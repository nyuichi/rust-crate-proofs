#!/usr/bin/env python3
"""AN proof-only three-owner normal Drop elaboration; mappings are unchecked.

The independent source/MIR checker must establish actual native correspondence.
This generator preserves AL's complete promotion core and AM's terminal helpers.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent
AL = ROOT.parent / "original-promotable-first-clone-2026-10-09"
AM = ROOT.parent / "original-promotable-automatic-drop-2026-10-09"
BASE_SHA = "2c51555cc0c5eb9da7719609b2b2826e3e354251b802ceb3b440efa2b5f5f3f2"
HELPER_SHA = "f7e410dcf3c2900464249a3d02ca04deae18cfaf59054ba1f20a9bdf9848d0f7"
FEATURES = ("", "omit_second", "omit_first", "omit_root", "swap_child_places",
            "duplicate_second", "duplicate_first", "duplicate_root", "early_root",
            "raw_branch", "stale_view", "missing_registration", "readonly_reseal")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def frozen_inputs():
    base = (ROOT / "src/promotion.rs").read_bytes()
    helper = (ROOT / "src/terminal_helpers.rs").read_bytes()
    if sha(base) != BASE_SHA or base != (AL / "src/promotion.rs").read_bytes():
        raise ValueError("AL promotion prefix changed")
    if sha(helper) != HELPER_SHA or helper != (AM / "generated/terminal-helper.rs").read_bytes():
        raise ValueError("AM terminal helpers changed")
    for old in (AM / "src").glob("*.rs"):
        if old.name == "lib.rs":
            continue
        if (ROOT / "src" / old.name).read_bytes() != old.read_bytes():
            raise ValueError("frozen support source changed: " + old.name)
    return base.decode(), helper.decode()


def client_source(feature):
    second = "    bytes_child_terminal_drop(second,scope.borrow_mut(),second_receipt.borrow_mut());\n"
    first = "    bytes_child_terminal_drop(first,scope.borrow_mut(),first_receipt.borrow_mut());\n"
    root = "    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());\n"
    client = """
/// Native lexical second/first Drop, surviving root read, saved return and
/// final root Drop. Every ledger key comes from the actual returned ticket.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_reclone_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let first=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let first_id=snapshot!(first.child_id());
    let before_reclone=snapshot!((*scope.observation()).0);
    let second=reclone_root(&original,scope.borrow_mut());
    let second_id=snapshot!(second.child_id());
    let second_fraction=snapshot!(second.child_fraction());
    let live_three=snapshot!((*scope.observation()).0);
    proof_assert!(!(*before_reclone).contains(*second_id));
    proof_assert!(*live_three==(*before_reclone).insert(*second_id,Excl(*second_fraction)));
    proof_assert!((*live_three).len()==3);
    let mut second_receipt=ghost! {None::<Completion>};
""" + second + """    proof_assert!(second_receipt.inner_logic()!=None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_three).remove(*second_id));
    proof_assert!(!(*scope.observation()).0.contains(*second_id));
    proof_assert!((*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==2);
    let live_two=snapshot!((*scope.observation()).0);
    let mut first_receipt=ghost! {None::<Completion>};
""" + first + """    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_two).remove(*first_id));
    proof_assert!(!(*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
""" + root + """    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}
"""
    calls = {"second": second, "first": first, "root": root}
    if feature.startswith("omit_"):
        client = client.replace(calls[feature.removeprefix("omit_")], "")
    elif feature.startswith("duplicate_"):
        call = calls[feature.removeprefix("duplicate_")]
        client = client.replace(call, call + call)
    elif feature == "swap_child_places":
        client = client.replace(second, second.replace("(second,", "(first,"))
        client = client.replace(first, first.replace("(first,", "(second,"))
    elif feature == "early_root":
        declaration = "    let mut root_receipt=ghost! {None::<Completion>};\n"
        client = client.replace(root, "").replace(declaration, "")
        client = client.replace("    let observed=borrowed.to_vec();\n",
                                declaration + root + "    let observed=borrowed.to_vec();\n")
    return client


def extension_source(feature):
    extension = (ROOT / "src/reclone_extension.rs").read_text()
    if feature == "raw_branch":
        assert extension.count("if kind==0usize {") == 2
        extension = extension.replace("if kind==0usize {", "if kind!=0usize {")
    elif feature == "stale_view":
        marker = "    let expected=snapshot!(phase.root.shared as *mut ());"
        assert extension.count(marker) == 2
        extension = extension.replace(marker, "    ghost! {phase.current=SyncView::new().into_inner();};\n" + marker)
    elif feature == "missing_registration":
        call = """            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());"""
        assert extension.count(call) == 1
        extension = extension.replace(call, "            let _=state; let _=c;")
    elif feature == "readonly_reseal":
        # This attempts to steal the affine owned history through a borrowed
        # Shared phase. Expected frontend E0507; it is an ownership-extraction
        # control, not a solver proof of the separate all-history constraint.
        marker = "    let stored=load_visible_snapshot(data,expected,own,current);"
        assert extension.count(marker) == 2
        extension = extension.replace(marker, marker + "\n    let _readonly=pointer_event::bind_read_only(data,stored,ghost! {phase.own});")
    return extension


def native_mapping():
    native = ROOT / "native.rs"
    paths = sorted((ROOT / "native-mir").glob("*ElaborateDrops.after.mir"))
    clients = [p for p in paths if ".promoted_reclone_scope." in p.name]
    places, edges = {}, []
    if len(clients) == 1:
        text = clients[0].read_text()
        places = dict(re.findall(r"debug\s+(\w+)\s*=>\s*(_\d+)\s*;", text))
        for block, cleanup, body in re.findall(r"\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}", text, re.S):
            if cleanup:
                continue
            for place, successor, unwind in re.findall(r"drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]", body):
                owner = next((x for x in ("second", "first", "original") if places.get(x) == place), None)
                if owner:
                    edges.append(dict(block=block, place=place, owner=owner, successor=successor,
                                      unwind=unwind.strip(), scope="scope", output=("root" if owner == "original" else owner) + "_receipt"))
    return dict(native_source="native.rs", native_source_sha256=sha(native.read_bytes()) if native.is_file() else None,
                native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients) == 1 else None,
                native_mir_ready=[e["owner"] for e in edges] == ["second", "first", "original"],
                debug_places=places, normal_edges=edges,
                mir=[dict(path=p.relative_to(ROOT).as_posix(), sha256=sha(p.read_bytes())) for p in paths])


def generate(feature):
    base, helpers = frozen_inputs()
    extension, client = extension_source(feature), client_source(feature)
    active = base + "\n" + helpers + extension + client
    output = ROOT / "generated"
    output.mkdir(exist_ok=True)
    for name, text in (("active.rs", active), ("terminal-helper.rs", helpers),
                       ("reclone-extension.rs", extension), ("elaborated-client.rs", client)):
        (output / name).write_text(text)
    if not feature:
        (output / "positive.rs").write_text(active)
    mapping = dict(feature=feature, status="generated_unchecked", stage="after-ElaborateDrops",
                   base_source="src/promotion.rs", base_source_sha256=sha(base.encode()),
                   terminal_helpers_sha256=sha(helpers.encode()), extension_source="src/reclone_extension.rs",
                   extension_sha256=sha(extension.encode()), client_sha256=sha(client.encode()),
                   active="generated/active.rs", active_sha256=sha(active.encode()),
                   helpers=["bytes_child_terminal_drop", "bytes_root_terminal_drop"],
                   callbacks=["even_reclone_checked", "odd_reclone_checked"],
                   second_clone_events=["Acquire root pointer load", "Relaxed refcount increment", "fresh child pointer construction"],
                   return_evaluation=dict(shadow="let saved_return=observed", root_effect_after_evaluation=True),
                   excluded=["unwind completion", "concurrent first-promotion loser", "arbitrary concurrent closure", "whole crate"],
                   tcb=["native compiler/MIR and normal terminal-place elaboration", "address nonobservation and no independent field-drop glue", "AL generic pointer/field/physical/erased-callback boundaries"],
                   **native_mapping())
    (output / "mapping.json").write_text(json.dumps(mapping, indent=2) + "\n")
    print(json.dumps({k: mapping[k] for k in ("feature", "active_sha256", "extension_sha256", "client_sha256", "native_mir_ready")}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--feature", default="", choices=FEATURES)
    generate(parser.parse_args().feature)
