#!/usr/bin/env python3
"""One refreshed-mapping control for AU's named direct-call binding gate.

The input is cloned in memory. This does not edit proof inputs, invoke Cargo,
or run Creusot/Why3. The positive full correspondence run is a separate gate.
"""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import pathlib
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_correspondence.py"
FIXTURE_PATH = ROOT / "fixtures/checker-controls.json"
RECEIPT_PATH = ROOT / "generated/checker-controls-receipt.json"
EXPECTED_POSITIVE_MAPPING_SHA256 = "07e58146a8881572db20d3d9d18d99246f4e6911c5277cd56a86e102d559ecd7"
EXPECTED_REJECTION = "named-call mapping field 'native_callee' differs from native source/MIR facts"

spec = importlib.util.spec_from_file_location("au_checker_controls_target", CHECKER_PATH)
if spec is None or spec.loader is None:
    raise SystemExit("cannot import the AU correspondence checker")
C = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = C
spec.loader.exec_module(C)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def main() -> int:
    fixture_bytes = FIXTURE_PATH.read_bytes()
    fixture = json.loads(fixture_bytes)
    require(fixture.get("schema") == "au_named_direct_call_control_v1" and
            len(fixture.get("controls", [])) == 1,
            "AU control fixture must contain exactly one named-summary mutation")
    control = fixture["controls"][0]
    require(control.get("id") == "refreshed_native_callee_identity" and
            control.get("field") == "named_call_summaries[0].native_callee" and
            control.get("replacement") == "forged_clone_suffix" and
            control.get("expected_rejection") == EXPECTED_REJECTION,
            "AU forged-summary fixture differs from the reviewed control")
    inputs = C._load_inputs(need_capture=False)
    base_mapping = inputs["mapping_bytes"]
    require(sha(base_mapping) == EXPECTED_POSITIVE_MAPPING_SHA256,
            "AU control base mapping is not the frozen positive mapping")
    mutated = copy.deepcopy(json.loads(base_mapping))
    summaries = mutated.get("named_call_summaries")
    require(isinstance(summaries, list) and len(summaries) == 1 and
            isinstance(summaries[0], dict),
            "AU control cannot resolve the single named-call summary row")
    # Serialize a fresh mapping document after the field mutation. The attack
    # therefore presents a fully refreshed mapping byte stream, not a stale
    # hash or malformed JSON receipt.
    summaries[0]["native_callee"] = control["replacement"]
    inputs["mapping_bytes"] = C.mapping_json(mutated)
    mutated_mapping_sha = sha(inputs["mapping_bytes"])
    try:
        C.audit_inputs(inputs, cargo_mode="skip")
    except C.CheckError as exc:
        reason = str(exc)
        require(EXPECTED_REJECTION in reason,
                f"AU forged-summary control rejected at an unrelated gate: {reason}")
    else:
        raise RuntimeError("AU correspondence admitted a refreshed forged native callee identity")
    receipt: dict[str, Any] = {
        "status": "pass",
        "control_count": 1,
        "rejected_as_expected": 1,
        "accepted": [],
        "errors": [],
        "baseline_positive_mapping_sha256": sha(base_mapping),
        "mutated_mapping_sha256": mutated_mapping_sha,
        "expected_rejection": EXPECTED_REJECTION,
        "control_id": control["id"],
        "checker_sha256": sha(CHECKER_PATH.read_bytes()),
        "fixture_sha256": sha(fixture_bytes),
        "control_scope": "one in-memory refreshed native-callee identity forgery through the complete source/native/proof-reuse audit; no Cargo capture or prover",
    }
    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
