#!/usr/bin/env python3
"""AR in-memory source, API/frame, native-map and compile-route controls."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import pathlib
import sys
from unittest import mock
from typing import Any, Callable

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_correspondence.py"
MANIFEST_PATH = ROOT / "fixtures/checker-controls.json"
DEFAULT_RECEIPT = ROOT / "generated/checker-controls-receipt.json"


def load_checker():
    spec = importlib.util.spec_from_file_location("ar_controls_target", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the AR correspondence checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


C = load_checker()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one mutation anchor, found {count}: {old!r}")
    return source.replace(old, new, 1)


def reject(control_id: str, action: Callable[[], Any]) -> dict[str, str]:
    try:
        result = action()
    except C.CheckError as exc:
        return {"id": control_id, "status": "rejected_as_expected", "reason": str(exc)}
    except Exception as exc:
        raise RuntimeError(f"{control_id}: checker crashed rather than rejecting: {type(exc).__name__}: {exc}") from exc
    raise RuntimeError(f"{control_id}: checker accepted mutation (result={result!r})")


def full_audit_rejection(expected_reason: str) -> dict[str, Any]:
    result = C.audit()
    if result.get("status") == "reject":
        reason = str(result.get("reason", "full AR audit rejected the in-memory mutation"))
        if expected_reason not in reason:
            raise RuntimeError(f"full audit rejected for an unrelated reason: {reason}")
        raise C.CheckError(reason)
    return result


def run_controls() -> dict[str, Any]:
    manifest = json.loads(MANIFEST_PATH.read_text())
    controls = manifest.get("controls")
    if manifest.get("version") != 1 or not isinstance(controls, list):
        raise RuntimeError("AR checker-control fixture has invalid version or shape")
    ids = [row.get("id") for row in controls]
    if not ids or any(not isinstance(x, str) for x in ids) or len(set(ids)) != len(ids):
        raise RuntimeError("AR checker-control IDs must be unique nonempty strings")

    baseline = C.audit()
    if baseline.get("status") != "pass":
        raise RuntimeError(f"AR complete source/native correspondence baseline did not pass: {baseline.get('reason')}")
    aq, _ = C.preflight_published_aq()
    ancestor = (C.AQ_ROOT / "generated/positive.rs").read_text()
    prefix = C.AR_PREFIX.read_text()
    extension = C.AR_EXTENSION.read_text()
    pointer = C.AR_POINTER.read_text()
    client = C.AR_CLIENT.read_text()
    active = C.ACTIVE.read_text()
    positive = (C.ROOT / "generated/positive.rs").read_text()
    mapping = json.loads(C.MAPPING.read_text())
    lib = (C.ROOT / "src/lib.rs").read_text()
    native = C.audit_native_capture()
    ar_source_tree = C._regular_tree_files(C.ROOT / "src", "AR control")

    changed_tree = dict(ar_source_tree)
    changed_tree["event.rs"] += b"\n#[trusted] fn hidden_support_effect() {}\n"
    actions = {}
    actions["support_inherited_module_false_trust"] = lambda changed_tree=changed_tree: C.assert_ar_support_closure(
        aq, ar_tree_override=changed_tree)

    changed_tree = dict(ar_source_tree)
    changed_tree["event.rs"] += b"\nmacro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n"
    actions["support_inherited_module_macro_injection"] = lambda changed_tree=changed_tree: C.assert_ar_support_closure(
        aq, ar_tree_override=changed_tree)

    changed_tree = dict(ar_source_tree)
    old_attr = b"#[cfg_attr(creusot, ensures(result == ptr.addr_logic()))]"
    if changed_tree["provenance_specs.rs"].count(old_attr) != 1:
        raise RuntimeError("support_provenance_false_post: expected one pointer-address contract anchor")
    changed_tree["provenance_specs.rs"] = changed_tree["provenance_specs.rs"].replace(
        old_attr, b"#[cfg_attr(creusot, ensures(false))]", 1)
    actions["support_provenance_false_post"] = lambda changed_tree=changed_tree: C.assert_ar_support_closure(
        aq, ar_tree_override=changed_tree)

    changed_tree = dict(ar_source_tree)
    changed_tree["event.rs"] += b'\ninclude!("../../unreviewed/event_impl.rs");\n'
    actions["support_hidden_include_route"] = lambda changed_tree=changed_tree: C.assert_ar_support_closure(
        aq, ar_tree_override=changed_tree)

    changed_tree = dict(ar_source_tree)
    changed_tree["closed_macro.rs"] = b"macro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n"
    actions["support_extra_unrouted_source_file"] = lambda changed_tree=changed_tree: C.assert_ar_support_closure(
        aq, ar_tree_override=changed_tree)

    changed_tree = dict(ar_source_tree)
    changed_tree["lib.rs"] = changed_tree["lib.rs"].replace(
        b"#[cfg(creusot)] mod cursor_pointer;",
        b'#[cfg(creusot)] #[path="../../unreviewed/cursor_pointer.rs"] mod cursor_pointer;', 1)
    actions["support_cursor_module_route_redirect"] = lambda changed_tree=changed_tree: C.assert_ar_support_closure(
        aq, ar_tree_override=changed_tree)

    end_to_end_tree = dict(ar_source_tree)
    end_to_end_tree["event.rs"] += b"\n#[trusted] fn hidden_full_audit_effect() {}\n"

    def full_audit_support_mutation() -> Any:
        original_tree_reader = C._regular_tree_files
        def injected_tree_reader(directory: pathlib.Path, label: str) -> dict[str, bytes]:
            if directory.resolve() == (C.ROOT / "src").resolve() and label == "AR":
                return end_to_end_tree
            return original_tree_reader(directory, label)
        with mock.patch.object(C, "_regular_tree_files", injected_tree_reader):
            return full_audit_rejection("AR inherited AQ support/source module changed: event.rs")

    actions["full_audit_inherited_support_mutation"] = full_audit_support_mutation

    mutations = [
        ("enum_missing_vacant", ", Vacant", ""),
        ("enum_vacant_has_payload", ", Vacant", ", Vacant(ChildProof)"),
        ("enum_extra_variant", ", Vacant", ", Vacant, Hidden"),
        ("enum_root_payload_changed", "Root(RootDescriptor)", "Root(ChildProof)"),
    ]
    for control_id, old, new in mutations:
        changed = replace_once(prefix, old, new, control_id)
        actions[control_id] = lambda changed=changed: C.assert_vacant_enum_transformation(
            aq, ancestor, changed, mapping)

    changed_prefix = prefix + "\nfn hidden_api()->usize{0}\n"
    actions["prefix_extra_method"] = lambda: C.assert_vacant_enum_transformation(
        aq, ancestor, changed_prefix, mapping)

    changed_lib = replace_once(lib, "#[cfg(creusot)] mod cursor_pointer;",
        "#[cfg(creusot)] mod cursor_pointer;\n#[path=\"evil.rs\"] mod cursor_pointer;", "lib_extra_module_route")
    actions["lib_extra_module_route"] = lambda: C.assert_lib_route(aq, changed_lib)

    mapping_controls = [
        ("mapping_ancestor_redirect", "source_transform.ancestor_source", "../unreviewed/positive.rs"),
        ("mapping_ancestor_hash_changed", "source_transform.ancestor_sha256", "0" * 64),
        ("mapping_vacant_literal_changed", "source_transform.new", "enum OriginalSharedProof { Vacant }")
    ]

    def set_path(value: dict[str, Any], path: str, item: Any) -> None:
        node: Any = value
        parts = path.split(".")
        for part in parts[:-1]:
            node = node[part]
        node[parts[-1]] = item

    for control_id, path, value in mapping_controls:
        changed = copy.deepcopy(mapping)
        set_path(changed, path, value)
        if control_id == "mapping_vacant_literal_changed":
            actions[control_id] = lambda changed=changed: C.assert_vacant_enum_transformation(
                aq, ancestor, prefix, changed)
        else:
            actions[control_id] = lambda changed=changed: C.assert_ar_composition(
                aq, changed, prefix, extension, client, active, positive, pointer)

    changed_extension = extension + "\n#[trusted] fn hidden_effect(){}\n"
    actions["composition_extension_changed"] = lambda: C.assert_ar_composition(
        aq, mapping, prefix, changed_extension, client, active, positive, pointer)

    changed_pointer = pointer + "\n#[trusted] fn hidden_pointer_axiom(){}\n"
    actions["composition_pointer_changed"] = lambda: C.assert_ar_composition(
        aq, mapping, prefix, extension, client, active, positive, changed_pointer)

    weakened = replace_once(extension,
        "#[ensures(result == (self@.len() == 0))]",
        "#[ensures(result == (self@.len() != 0))]", "slice_empty_spec_weakened")
    actions["slice_empty_spec_weakened"] = lambda: C.assert_slice_is_empty_spec(aq, weakened)

    false_post = replace_once(extension,
        "#[ensures(result == (self@.len() == 0))]",
        "#[ensures(false)]", "slice_empty_spec_false")
    actions["slice_empty_spec_false"] = lambda: C.assert_slice_is_empty_spec(aq, false_post)

    macro_override = replace_once(extension,
        "creusot_std::macros::extern_spec!",
        "macro_rules! extern_spec { ($($tokens:tt)*) => {}; } extern_spec!",
        "slice_empty_spec_macro_override")
    actions["slice_empty_spec_macro_override"] = lambda: C.assert_slice_is_empty_spec(aq, macro_override)

    precondition = replace_once(extension,
        "#[ensures(result == (self@.len() == 0))]",
        "#[requires(self@.len() > 0)]\n        #[ensures(result == (self@.len() == 0))]",
        "slice_empty_spec_extra_precondition")
    actions["slice_empty_spec_extra_precondition"] = lambda: C.assert_slice_is_empty_spec(aq, precondition)

    changed_inc = replace_once(extension, "value.ptr=ptr;", "value.ptr=value.ptr;", "api_cursor_pointer_store_changed")
    actions["api_cursor_pointer_store_changed"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_inc, pointer, client, mapping)

    changed_read = replace_once(extension, "if value.len==0 {", "if value.len!=0 {", "api_read_empty_branch_swapped")
    actions["api_read_empty_branch_swapped"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_read, pointer, client, mapping)

    changed_advance = replace_once(extension, "assert!(count<=value.len,", "assert!(count==0,", "api_advance_guard_weakened")
    actions["api_advance_guard_weakened"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_advance, pointer, client, mapping)

    changed_empty_range = replace_once(extension, "if end==view_begin {", "if false {", "api_empty_range_branch_removed")
    actions["api_empty_range_branch_removed"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_empty_range, pointer, client, mapping)

    changed_pointer_lease = replace_once(pointer,
        "AdvanceLease::Zero=>count==0usize && bound.inner_logic()@==None",
        "AdvanceLease::Zero=>bound.inner_logic()@==None", "api_zero_lease_unbounded")
    actions["api_zero_lease_unbounded"] = lambda: C.assert_ar_shadow_surface(
        aq, extension, changed_pointer_lease, client, mapping)

    changed_callback = replace_once(extension, "let native=value.vtable.drop;", "let native=shared_table().drop;",
                                    "api_dynamic_callback_route_changed")
    actions["api_dynamic_callback_route_changed"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_callback, pointer, client, mapping)

    changed_owner = replace_once(extension, "a==b,", "false,", "api_owner_identity_changed")
    actions["api_owner_identity_changed"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_owner, pointer, client, mapping)

    changed_client = replace_once(client, "while i<steps.len() {", "while i<steps.len() { if i>=16 {break;} ",
                                   "client_fixed_advance_quota")
    actions["client_fixed_advance_quota"] = lambda: C.assert_ar_shadow_surface(
        aq, extension, pointer, changed_client, mapping)

    changed_ghost = extension + "\n#[trusted] fn hidden_effect() {}\n"
    actions["api_extension_hidden_trust"] = lambda: C.assert_ar_shadow_surface(
        aq, changed_ghost, pointer, client, mapping)

    changed_alpha = copy.deepcopy(mapping)
    changed_alpha["native_alpha_renaming"]["slice_cursor_entry"]["begin"] = "begin"
    actions["mapping_alpha_rename_changed"] = lambda: C.assert_ar_shadow_surface(
        aq, extension, pointer, client, changed_alpha)

    changed_edges = copy.deepcopy(mapping)
    changed_edges["normal_edges"][0]["owner"] = "owner"
    actions["mapping_native_drop_edge_changed"] = lambda: C.assert_native_mapping(changed_edges, native)

    changed_frame = copy.deepcopy(mapping)
    changed_frame["ownership_frame"] = "same byte length is ownership"
    actions["mapping_api_frame_changed"] = lambda: C.assert_ar_shadow_surface(
        aq, extension, pointer, client, changed_frame)

    manifest_text = (C.ROOT / "Cargo.toml").read_text()
    lock_bytes = (C.ROOT / "Cargo.lock").read_bytes()
    build_bytes = (C.ROOT / "build.rs").read_bytes()
    actions["manifest_patch_redirect"] = lambda: C.assert_probe_inputs(
        manifest_text=manifest_text + '\n[patch.crates-io]\ncreusot-std = { path = "../../unreviewed" }\n',
        lock_bytes=lock_bytes, build_bytes=build_bytes, environment={})
    actions["manifest_default_negative_feature"] = lambda: C.assert_probe_inputs(
        manifest_text=manifest_text.replace("negative_missing_acquire = []", 'negative_missing_acquire = []\ndefault = ["negative_missing_acquire"]'),
        lock_bytes=lock_bytes, build_bytes=build_bytes, environment={})
    actions["manifest_diagnostic_environment"] = lambda: C.assert_probe_inputs(
        manifest_text=manifest_text, lock_bytes=lock_bytes, build_bytes=build_bytes,
        environment={"BYTES_SCOPE_DIAGNOSTIC":"1"})

    changed_client = client + "\nmacro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n"
    actions["composition_client_macro_added"] = lambda: C.assert_ar_composition(
        aq, mapping, prefix, extension, changed_client, active, positive, pointer)

    changed_active = active + "\nfn hidden_runtime_effect(){}\n"
    actions["composition_active_tail_added"] = lambda: C.assert_ar_composition(
        aq, mapping, prefix, extension, client, changed_active, positive, pointer)

    live_build = C.assert_ar_live_build(aq, require_capture=False)
    live_receipt = live_build["receipt"]
    live_artifacts = {
        "public_records.rs": pathlib.Path(live_receipt["compiled_input_path"]).read_bytes(),
        "cargo-run-build-fingerprint.json": pathlib.Path(live_receipt["cargo_build_fingerprint"]).read_bytes(),
        "cargo-build-output.txt": pathlib.Path(live_receipt["build_output_path"]).read_bytes(),
        "cargo-root-output.txt": pathlib.Path(live_receipt["root_output_path"]).read_bytes(),
    }
    selected_record = (C.ROOT / "generated/public_records.rs").read_bytes()
    selected_source_map = (C.ROOT / "generated/source-map.json").read_bytes()
    expected_input_hashes = live_receipt["production_rerun_input_sha256"]

    def compiled_action(receipt: dict[str, Any], artifacts: dict[str, bytes]) -> Any:
        receipt_bytes = (json.dumps(receipt, indent=2) + "\n").encode()
        return C.assert_ar_compiled_capture(receipt, artifacts_override=artifacts,
            stored_receipt_bytes=receipt_bytes, expected_record=selected_record,
            source_map_bytes=selected_source_map, expected_input_hashes=expected_input_hashes)

    changed_receipt = copy.deepcopy(live_receipt)
    redirected_out = "/tmp/ar-unreviewed-build/out"
    changed_receipt["actual_out_dir"] = redirected_out
    changed_receipt["compiled_input_path"] = redirected_out + "/public_records.rs"
    changed_receipt["root_output_path"] = "/tmp/ar-unreviewed-build/root-output"
    changed_artifacts = dict(live_artifacts)
    changed_artifacts["cargo-root-output.txt"] = redirected_out.encode()
    changed_receipt["captured_root_output_sha256"] = sha(changed_artifacts["cargo-root-output.txt"])
    changed_receipt["root_output_sha256"] = sha(changed_artifacts["cargo-root-output.txt"])
    actions["compiled_capture_detached_out_dir_fresh_hashes"] = lambda receipt=changed_receipt, artifacts=changed_artifacts: compiled_action(
        receipt, artifacts)

    changed_receipt = copy.deepcopy(live_receipt)
    changed_artifacts = dict(live_artifacts)
    fp = json.loads(changed_artifacts["cargo-run-build-fingerprint.json"])
    fp["local"] = [dict(row, RerunIfChanged=dict(row["RerunIfChanged"], output="debug/build/unreviewed/output"))
                    if isinstance(row, dict) and isinstance(row.get("RerunIfChanged"), dict) else row
                    for row in fp.get("local", [])]
    changed_artifacts["cargo-run-build-fingerprint.json"] = (json.dumps(fp, separators=(",", ":")) + "\n").encode()
    fp_hash = sha(changed_artifacts["cargo-run-build-fingerprint.json"])
    changed_receipt["captured_cargo_build_fingerprint_sha256"] = fp_hash
    changed_receipt["cargo_build_fingerprint_sha256"] = fp_hash
    actions["compiled_fingerprint_output_redirect_fresh_hashes"] = lambda receipt=changed_receipt, artifacts=changed_artifacts: compiled_action(
        receipt, artifacts)

    changed_receipt = copy.deepcopy(live_receipt)
    changed_artifacts = dict(live_artifacts)
    fp = json.loads(changed_artifacts["cargo-run-build-fingerprint.json"])
    fp["local"] = [dict(row, RerunIfChanged=dict(row["RerunIfChanged"],
                    paths=["../../unreviewed/bytes.rs"] + row["RerunIfChanged"].get("paths", [])[1:]))
                    if isinstance(row, dict) and isinstance(row.get("RerunIfChanged"), dict) else row
                    for row in fp.get("local", [])]
    changed_artifacts["cargo-run-build-fingerprint.json"] = (json.dumps(fp, separators=(",", ":")) + "\n").encode()
    fp_hash = sha(changed_artifacts["cargo-run-build-fingerprint.json"])
    changed_receipt["captured_cargo_build_fingerprint_sha256"] = fp_hash
    changed_receipt["cargo_build_fingerprint_sha256"] = fp_hash
    actions["compiled_fingerprint_source_redirect_fresh_hashes"] = lambda receipt=changed_receipt, artifacts=changed_artifacts: compiled_action(
        receipt, artifacts)

    changed_receipt = copy.deepcopy(live_receipt)
    changed_artifacts = dict(live_artifacts)
    changed_artifacts["public_records.rs"] += b"\n// unexpected captured record tail\n"
    record_hash = sha(changed_artifacts["public_records.rs"])
    changed_receipt["captured_input_sha256"] = record_hash
    changed_receipt["compiled_input_sha256"] = record_hash
    changed_receipt["reconstructed_generated_sha256"] = record_hash
    actions["compiled_public_record_forged_fresh_hashes"] = lambda receipt=changed_receipt, artifacts=changed_artifacts: compiled_action(
        receipt, artifacts)

    changed_receipt = copy.deepcopy(live_receipt)
    changed_artifacts = dict(live_artifacts)
    changed_artifacts["cargo-root-output.txt"] = b"/tmp/unreviewed/out"
    root_hash = sha(changed_artifacts["cargo-root-output.txt"])
    changed_receipt["captured_root_output_sha256"] = root_hash
    changed_receipt["root_output_sha256"] = root_hash
    actions["compiled_root_output_contents_forged_fresh_hash"] = lambda receipt=changed_receipt, artifacts=changed_artifacts: compiled_action(
        receipt, artifacts)

    def full_audit_compiled_capture_mutation() -> Any:
        original_capture_check = C.assert_ar_compiled_capture
        original_live_build = C.assert_ar_live_build
        def injected_live_build(aq_module: Any, require_capture: bool = True,
                                capture: bool = False) -> dict[str, Any]:
            live = original_live_build(aq_module, require_capture=False, capture=False)
            receipt = live["receipt"]
            altered = {
                "public_records.rs": pathlib.Path(receipt["compiled_input_path"]).read_bytes(),
                "cargo-run-build-fingerprint.json": pathlib.Path(receipt["cargo_build_fingerprint"]).read_bytes(),
                "cargo-build-output.txt": pathlib.Path(receipt["build_output_path"]).read_bytes(),
                "cargo-root-output.txt": pathlib.Path(receipt["root_output_path"]).read_bytes(),
            }
            forged = copy.deepcopy(receipt)
            detached = "/tmp/ar-full-audit-unreviewed/out"
            forged["actual_out_dir"] = detached
            forged["compiled_input_path"] = detached + "/public_records.rs"
            forged["root_output_path"] = "/tmp/ar-full-audit-unreviewed/root-output"
            altered["cargo-root-output.txt"] = detached.encode()
            root_hash = sha(altered["cargo-root-output.txt"])
            forged["root_output_sha256"] = root_hash
            forged["captured_root_output_sha256"] = root_hash
            forged_bytes = (json.dumps(forged, indent=2) + "\n").encode()
            original_capture_check(forged, altered, forged_bytes,
                pathlib.Path(C.ROOT / "generated/public_records.rs").read_bytes(),
                pathlib.Path(C.ROOT / "generated/source-map.json").read_bytes(),
                receipt["production_rerun_input_sha256"])
            raise RuntimeError("full AR audit failed to reject its injected compiled-artifact mutation")
        with mock.patch.object(C, "assert_ar_live_build", injected_live_build):
            return full_audit_rejection("AR captured Cargo root-output/OUT_DIR/public_records paths do not resolve consistently")

    actions["full_audit_compiled_capture_mutation"] = full_audit_compiled_capture_mutation

    if set(actions) != set(ids):
        raise RuntimeError(f"AR control implementation/fixture IDs differ: missing={sorted(set(ids)-set(actions))}, extra={sorted(set(actions)-set(ids))}")
    results = [reject(row["id"], actions[row["id"]]) for row in controls]
    return {"status": "pass" if len(results) == len(controls) else "fail",
        "checker_sha256": sha(CHECKER_PATH.read_bytes()),
        "fixture_sha256": sha(MANIFEST_PATH.read_bytes()),
        "baseline_scope": "closed AR source/API/native/Cargo-input correspondence; proof admission and unwind behavior are separate",
        "baseline_status": "pass", "control_count": len(results),
        "rejected_as_expected": sum(row["status"] == "rejected_as_expected" for row in results),
        "controls": results, "mutations_in_memory_only": True,
        "cargo_or_rust_build_invoked": False, "solver_invoked": False,
        "full_AR_api_native_gate": "pass"}


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
