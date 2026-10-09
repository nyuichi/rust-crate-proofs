#!/usr/bin/env python3
"""Independent AM source/native correspondence gate.

The checker binds the frozen AL ownership core, the generated AM terminal
adapters/client, the selected module route, the actual default-native source
and MIR, and the active Cargo OUT_DIR record. It does not prove Creusot/Why3
obligations or generic MIR/Drop correspondence.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import shutil
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
AL_ROOT = ROOT.parent / "original-promotable-first-clone-2026-10-09"
AL_CHECKER = AL_ROOT / "check_correspondence.py"
AL_CHECKER_SHA256 = "9d876d21f712441d71cc309411a853874c852c139219363b0c71a621da480867"
AL_NATIVE_CHECKER_SHA256 = "91adb92c5ed5ff27f5c6169ff770ee01cf368febf1dd5b38b5371167cd446d88"
FROZEN_MANIFEST_SHA256 = "b70873f3fd5d779be2a3b04fae5f745260c078c4dd2d096840575d8687f1cd06"
FROZEN_MANIFEST = ROOT / "frozen-al-source.json"
ACTIVE_PATH = ROOT / "generated/active.rs"
HELPER_PATH = ROOT / "generated/terminal-helper.rs"
CLIENT_PATH = ROOT / "generated/elaborated-client.rs"
MAPPING_PATH = ROOT / "generated/mapping.json"
NATIVE_CHECKER_PATH = ROOT / "check_native.py"
REVIEWED_PRODUCTION_MANIFEST_SHA256 = "e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306"


class CheckError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_module(name: str, path: pathlib.Path):
    require(path.is_file(), f"required checker input is missing: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot import checker: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


AL = load_module("am_frozen_al_correspondence", AL_CHECKER)
AI = AL.AI


def assert_frozen_al_sources(source_overrides: dict[str, bytes] | None = None) -> dict[str, Any]:
    manifest_bytes = FROZEN_MANIFEST.read_bytes()
    require(sha(manifest_bytes) == FROZEN_MANIFEST_SHA256,
            "frozen AL source manifest changed")
    manifest = json.loads(manifest_bytes)
    require(manifest.get("base_commit") == "361c7cd261507ac0a705b3b836f73240070891c6" and
            manifest.get("base_probe") == "../original-promotable-first-clone-2026-10-09",
            "frozen AL provenance identity changed")
    files = manifest.get("rust_modules")
    require(isinstance(files, dict) and set(files) == {
        "provenance_specs.rs", "free_effect.rs", "lifecycle.rs", "erased_call.rs",
        "physical_projection.rs", "public_shared.rs", "promotion.rs", "field_event.rs",
        "event.rs", "lib.rs", "owned_pointer.rs", "native_client.rs",
    }, "frozen AL Rust-module inventory changed")
    checked = {}
    source_overrides = source_overrides or {}
    for name, expected_hash in files.items():
        old_path = AL_ROOT / "src" / name
        new_path = ROOT / "src" / name
        require(old_path.is_file() and new_path.is_file(), f"frozen AL source missing: {name}")
        old_bytes = old_path.read_bytes()
        new_bytes = source_overrides[name] if name in source_overrides else new_path.read_bytes()
        require(sha(old_bytes) == expected_hash,
                f"published AL source no longer matches its frozen identity: {name}")
        if name != "lib.rs":
            require(new_bytes == old_bytes and sha(new_bytes) == expected_hash,
                    f"AM changed frozen AL proof/support source: {name}")
        checked[name] = expected_hash
    require(sha(AL_CHECKER.read_bytes()) == AL_CHECKER_SHA256,
            "reused published AL correspondence checker changed")
    require(sha((AL_ROOT / "check_native.py").read_bytes()) == AL_NATIVE_CHECKER_SHA256,
            "reused published AL native checker changed")
    return {"base_commit": manifest["base_commit"], "byte_identical_modules": len(files) - 1,
            "lib_route_change_separately_checked": True, "module_sha256": checked}


def assert_cargo_routes(probe_text: str, production_text: str) -> dict[str, Any]:
    try:
        probe = tomllib.loads(probe_text)
        production = tomllib.loads(production_text)
    except tomllib.TOMLDecodeError as exc:
        raise CheckError(f"Cargo manifest is invalid: {exc}") from exc
    package = probe.get("package", {})
    require(package == {"name": "bytes-original-promotable-automatic-drop", "version": "0.1.0",
                       "edition": "2021", "publish": False},
            "AM Cargo package identity/options changed")
    require(probe.get("dependencies") == {"creusot-std": "=0.13.0"} and
            probe.get("workspace") == {} and "build" not in package and "lib" not in probe,
            "AM Cargo target, dependency, or build route changed")
    require(probe.get("features") == {
        "negative_missing_acquire": [], "negative_missing_payload_free": [],
        "negative_missing_control_free": [],
    }, "AM Cargo feature surface changed")
    prod_package = production.get("package", {})
    prod_lib = production.get("lib", {})
    require(prod_package.get("name") == "bytes" and prod_package.get("version") == "1.11.1" and
            prod_package.get("build") is False and prod_lib.get("path") == "src/lib.rs",
            "actual production Cargo route changed")
    return {"probe_package": package["name"], "probe_lib_entry": "Cargo default src/lib.rs",
            "probe_build_entry": "Cargo default build.rs", "production_lib_entry": "src/lib.rs"}


def assert_promotion_module_route(lib: str, active: str) -> dict[str, Any]:
    old_lib = (AL_ROOT / "src/lib.rs").read_text()
    old_route = "#[cfg(creusot)] mod promotion;"
    selected_route = '#[cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;'
    require(old_lib.count(old_route) == 1,
            "published AL lib.rs no longer has its unique selected promotion declaration")
    expected_lib = old_lib.replace(old_route, selected_route, 1)
    require(lib == expected_lib,
            "AM lib.rs may change only the selected promotion module path from the frozen AL source")
    route_pattern = re.compile(
        r"#\s*\[\s*cfg\s*\(\s*creusot\s*\)\s*\]\s*"
        r"#\s*\[\s*path\s*=\s*\"\.\./generated/active\.rs\"\s*\]\s*"
        r"mod\s+promotion\s*;"
    )
    route_matches = list(route_pattern.finditer(lib))
    require(len(route_matches) == 1,
            "promotion must have exactly the reviewed cfg and generated active-file route")
    require(AI.rust_path_for_module(lib, "promotion") == "../generated/active.rs",
            "promotion path literal changed")
    lib_tokens = AI.rust_tokens(lib)
    declarations = []
    wanted_mod = ["mod", "promotion", ";"]
    for i in range(len(lib_tokens) - 2):
        if lib_tokens[i:i + 3] != wanted_mod:
            continue
        attrs = []
        cursor = i
        while cursor >= 2 and lib_tokens[cursor - 1] == "]":
            depth = 0
            opening = None
            for j in range(cursor - 1, -1, -1):
                if lib_tokens[j] == "]":
                    depth += 1
                elif lib_tokens[j] == "[":
                    depth -= 1
                    if depth == 0:
                        opening = j
                        break
            require(opening is not None and opening > 0 and lib_tokens[opening - 1] == "#",
                    "cannot parse promotion module outer attributes")
            attrs.insert(0, lib_tokens[opening - 1:cursor])
            cursor = opening - 1
        declarations.append(attrs)
    expected_attrs = [AI.rust_tokens("#[cfg(creusot)]"),
                      AI.rust_tokens('#[path="../generated/active.rs"]')]
    require(declarations == [expected_attrs],
            "promotion module declaration has an alternate or extra outer attribute")
    require(len(re.findall(r"\bmod\s+promotion\b", AI.mask_noncode(lib))) == 1,
            "promotion module has a duplicate, inline, or alternate declaration")
    resolved = (ROOT / "src" / "../generated/active.rs").resolve()
    require(resolved == ACTIVE_PATH.resolve() and resolved.is_file(),
            "selected promotion route does not resolve to generated/active.rs")
    # Reuse AL's exact support-module route audit after replacing only the
    # independently pinned selected-module path attribute with its equivalent
    # cfg-only declaration for that legacy checker.
    match = route_matches[0]
    normalized = lib[:match.start()] + "#[cfg(creusot)] mod promotion;" + lib[match.end():]
    AL.assert_routes(normalized, (AL_ROOT / "src/promotion.rs").read_text())
    AL.assert_selected_include_routes(active, ACTIVE_PATH, "generated/active.rs")
    return {"promotion_cfg": "creusot", "promotion_path_literal": "../generated/active.rs",
            "resolved_selected_source": str(resolved), "support_module_routes_pinned": True,
            "active_record_include_literals_pinned": True}


def _expected_helper_tokens(base: str) -> list[str]:
    expected: list[str] = []
    for cleanup, helper, signature, body in (
        ("cleanup_child", "bytes_child_terminal_drop",
         "fn bytes_child_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) ",
         "{cleanup_child(value,scope,output)}"),
        ("cleanup_root", "bytes_root_terminal_drop",
         "fn bytes_root_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) ",
         "{cleanup_root(value,scope,output)}"),
    ):
        for attribute in AI.function_outer_attributes(base, cleanup):
            expected.extend(attribute)
        expected.extend(AI.rust_tokens(signature))
        expected.extend(AI.rust_tokens(body))
    return expected


EXPECTED_CLIENT_ATTRIBUTES = [
    AI.rust_tokens("#[requires(input@.len()>0)]"),
    AI.rust_tokens("#[ensures(result@==input@)]"),
]
EXPECTED_CLIENT_BODY = r"""{
    let (original,mut scope)=from_box_scoped(input);
    let child=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(child.child_id());
    let live_before=snapshot!((*scope.observation()).0);
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(child,scope.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && !child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_before).remove(*child_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!(!(*scope.observation()).0.contains(*child_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}"""
EXPECTED_COMPLETE_CLIENT_SOURCE = r"""#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_automatic_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let child=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(child.child_id());
    let live_before=snapshot!((*scope.observation()).0);
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(child,scope.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && !child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_before).remove(*child_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!(!(*scope.observation()).0.contains(*child_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}"""


def assert_active_composition(base: str, helper: str, client: str, active: str) -> dict[str, Any]:
    expected_helper_names = ["bytes_child_terminal_drop", "bytes_root_terminal_drop"]
    helper_names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", AI.mask_noncode(helper))
    require(helper_names == expected_helper_names, "terminal helper file has an open function surface")
    require(AI.rust_tokens(helper) == _expected_helper_tokens(base),
            "terminal wrappers must exactly reuse the checked cleanup contracts and single forward calls")
    for helper_name, cleanup, signature, body in (
        ("bytes_child_terminal_drop", "cleanup_child",
         "fn bytes_child_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>)",
         "{cleanup_child(value,scope,output)}"),
        ("bytes_root_terminal_drop", "cleanup_root",
         "fn bytes_root_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>)",
         "{cleanup_root(value,scope,output)}"),
    ):
        require(AI.function_signature(helper, helper_name) == AI.rust_tokens(signature),
                f"{helper_name} signature changed")
        require(AI.function_outer_attributes(helper, helper_name) == AI.function_outer_attributes(base, cleanup),
                f"{helper_name} must use the exact body-proved cleanup contract")
        got, _, _ = AI.find_function(helper, helper_name)
        require(got == AI.body_from_braced_snippet(body), f"{helper_name} is not a single ordinary forward call")

    client_names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", AI.mask_noncode(client))
    require(client_names == ["promoted_automatic_scope"], "elaborated client has an open function surface")
    require(AI.rust_tokens(client) == AI.rust_tokens(EXPECTED_COMPLETE_CLIENT_SOURCE),
            "elaborated client token stream must match the complete reviewed source with no added macro or item")
    require(AI.function_signature(client, "promoted_automatic_scope") == AI.rust_tokens(
        "fn promoted_automatic_scope(input:Box<[u8]>)->Vec<u8>"),
        "automatic-drop proof client signature changed")
    require(AI.function_outer_attributes(client, "promoted_automatic_scope") == EXPECTED_CLIENT_ATTRIBUTES,
            "automatic-drop proof client admission or result contract changed")
    got_body, _, _ = AI.find_function(client, "promoted_automatic_scope")
    require(got_body == AI.body_from_braced_snippet(EXPECTED_CLIENT_BODY),
            "proof client event, snapshot, borrowed-read, return, or drop order changed")
    require(not any(tok in AI.rust_tokens(helper + client) for tok in
                    ("trusted", "assume", "axiom", "extern_spec", "externspec", "checktrusted")),
            "generated terminal tail added an unreviewed proof escape")
    require(not re.search(r"\bimpl\s+Drop\s+for\s+Bytes\b", AI.mask_noncode(helper + client)),
            "proof shadow may not invent an implicit Bytes destructor")
    require(active == base + "\n" + helper + client,
            "selected generated active module is not the exact frozen core plus checked tail files")
    return {"frozen_AL_prefix_byte_exact": True, "child_and_root_forward_adapters_exact": True,
            "client_attributes_and_complete_body_exact": True, "ticket_IDs_derived_from_live_cursor": True,
            "child_kept_alive_before_parent_read": True, "saved_return_precedes_root_drop": True,
            "terminal_tail_has_no_added_trust_or_Bytes_Drop": True}


def derive_native_edges(client_mir: str, native_checker: Any) -> tuple[list[dict[str, str]], dict[str, str], dict[str, str]]:
    debug_places = dict(re.findall(r"(?m)^\s*debug\s+(\w+)\s*=>\s*(_\d+)\s*;", client_mir))
    require(debug_places.get("child") is not None and debug_places.get("original") is not None,
            "native MIR lacks the selected child/original local bindings")
    blocks = native_checker.mir_blocks(client_mir, "client")
    edges: list[dict[str, str]] = []
    for (block_num, cleanup), body in sorted(blocks.items()):
        if cleanup:
            continue
        for place, successor, unwind in re.findall(
                r"drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?[ ]*([^\]]+)\]", body):
            owner = next((name for name in ("child", "original") if debug_places.get(name) == place), None)
            if owner is not None:
                edges.append({"block": f"bb{block_num}", "place": place, "owner": owner,
                              "successor": successor, "unwind": unwind.strip()})
    require([edge["owner"] for edge in edges] == ["child", "original"],
            "native normal MIR must have exactly child Drop then root Drop")
    return_evals = []
    for (block_num, cleanup), body in blocks.items():
        if cleanup:
            continue
        for statement in re.findall(r"(?m)^\s*(_0\s*=\s*move\s+_\d+;)\s*$", body):
            return_evals.append({"block": f"bb{block_num}", "statement": statement})
    require(len(return_evals) == 1, "native MIR must evaluate exactly one saved return value")
    return edges, debug_places, return_evals[0]


def assert_mapping_data(mapping: dict[str, Any], base: str, helper: str, client: str,
                        active: str, capture: dict[str, Any], client_mir: str,
                        native_checker: Any) -> dict[str, Any]:
    expected_keys = {
        "feature", "status", "base_source", "base_source_sha256", "active", "active_sha256",
        "helper_sha256", "client_sha256", "helpers", "helper_interpretation", "stage",
        "return_evaluation", "excluded", "tcb", "native_source", "native_source_sha256",
        "native_client_mir", "native_mir_ready", "debug_places", "normal_edges", "mir",
    }
    require(set(mapping) == expected_keys, "mapping receipt field surface changed")
    require(mapping["feature"] == "" and mapping["status"] == "generated_unchecked" and
            mapping["stage"] == "after-ElaborateDrops",
            "mapping must identify the positive default feature and selected elaboration stage")
    require(mapping["base_source"] == "src/promotion.rs" and
            mapping["base_source_sha256"] == sha((ROOT / "src/promotion.rs").read_bytes()) and
            mapping["active"] == "generated/active.rs" and mapping["active_sha256"] == sha(active.encode()),
            "mapping source path/hash fields differ from selected live proof sources")
    require(mapping["helper_sha256"] == sha(helper.encode()) and mapping["client_sha256"] == sha(client.encode()),
            "mapping helper/client source hashes differ from checked files")
    require(mapping["helpers"] == ["bytes_child_terminal_drop", "bytes_root_terminal_drop"] and
            mapping["helper_interpretation"] ==
            "ordinary body-proved consuming forward calls, no trusted Bytes Drop effect",
            "mapping helper identity or stated interpretation changed")
    require(mapping["excluded"] == ["unwind completion", "concurrent first-promotion loser",
            "arbitrary Drop/move equivalence", "whole crate"],
            "mapping exclusions changed")
    require(mapping["tcb"] == ["native compiler/MIR and normal terminal-place elaboration",
            "receiver/data-field address nonobservation and no independent field-drop glue",
            "AL generic pointer/field/physical/erased-callback boundaries"],
            "mapping TCB disclosure changed")
    edges, debug_places, return_evaluation = derive_native_edges(client_mir, native_checker)
    require(mapping["native_source"] == "native.rs" and
            mapping["native_source_sha256"] == capture.get("native_source_sha256"),
            "mapping native source binding differs from the independently checked capture")
    client_mir_path = "native-mir/" + native_checker.MIR_FILES["client"]
    require(mapping["native_client_mir"] == client_mir_path and mapping["native_mir_ready"] is True,
            "mapping selected MIR route/ready flag changed")
    require(mapping["debug_places"] == debug_places, "mapping debug place identities were not rederived")
    expected_edges = [dict(edge, scope="scope", output=("child_receipt" if edge["owner"] == "child" else "root_receipt"))
                      for edge in edges]
    require(mapping["normal_edges"] == expected_edges,
            "mapping drop places/order/successors or consuming proof channels differ from native MIR/client")
    expected_return = {"shadow": "let saved_return=observed", "root_effect_after_evaluation": True,
                       "child_effect_before_root_read": True}
    require(mapping["return_evaluation"] == expected_return and
            return_evaluation == {"block": "bb5", "statement": "_0 = move _7;"},
            "mapping return evaluation differs from actual MIR or checked proof client")
    selected = capture.get("selected")
    require(isinstance(selected, list) and len(selected) == 19,
            "native capture must enumerate all 19 selected MIR inputs")
    expected_mir = []
    for row in selected:
        require(set(row) == {"label", "path", "sha256"} and row["path"].startswith("native-mir/"),
                "native capture selected MIR row has an unexpected shape or route")
        path = (ROOT / row["path"]).resolve()
        require(path.parent == (ROOT / "native-mir").resolve() and path.is_file() and
                sha(path.read_bytes()) == row["sha256"],
                f"captured MIR bytes differ from source-selected hash: {row['path']}")
        expected_mir.append({"path": row["path"], "sha256": row["sha256"]})
    expected_mir.sort(key=lambda row: row["path"])
    require(mapping["mir"] == expected_mir, "mapping MIR list differs from captured native source files")
    return {"mapping_fields_closed": True, "native_drop_edges_rederived": expected_edges,
            "return_evaluation_rederived": return_evaluation, "selected_MIR_inputs": len(expected_mir)}


def assert_reviewed_native_production(native_result: dict[str, Any]) -> dict[str, Any]:
    reviewed = native_result.get("reviewed_production_inputs")
    require(isinstance(reviewed, dict) and set(reviewed) == {
        "manifest_sha256", "base_commit", "source_files", "manifest_and_lock_pinned",
        "global_import_macro_and_include_bindings_frozen",
    }, "native audit omitted or changed its reviewed production-input summary")
    require(reviewed == {
        "manifest_sha256": REVIEWED_PRODUCTION_MANIFEST_SHA256,
        "base_commit": "361c7cd261507ac0a705b3b836f73240070891c6",
        "source_files": 61,
        "manifest_and_lock_pinned": True,
        "global_import_macro_and_include_bindings_frozen": True,
    }, "native audit did not bind the complete reviewed production source, manifest, lock, import, and include set")
    return {"status": "pass", **reviewed}


def assert_am_compiled_records(native_bundle: dict[str, Any]) -> dict[str, Any]:
    probe_manifest_path = ROOT / "Cargo.toml"
    production_manifest_path = CRATE_ROOT / "Cargo.toml"
    cargo_routes = assert_cargo_routes(probe_manifest_path.read_text(), production_manifest_path.read_text())
    build_path, extractor_path = ROOT / "build.rs", ROOT / "extract_public.py"
    build_bytes, extractor_bytes = build_path.read_bytes(), extractor_path.read_bytes()
    rerun_paths = AL.assert_build_script_surface(build_bytes)
    AL.assert_extractor_source_surface(extractor_bytes)
    expected_inputs = {
        "../../../src/bytes.rs": CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": extractor_path,
    }
    require(rerun_paths == list(expected_inputs), "AM build rerun input list changed")
    input_hashes = {}
    for literal, expected in expected_inputs.items():
        resolved = (ROOT / literal).resolve()
        require(resolved == expected.resolve() and resolved.is_file(),
                f"build rerun literal resolves away from actual production input: {literal}")
        input_hashes[literal] = sha(resolved.read_bytes())

    generated_path = ROOT / "generated/public_records.rs"
    source_map_path = ROOT / "generated/source-map.json"
    require(generated_path.is_file() and source_map_path.is_file(),
            "AM generated public record or source map is missing")
    # Independently replay source extraction against AM's actual generated
    # files. This uses the reviewed parser only and does not rewrite the AL
    # evidence tree as a side effect.
    AI.audit_generated_extractions(
        native_bundle["production_source"], ROOT / "generated",
        (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text(),
        (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text(),
        (CRATE_ROOT / "src/bytes_mut.rs").read_text(),
    )
    bytes_record_source = (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text()
    vtable_record_source = (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text()
    bytes_mut_source = (CRATE_ROOT / "src/bytes_mut.rs").read_text()
    mutable_shared = AI.extract_struct_source(bytes_mut_source, "struct Shared {", "BytesMut::Shared")
    mutable_record = AI.extract_struct_source(bytes_mut_source, "pub struct BytesMut {", "BytesMut")
    expected_record = (
        bytes_record_source + "\n" + vtable_record_source + "\n" +
        "mod mutable_record {\nuse alloc::vec::Vec;\n"
        "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        mutable_shared + "\n" + mutable_record + "\n}\nuse mutable_record::BytesMut;\n"
    ).encode()
    generated_bytes = generated_path.read_bytes()
    require(generated_bytes == expected_record,
            "AM generated public_records.rs differs byte-for-byte from independent production reconstruction")
    try:
        source_map = json.loads(source_map_path.read_text())
    except (json.JSONDecodeError, OSError) as exc:
        raise CheckError(f"AM public record source map cannot be read: {exc}") from exc
    require(source_map.get("generated/public_records.rs", {}).get("sha256") == sha(generated_bytes),
            "AM source map does not bind its public record bytes")

    target_env = os.environ.get("CARGO_TARGET_DIR")
    target_dir = pathlib.Path(target_env).resolve() if target_env else (ROOT.parents[5] / "bytes-proof-tools/targets/bytes").resolve()
    require(target_dir.is_dir(), f"active Cargo target directory is missing: {target_dir}")
    fingerprints = list((target_dir / "debug/.fingerprint").glob(
        "bytes-original-promotable-automatic-drop-*/run-build-script-build-script-build.json"))
    require(len(fingerprints) == 1, f"expected unique AM Cargo build fingerprint, found {len(fingerprints)}")
    fingerprint_path = fingerprints[0]
    fingerprint_bytes = fingerprint_path.read_bytes()
    try:
        fingerprint = json.loads(fingerprint_bytes)
    except (json.JSONDecodeError, OSError) as exc:
        raise CheckError(f"AM Cargo build fingerprint is invalid: {exc}") from exc
    run_records = [entry["RerunIfChanged"] for entry in fingerprint.get("local", [])
                   if isinstance(entry, dict) and "RerunIfChanged" in entry]
    require(len(run_records) == 1 and run_records[0].get("paths") == rerun_paths,
            "AM Cargo fingerprint rerun inputs differ from reviewed build.rs inputs")
    output_rel = pathlib.Path(run_records[0].get("output", ""))
    require(not output_rel.is_absolute() and output_rel.parts[:2] == ("debug", "build"),
            "AM Cargo build output is outside target/debug/build")
    output_path = (target_dir / output_rel).resolve()
    output_dir = output_path.parent
    require(fingerprint_path.parent.name == output_dir.name and output_path.is_file(),
            "AM build output and Cargo fingerprint identify different build-script runs")
    output_bytes = output_path.read_bytes()
    expected_output = [
        "cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate",
        "cargo:rerun-if-changed=../../../src/bytes.rs",
        "cargo:rerun-if-changed=../../../src/bytes/bytes_record.rs",
        "cargo:rerun-if-changed=../../../src/bytes/vtable_record.rs",
        "cargo:rerun-if-changed=../../../src/bytes_mut.rs",
        "cargo:rerun-if-changed=extract_public.py",
    ]
    require(output_bytes == ("\n".join(expected_output) + "\n").encode(),
            "AM actual Cargo build-script output directives changed")
    root_output_path = output_dir / "root-output"
    require(root_output_path.is_file(), "AM Cargo root-output receipt is missing")
    root_output_bytes = root_output_path.read_bytes()
    require(root_output_bytes == str(output_dir / "out").encode(),
            "AM Cargo root-output does not identify the selected OUT_DIR")
    compiled_path = output_dir / "out/public_records.rs"
    require(compiled_path.is_file(), "AM actual compiled OUT_DIR/public_records.rs is missing")
    compiled_bytes = compiled_path.read_bytes()
    require(compiled_bytes == expected_record == generated_bytes,
            "AM compiled OUT_DIR record differs from independent AL and generated-source bytes")

    capture_dir = ROOT / "generated/compiled-inputs"
    capture_dir.mkdir(parents=True, exist_ok=True)
    artifacts = {
        "public_records.rs": compiled_bytes,
        "cargo-run-build-fingerprint.json": fingerprint_bytes,
        "cargo-build-output.txt": output_bytes,
        "cargo-root-output.txt": root_output_bytes,
    }
    for filename, data in artifacts.items():
        (capture_dir / filename).write_bytes(data)
    receipt = {
        "status": "pass", "cargo_package": cargo_routes["probe_package"],
        "probe_manifest": str(probe_manifest_path), "crate_entry": "Cargo default src/lib.rs",
        "build_script": str(build_path), "build_script_sha256": sha(build_bytes),
        "extractor": str(extractor_path), "extractor_sha256": sha(extractor_bytes),
        "production_manifest": str(production_manifest_path), "production_lib_entry": "src/lib.rs",
        "cargo_target_dir": str(target_dir), "cargo_build_fingerprint": str(fingerprint_path),
        "cargo_build_fingerprint_sha256": sha(fingerprint_bytes),
        "captured_cargo_build_fingerprint_path": "generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_cargo_build_fingerprint_sha256": sha(artifacts["cargo-run-build-fingerprint.json"]),
        "build_output_path": str(output_path), "build_output_sha256": sha(output_bytes),
        "captured_build_output_path": "generated/compiled-inputs/cargo-build-output.txt",
        "captured_build_output_sha256": sha(artifacts["cargo-build-output.txt"]),
        "root_output_path": str(root_output_path), "root_output_sha256": sha(root_output_bytes),
        "captured_root_output_path": "generated/compiled-inputs/cargo-root-output.txt",
        "captured_root_output_sha256": sha(artifacts["cargo-root-output.txt"]),
        "actual_out_dir": str(compiled_path.parent), "compiled_input_path": str(compiled_path),
        "compiled_input_sha256": sha(compiled_bytes),
        "captured_input_path": "generated/compiled-inputs/public_records.rs",
        "captured_input_sha256": sha(artifacts["public_records.rs"]),
        "generated_record_sha256": sha(generated_bytes),
        "source_map_sha256": sha(source_map_path.read_bytes()),
        "production_rerun_input_sha256": input_hashes,
        "compiled_input_equal_reconstructed_and_generated": True,
    }
    (capture_dir / "public-records-build-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return {"cargo_routes": cargo_routes, "rerun_inputs_resolve_to_production": True,
            "actual_OUT_DIR_record_matches_reconstruction": True, "compiled_input_receipt": receipt}


def audit(shadow_path: pathlib.Path) -> dict[str, Any]:
    require(shadow_path.resolve() == ACTIVE_PATH.resolve(),
            f"selected shadow must be the active generated module {ACTIVE_PATH}")
    frozen = assert_frozen_al_sources()
    base_path = ROOT / "src/promotion.rs"
    base = base_path.read_text()
    require(base.encode() == (AL_ROOT / "src/promotion.rs").read_bytes(),
            "selected AL promotion core is no longer byte-identical to published AL")
    active = ACTIVE_PATH.read_text()
    helper = HELPER_PATH.read_text()
    client = CLIENT_PATH.read_text()
    lib = (ROOT / "src/lib.rs").read_text()
    routes = assert_promotion_module_route(lib, active)
    composition = assert_active_composition(base, helper, client, active)

    # Re-run the published, fixed production/source correspondence on its
    # original selected AL client; it remains byte-exact and independently
    # binds actual Bytes construction/clone/read/drop callbacks and adapters.
    al_bundle = AL.NATIVE.load_bundle()
    al_native = AL.NATIVE.audit_bundle(al_bundle)
    am_route = '#[cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;'
    al_route = "#[cfg(creusot)] mod promotion;"
    require(lib.count(am_route) == 1, "AM selected promotion route changed before AL support audit")
    core_routes_lib = lib.replace(am_route, al_route, 1)
    route_and_adapters = AL.assert_support_adapters(core_routes_lib, base, al_bundle)
    core_operation_mapping = AL.assert_operation_mapping(base, al_native, al_bundle)
    core_summary = {
        "reviewed_token_surfaces": AL.assert_reviewed_token_surfaces(),
        "proof_state": AL.assert_proof_state_types(base),
        "trust_boundary": AL.assert_trust_boundary(base),
        "closed_function_surface": True,
        "support_adapters": route_and_adapters,
        "operation_mapping": core_operation_mapping,
    }
    AL.assert_closed_function_surface(base)

    native_checker = load_module("am_automatic_drop_native_checker", NATIVE_CHECKER_PATH)
    native_bundle = native_checker.load_bundle()
    native_result = native_checker.audit_bundle(native_bundle)
    require(native_result.get("status") == "pass", "AM native source/MIR audit did not pass")
    reviewed_native_inputs = assert_reviewed_native_production(native_result)
    require(native_result.get("production_source", {}).get("constructor_clone_cleanup_read_source_exact") is True,
            "AM native checker did not admit the exact production Bytes call chain")
    require(native_result.get("terminal_field_profile", {}).get("compile_time_no_independent_field_drop_glue") is True,
            "AM native checker did not establish the selected default Bytes fields have no extra drop glue")
    require(native_result.get("native_mir", {}).get("normal_terminal_edges_checked") is True or
            native_result.get("client", {}).get("normal_implicit_handle_drop") is True,
            "AM native checker did not establish the automatic-drop client path")
    native_client_mir = native_bundle["mir_sources"]["client"]
    capture = native_bundle["capture"]
    mapping_facts = assert_mapping_data(json.loads(MAPPING_PATH.read_text()), base, helper, client,
                                        active, capture, native_client_mir, native_checker)
    compiled = assert_am_compiled_records(native_bundle)
    return {
        "status": "pass",
        "checker_scope": "AM first-promotion proof shadow mapped to one closed default-native normal-return automatic-Drop witness",
        "frozen_AL": frozen,
        "selected_module_route": routes,
        "generated_shadow": composition,
        "AL_core_audit": core_summary,
        "native_source_and_MIR_audit": native_result,
        "reviewed_native_production_inputs": reviewed_native_inputs,
        "mapping_reconstruction": mapping_facts,
        "compiled_input_routes": compiled,
        "limitations": [
            "this checker binds sources, generated mapping, native MIR and compiled inputs; it does not prove Creusot/Why3 obligations",
            "normal terminal-place Drop equivalence, receiver/data address nonobservation, field-drop absence, callback erasure, compiler/MIR adequacy, allocator and pointer provenance remain stated TCB",
            "unwind behavior, CAS loser behavior, arbitrary Drop/move equivalence, concurrency and the whole crate are excluded",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", "--active", dest="shadow", type=pathlib.Path, default=ACTIVE_PATH)
    parser.add_argument("--mapping", type=pathlib.Path, default=MAPPING_PATH)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        require(args.mapping.resolve() == MAPPING_PATH.resolve(),
                "mapping override is not admitted; correspondence derives from the selected generated route")
        result = audit(args.shadow)
    except Exception as exc:
        result = {"status": "reject", "checker_scope": "AM automatic-Drop source/native correspondence",
                  "reason": str(exc)}
        rendered = json.dumps(result, indent=2) + "\n"
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(rendered)
        print(rendered, end="")
        return 1
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
