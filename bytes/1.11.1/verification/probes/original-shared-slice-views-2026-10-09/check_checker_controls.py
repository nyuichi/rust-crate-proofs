#!/usr/bin/env python3
"""Replay AQ correspondence-checker mutations using in-memory source values only."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import pathlib
import sys
from typing import Any, Callable

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_correspondence.py"
MANIFEST_PATH = ROOT / "fixtures/checker-controls.json"
DEFAULT_RECEIPT = ROOT / "generated/checker-controls-receipt.json"


def load_module(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import checker module {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


C = load_module("aq_correspondence_controls_target", CHECKER_PATH)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one mutation anchor, found {count}: {old!r}")
    return source.replace(old, new, 1)


def replace_bytes_once(source: bytes, old: bytes, new: bytes, label: str) -> bytes:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one byte mutation anchor, found {count}: {old!r}")
    return source.replace(old, new, 1)


def rejected(control_id: str, action: Callable[[], Any]) -> dict[str, str]:
    try:
        result = action()
    except C.CheckError as exc:
        return {"id": control_id, "status": "rejected_as_expected", "reason": str(exc)}
    except Exception as exc:
        raise RuntimeError(f"{control_id}: checker crashed instead of rejecting: {type(exc).__name__}: {exc}") from exc
    raise RuntimeError(f"{control_id}: checker accepted mutation (result={result!r})")


def run_controls() -> dict[str, Any]:
    manifest = json.loads(MANIFEST_PATH.read_text())
    controls = manifest.get("controls")
    if manifest.get("version") != 1 or not isinstance(controls, list):
        raise RuntimeError("AQ checker-control manifest version/shape is invalid")
    expected_ids = [row.get("id") for row in controls]
    if not expected_ids or any(not isinstance(item, str) for item in expected_ids) or len(set(expected_ids)) != len(expected_ids):
        raise RuntimeError("AQ checker-control IDs must be unique nonempty strings")

    # Establish the actual positive input baseline before mutating in-memory
    # values. This also snapshots the four real Cargo inputs through audit().
    baseline = C.audit()
    if baseline.get("status") != "pass":
        raise RuntimeError(f"AQ current positive source/native baseline did not pass: {baseline.get('reason')}")
    prefix = (ROOT / "src/promotion.rs").read_text()
    parent = (C.AP_ROOT / "generated/positive.rs").read_text()
    extension = (ROOT / "src/slice_extension.rs").read_text()
    pointer = (ROOT / "src/view_pointer.rs").read_text()
    generator = (ROOT / "elaborate.py").read_bytes()
    client = (ROOT / "generated/elaborated-client.rs").read_text()
    active = (ROOT / "generated/active.rs").read_text()
    lib = (ROOT / "src/lib.rs").read_text()
    mapping = json.loads((ROOT / "generated/mapping.json").read_text())
    production_source = (C.CRATE_ROOT / "src/bytes.rs").read_text()
    source_map = json.loads((ROOT / "generated/source-map.json").read_text())
    native_bindings = (ROOT / "generated/native_view_bindings.rs").read_text()
    compiled_receipt = baseline.get("compiled_inputs", {}).get("receipt")
    native = baseline.get("native_audit")
    if not isinstance(compiled_receipt, dict) or not isinstance(native, dict):
        raise RuntimeError("AQ positive baseline omitted compiled-input/native audit objects")
    variants = tuple(name for name, _ in C.enum_variant_rows(
        C.rust_item_span(prefix, "enum", "OriginalSharedProof")[2]))
    positive_files = {"prefix": prefix, "extension": extension, "pointer": pointer,
        "client": client, "active": active, "lib": lib, "mapping": mapping,
        "generator": generator,
        "production": production_source, "source_map": source_map,
        "native_bindings": native_bindings, "receipt": compiled_receipt, "native": native}

    results: list[dict[str, str]] = []
    by_id: dict[str, Callable[[], Any]] = {}

    def enum_control(name: str, old: str, new: str) -> None:
        mutated = replace_once(prefix, old, new, name)
        by_id[name] = lambda mutated=mutated: C.assert_enum_prefix_transformation(parent, mutated, variants)

    enum_control("root_payload_changed", "Root(RootDescriptor)", "Root(ChildProof)")
    enum_control("child_payload_changed", "Child(ChildProof)", "Child(RootDescriptor)")
    enum_control("view_has_owner_ticket", "View(ChildProof,raw_vec::BoundPtr)",
                 "View(ChildProof,raw_vec::BoundPtr,lifecycle::Ticket<Payload>)")
    enum_control("empty_has_physical_authority", "Empty(EmptyViewProof)",
                 "Empty(EmptyViewProof,raw_vec::PhysicalRegion)")
    enum_control("extra_proof_variant", "Empty(EmptyViewProof)", "Empty(EmptyViewProof),Hidden(ChildProof)")
    enum_control("proof_variant_order_changed", "Root(RootDescriptor), Child(ChildProof),",
                 "Child(ChildProof), Root(RootDescriptor),")
    by_id["prefix_hidden_item"] = lambda: C.assert_enum_prefix_transformation(
        parent, prefix + "\nfn hidden_resource_getter()->lifecycle::Ticket<Payload>{unimplemented!()}\n", variants)

    lib_controls = {
        "lib_view_module_redirect": ("#[cfg(creusot)] mod view_pointer;", "#[cfg(creusot)] #[path=\"elsewhere.rs\"] mod view_pointer;"),
        "lib_extra_unreviewed_route": ("#[cfg(creusot)] mod view_pointer;", "#[cfg(creusot)] mod view_pointer;\n#[path=\"shadow.rs\"] mod promotion;"),
        "lib_wrong_cfg_selection": ("#[cfg(creusot)] mod view_pointer;", "#[cfg(not(creusot))] mod view_pointer;"),
    }
    for name, (old, new) in lib_controls.items():
        mutated = replace_once(lib, old, new, name)
        by_id[name] = lambda mutated=mutated: C.assert_lib_route(mutated, (C.AP_ROOT / "src/lib.rs").read_text())

    extension_controls = [
        ("extension_false_contract", "#[ensures(result.len==0usize", "#[ensures(false)]\n#[ensures(result.len==0usize"),
        ("extension_trusted_body_helper", "fn slice_view(", "#[trusted]\nfn slice_view("),
        ("extension_runtime_raw_call", "let len=source.len;", "unsafe { core::ptr::read(source.ptr); }\n    let len=source.len;"),
        ("extension_empty_ticket_field", "binding:Ghost<pointer_event::ReadOnlyPointer>,", "binding:Ghost<pointer_event::ReadOnlyPointer>, ticket:lifecycle::Ticket<Payload>,"),
        ("extension_empty_content_nonempty", "OriginalSharedProof::Empty(_)=>Seq::empty()", "OriginalSharedProof::Empty(_)=>Seq::singleton(1u8)"),
        ("extension_new_unreviewed_callback", "fn static_view_table()->", "fn hidden_callback(){}\n#[logic] fn static_view_table()->"),
        ("extension_wrong_native_view_dispatch", "erased_call::invoke3(native,(&source.data,source.ptr,source.len)",
         "erased_call::invoke3(shared_table().clone,(&source.data,source.ptr,source.len)"),
        ("extension_registration_false_post", "#[ensures(result.0==shared_table())]", "#[ensures(false)]\n#[ensures(result.0==shared_table())]"),
        ("extension_skip_native_slice_clone", "clone_shared_view(source,ghost!", "source.clone();\n    clone_shared_view(source,ghost!"),
        ("extension_empty_pointer_uses_live_add", "wrapping_bounded(source.ptr,view_begin,bound.borrow())", "add_live(source.ptr,view_begin,bound.borrow(),region.borrow())"),
    ]
    for name, old, new in extension_controls:
        mutated = replace_once(extension, old, new, name)
        by_id[name] = lambda mutated=mutated: C.assert_view_extension(mutated, pointer, client)

    pointer_controls = [
        ("pointer_add_swapped_for_wrapping", "pointer.add(count)", "pointer.wrapping_add(count)"),
        ("pointer_empty_offset_swapped", "pointer.wrapping_add(count)", "pointer.add(count)"),
        ("pointer_provenance_removed_by_cast", "null::<u8>().wrapping_add(pointer as usize)", "pointer as *mut u8"),
        ("pointer_null_reifier_changed", "core::ptr::null_mut()", "core::ptr::dangling_mut::<()>()"),
        ("pointer_null_logic_unbound", "result==null_word() && result.is_null_logic()", "result==null_word()"),
        ("pointer_added_trusted_surface", "pub fn shifted(", "#[trusted]\npub fn shifted("),
    ]
    for name, old, new in pointer_controls:
        mutated = replace_once(pointer, old, new, name)
        by_id[name] = lambda mutated=mutated: C.assert_view_extension(extension, mutated, client)

    client_controls = [
        ("client_assert_macro_override", "/// All valid concrete ranges", "macro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n/// All valid concrete ranges"),
        ("client_first_slice_omitted", "let first=slice_view(&owner,a..b,detached.borrow_mut());", "let first=owner;"),
        ("client_second_slice_omitted", "let selected=slice_view(&first,c..d,detached.borrow_mut());", "let selected=first;"),
        ("client_extra_unknown_callback", "let selected=slice_view(&first,c..d,detached.borrow_mut());", "unknown_callback();\n    let selected=slice_view(&first,c..d,detached.borrow_mut());"),
        ("client_drop_order_changed", "bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());", "bytes_view_terminal_drop(selected,detached.borrow_mut(),owner_receipt.borrow_mut());"),
        ("client_output_read_after_drop", "let borrowed=read_view(&selected,detached.borrow());", "bytes_view_terminal_drop(selected,detached.borrow_mut(),selected_receipt.borrow_mut());\n    let borrowed=read_view(&selected,detached.borrow());"),
        ("client_extra_raw_alias", "let selected=slice_view(&first,c..d,detached.borrow_mut());", "let raw=unsafe { selected.ptr };\n    let selected=slice_view(&first,c..d,detached.borrow_mut());"),
    ]
    for name, old, new in client_controls:
        mutated = replace_once(client, old, new, name)
        by_id[name] = lambda mutated=mutated: C.assert_view_extension(extension, pointer, mutated)

    by_id["active_extra_code"] = lambda: C.assert_active_composition(
        prefix, extension, client, active + "\nfn hidden()->bool{true}\n", active)
    by_id["active_prefix_wrong"] = lambda: C.assert_active_composition(
        prefix + "\n// inherited prefix drift\n", extension, client, active, active)

    mapping_controls: list[tuple[str, str, Any]] = [
        ("mapping_ancestor_redirect", "source_transform.ancestor_source", "../elsewhere/positive.rs"),
        ("mapping_parent_hash_wrong", "source_transform.ancestor_sha256", "0" * 64),
        ("mapping_selected_prefix_hash_wrong", "selected_prefix_sha256", "f" * 64),
        ("mapping_pointer_module_redirect", "pointer_support.path", "src/not_view_pointer.rs"),
        ("mapping_terminal_helper_hash_wrong", "terminal_helpers_sha256", "0" * 64),
        ("mapping_alpha_rename_wrong", "native_alpha_renaming.slice.begin", "begin"),
        ("mapping_extra_alpha_alias", "native_alpha_renaming", {"slice":{"begin":"view_begin","end":"end"}}),
        ("mapping_negative_feature_selected", "feature", "view_capacity"),
        ("mapping_claims_wider_scope", "excluded", ["none"]),
        ("mapping_debug_place_changed", "debug_places.selected", "_999"),
    ]

    def set_path(obj: Any, path: str, value: Any) -> None:
        parts = path.split(".")
        node = obj
        for part in parts[:-1]:
            node = node[int(part)] if isinstance(node, list) else node[part]
        last = parts[-1]
        if isinstance(node, list): node[int(last)] = value
        else: node[last] = value

    for name, path, value in mapping_controls:
        mutated = copy.deepcopy(mapping)
        set_path(mutated, path, value)
        by_id[name] = lambda mutated=mutated: C.assert_mapping_sources(
            mutated, prefix, extension, client, active, pointer)

    mutated_drop_edge = copy.deepcopy(mapping)
    set_path(mutated_drop_edge, "normal_edges.2.place", "_99")
    by_id["mapping_drop_edge_retargeted"] = lambda: C.assert_native_mapping(
        mutated_drop_edge, native)
    mutated_mir_not_ready = copy.deepcopy(mapping)
    set_path(mutated_mir_not_ready, "native_mir_ready", False)
    by_id["mapping_native_mir_not_ready"] = lambda: C.assert_native_mapping(
        mutated_mir_not_ready, native)

    by_id["generator_view_capacity_feature_drift"] = lambda: C.assert_generator_source(
        replace_bytes_once(generator, b"'missing_view_registration', 'view_capacity', 'empty_register')",
            b"'missing_view_registration', 'view_capacity_wrong', 'empty_register')", "generator_view_capacity_feature_drift"))
    by_id["generator_empty_register_feature_drift"] = lambda: C.assert_generator_source(
        replace_bytes_once(generator, b"if feature=='empty_register':", b"if feature=='empty_unregister':",
            "generator_empty_register_feature_drift"))

    native_controls: list[tuple[str, str, Any]] = [
        ("native_edge_place_mismatch", "client.normal_edges.1.place", "_42"),
        ("native_edge_missing", "client.normal_edges.3", None),
        ("native_range_wrong_native_type", "client.range_specialization.native_type", "Other"),
        ("native_source_return_route_changed", "paths_and_capture.resolved_capture_inputs.native_source", "/tmp/not-selected.rs"),
        ("native_saved_return_order_changed", "client.selected_read_and_saved_return.selected_drop", "bb8"),
        ("native_local_role_changed", "client.local_roles.Selected", "_999"),
        ("native_slice_checked_add_count", "native_audit.slice_mir.excluded_start_and_included_end_checked_add_calls", 1),
        ("native_empty_branch_clones", "native_audit.slice_mir.empty_branch.clone", True),
        ("native_nonempty_branch_uses_empty_ctor", "native_audit.slice_mir.nonempty_branch.empty_constructor", True),
        ("native_empty_authority_claim", "native_audit.empty_view.empty_has_no_ticket_or_allocation_authority_claim", False),
        ("native_selected_mir_omitted", "native_mir.selected_mir_count_including_client", 24),
    ]
    for name, path, value in native_controls:
        mutated = copy.deepcopy(native)
        if value is None:
            parent_path, index = path.rsplit(".", 1)
            parent_obj = mutated
            for part in parent_path.split("."):
                parent_obj = parent_obj[int(part)] if isinstance(parent_obj, list) else parent_obj[part]
            parent_obj.pop(int(index))
        else:
            set_path(mutated, path, value)
        by_id[name] = lambda mutated=mutated: C.assert_native_mapping(mapping, mutated)

    production_controls = [
        ("production_view_offset_changed", "ret.ptr.add(begin)", "ret.ptr.add(begin+1)"),
        ("production_empty_constructor_changed", "Bytes::new_empty_with_ptr(self.ptr.wrapping_add(begin))",
         "Bytes::new_empty_with_ptr(ret.ptr.wrapping_add(begin))"),
    ]
    for name, old, new in production_controls:
        mutated = replace_once(production_source, old, new, name)
        by_id[name] = lambda mutated=mutated: C.assert_native_view_source_inputs(
            mutated, source_map, native_bindings)
    extractor = (ROOT / "extract_public.py").read_bytes()
    by_id["production_extractor_source_changed"] = lambda: C.assert_aq_extractor_source_surface(
        replace_bytes_once(extractor, b"public_records.rs", b"public_recordX.rs",
                           "production_extractor_source_changed"))
    bad_map = copy.deepcopy(source_map)
    bad_map["view_bodies"]["slice"] = "0" * 64
    by_id["production_source_map_spoofed"] = lambda: C.assert_native_view_source_inputs(
        production_source, bad_map, native_bindings)
    by_id["extracted_native_body_drift"] = lambda: C.assert_native_view_source_inputs(
        production_source, source_map, native_bindings + "\nfn hidden(){}\n")

    probe_manifest = (ROOT / "Cargo.toml").read_bytes()
    native_manifest = (ROOT / "native-test/Cargo.toml").read_bytes()
    by_id["probe_manifest_patch_override"] = lambda: C.assert_probe_manifest(
        replace_bytes_once(probe_manifest,b"[workspace]",b"[patch.crates-io]\ncreusot-std={path=\"../../evil\"}\n[workspace]", "probe_manifest_patch_override"),
        native_manifest)
    by_id["probe_default_feature_selected"] = lambda: C.assert_probe_manifest(
        replace_bytes_once(probe_manifest,b"negative_missing_acquire = []",b"negative_missing_acquire = [\"negative_missing_control_free\"]", "probe_default_feature_selected"),
        native_manifest)
    by_id["native_manifest_library_redirect"] = lambda: C.assert_probe_manifest(
        probe_manifest, replace_bytes_once(native_manifest,b'path = "../native.rs"',b'path = "../../evil.rs"', "native_manifest_library_redirect"))
    lock_bytes=(ROOT/"Cargo.lock").read_bytes()
    by_id["probe_lock_resolution_changed"] = lambda: C.assert_probe_lock(
        replace_bytes_once(lock_bytes,b'name = "creusot-std"\nversion = "0.13.0"',
            b'name = "creusot-std"\nversion = "9.99.0"', "probe_lock_resolution_changed"))

    receipt_mutations = [
        ("compiled_record_hash_receipt_wrong", "captured_input_sha256", "0" * 64),
        ("compiled_build_output_path_redirect", "captured_build_output_path", "generated/unreviewed-output.txt"),
        ("compiled_fingerprint_hash_receipt_wrong", "captured_cargo_build_fingerprint_sha256", "f" * 64),
        ("compiled_out_dir_path_mismatch", "actual_out_dir", "/tmp/unrelated/out"),
        ("compiled_target_directory_redirected", "cargo_target_dir", "/tmp/unpinned-target"),
        ("compiled_artifact_count_inflated", "captured_actual_Cargo_artifact_count", 5),
    ]
    for name, key, value in receipt_mutations:
        mutated = copy.deepcopy(compiled_receipt)
        mutated[key] = value
        stored = (json.dumps(mutated, indent=2) + "\n").encode()
        by_id[name] = lambda mutated=mutated, stored=stored: C.assert_compiled_capture(
            mutated, stored_receipt_bytes=stored)
    captured_artifacts = {name:(ROOT / "generated/compiled-inputs" / name).read_bytes() for name in (
        "public_records.rs", "cargo-run-build-fingerprint.json", "cargo-build-output.txt", "cargo-root-output.txt")}
    forged_receipt = copy.deepcopy(compiled_receipt)
    forged_out = "/tmp/unrelated-package/out"
    forged_root = forged_out.encode()
    forged_receipt.update({"actual_out_dir":forged_out,
        "compiled_input_path":forged_out + "/public_records.rs",
        "root_output_path":"/tmp/unrelated-package/root-output",
        "root_output_sha256":sha(forged_root), "captured_root_output_sha256":sha(forged_root)})
    forged_artifacts = dict(captured_artifacts)
    forged_artifacts["cargo-root-output.txt"] = forged_root
    stored_forged_receipt = (json.dumps(forged_receipt, indent=2) + "\n").encode()
    by_id["compiled_receipt_root_outside_selected_package"] = lambda: C.assert_compiled_capture(
        forged_receipt, forged_artifacts, stored_forged_receipt)

    if set(by_id) != set(expected_ids):
        raise RuntimeError(f"AQ checker control implementation/fixture IDs differ: missing={sorted(set(expected_ids)-set(by_id))}, extra={sorted(set(by_id)-set(expected_ids))}")
    for row in controls:
        results.append(rejected(row["id"], by_id[row["id"]]))
    return {"status":"pass" if len(results) == len(controls) else "fail",
        "checker_sha256":sha(CHECKER_PATH.read_bytes()),
        "fixture_sha256":sha(MANIFEST_PATH.read_bytes()),
        "baseline_status":"pass", "baseline_source_sha256":{
            "active":sha(active.encode()),"extension":sha(extension.encode()),"client":sha(client.encode()),
            "mapping":sha((ROOT/"generated/mapping.json").read_bytes())},
        "control_count":len(results), "rejected_as_expected":sum(r["status"]=="rejected_as_expected" for r in results),
        "controls":results, "mutations_in_memory_only":True,
        "source_mutated_on_disk":False, "cargo_or_rust_build_invoked":False, "solver_invoked":False}


def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_RECEIPT)
    args = parser.parse_args()
    result = run_controls()
    rendered = json.dumps(result, indent=2) + "\n"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
