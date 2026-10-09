#!/usr/bin/env python3
"""Replay the one AV refreshed-mapping structural control; no build or prover."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER = ROOT / "check_correspondence.py"
FIXTURE = ROOT / "fixtures/checker-controls.json"
RECEIPT = ROOT / "generated/checker-control-receipt.json"
POSITIVE_MAPPING_SHA256 = "0273f1eb0bf269ec49f494850138fa36a3f160bb972066afaff6141abc3d6834"
EXPECTED_REJECTION = (
    "AV helper inventory, ownership frame, allocation recovery, callback selection, "
    "return order, or exclusions changed"
)

spec = importlib.util.spec_from_file_location("av_structural_control_target", CHECKER)
if spec is None or spec.loader is None:
    raise SystemExit("cannot import AV correspondence checker")
C = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = C
spec.loader.exec_module(C)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def main() -> int:
    fixture_bytes = FIXTURE.read_bytes()
    fixture = json.loads(fixture_bytes)
    require(fixture.get("schema") == "av_suffix_structural_control_v1" and
            len(fixture.get("controls", [])) == 1,
            "AV structural control fixture must contain exactly one control")
    control = fixture["controls"][0]
    require(control == {
        "id": "refreshed_suffix_capacity_recovery_claim",
        "field": "allocation_recovery",
        "replacement": "offset_from(current view, mutable byte buffer) + view length; reduced capacity",
        "expected_rejection": EXPECTED_REJECTION,
    }, "AV structural control differs from the reviewed mapping mutation")

    inputs = C._load_av_inputs(need_capture=False)
    base_mapping = inputs["mapping_bytes"]
    require(sha(base_mapping) == POSITIVE_MAPPING_SHA256,
            "AV control base mapping is not the frozen positive mapping")
    mapping = json.loads(base_mapping)
    mapping["allocation_recovery"] = control["replacement"]
    _, au_files, _ = C.assert_published_au()

    result = C.av_structural_control(inputs, mapping, au_files)
    require(result.get("control_id") == control["id"] and
            result.get("reason") == EXPECTED_REJECTION and
            result.get("status") == "rejected_as_expected",
            "AV structural control did not reject the refreshed allocation-recovery mutation")

    receipt = {
        "schema": "av-suffix-structural-control-v1",
        "status": "pass",
        "control_count": 1,
        "rejected_as_expected": 1,
        "accepted": [],
        "errors": [],
        "control_id": control["id"],
        "expected_rejection": EXPECTED_REJECTION,
        "baseline_positive_mapping_sha256": sha(base_mapping),
        "mutated_mapping_sha256": result["refreshed_mapping_sha256"],
        "checker_sha256": sha(CHECKER.read_bytes()),
        "fixture_sha256": sha(fixture_bytes),
        "control_scope": "one refreshed mapping allocation-recovery forgery through the AV source/native boundary; no Cargo capture or prover",
    }
    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
