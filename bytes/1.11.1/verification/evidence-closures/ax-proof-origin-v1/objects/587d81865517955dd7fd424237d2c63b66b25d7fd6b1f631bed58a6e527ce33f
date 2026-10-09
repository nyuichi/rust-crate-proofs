#!/usr/bin/env python3
"""One in-memory AX source/native boundary control; no Cargo or prover."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import pathlib
import sys

SCRIPT = pathlib.Path(__file__).resolve()
_candidate_root = SCRIPT.parents[2] if len(SCRIPT.parents) > 2 else pathlib.Path("/")
ROOT = (_candidate_root if (_candidate_root / "src/promotion.rs").is_file() else
    pathlib.Path("/workspace/bytes-work/bytes/1.11.1/verification/probes/original-raw-suffix-drop-2026-10-09"))
FIXTURE = (SCRIPT.parent / "fixture.json" if (SCRIPT.parent / "fixture.json").is_file() else
    pathlib.Path("/workspace/work/ax-raw-free-control.json"))
CHECKER_SHA = "f63fcc1b6382be16eff6cf31a3ea4eb05a5648c615f2d681d1069817878c0922"
NATIVE_SHA = "309e0dd8a807c1c96f3b791663d62e11d4bc08bcdf01e9fd9d482f320e5b2320"


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def main() -> int:
    fixture = json.loads(FIXTURE.read_text())
    assert fixture["schema"] == "ax-source-correspondence-control-v1"
    checker_raw = (ROOT / "check_correspondence.py").read_bytes()
    native_raw = (ROOT / "check_native.py").read_bytes()
    assert sha(checker_raw) == CHECKER_SHA, "AX correspondence checker changed"
    assert sha(native_raw) == NATIVE_SHA, "AX native checker changed"

    spec = importlib.util.spec_from_file_location("ax_control_checker", ROOT / "check_correspondence.py")
    assert spec is not None and spec.loader is not None
    checker = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = checker
    spec.loader.exec_module(checker)
    native = checker.import_native_checker()
    native_report = native.audit_bundle(native.load_bundle())

    source_path = ROOT / "src/promotion.rs"
    original = source_path.read_bytes()
    actual_pin = sha(original)
    assert actual_pin == "9440ac43f7a116e4bd36b5affb449334be8872e34f9e9cb7a9dc0c923311d414"
    marker = (b"unsafe {physical_projection::deallocate(\n"
              b"        base,capacity,ghost! {descriptor.base},ghost! {(recovery.into_inner(),physical.into_inner())})}")
    assert original.count(marker) == 1, "physical-free mutation anchor is not unique"
    mutant = original.replace(marker, b"Ghost::conjure()", 1)
    mutant_hash = sha(mutant)

    mapping = json.loads((ROOT / "generated/mapping.json").read_text())
    mapping = copy.deepcopy(mapping)
    mapping["source"]["promotion_sha256"] = mutant_hash
    mapping["source"]["appended_bytes_sha256"] = sha(mutant[len((ROOT.parent / "original-promotable-suffix-promotion-2026-10-09/generated/positive.rs").read_bytes()):])
    mapping["source"]["bindings"]["promotion.rs"] = mutant_hash
    suffix_start = mutant.find(b"fn free_raw_suffix_checked")
    assert suffix_start >= 0
    mapping["source"]["item_suffix_hashes"]["free_raw_suffix_checked"] = sha(mutant[suffix_start:])

    original_read = checker.read_file
    original_pin = checker.AX_PROMOTION_SHA
    resolved_target = source_path.resolve()

    def in_memory_read(path: pathlib.Path, label: str) -> bytes:
        if pathlib.Path(path).resolve() == resolved_target:
            return mutant
        return original_read(path, label)

    checker.read_file = in_memory_read
    checker.AX_PROMOTION_SHA = mutant_hash
    try:
        try:
            checker.source_correspondence(mapping, native_report)
        except checker.CheckError as exc:
            reason = str(exc)
            assert fixture["expected_rejection"] in reason, f"wrong rejection gate: {reason}"
        else:
            raise AssertionError("refreshed-hash physical-free omission was accepted")
    finally:
        checker.read_file = original_read
        checker.AX_PROMOTION_SHA = original_pin

    print(json.dumps({"status": "pass", "control_id": fixture["id"],
        "checker_sha256": CHECKER_SHA, "native_checker_sha256": NATIVE_SHA,
        "control_script_sha256": sha(SCRIPT.read_bytes()), "fixture_sha256": sha(FIXTURE.read_bytes()),
        "source_before_sha256": actual_pin, "source_mutant_sha256": mutant_hash,
        "expected_rejection": fixture["expected_rejection"], "rejected_as_expected": True,
        "cargo_or_prover_run": False}, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
