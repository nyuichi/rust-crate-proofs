#!/usr/bin/env python3
"""In-memory adversarial controls for the AT correspondence gate.

All mutations are byte dictionaries and copied JSON values. The suite does
not write Rust/proof inputs, invoke Cargo, or invoke a prover.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import pathlib
import sys
from typing import Any, Callable
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER = ROOT / "check_correspondence.py"
MANIFEST_PATH = ROOT / "fixtures/checker-controls.json"
DEFAULT_RECEIPT = ROOT / "generated/checker-controls-receipt.json"

spec = importlib.util.spec_from_file_location("at_checker_controls_target", CHECKER)
assert spec and spec.loader
C = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = C
spec.loader.exec_module(C)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def replace_once(data: bytes, old: bytes, new: bytes, label: str) -> bytes:
    count = data.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one mutation anchor, found {count}")
    return data.replace(old, new, 1)


def expect_reject(identifier: str, action: Callable[[], Any],
                  expected: str | None = None) -> dict[str, str]:
    try:
        result = action()
    except C.CheckError as exc:
        reason = str(exc)
        if expected and expected not in reason:
            raise RuntimeError(
                f"{identifier}: rejected at the wrong gate; expected {expected!r}, got {reason!r}") from exc
        return {"id": identifier, "status": "rejected_as_expected", "reason": reason}
    except Exception as exc:
        raise RuntimeError(f"{identifier}: checker error: {type(exc).__name__}: {exc}") from exc
    raise RuntimeError(f"{identifier}: accepted mutation: {result!r}")


def full_audit_reject(inputs: dict[str, Any], context: dict[str, Any],
                      archive_report: dict[str, Any], archive: dict[str, bytes],
                      identifier: str, expected: str) -> dict[str, str]:
    def action() -> Any:
        with mock.patch.object(C, "assert_as_canonical_archive",
                               return_value=(archive_report, archive)), \
             mock.patch.object(C, "assert_published_as_ancestry",
                               return_value=context):
            return C.audit_inputs(inputs, cargo_mode="captured")
    try:
        result = action()
    except C.CheckError as exc:
        if expected not in str(exc):
            raise RuntimeError(
                f"{identifier}: full audit rejected at the wrong gate: {exc}") from exc
        raise
    raise RuntimeError(f"{identifier}: full audit accepted mutation: {result!r}")


def run() -> dict[str, Any]:
    fixture = json.loads(MANIFEST_PATH.read_text())
    ids = [row.get("id") for row in fixture.get("controls", [])]
    if fixture.get("version") != 1 or ids != AT_IDS or len(set(ids)) != len(ids):
        raise RuntimeError(
            f"AT fixture does not exactly match the implemented {len(AT_IDS)} controls")

    # The single unmutated run establishes that each check is rejecting a
    # plausible change from an accepted source/configuration state.
    baseline = C.audit(cargo_mode="captured")
    if baseline.get("status") != "pass":
        raise RuntimeError(f"AT complete positive baseline failed: {baseline!r}")
    archive_report, archive = C.assert_as_canonical_archive()
    context = C.assert_published_as_ancestry(archive)
    inputs = C._load_inputs()
    as_checker = context["_as_checker"]
    ar = context["_ar_checker"]
    aq = context["_aq_ar"]
    source_tree = inputs["source_tree"]
    mapping = json.loads(inputs["mapping_bytes"])
    active = inputs["active"]
    positive = inputs["positive"]

    def composition_check(changed: dict[str, Any]) -> Any:
        changed_map = json.loads(changed["mapping_bytes"])
        return C.assert_at_composition(changed["source_tree"], changed["generator"],
            changed_map, changed["active"], changed["positive"],
            changed["extension_generated"], changed["terminal_generated"],
            changed["client"], aq, ar)

    def source_tree_check(changed_tree: dict[str, bytes]) -> Any:
        return C.assert_at_source_closure(archive, changed_tree)

    extension = source_tree["owned_clone_extension.rs"]
    client = inputs["client"]
    actions: dict[str, tuple[Callable[[], Any], str | None]] = {}

    changed = copy.deepcopy(inputs)
    changed["source_tree"]["promotion.rs"] = replace_once(
        changed["source_tree"]["promotion.rs"],
        b"//! Closed first-promotion refinement: root Bytes plus an external affine scope.",
        b"//! Mutated inherited prefix for the source-closure control.",
        "AS complete prefix")
    actions["source_prefix_changed"] = (
        lambda x=changed: source_tree_check(x["source_tree"]),
        "changed inherited AS Rust source")

    changed = copy.deepcopy(inputs)
    changed["source_tree"]["event.rs"] += b"\n// hidden item: #[trusted] fn injected() {}\n"
    actions["inherited_support_mutated"] = (
        lambda x=changed: source_tree_check(x["source_tree"]),
        "changed inherited AS Rust source")

    changed = copy.deepcopy(inputs)
    changed["source_tree"]["rogue.rs"] = b"#[trusted] fn hidden() {}\n"
    actions["extra_unrouted_module"] = (
        lambda x=changed: source_tree_check(x["source_tree"]),
        "source inventory")

    changed = copy.deepcopy(inputs)
    changed["source_tree"]["lib.rs"] += b'\n#[path="../../unreviewed.rs"] mod alias;\n'
    actions["alternate_module_route"] = (
        lambda x=changed: source_tree_check(x["source_tree"]),
        "changed inherited AS Rust source")

    def extension_change(identifier: str, old: bytes, new: bytes,
                         expected: str = "AT selected extension/client/active source") -> None:
        changed = copy.deepcopy(inputs)
        changed["source_tree"]["owned_clone_extension.rs"] = replace_once(
            extension, old, new, identifier)
        changed["extension_generated"] = changed["source_tree"]["owned_clone_extension.rs"]
        # Keep the shadow composition coherent so rejection proves the selected
        # source/body checks, not a stale concatenated output.
        changed["active"] = (changed["source_tree"]["promotion.rs"] + b"\n" +
            changed["extension_generated"] + changed["client"])
        changed["positive"] = changed["active"]
        actions[identifier] = (
            lambda x=changed: composition_check(x), expected)

    extension_change("extension_added_trusted_callback",
        b"fn clone_owned_api(", b"#[trusted]\nfn clone_owned_api(")
    extension_change("extension_false_clone_postcondition",
        b"#[ensures(result.api_view_valid() && result.view_owned() && result.view_content()==source.view_content())]",
        b"#[ensures(false)]\n#[ensures(result.api_view_valid() && result.view_owned() && result.view_content()==source.view_content())]")
    extension_change("extension_callback_registration_removed",
        b"erased_call::invoke3(native,", b"erased_call::invoke2(native,")
    extension_change("extension_wrong_native_shared_source",
        b"pointer_event::load_relaxed(data,", b"pointer_event::load_acquire(data,")
    extension_change("extension_wrong_view_offset",
        b"bound.into_inner())", b"source.bound)")
    extension_change("extension_fresh_ticket_removed",
        b"lifecycle::State::on_register", b"lifecycle::State::on_acquire")
    extension_change("extension_add_hidden_macro_text",
        b"// AT: allocation identity", b"macro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n// AT: allocation identity")

    changed = copy.deepcopy(inputs)
    changed["client"] = b"macro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n" + client
    actions["client_proof_assert_macro_override"] = (
        lambda x=changed: composition_check(x),
        "AT selected extension/client/active source")

    changed = copy.deepcopy(inputs)
    changed["client"] = replace_once(client,
        b"let next=clone_owned_api(&value,detached.borrow_mut());",
        b"let next=clone_owned_api(&value,detached.borrow_mut()); let other=clone_owned_api(&value,detached.borrow_mut());",
        "extra owned clone")
    actions["client_extra_clone_event"] = (
        lambda x=changed: composition_check(x),
        "AT selected extension/client/active source")

    changed = copy.deepcopy(inputs)
    changed["client"] = replace_once(client, b"value=next;", b"i+=1; value=next;", "assignment order")
    actions["client_assignment_effect_reordered"] = (
        lambda x=changed: composition_check(x),
        "AT selected extension/client/active source")

    changed = copy.deepcopy(inputs)
    changed["client"] = replace_once(client,
        b"#[invariant(value.api_view_valid() && value.view_owned() && value.view_accepts(detached.inner_logic()))]",
        b"#[invariant(value.api_view_valid() && value.view_accepts(detached.inner_logic()))]",
        "owned-view invariant")
    actions["client_owned_view_invariant_removed"] = (
        lambda x=changed: composition_check(x),
        "AT selected extension/client/active source")

    changed = copy.deepcopy(inputs)
    changed["extension_generated"] += b"\nfn unreviewed_shadow_fn() {}\n"
    actions["generated_extension_differs_from_source"] = (
        lambda x=changed: composition_check(x),
        "AT active/positive/extension/client composition")

    changed = copy.deepcopy(inputs)
    changed["terminal_generated"] += b"\nfn unreviewed_terminal() {}\n"
    actions["generated_terminal_helper_differs"] = (
        lambda x=changed: composition_check(x),
        "AT active/positive/extension/client composition")

    def changed_mapping(identifier: str, mutate: Callable[[dict[str, Any]], None],
                        expected: str = "AT mapping") -> None:
        changed = copy.deepcopy(inputs)
        changed_map = json.loads(changed["mapping_bytes"])
        mutate(changed_map)
        changed["mapping_bytes"] = (json.dumps(changed_map, indent=2) + "\n").encode()
        actions[identifier] = (lambda x=changed: composition_check(x), expected)

    changed_mapping("mapping_ancestor_path_redirected",
        lambda m: m.__setitem__("ancestor_source", "../unreviewed/positive.rs"),
        "AT mapping does not bind")
    changed_mapping("mapping_prefix_hash_changed",
        lambda m: m.__setitem__("selected_prefix_sha256", "0" * 64),
        "AT mapping does not bind")
    changed_mapping("mapping_extension_hash_changed",
        lambda m: m.__setitem__("extension_sha256", "0" * 64),
        "AT mapping does not bind")
    changed_mapping("mapping_client_hash_changed",
        lambda m: m.__setitem__("client_sha256", "0" * 64),
        "AT mapping does not bind")
    changed_mapping("mapping_source_inventory_changed",
        lambda m: m["support_inventory"].__setitem__("src/owned_clone_extension.rs", "0" * 64),
        "AT mapping does not bind")
    changed_mapping("mapping_unreviewed_helper_added",
        lambda m: m["helpers"].append("hidden_runtime_callback"),
        "AT mapping helper")
    changed_mapping("mapping_assignment_drop_order_changed",
        lambda m: m["assignment_effect"].__setitem__("order",
            ["drop old value", "evaluate next clone", "install next", "increment i"]),
        "AT mapping helper")
    changed_mapping("mapping_claim_generalized",
        lambda m: m.__setitem__("full_original_admitted", True),
        "AT mapping does not bind")
    changed_mapping("mapping_unknown_native_claim_added",
        lambda m: m.__setitem__("native_unwind_admitted", True),
        "AT mapping contains unknown")

    # Native facts use the complete bundle checker, which reparses selected
    # MIR. These mutations keep the cached source archive untouched.
    changed_map = copy.deepcopy(mapping)
    if changed_map.get("normal_edges"):
        changed_map["normal_edges"][0]["owner"] = "wrong-owner"
    actions["mapping_native_drop_owner_changed"] = (
        lambda m=changed_map: C.assert_native(m), "mapping")
    changed_map = copy.deepcopy(mapping)
    if changed_map.get("normal_edges"):
        repeated = next((row for row in changed_map["normal_edges"] if row.get("repeated")), None)
        if repeated is None:
            repeated = changed_map["normal_edges"][-1]
        repeated["repeated"] = False
    actions["mapping_repeated_assignment_drop_unbound"] = (
        lambda m=changed_map: C.assert_native(m), "mapping")
    changed_map = copy.deepcopy(mapping)
    changed_map["native_client_mir"] = "native-mir/unreviewed.mir"
    actions["mapping_native_client_redirected"] = (
        lambda m=changed_map: C.assert_native(m), "mapping")
    changed_map = copy.deepcopy(mapping)
    changed_map["native_source_sha256"] = "0" * 64
    actions["mapping_native_source_changed"] = (
        lambda m=changed_map: C.assert_native(m), "mapping")

    manifest = inputs["manifest"].decode()
    lock = inputs["lock"]
    build = inputs["build"]
    extractor = inputs["extractor"]
    generator = inputs["generator"]
    launcher = inputs["launcher"]
    for identifier, mutation in (
        ("cargo_patch_redirect", manifest + '\n[patch.crates-io]\ncreusot-std={path="../../evil"}\n'),
        ("cargo_target_dependency_redirect", manifest + '\n[target.x86_64-unknown-linux-gnu.dependencies]\ncreusot-std={path="../../evil"}\n'),
        ("cargo_default_negative_feature", manifest + '\ndefault=["negative_missing_acquire"]\n'),
        ("cargo_extra_feature_surface", manifest + '\nextra_clone_mode=[]\n'),
        ("cargo_wrong_package_name", manifest.replace(C.PACKAGE, "bytes-other", 1)),
    ):
        actions[identifier] = (
            lambda m=mutation: C.assert_probe_manifest(m.encode(), lock, build, extractor,
                generator, launcher, {}),
            "AT Cargo package")
    actions["cargo_build_script_mutated"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build + b"\nfn reroute() {}\n",
            extractor, generator, launcher, {}),
        "AT changed its inherited")
    actions["cargo_lock_mutated"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock + b"\n# changed\n",
            build, extractor, generator, launcher, {}),
        "AT lockfile differs")
    actions["env_generator_feature_admission"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build, extractor,
            generator, launcher, {"BYTES_DROP_FEATURE": "zero_to_static"}),
        "AT cannot admit")
    actions["env_source_control_admission"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build, extractor,
            generator, launcher, {"BYTES_SCOPE_SOURCE_CONTROL": "omit_root_recovery_publication"}),
        "AT cannot admit")
    actions["env_diagnostic_admission"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build, extractor,
            generator, launcher, {"BYTES_SCOPE_DIAGNOSTIC": "1"}),
        "AT cannot admit")
    actions["env_checker_skip_admission"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build, extractor,
            generator, launcher, {"BYTES_DROP_CHECKER_SKIP": "1"}),
        "AT cannot admit")
    actions["env_translation_only_admission"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build, extractor,
            generator, launcher, {"BYTES_TRANSLATE_ONLY": "1"}),
        "AT cannot admit")
    actions["env_cargo_feature_admission"] = (
        lambda: C.assert_probe_manifest(inputs["manifest"], lock, build, extractor,
            generator, launcher, {"BYTES_CARGO_FEATURES": "negative_missing_acquire"}),
        "AT cannot admit")

    # Four actual Cargo artifacts, their captured paths and the generated
    # public-record source are reconstructed through the full selected lineage.
    expected_record, input_hashes = C._expected_public_record(as_checker, ar)
    capture = inputs["compiled_capture"]
    receipt = json.loads(capture["receipt_bytes"])
    source_map = capture["source_map"]

    def compiled_check(r: dict[str, Any], artifacts: dict[str, bytes]) -> Any:
        stored = (json.dumps(r, indent=2) + "\n").encode()
        return C.assert_compiled_capture(r, artifacts, stored, expected_record,
            source_map, input_hashes)

    changed_receipt = copy.deepcopy(receipt)
    changed_artifacts = dict(capture["artifacts"])
    detached = "/tmp/at-detached/out"
    changed_receipt.update(actual_out_dir=detached,
        compiled_input_path=detached + "/public_records.rs",
        root_output_path="/tmp/at-detached/root-output")
    changed_artifacts["cargo-root-output.txt"] = detached.encode()
    for field in ("root_output_sha256", "captured_root_output_sha256"):
        changed_receipt[field] = sha(changed_artifacts["cargo-root-output.txt"])
    actions["compiled_out_dir_detached_fresh_hashes"] = (
        lambda r=changed_receipt, a=changed_artifacts: compiled_check(r, a),
        "AT captured Cargo root-output")

    changed_receipt = copy.deepcopy(receipt)
    changed_artifacts = dict(capture["artifacts"])
    fp = json.loads(changed_artifacts["cargo-run-build-fingerprint.json"])
    fp["local"] = [dict(row,
        RerunIfChanged=dict(row["RerunIfChanged"], output="debug/build/unreviewed/output"))
        if isinstance(row, dict) and isinstance(row.get("RerunIfChanged"), dict) else row
        for row in fp.get("local", [])]
    changed_artifacts["cargo-run-build-fingerprint.json"] = (
        json.dumps(fp, separators=(",", ":")) + "\n").encode()
    digest = sha(changed_artifacts["cargo-run-build-fingerprint.json"])
    changed_receipt["cargo_build_fingerprint_sha256"] = digest
    changed_receipt["captured_cargo_build_fingerprint_sha256"] = digest
    actions["compiled_fingerprint_output_redirect_fresh_hashes"] = (
        lambda r=changed_receipt, a=changed_artifacts: compiled_check(r, a),
        "AT Cargo package, fingerprint, target and build-output paths do not join")

    changed_receipt = copy.deepcopy(receipt)
    changed_artifacts = dict(capture["artifacts"])
    fp = json.loads(changed_artifacts["cargo-run-build-fingerprint.json"])
    fp["local"] = [dict(row,
        RerunIfChanged=dict(row["RerunIfChanged"],
            paths=["../../unreviewed.rs"] + row["RerunIfChanged"].get("paths", [])[1:]))
        if isinstance(row, dict) and isinstance(row.get("RerunIfChanged"), dict) else row
        for row in fp.get("local", [])]
    changed_artifacts["cargo-run-build-fingerprint.json"] = (
        json.dumps(fp, separators=(",", ":")) + "\n").encode()
    digest = sha(changed_artifacts["cargo-run-build-fingerprint.json"])
    changed_receipt["cargo_build_fingerprint_sha256"] = digest
    changed_receipt["captured_cargo_build_fingerprint_sha256"] = digest
    actions["compiled_fingerprint_source_redirect_fresh_hashes"] = (
        lambda r=changed_receipt, a=changed_artifacts: compiled_check(r, a),
        "AT Cargo fingerprint does not bind")

    changed_receipt = copy.deepcopy(receipt)
    changed_artifacts = dict(capture["artifacts"])
    changed_artifacts["public_records.rs"] += b"\n// forged record\n"
    digest = sha(changed_artifacts["public_records.rs"])
    changed_receipt["compiled_input_sha256"] = digest
    changed_receipt["reconstructed_generated_sha256"] = digest
    changed_receipt["captured_input_sha256"] = digest
    actions["compiled_public_record_forged_fresh_hashes"] = (
        lambda r=changed_receipt, a=changed_artifacts: compiled_check(r, a),
        "AT compiled OUT_DIR record is not reconstructed from selected production source inputs")

    changed_receipt = copy.deepcopy(receipt)
    changed_artifacts = dict(capture["artifacts"])
    changed_artifacts["cargo-root-output.txt"] = b"/tmp/at-unrelated/out"
    digest = sha(changed_artifacts["cargo-root-output.txt"])
    changed_receipt["root_output_sha256"] = digest
    changed_receipt["captured_root_output_sha256"] = digest
    actions["compiled_root_output_forged_fresh_hashes"] = (
        lambda r=changed_receipt, a=changed_artifacts: compiled_check(r, a),
        "AT captured Cargo root-output")

    # Full audit traversals demonstrate that selected extension, mapping,
    # manifest and archive-only compiler joins cannot be bypassed by a caller.
    changed = copy.deepcopy(inputs)
    changed_extension = changed["source_tree"]["owned_clone_extension.rs"] + b"\n#[trusted] fn unreviewed() {}\n"
    changed["source_tree"]["owned_clone_extension.rs"] = changed_extension
    changed["extension_generated"] = changed_extension
    changed["terminal_generated"] = changed_extension
    changed["active"] = changed["source_tree"]["promotion.rs"] + b"\n" + changed_extension + changed["client"]
    changed["positive"] = changed["active"]
    changed_map = json.loads(changed["mapping_bytes"])
    changed_map["extension_sha256"] = sha(changed_extension)
    changed_map["active_sha256"] = sha(changed["active"])
    changed_map["support_inventory"]["src/owned_clone_extension.rs"] = sha(changed_extension)
    changed["mapping_bytes"] = (json.dumps(changed_map, indent=2) + "\n").encode()
    actions["full_audit_extension_trust_mutation"] = (
        lambda x=changed: full_audit_reject(x, context, archive_report, archive,
            "full_audit_extension_trust_mutation", "AT selected extension/client/active source"),
        "AT selected extension/client/active source")

    changed = copy.deepcopy(inputs)
    changed_map = json.loads(changed["mapping_bytes"])
    if changed_map.get("normal_edges"):
        changed_map["normal_edges"][0]["owner"] = "wrong-owner"
    changed["mapping_bytes"] = (json.dumps(changed_map, indent=2) + "\n").encode()
    actions["full_audit_native_mapping_redirect"] = (
        lambda x=changed: full_audit_reject(x, context, archive_report, archive,
            "full_audit_native_mapping_redirect", "mapping"),
        "mapping")

    changed = copy.deepcopy(inputs)
    changed["manifest"] += b'\n[patch.crates-io]\ncreusot-std={path="../../evil"}\n'
    actions["full_audit_manifest_redirect"] = (
        lambda x=changed: full_audit_reject(x, context, archive_report, archive,
            "full_audit_manifest_redirect", "AT Cargo package"),
        "AT Cargo package")

    changed = copy.deepcopy(inputs)
    changed_capture = changed["compiled_capture"]
    changed_receipt = copy.deepcopy(json.loads(changed_capture["receipt_bytes"]))
    changed_artifacts = dict(changed_capture["artifacts"])
    target = "/tmp/at-full-audit/out"
    changed_receipt.update(actual_out_dir=target,
        compiled_input_path=target + "/public_records.rs",
        root_output_path="/tmp/at-full-audit/root-output")
    changed_artifacts["cargo-root-output.txt"] = target.encode()
    digest = sha(changed_artifacts["cargo-root-output.txt"])
    changed_receipt.update(root_output_sha256=digest, captured_root_output_sha256=digest)
    changed_capture["artifacts"] = changed_artifacts
    changed_capture["receipt_bytes"] = (json.dumps(changed_receipt, indent=2) + "\n").encode()
    actions["full_audit_compiled_join_redirect"] = (
        lambda x=changed: full_audit_reject(x, context, archive_report, archive,
            "full_audit_compiled_join_redirect",
            "AT captured Cargo root-output, OUT_DIR and compiled record paths"),
        "AT captured Cargo root-output")

    if set(actions) != set(AT_IDS):
        raise RuntimeError(
            f"AT control implementation/fixture mismatch: missing={set(AT_IDS)-set(actions)}, extra={set(actions)-set(AT_IDS)}")
    rows = [expect_reject(identifier, actions[identifier][0], actions[identifier][1])
        for identifier in AT_IDS]
    return {
        "status": "pass",
        "baseline_status": baseline.get("status"),
        "control_count": len(rows),
        "rejected_as_expected": len(rows),
        "accepted": [],
        "checker_errors": [],
        "controls": rows,
        "mutations_in_memory_only": True,
        "cargo_or_rust_build_invoked": False,
        "solver_invoked": False,
        "checker_sha256": sha(CHECKER.read_bytes()),
        "fixture_sha256": sha(MANIFEST_PATH.read_bytes()),
        "baseline_scope": "AS canonical lineage plus AT closed owned-View Clone, native normal-MIR map and four-artifact Cargo join",
    }


AT_IDS = [
    "source_prefix_changed", "inherited_support_mutated", "extra_unrouted_module",
    "alternate_module_route", "extension_added_trusted_callback",
    "extension_false_clone_postcondition", "extension_callback_registration_removed",
    "extension_wrong_native_shared_source", "extension_wrong_view_offset",
    "extension_fresh_ticket_removed", "extension_add_hidden_macro_text",
    "client_proof_assert_macro_override", "client_extra_clone_event",
    "client_assignment_effect_reordered", "client_owned_view_invariant_removed",
    "generated_extension_differs_from_source", "generated_terminal_helper_differs",
    "mapping_ancestor_path_redirected", "mapping_prefix_hash_changed",
    "mapping_extension_hash_changed", "mapping_client_hash_changed",
    "mapping_source_inventory_changed", "mapping_unreviewed_helper_added",
    "mapping_assignment_drop_order_changed", "mapping_claim_generalized",
    "mapping_unknown_native_claim_added", "mapping_native_drop_owner_changed",
    "mapping_repeated_assignment_drop_unbound", "mapping_native_client_redirected",
    "mapping_native_source_changed", "cargo_patch_redirect",
    "cargo_target_dependency_redirect", "cargo_default_negative_feature",
    "cargo_extra_feature_surface", "cargo_wrong_package_name",
    "cargo_build_script_mutated", "cargo_lock_mutated",
    "env_generator_feature_admission", "env_source_control_admission",
    "env_diagnostic_admission", "env_checker_skip_admission",
    "env_translation_only_admission", "env_cargo_feature_admission",
    "compiled_out_dir_detached_fresh_hashes",
    "compiled_fingerprint_output_redirect_fresh_hashes",
    "compiled_fingerprint_source_redirect_fresh_hashes",
    "compiled_public_record_forged_fresh_hashes",
    "compiled_root_output_forged_fresh_hashes",
    "full_audit_extension_trust_mutation", "full_audit_native_mapping_redirect",
    "full_audit_manifest_redirect", "full_audit_compiled_join_redirect",
]


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_RECEIPT)
    args = parser.parse_args()
    result = run()
    rendered = json.dumps(result, indent=2) + "\n"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered)
    print(rendered, end="")
