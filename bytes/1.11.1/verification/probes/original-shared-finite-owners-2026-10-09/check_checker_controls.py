#!/usr/bin/env python3
"""Replay AP correspondence-checker controls without Cargo or a prover."""
from __future__ import annotations
import copy
import hashlib
import json
import pathlib
import subprocess
import sys
from typing import Callable

import check_correspondence as C

ROOT = pathlib.Path(__file__).resolve().parent
MANIFEST = ROOT / "fixtures/checker-controls.json"
RECEIPT = ROOT / "generated/checker-controls-receipt.json"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def live_inputs() -> dict:
    return {
        "prefix": (ROOT / "src/promotion.rs").read_text(),
        "extension": (ROOT / "src/finite_extension.rs").read_text(),
        "client": (ROOT / "generated/elaborated-client.rs").read_text(),
        "helper": (ROOT / "generated/finite-extension.rs").read_text(),
        "terminal_helper": (ROOT / "generated/terminal-helper.rs").read_text(),
        "active": (ROOT / "generated/active.rs").read_text(),
        "lib": (ROOT / "src/lib.rs").read_text(),
        "native_source": (ROOT / "native.rs").read_text(),
        "mir": C.CLIENT_MIR.read_text(),
        "mapping": json.loads((ROOT / "generated/mapping.json").read_text()),
        "capture": json.loads((ROOT / "native-mir/capture.json").read_text()),
    }


def rejected(name: str, action: Callable[[], object]) -> dict:
    try:
        result = action()
    except C.CheckError as exc:
        return {"id": name, "result": "rejected", "reason": str(exc)}
    except Exception as exc:  # unexpected checker crashes are not successful controls
        raise RuntimeError(f"control {name} crashed instead of rejecting: {type(exc).__name__}: {exc}") from exc
    if isinstance(result, dict) and result.get("status") == "reject":
        return {"id": name, "result": "rejected", "reason": result.get("reason", "structured reject")}
    raise RuntimeError(f"control {name} was accepted")


def with_changed(source: dict, key: str, old: str, new: str) -> dict:
    changed = copy.deepcopy(source)
    if old not in changed[key]:
        raise RuntimeError(f"control setup cannot find selected text in {key}: {old!r}")
    changed[key] = changed[key].replace(old, new, 1)
    return changed


def run() -> dict:
    fixture_raw = MANIFEST.read_bytes()
    fixture = json.loads(fixture_raw)
    baseline_inputs = live_inputs()
    pinned = {"extension": C.AP_EXTENSION_SHA256, "client": C.AP_CLIENT_SHA256,
              "active": C.AP_ACTIVE_SHA256, "mir": C.AP_CLIENT_MIR_SHA256}
    actual = {"extension": sha(baseline_inputs["extension"].encode()),
              "client": sha(baseline_inputs["client"].encode()),
              "active": sha(baseline_inputs["active"].encode()),
              "mir": sha(baseline_inputs["mir"].encode())}
    if actual != pinned:
        raise RuntimeError("checker controls require the restored AP positive source/MIR inputs")
    positive = C.audit(baseline_inputs)
    if positive.get("status") != "pass":
        raise RuntimeError(f"positive full correspondence baseline rejected: {positive.get('reason')}")
    baseline_receipt = positive.get("compiled_inputs", {}).get("receipt", {})

    expected_ids = [row["id"] for row in fixture.get("controls", [])]
    results = []
    f = C.load_fixture()

    def extension_case(name: str, old: str, new: str) -> None:
        source = baseline_inputs["extension"]
        if old not in source:
            raise RuntimeError(f"control setup missing extension anchor for {name}")
        mutated = source.replace(old, new, 1)
        results.append(rejected(name, lambda: C.assert_extension_source(mutated, f)))

    def client_case(name: str, old: str, new: str) -> None:
        source = baseline_inputs["client"]
        if old not in source:
            raise RuntimeError(f"control setup missing client anchor for {name}")
        mutated = source.replace(old, new, 1)
        results.append(rejected(name, lambda: C.assert_client_source(mutated, f)))

    def audit_case(name: str, mutate: Callable[[dict], dict]) -> None:
        changed = mutate(copy.deepcopy(baseline_inputs))
        results.append(rejected(name, lambda: C.audit(changed)))

    extension_case("lemma_trusted", "#[check(ghost)]", "#[trusted]\n#[check(ghost)]")
    extension_case("lemma_false_contract", "#[ensures(finite_inventory((*owners).push_back(*peer),*survivor,*after))]",
                   "#[ensures(false)]\n#[ensures(finite_inventory((*owners).push_back(*peer),*survivor,*after))]")
    lemma_old = "    proof_assert!((*after).accepts(*survivor));"
    lemma_new = "    let escaped=(*peer);\n" + lemma_old
    extension_case("lemma_runtime_owner_effect", lemma_old, lemma_new)
    extension_case("lemma_resource_extraction", lemma_old,
                   "    let extracted=(*before).extract();\n" + lemma_old)
    extension_case("lemma_resource_ticket_boundary", "Snapshot<DetachedScope>)", "Ticket<DetachedScope>)")
    extension_case("lemma_inventory_formula", "0<=i && i<(*owners).len() ==>", "0<=i && i<=(*owners).len() ==>")
    extension_case("extra_clone_callback_event", "shallow_clone_arc_checked(shared.cast(),ptr,len,",
                   "shallow_clone_arc_checked(shared.cast(),ptr,len,\n    shallow_clone_arc_checked(shared.cast(),ptr,len,")
    extension_case("empty_vec_receipt_mint", "let _=value;", "free_recovered(value);\n    let _=value;")
    extension_case("empty_vec_precondition_omitted", "#[requires(value@.len()==0)]\nfn empty_vec_terminal_drop",
                   "fn empty_vec_terminal_drop")

    client_case("client_macro_override", "/// Runtime count creates an arbitrary finite inventory.",
                "macro_rules! proof_assert { ($($tokens:tt)*) => {}; }\n/// Runtime count creates an arbitrary finite inventory.")
    client_case("client_omits_inventory_lemma", "prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),\n            old_scope,snapshot!(detached.inner_logic()));",
                "/* omitted lemma */")
    client_case("client_push_precedes_lemma", "ghost! {prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),\n            old_scope,snapshot!(detached.inner_logic()));};\n        owners.push(next);",
                "owners.push(next);\n        ghost! {prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),\n            old_scope,snapshot!(detached.inner_logic()));};")
    client_case("client_extra_clone", "owners.push(next);", "let escaped=clone_surviving_child(&survivor,detached.borrow_mut());\n        owners.push(next);")
    client_case("client_missing_peer_retirement", "bytes_detached_child_terminal_drop(peer,detached.borrow_mut(),peer_receipt.borrow_mut());",
                "proof_assert!(true);")
    client_case("client_missing_inventory_loop_invariant", "#[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]",
                "#[invariant(true)]")
    client_case("client_false_public_post", "#[ensures(result@==input@)]", "#[ensures(false)]")
    client_case("client_unknown_call", "owners.push(next);", "unknown_callback(next);\n        owners.push(next);")
    client_case("client_forget_live_owner", "owners.push(next);", "core::mem::forget(next);\n        owners.push(next);")
    client_case("client_raw_operation", "owners.push(next);", "unsafe { owners.push(next); }")

    audit_case("module_route_alias", lambda d: {**d, "lib": d["lib"].replace(
        "#[path = \"../generated/active.rs\"]", "#[path = \"../generated/elsewhere.rs\"]", 1)})
    audit_case("generated_extension_drift", lambda d: {**d, "helper": d["helper"] + "\n// extra"})
    audit_case("terminal_extension_drift", lambda d: {**d, "terminal_helper": d["terminal_helper"] + "\n// extra"})
    audit_case("active_unreviewed_drop", lambda d: {**d, "active": d["active"] + "\nimpl Drop for Bytes {}\n"})
    audit_case("native_client_wrong_clone_source", lambda d: {**d, "native_source": d["native_source"].replace(
        "owners.push(survivor.clone())", "owners.push(input.clone())", 1)})

    def mapping_mutation(key: str, transform: Callable[[dict], None]) -> Callable[[], object]:
        mapping = copy.deepcopy(baseline_inputs["mapping"])
        transform(mapping)
        return lambda: C.assert_mapping(mapping, baseline_inputs["prefix"], baseline_inputs["extension"],
            baseline_inputs["client"], baseline_inputs["active"], baseline_inputs["capture"], baseline_inputs["mir"])

    results.append(rejected("mapping_creation_backedge_redirect",
        mapping_mutation("loop_regions", lambda m: m["loop_regions"].update(creation_backedge=["bb9", "bb10"]))))
    results.append(rejected("mapping_wrong_peer_drop_owner",
        mapping_mutation("normal_edges", lambda m: m["normal_edges"][1].update(owner="survivor"))))
    results.append(rejected("mapping_broadened_success_claim",
        mapping_mutation("excluded", lambda m: m.update(excluded=[]))))

    def changed_mir(text: str) -> Callable[[], object]:
        return lambda: C.derive_client_mir(text, baseline_inputs["capture"])

    results.append(rejected("mir_hidden_normal_call", changed_mir(baseline_inputs["mir"].replace(
        "StorageDead(_11);", "drop(_26) -> [return: bb15, unwind: bb24];\n        StorageDead(_11);", 1))))
    results.append(rejected("mir_duplicate_peer_drop", changed_mir(baseline_inputs["mir"].replace(
        "drop(_26) -> [return: bb15, unwind: bb24];", "drop(_26) -> [return: bb15, unwind: bb24];\n        drop(_26) -> [return: bb15, unwind: bb24];", 1))))
    results.append(rejected("mir_comment_mutation", changed_mir(baseline_inputs["mir"] + "\n// fake drop(_26)\n")))

    probe_manifest = (ROOT / "Cargo.toml").read_bytes()
    native_manifest = (ROOT / "native-test/Cargo.toml").read_bytes()
    results.append(rejected("probe_manifest_extra_patch_route", lambda: C.assert_probe_manifest(
        {}, probe_manifest + b"\n[patch.crates-io]\ncreusot-std = { path = \"../unreviewed\" }\n", native_manifest)))
    results.append(rejected("probe_manifest_extra_feature", lambda: C.assert_probe_manifest(
        {}, probe_manifest.replace(b"negative_missing_control_free = []",
                                   b"negative_missing_control_free = []\nunchecked = []"), native_manifest)))
    results.append(rejected("native_manifest_target_override", lambda: C.assert_probe_manifest(
        {}, probe_manifest, native_manifest + b"\n[patch.crates-io]\nbytes = { path = \"../unreviewed-bytes\" }\n")))

    for arg in ("--shadow", "--mapping"):
        wrong = ROOT / "generated/not-the-selected-input.json"
        proc = subprocess.run([sys.executable, str(ROOT / "check_correspondence.py"), arg, str(wrong)],
                             cwd=ROOT, capture_output=True, text=True, check=False)
        try:
            payload = json.loads(proc.stdout)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"wrong-route CLI control {arg} emitted no structured rejection") from exc
        if proc.returncode == 0 or payload.get("status") != "reject":
            raise RuntimeError(f"wrong-route CLI control {arg} was accepted")
        results.append({"id": f"cli_{arg[2:]}_alternate_path", "result": "rejected",
                        "reason": payload.get("reason", "structured route rejection")})

    actual_ids = [item["id"] for item in results]
    if actual_ids != expected_ids:
        raise RuntimeError(f"control fixture/list mismatch: fixture={expected_ids}; actual={actual_ids}")
    return {"status": "pass", "controls": len(results), "rejected_as_expected": len(results),
            "solver_invoked": False, "cargo_invoked": False, "source_mutated_on_disk": False,
            "positive_baseline": {"status": "pass", "source_inputs_sha256": actual,
                "manifest": positive.get("manifest"),
                "compiled_artifacts": {"count": positive.get("compiled_inputs", {}).get("captured_actual_build_artifact_count"),
                    "public_records_sha256": baseline_receipt.get("captured_input_sha256"),
                    "fingerprint_sha256": baseline_receipt.get("captured_cargo_build_fingerprint_sha256"),
                    "build_output_sha256": baseline_receipt.get("captured_build_output_sha256"),
                    "root_output_sha256": baseline_receipt.get("captured_root_output_sha256")}},
            "results": results}


def main() -> int:
    result = run()
    result["checker_sha256"] = sha((ROOT / "check_correspondence.py").read_bytes())
    result["fixture_sha256"] = sha(MANIFEST.read_bytes())
    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
