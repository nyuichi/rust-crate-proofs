#!/usr/bin/env python3
"""Generate AM's bounded, proof-only normal terminal effects.

AL remains byte exact. The independent source/MIR checker, not this generator's
mapping receipt, certifies each terminal place and erased Ghost channel.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent
FEATURES = ("", "omit_child", "omit_root", "swap_places", "swap_adapters",
            "duplicate_child", "duplicate_root", "early_root")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def frozen_base():
    manifest = json.loads((ROOT / "frozen-al-source.json").read_text())
    for name, expected in manifest["rust_modules"].items():
        if name == "lib.rs":
            continue  # The independent checker certifies the sole route change.
        actual = (ROOT / "src" / name).read_bytes()
        if sha(actual) != expected:
            raise ValueError(f"frozen AL module changed: {name}")
    return (ROOT / "src/promotion.rs").read_text()


def cleanup_contract(base, name):
    marker = "\nfn " + name + "("
    if base.count(marker) != 1:
        raise ValueError(f"cleanup declaration is not unique: {name}")
    end = base.index(marker)
    start = base.rfind("\n\n", 0, end) + 2
    contract = base[start:end]
    if not contract.startswith("#[requires(") or "#[trusted]" in contract:
        raise ValueError(f"unexpected cleanup contract: {name}")
    return contract


def terminal_helpers(base):
    child = cleanup_contract(base, "cleanup_child") + """
fn bytes_child_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) {
    cleanup_child(value,scope,output)
}
"""
    root = cleanup_contract(base, "cleanup_root") + """
fn bytes_root_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) {
    cleanup_root(value,scope,output)
}
"""
    return child + "\n" + root


def elaborated_client(feature):
    child = "    bytes_child_terminal_drop(child,scope.borrow_mut(),child_receipt.borrow_mut());\n"
    root = "    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());\n"
    client = """
/// Proof-only elaboration: inner-scope child Drop, root read, saved return,
/// then root Drop. The independent checker certifies the native normal edges.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_automatic_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let child=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(child.child_id());
    let live_before=snapshot!((*scope.observation()).0);
    let mut child_receipt=ghost! {None::<Completion>};
""" + child + """    proof_assert!(child_receipt.inner_logic()!=None && !child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_before).remove(*child_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!(!(*scope.observation()).0.contains(*child_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
""" + root + """    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}
"""
    if feature == "omit_child":
        client = client.replace(child, "")
    elif feature == "omit_root":
        client = client.replace(root, "")
    elif feature == "swap_places":
        client = client.replace(child, child.replace("(child,", "(original,"))
        client = client.replace(root, root.replace("(original,", "(child,"))
    elif feature == "swap_adapters":
        client = client.replace(child, child.replace("bytes_child_terminal_drop", "bytes_root_terminal_drop"))
        client = client.replace(root, root.replace("bytes_root_terminal_drop", "bytes_child_terminal_drop"))
    elif feature == "duplicate_child":
        client = client.replace(child, child + child)
    elif feature == "duplicate_root":
        client = client.replace(root, root + root)
    elif feature == "early_root":
        declaration = "    let mut root_receipt=ghost! {None::<Completion>};\n"
        client = client.replace(root, "").replace(declaration, "")
        client = client.replace("    let observed=borrowed.to_vec();\n",
                                declaration + root + "    let observed=borrowed.to_vec();\n")
    elif feature:
        raise ValueError(feature)
    return client


def native_mapping():
    native = ROOT / "native.rs"
    mir_paths = sorted((ROOT / "native-mir").glob("*ElaborateDrops.after.mir"))
    clients = [p for p in mir_paths if ".promoted_automatic_scope." in p.name]
    rows = []
    debug_places = {}
    if len(clients) == 1:
        text = clients[0].read_text()
        debug_places = dict(re.findall(r"debug\s+(\w+)\s*=>\s*(_\d+)\s*;", text))
        for block, cleanup, contents in re.findall(r"\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}", text, re.S):
            if cleanup:
                continue
            for place, successor, unwind in re.findall(r"drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]", contents):
                owner = next((name for name in ("child", "original") if debug_places.get(name) == place), None)
                if owner:
                    rows.append(dict(block=block, place=place, owner=owner,
                                     successor=successor, unwind=unwind.strip(),
                                     scope="scope", output=owner.replace("original", "root") + "_receipt"))
    return dict(
        native_source="native.rs", native_source_sha256=sha(native.read_bytes()) if native.is_file() else None,
        native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients) == 1 else None,
        native_mir_ready=len(rows) == 2 and {row["owner"] for row in rows} == {"child", "original"},
        debug_places=debug_places, normal_edges=rows,
        mir=[dict(path=p.relative_to(ROOT).as_posix(), sha256=sha(p.read_bytes())) for p in mir_paths])


def generate(feature):
    base = frozen_base()
    helpers = terminal_helpers(base)
    client = elaborated_client(feature)
    active = base + "\n" + helpers + client
    generated = ROOT / "generated"
    generated.mkdir(exist_ok=True)
    for name, text in (("active.rs", active), ("terminal-helper.rs", helpers), ("elaborated-client.rs", client)):
        (generated / name).write_text(text)
    if not feature:
        (generated / "positive.rs").write_text(active)
    mapping = dict(
        feature=feature, status="generated_unchecked", base_source="src/promotion.rs",
        base_source_sha256=sha(base.encode()), active="generated/active.rs", active_sha256=sha(active.encode()),
        helper_sha256=sha(helpers.encode()), client_sha256=sha(client.encode()),
        helpers=["bytes_child_terminal_drop", "bytes_root_terminal_drop"],
        helper_interpretation="ordinary body-proved consuming forward calls, no trusted Bytes Drop effect",
        stage="after-ElaborateDrops", return_evaluation=dict(shadow="let saved_return=observed", root_effect_after_evaluation=True, child_effect_before_root_read=True),
        excluded=["unwind completion", "concurrent first-promotion loser", "arbitrary Drop/move equivalence", "whole crate"],
        tcb=["native compiler/MIR and normal terminal-place elaboration", "receiver/data-field address nonobservation and no independent field-drop glue", "AL generic pointer/field/physical/erased-callback boundaries"],
        **native_mapping())
    (generated / "mapping.json").write_text(json.dumps(mapping, indent=2) + "\n")
    print(json.dumps({k: mapping[k] for k in ("feature", "active_sha256", "helper_sha256", "client_sha256", "native_mir_ready")}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--feature", choices=FEATURES, default="")
    generate(parser.parse_args().feature)
