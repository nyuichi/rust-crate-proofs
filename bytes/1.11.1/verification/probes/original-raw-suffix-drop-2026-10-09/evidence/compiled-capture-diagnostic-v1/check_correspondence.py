#!/usr/bin/env python3
"""AX raw suffix normal-Drop source/native/Cargo correspondence gate.

The gate checks the frozen AV proof prefix, the AX source-shaped operations,
the independently parsed native MIR witness, and the actual Cargo public-record
build input. It does not claim whole-crate verification or unwind coverage.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
PROBES = ROOT.parent
AV_ROOT = PROBES / "original-promotable-suffix-promotion-2026-10-09"
AV_CHECKER_PATH = AV_ROOT / "check_correspondence.py"
AV_CHECKER_SHA = "bf3e513f5a0e783b2beb638ce96f25a85ecc3f8e1955ced329533f783c133ef3"
AV_NATIVE_SHA = "f1cfaf1ac60107643e936d22a098c06671ac6f2b0178b9cb6354444f5f4d31df"
AV_POSITIVE_SHA = "67b6643e09f88fd4129db990418207aeecf9c81a90d51113b6b8759f513f2e5e"
AV_ARCHIVE_SHA = "a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489"
AV_RECEIPT_SHA = "a9e2852bfec95740eb2e7a2751e185a198267cc89f4ff829bf4724aba5c40f64"
AV_AUDIT_SHA = "3de128f828f4d142d06b2bae4b7a4ce2a9bf349046b0cf017805a8b19f728ecc"
AV_REPLAY_SHA = "dcbd4844eefd02c43e03f9e2848f8c232fd91e074afe778ccf827c22886e197d"
AX_PROMOTION_SHA = "9440ac43f7a116e4bd36b5affb449334be8872e34f9e9cb7a9dc0c923311d414"
AX_NATIVE_CHECKER_SHA = "309e0dd8a807c1c96f3b791663d62e11d4bc08bcdf01e9fd9d482f320e5b2320"
PACKAGE = "bytes-original-raw-suffix-drop"
AV_PACKAGE = "bytes-original-promotable-suffix-promotion"
TARGET_DIR = pathlib.Path("/workspace/bytes-proof-tools/targets/bytes").resolve()
CAPTURE_DIR = ROOT / "generated/compiled-inputs"
ARTIFACT_FIELDS = {
    "public_records.rs": "captured_input_sha256",
    "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
    "cargo-build-output.txt": "captured_build_output_sha256",
    "cargo-root-output.txt": "captured_root_output_sha256",
}
RERUN_PATHS = [
    "../../../src/bytes.rs", "../../../src/bytes/bytes_record.rs",
    "../../../src/bytes/vtable_record.rs", "../../../src/bytes_mut.rs", "extract_public.py",
]
BUILD_DIRECTIVES = [
    "cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
    "cargo:rustc-cfg=bytes_original_shared_gate",
] + ["cargo:rerun-if-changed=" + path for path in RERUN_PATHS]


class CheckError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_file(path: pathlib.Path, label: str) -> bytes:
    require(path.is_file() and not path.is_symlink(), f"{label} missing or redirected")
    return path.read_bytes()


def load_av_checker():
    raw = read_file(AV_CHECKER_PATH, "published AV correspondence checker")
    require(sha(raw) == AV_CHECKER_SHA, "published AV correspondence checker changed before import")
    spec = importlib.util.spec_from_file_location("ax_pinned_av_checker", AV_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load pinned AV correspondence checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    require(sha(read_file(AV_ROOT / "check_native.py", "published AV native checker")) == AV_NATIVE_SHA,
            "published AV native checker changed before admission")
    return module


AV = load_av_checker()


def regular_tree(directory: pathlib.Path, label: str) -> dict[str, bytes]:
    require(directory.is_dir() and not directory.is_symlink(), f"{label} source directory missing or redirected")
    result: dict[str, bytes] = {}
    for path in directory.rglob("*"):
        require(not path.is_symlink(), f"{label} source contains symlink: {path}")
        if path.is_dir():
            continue
        require(path.is_file(), f"{label} source contains nonregular file: {path}")
        result[path.relative_to(directory).as_posix()] = path.read_bytes()
    return result


def published_av() -> tuple[Any, dict[str, bytes], dict[str, Any]]:
    # AV.audit performs the archived-AU lineage replay, AV source/native/Cargo
    # checks, and the complete 167-target proof inventory without invoking tools.
    evidence = AV_ROOT / "evidence"
    for path, expected, label in (
        (evidence / "av-positive-reuse-canonical-v1.tar.gz", AV_ARCHIVE_SHA, "AV canonical archive"),
        (evidence / "av-positive-reuse-canonical-v1.json", AV_RECEIPT_SHA, "AV canonical receipt"),
        (evidence / "AV_CANONICAL_AUDIT.json", AV_AUDIT_SHA, "AV independent audit"),
        (evidence / "av-positive-reuse-canonical-v1-audit.json", AV_REPLAY_SHA, "AV replay"),
    ):
        require(sha(read_file(path, label)) == expected, f"{label} hash differs from published AV")
    report = AV.audit(cargo_mode="captured")
    require(report.get("status") == "pass" and
            report.get("AV_proof_inventory", {}).get("target_count") == 167 and
            report.get("AV_proof_inventory", {}).get("prover_leaves") == 1683 and
            report.get("AV_compiled_production_input", {}).get("status") == "pass",
            "published AV source/native/Cargo/proof audit failed")
    au_checker, au_files, au_info = AV.assert_published_au()
    return au_checker, au_files, {"audit": report, "au_info": au_info}


def av_target_rows() -> tuple[dict[str, Any], list[str]]:
    raw = read_file(AV_ROOT / "evidence/av-positive-reuse-canonical-v1.json", "AV canonical target receipt")
    require(sha(raw) == AV_RECEIPT_SHA, "AV canonical target receipt hash changed")
    receipt = json.loads(raw)
    require(receipt.get("archive_sha256") == AV_ARCHIVE_SHA and receipt.get("status") == "admitted_reuse" and
            receipt.get("statistics") == {"files": 167, "prover": 1683, "null": 0, "structural": 0},
            "AV canonical target receipt status/statistics changed")
    prefix = f"probe/verif/{AV_PACKAGE.replace('-', '_')}_rlib/"
    relative: list[str] = []
    for row in receipt.get("targets", []):
        coma = row.get("coma") if isinstance(row, dict) else None
        require(isinstance(coma, str) and coma.startswith(prefix), "AV canonical target escapes parent package")
        relative.append(coma[len(prefix):])
    require(len(relative) == 167 and len(set(relative)) == 167, "AV canonical target inventory is not 167 unique paths")
    return receipt, sorted(relative)


def extract_function(source: str, name: str) -> tuple[str, str]:
    matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\b", source))
    require(len(matches) == 1, f"expected exactly one selected Rust function {name}, found {len(matches)}")
    start = matches[0].start()
    brace = source.find("{", matches[0].end())
    require(brace >= 0, f"function {name} has no body")
    depth = 0
    string = False
    escaped = False
    line_comment = False
    block_depth = 0
    i = brace
    while i < len(source):
        c = source[i]
        n = source[i + 1] if i + 1 < len(source) else ""
        if line_comment:
            if c == "\n": line_comment = False
        elif block_depth:
            if c == "/" and n == "*": block_depth += 1; i += 1
            elif c == "*" and n == "/": block_depth -= 1; i += 1
        elif string:
            if escaped: escaped = False
            elif c == "\\": escaped = True
            elif c == '"': string = False
        elif c == "/" and n == "/": line_comment = True; i += 1
        elif c == "/" and n == "*": block_depth = 1; i += 1
        elif c == '"': string = True
        elif c == "{": depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return source[start:i + 1], source[brace + 1:i]
        i += 1
    raise CheckError(f"unbalanced function body {name}")


def source_correspondence(mapping: dict[str, Any], native_report: dict[str, Any]) -> dict[str, Any]:
    av_prefix = read_file(AV_ROOT / "generated/positive.rs", "AV positive source")
    promotion_raw = read_file(ROOT / "src/promotion.rs", "AX selected promotion source")
    require(sha(av_prefix) == AV_POSITIVE_SHA and promotion_raw.startswith(av_prefix),
            "AX selected source is not the complete published AV positive prefix")
    require(sha(promotion_raw) == AX_PROMOTION_SHA, "frozen AX promotion source changed")
    promotion = promotion_raw.decode()
    suffix = promotion[len(av_prefix):]
    client_item, client_body = extract_function(suffix, "raw_suffix_scope")
    require("from_box_scoped(input)" in client_body and "SuffixScope::new(scope)" in client_body and
            "advance_raw_suffix(&mutvalue,amount" in re.sub(r"\s+", "", client_body) and
            "chunk_raw_suffix(&value,suffix.borrow()).to_vec()" in re.sub(r"\s+", "", client_body) and
            "bytes_raw_suffix_terminal_drop(value,suffix,receipt.borrow_mut())" in re.sub(r"\s+", "", client_body) and
            client_body.rfind("bytes_raw_suffix_terminal_drop") > client_body.find(".to_vec()") and
            not re.search(r"\b(?:clone|shallow_clone|clone_suffix)\s*\(", client_body),
            "AX shadow client does not preserve native From→advance→read→saved Vec→normal Drop order")

    constructor, constructor_body = extract_function(promotion, "from_box_scoped")
    required_constructor = ["input.len()==0", "raw_vec::detach_boxed_slice(input)",
        "raw.into_bound_ptr_at_zero()", "pointer_event::new_pointer(word,current.borrow_mut())",
        "Bytes {ptr,len,data,vtable", "Phase::Raw"]
    require(all(token in constructor_body for token in required_constructor),
            "from_box_scoped no longer binds the nonempty Box detach to one raw owner/AtomicPtr data field")
    advance, advance_body = extract_function(suffix, "advance_raw_suffix")
    inc, inc_body = extract_function(suffix, "inc_start_raw_suffix")
    compact_advance = re.sub(r"\s+", "", advance_body)
    compact_inc = re.sub(r"\s+", "", inc_body)
    require("amount<=value.len()" in compact_advance and "inc_start_raw_suffix(value,amount,scope)" in compact_advance and
            "value.len-=amount" in compact_inc and "cursor_pointer::add(value.ptr,amount" in compact_inc and
            "value.ptr=ptr" in compact_inc and "value.data=" not in compact_inc and "value.vtable=" not in compact_inc,
            "AX advance shadow does not join native len guard/subtraction/pointer-add on the same owner")
    require("(^value).data==value.data" in suffix and "(^value).vtable==value.vtable" in suffix,
            "AX advance contract does not frame native data/vtable fields")

    chunk, chunk_body = extract_function(suffix, "chunk_raw_suffix")
    read, read_body = extract_function(suffix, "raw_suffix_as_slice")
    require("raw_suffix_as_slice(value,scope)" in re.sub(r"\s+", "", chunk_body) and
            "physical_projection::borrow(value.ptr,value.len" in re.sub(r"\s+", "", read_body),
            "AX chunk/read shadow no longer maps to current ptr/len physical borrow")

    free, free_body = extract_function(suffix, "free_raw_suffix_checked")
    compact_free = re.sub(r"\s+", "", free_body)
    require("suffix_pointer::distance(pointer,base" in compact_free and "distanceasusize+len" in compact_free and
            "physical_projection::deallocate(base,capacity" in compact_free and
            "#[ensures(result.inner_logic().pointer()==base)]" in suffix and
            "#[ensures(result.inner_logic().size()==input.inner_logic().0.capacity@)]" in suffix,
            "AX raw suffix free does not recover and deallocate exactly the original base/capacity")

    even, even_body = extract_function(suffix, "even_raw_suffix_drop_checked")
    odd, odd_body = extract_function(suffix, "odd_raw_suffix_drop_checked")
    for name, body in (("even", even_body), ("odd", odd_body)):
        compact = re.sub(r"\s+", "", body)
        wanted = ("input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize==0usize" if name == "even" else
                  "input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize!=0usize")
        require("owned_pointer::get_mut_finish(data,own)" in compact and
                wanted in suffix and
                "free_raw_suffix_checked(base,offset,len" in compact,
                f"AX {name} raw callback lacks same-field finish/parity/raw-free sequence")
    terminal, terminal_body = extract_function(suffix, "bytes_raw_suffix_terminal_drop")
    compact_terminal = re.sub(r"\s+", "", terminal_body)
    selector = compact_terminal[compact_terminal.index("letaddress="):compact_terminal.index("erased_call::invoke3")]
    require("letnative=value.vtable.drop" in compact_terminal and "descriptor.base.raw_pointer()" in terminal_body and
            "even_raw_suffix_drop_registration" in terminal_body and "odd_raw_suffix_drop_registration" in terminal_body and
            "erased_call::invoke3(native,(&mutvalue.data,value.ptr,value.len)" in compact_terminal and
            "ifaddress&1usize==0usize" in compact_terminal and
            compact_terminal.index("ifaddress&1usize==0usize") < compact_terminal.index("even_raw_suffix_drop_registration") <
            compact_terminal.index("odd_raw_suffix_drop_registration") and
            "value.ptr" not in selector and "value.vtable" not in selector,
            "AX terminal Drop is not one stored-vtable call selected only from immutable base parity")
    require("physical_projection::FreeReceipt" in terminal and "Completion" not in terminal,
            "AX raw terminal effect changed from physical free receipt")

    client_report = native_report["native_audit"]["client"]
    require(client_report.get("normal_cfg_exact") is True and client_report.get("function") == "raw_suffix_scope" and
            client_report.get("debug_places") == {"input": "_1", "amount": "_2", "value": "_3"} and
            client_report.get("normal_drop_order") == ["value"] and
            client_report.get("saved_return_precedes_drop") is True,
            "native MIR report does not bind the AX ordinary raw suffix Drop witness")
    ops = client_report["operations"]
    require(ops["from_box"]["input"] == "_1" and ops["from_box"]["result"] == "_3" and
            ops["advance"]["receiver"] == "_3" and ops["advance"]["amount"] == "_2" and
            ops["chunk"]["receiver"] == "_3" and ops["to_vec"]["result"] == "_0" and
            client_report["normal_edges"] == [{"block":"bb4","place":"_3","owner":"value","successor":"bb5",
                "unwind":"bb9","scope":"raw_suffix","role":"raw_suffix_terminal_drop"}],
            "native MIR operation/normal Drop roles changed")

    expected_bindings = [
        {"native_operation":"From<Box<[u8]>>::from","native_block":"bb0","native_input":"_1","native_result":"_3",
         "proof_operation":"from_box_scoped","proof_input":"input","proof_result":"value"},
        {"native_operation":"Buf::advance","native_block":"bb1","native_receiver":"_3","native_amount":"_2",
         "proof_operation":"advance_raw_suffix","proof_receiver":"value","proof_amount":"amount",
         "mutation_operation":"inc_start_raw_suffix"},
        {"native_operation":"Buf::chunk","native_block":"bb2","native_receiver":"_3","native_result":"_9",
         "proof_operation":"chunk_raw_suffix","proof_receiver":"value","read_operation":"raw_suffix_as_slice"},
        {"native_operation":"slice::to_vec","native_block":"bb3","native_result":"_0",
         "saved_return_before_drop":True,"proof_operation":"Vec::to_vec"},
        {"native_operation":"Bytes normal Drop","native_block":"bb4","native_place":"_3","native_successor":"bb5",
         "proof_operation":"bytes_raw_suffix_terminal_drop","parity_selector":"immutable_original_base_in_ghost",
         "callbacks":["even_raw_suffix_drop_checked","odd_raw_suffix_drop_checked"],
         "physical_free":"free_raw_suffix_checked","receipt":"physical_projection::FreeReceipt"},
    ]
    capture = native_report["capture"]
    selected = sorted(({"path": row["path"], "sha256": row["sha256"]} for row in capture["selected_paths"].values()),
                     key=lambda row: row["path"])
    require(mapping.get("schema") == "ax-raw-suffix-source-native-map-v1" and
            mapping.get("status") == "generated_unchecked" and mapping.get("shadow") == "src/promotion.rs" and
            mapping.get("native_source") == "native.rs" and mapping.get("native_source_sha256") == client_report["source_sha256"] and
            mapping.get("native_client_mir_sha256") == client_report["mir_sha256"] and
            mapping.get("debug_places") == client_report["debug_places"] and
            mapping.get("operations") == ops and mapping.get("normal_edges") == client_report["normal_edges"] and
            mapping.get("mir_blocks") == client_report["mir_blocks"] and mapping.get("mir") == selected and
            mapping.get("bindings") == expected_bindings,
            "AX mapping is not independently rederived from source operations and native MIR")
    return {"status":"pass", "selected_source_sha256":sha(promotion_raw), "av_prefix_sha256":sha(av_prefix),
        "operation_bindings":expected_bindings, "source_body_sha256":{name:sha(item.encode()) for name,item in {
            "from_box_scoped":constructor,"advance_raw_suffix":advance,"inc_start_raw_suffix":inc,
            "chunk_raw_suffix":chunk,"raw_suffix_as_slice":read,"free_raw_suffix_checked":free,
            "even_raw_suffix_drop_checked":even,"odd_raw_suffix_drop_checked":odd,
            "bytes_raw_suffix_terminal_drop":terminal,"raw_suffix_scope":client_item}.items()},
        "native_client":{"selected_mir_count":capture["selected_mir_count"],"production_mir_count":capture["production_mir_count"],
            "normal_edges":client_report["normal_edges"],"saved_return_precedes_drop":True}}


def audit_source_and_native(mapping: dict[str, Any]) -> dict[str, Any]:
    av_src = regular_tree(AV_ROOT / "src", "published AV")
    ax_src = regular_tree(ROOT / "src", "AX")
    require(set(ax_src) == set(av_src), "AX source inventory differs from complete AV source inventory")
    av_positive = read_file(AV_ROOT / "generated/positive.rs", "AV positive source")
    require(sha(av_positive) == AV_POSITIVE_SHA, "AV positive source prefix hash changed")
    for name, data in av_src.items():
        if name == "promotion.rs":
            require(ax_src[name].startswith(av_positive), "AX promotion does not preserve full AV positive source prefix")
        elif name == "lib.rs":
            old = b'#[cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;'
            new = b"#[cfg(creusot)] mod promotion;"
            require(data.count(old) == 1 and ax_src[name] == data.replace(old,new,1),
                    "AX lib.rs changed beyond its exact selected promotion module route")
        else:
            require(ax_src[name] == data, f"AX changed inherited AV support file src/{name}")
    manifest_raw = read_file(ROOT / "Cargo.toml", "AX Cargo manifest")
    manifest = tomllib.loads(manifest_raw.decode())
    require(manifest == {"package":{"name":PACKAGE,"version":"0.1.0","edition":"2021","publish":False},
        "dependencies":{"creusot-std":"=0.13.0"},"workspace":{},"features":{
            "negative_missing_acquire":[],"negative_missing_payload_free":[],"negative_missing_control_free":[]}},
        "AX manifest has an unreviewed dependency, feature, target, or Cargo patch route")
    require(sha(read_file(ROOT / "Cargo.lock", "AX Cargo lock")) == "0b8252a36053aae736bce69369284ee6ed4c3e3c9d79126e33db441ead6416ee",
            "AX lockfile changed")
    native = import_native_checker()
    native_report = native.audit_bundle(native.load_bundle())
    require(native_report.get("status") == "pass", "AX native source/MIR gate failed")
    source_report = source_correspondence(mapping, native_report)
    return {"source_inventory":len(ax_src),"complete_av_support_byte_exact":True,
        "promotion_prefix":"complete AV generated/positive.rs", "lib_route":"one exact direct-module route",
        "manifest_and_lock_pinned":True,"native":native_report,"source_native_join":source_report}


def import_native_checker():
    path = ROOT / "check_native.py"
    require(sha(read_file(path,"AX native checker")) == AX_NATIVE_CHECKER_SHA,
            "AX native checker changed after its reviewed freeze")
    spec=importlib.util.spec_from_file_location("ax_pinned_native_checker",path)
    require(spec is not None and spec.loader is not None,"cannot load AX native checker")
    mod=importlib.util.module_from_spec(spec);sys.modules[spec.name]=mod;spec.loader.exec_module(mod)
    return mod


def target_inventory(mapping: dict[str, Any]) -> dict[str, Any]:
    receipt, inherited = av_target_rows()
    policy_path = ROOT / "generated/proof-targets.json"
    all_comas = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT/"verif").rglob("*.coma"))
    require(all_comas, "AX translation produced no Coma targets")
    current_prefix = f"verif/{PACKAGE.replace('-','_')}_rlib/"
    inherited_current={current_prefix+x for x in inherited}
    require(inherited_current <= set(all_comas),"one or more inherited AV proof targets disappeared in AX translation")
    new = set(all_comas)-inherited_current
    require(new and all(path.startswith(current_prefix+"promotion/") for path in new),
            "AX added an unreviewed non-promotion target outside the appended raw-suffix source")
    promotion=(ROOT/"src/promotion.rs").read_text()
    suffix=promotion[len(read_file(AV_ROOT/"generated/positive.rs","AV positive source")):]
    source_functions=set(re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)",suffix))
    new_relative={path[len(current_prefix):] for path in new}
    for path in new_relative:
        stem=path[:-5] if path.endswith(".coma") else path
        name=stem.split("/")[-1]
        require(name in source_functions,
                f"new AX target has no matching appended source function: {path}")
    require(any(path.endswith("/raw_suffix_scope.coma") for path in new_relative),
            "translated AX targets omit the selected raw_suffix_scope proof")
    # run-proof.sh checks correspondence immediately after `--only=coma`, then
    # writes proof-targets.json. If that file already exists it may describe a
    # prior run, so the current .coma inventory is authoritative at this stage.
    policy_sha=None
    if policy_path.is_file() and not policy_path.is_symlink():
        policy=json.loads(policy_path.read_text())
        if set(policy.get("translated",[]))==set(all_comas):
            require(set(policy.get("included",[]))==set(all_comas) and policy.get("excluded")=={} and
                    policy.get("features")==[] and policy.get("correspondence_exit_status")==0 and
                    policy.get("diagnostic") is False,
                    "fresh AX proof-target policy has exclusions, features, or skipped correspondence")
            policy_sha=sha(policy_path.read_bytes())
    inherited_file=json.loads(read_file(ROOT/"inherited-targets.json","AX inherited target table"))
    require(inherited_file.get("ancestor_archive_sha256")==AV_ARCHIVE_SHA and
            inherited_file.get("ancestor_receipt_sha256")==AV_RECEIPT_SHA and
            inherited_file.get("source_prefix_sha256")==AV_POSITIVE_SHA and
            inherited_file.get("relative_targets")==inherited and inherited_file.get("target_count")==167,
            "AX inherited target table differs from the published AV canonical receipt")
    return {"status":"pass","translated_coma_count":len(all_comas),"inherited_av_targets":len(inherited),
        "new_ax_targets":len(new),"target_policy_sha256":policy_sha,
        "target_policy_timing":"validated directly from current Coma inventory; launcher writes policy after checker",
        "inherited_target_receipt_sha256":AV_RECEIPT_SHA,"no_exclusions":True}


def expected_public_records() -> tuple[Any, bytes, dict[str,str]]:
    au_checker, _au_files, _info = published_av()
    expected, inputs = AV.av_expected_public_records(au_checker)
    return au_checker, expected, inputs


def cargo_artifacts_live(source_map: bytes, *, capture: bool) -> dict[str, Any]:
    _checker, expected, input_hashes = expected_public_records()
    matches=list(TARGET_DIR.glob(f"debug/.fingerprint/{PACKAGE}-*/run-build-script-build-script-build.json"))
    require(len(matches)==1,"AX package must have exactly one Cargo build-script fingerprint")
    fp=matches[0].resolve(); fp_bytes=read_file(fp,"AX Cargo fingerprint")
    fingerprint=json.loads(fp_bytes)
    rows=[r["RerunIfChanged"] for r in fingerprint.get("local",[])
        if isinstance(r,dict) and isinstance(r.get("RerunIfChanged"),dict)]
    require(len(rows)==1 and rows[0].get("paths")==RERUN_PATHS,"AX Cargo fingerprint source inputs changed")
    rel=pathlib.Path(str(rows[0].get("output","")))
    require(not rel.is_absolute() and ".." not in rel.parts and rel.parts[:2]==("debug","build"),
        "AX fingerprint output path escapes selected Cargo target")
    output=(TARGET_DIR/rel).resolve()
    require(output.is_file() and output.parent.name==fp.parent.name and output.parent.name.startswith(PACKAGE+"-") and
        fp.parent.parent.resolve()==(TARGET_DIR/"debug/.fingerprint").resolve() and
        output.parent.parent.resolve()==(TARGET_DIR/"debug/build").resolve(),
        "AX package, fingerprint, target and build output do not join")
    output_bytes=read_file(output,"AX Cargo build output")
    require(output_bytes==("\n".join(BUILD_DIRECTIVES)+"\n").encode(),"AX actual Cargo build directives differ")
    out=output.parent/"out"; compiled=out/"public_records.rs"; root_output=output.parent/"root-output"
    require(compiled.is_file() and not compiled.is_symlink() and root_output.is_file() and not root_output.is_symlink() and
        compiled.read_bytes()==expected and root_output.read_bytes()==str(out).encode(),
        "AX actual Cargo OUT_DIR public records do not match source reconstruction")
    artifacts={"public_records.rs":compiled.read_bytes(),"cargo-run-build-fingerprint.json":fp_bytes,
        "cargo-build-output.txt":output_bytes,"cargo-root-output.txt":root_output.read_bytes()}
    source_map_raw=source_map
    source_map_obj=json.loads(source_map_raw)
    require(source_map_obj.get("generated/public_records.rs",{}).get("sha256")==sha(expected),
        "AX extractor source-map does not bind the generated public records")
    receipt={"schema":"ax-compiled-public-records-v1","status":"pass","probe_package":PACKAGE,
        "build_script_sha256":sha(read_file(ROOT/"build.rs","AX build script")),
        "extractor_sha256":sha(read_file(ROOT/"extract_public.py","AX extractor")),
        "cargo_target_dir":str(TARGET_DIR),"cargo_build_fingerprint":str(fp),"build_output_path":str(output),
        "root_output_path":str(root_output),"actual_out_dir":str(out),"compiled_input_path":str(compiled),
        "compiled_input_sha256":sha(expected),"reconstructed_generated_sha256":sha(expected),
        "source_map_sha256":sha(source_map_raw),"production_rerun_input_sha256":input_hashes,
        "captured_actual_Cargo_artifact_count":4}
    for filename,key in ARTIFACT_FIELDS.items():receipt[key]=sha(artifacts[filename])
    CAPTURE_DIR.mkdir(parents=True,exist_ok=True)
    if capture:
        for filename,data in artifacts.items():(CAPTURE_DIR/filename).write_bytes(data)
        (CAPTURE_DIR/"public-records-build-receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    return audit_captured_artifacts(receipt,artifacts,source_map_raw,expected,input_hashes) | {
        "snapshot_written":capture,"live_fingerprint_and_output_checked":True}


def audit_captured_artifacts(receipt:dict[str,Any],artifacts:dict[str,bytes],source_map:bytes,
                             expected:bytes,input_hashes:dict[str,str])->dict[str,Any]:
    require(set(artifacts)==set(ARTIFACT_FIELDS) and receipt.get("schema")=="ax-compiled-public-records-v1" and
        receipt.get("status")=="pass" and receipt.get("probe_package")==PACKAGE and
        receipt.get("captured_actual_Cargo_artifact_count")==4,"AX compiled artifact receipt is malformed")
    for filename,key in ARTIFACT_FIELDS.items():
        require(receipt.get(key)==sha(artifacts[filename]),f"AX captured Cargo artifact hash mismatch: {filename}")
    fingerprint=json.loads(artifacts["cargo-run-build-fingerprint.json"])
    rows=[r["RerunIfChanged"] for r in fingerprint.get("local",[])
        if isinstance(r,dict) and isinstance(r.get("RerunIfChanged"),dict)]
    require(len(rows)==1 and rows[0].get("paths")==RERUN_PATHS,"AX captured fingerprint inputs changed")
    out=pathlib.Path(receipt.get("actual_out_dir",""));compiled=pathlib.Path(receipt.get("compiled_input_path",""))
    output=pathlib.Path(receipt.get("build_output_path",""));fp=pathlib.Path(receipt.get("cargo_build_fingerprint",""))
    rootout=pathlib.Path(receipt.get("root_output_path",""))
    require(receipt.get("cargo_target_dir")==str(TARGET_DIR) and output.name=="output" and
        fp.name=="run-build-script-build-script-build.json" and output.parent.name.startswith(PACKAGE+"-") and
        fp.parent.name==output.parent.name and (TARGET_DIR/pathlib.Path(str(rows[0].get("output")))).resolve()==output.resolve() and
        fp.parent.parent.resolve()==(TARGET_DIR/"debug/.fingerprint").resolve() and
        output.parent.parent.resolve()==(TARGET_DIR/"debug/build").resolve() and
        out.resolve()==(output.parent/"out").resolve() and compiled.name=="public_records.rs" and
        compiled.resolve()==out.resolve()/"public_records.rs" and rootout.resolve()==(out.parent/"root-output").resolve() and
        artifacts["cargo-root-output.txt"]==str(out).encode(),
        "AX Cargo target/fingerprint/output/OUT_DIR artifact paths do not join")
    require(artifacts["cargo-build-output.txt"]==("\n".join(BUILD_DIRECTIVES)+"\n").encode(),
        "AX captured Cargo build directives changed")
    sm=json.loads(source_map)
    require(artifacts["public_records.rs"]==expected and receipt.get("compiled_input_sha256")==sha(expected) and
        receipt.get("reconstructed_generated_sha256")==sha(expected) and receipt.get("source_map_sha256")==sha(source_map) and
        sm.get("generated/public_records.rs",{}).get("sha256")==sha(expected) and
        receipt.get("production_rerun_input_sha256")==input_hashes,
        "AX captured OUT_DIR public records do not reconstruct from pinned production sources")
    return {"status":"pass","captured_artifact_count":4,"fingerprint_inputs_exact":True,
        "build_output_exact":True,"OUT_DIR_join_exact":True,"public_records_reconstructed":True}


def audit_capture(*, live_capture:bool=False)->dict[str,Any]:
    source_map=read_file(ROOT/"generated/source-map.json","AX extractor source map")
    if live_capture:
        return cargo_artifacts_live(source_map,capture=True)
    receipt_bytes=read_file(CAPTURE_DIR/"public-records-build-receipt.json","AX captured Cargo receipt")
    receipt=json.loads(receipt_bytes)
    artifacts={name:read_file(CAPTURE_DIR/name,f"AX captured artifact {name}") for name in ARTIFACT_FIELDS}
    _checker,expected,input_hashes=expected_public_records()
    return audit_captured_artifacts(receipt,artifacts,source_map,expected,input_hashes)


def audit(*, shadow:pathlib.Path|None=None,mapping_path:pathlib.Path|None=None,
          capture_compiled:bool=False,captured_only:bool=False,ancestry_only:bool=False)->dict[str,Any]:
    require(shadow is None or shadow.resolve()==(ROOT/"src/promotion.rs").resolve(),
            "shadow must resolve to the selected src/promotion.rs module")
    require(mapping_path is None or mapping_path.resolve()==(ROOT/"generated/mapping.json").resolve(),
            "mapping must resolve to the selected generated/mapping.json")
    av_info=published_av()
    if ancestry_only:
        return {"status":"pass","checker_scope":"published AV source/native/Cargo/proof ancestry",
            "AV_archive_sha256":AV_ARCHIVE_SHA,"AV_receipt_sha256":AV_RECEIPT_SHA}
    mapping=json.loads(read_file(ROOT/"generated/mapping.json","AX source/native mapping"))
    source=audit_source_and_native(mapping)
    targets=target_inventory(mapping)
    if capture_compiled:
        cargo=audit_capture(live_capture=True)
    elif captured_only:
        cargo=audit_capture(live_capture=False)
    else:
        cargo=audit_capture(live_capture=False)
    return {"status":"pass","checker_scope":"AX nonempty owned Box to raw suffix advance/read/ordinary physical Drop; no whole-crate or unwind claim",
        "full_original_admitted":False,"published_AV":{"archive_sha256":AV_ARCHIVE_SHA,"receipt_sha256":AV_RECEIPT_SHA,
            "source_and_proof_replayed":True},"AX_source_and_native":source,"AX_target_inventory":targets,
        "AX_compiled_production_input":cargo,
        "claims":{"input":"nonempty Box<[u8]>; amount<=len includes one-past empty suffix while allocation capacity stays positive",
            "route":"From<Box> detaches one raw owner, advance subtracts len/adds pointer offset, chunk borrows current view, saved Vec precedes normal Drop",
            "free":"normal raw Drop calls the stored vtable once; physical receipt identifies the original allocation base/capacity",
            "exclusions":["empty Box/static route","first Clone/promotion","CAS loser","arbitrary representations","unwind","whole crate"]}}


def main()->int:
    parser=argparse.ArgumentParser(description="AX raw suffix Drop source/native/Cargo correspondence gate")
    parser.add_argument("--shadow",type=pathlib.Path,default=ROOT/"src/promotion.rs")
    parser.add_argument("--mapping",type=pathlib.Path,default=ROOT/"generated/mapping.json")
    parser.add_argument("--ancestry-only",action="store_true")
    parser.add_argument("--capture-compiled-inputs",action="store_true")
    parser.add_argument("--audit-compiled-capture-only",action="store_true")
    parser.add_argument("--output",type=pathlib.Path)
    args=parser.parse_args()
    try:
        report=audit(shadow=args.shadow,mapping_path=args.mapping,capture_compiled=args.capture_compiled_inputs,
            captured_only=args.audit_compiled_capture_only,ancestry_only=args.ancestry_only)
    except Exception as exc:
        report={"status":"reject","checker_scope":"AX selected raw suffix source/native/Cargo correspondence",
            "reason":f"{type(exc).__name__}: {exc}"}
    rendered=json.dumps(report,indent=2,ensure_ascii=False)+"\n"
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(rendered)
    print(rendered,end="")
    return 0 if report.get("status")=="pass" else 1


if __name__=="__main__":
    raise SystemExit(main())
