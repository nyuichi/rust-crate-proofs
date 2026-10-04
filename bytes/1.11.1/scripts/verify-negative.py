#!/usr/bin/env python3
"""Require intended unproved VCs, never syntax/setup failures, for probe negatives."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS = ROOT / "verification" / "artifacts"
LOGS = ARTIFACTS / "logs"
EVIDENCE = ARTIFACTS / "evidence"

# Each feature and goal names a real cfg-gated negative in that probe's src/lib.rs.
# wide-codecs now has its own wrong-order encoder and decoder controls, so both
# are exercised here rather than being treated as a positive-only component.
CASES = (
    ("helpers", "wrong_postcondition", "wrong_postcondition"),
    ("helpers", "reachable_false", "reachable_false"),
    ("storage", "wrong_byte", "wrong_byte"),
    ("storage", "missing_ownership", "recover_without_ownership"),
    ("deallocation", "wrong_layout", "wrong_layout"),
    ("deallocation", "wrong_capacity", "wrong_capacity"),
    ("bounded-ops", "wrong_prefix", "wrong_prefix"),
    ("slice-ops", "wrong_content", "wrong_content"),
    ("slice-ops", "wrong_advance", "wrong_advance"),
    ("slice-ops", "out_of_bounds", "out_of_bounds_advance"),
    ("cursor-ops", "wrong_position", "wrong_position"),
    ("cursor-ops", "overflow_example", "unguarded_overflow"),
    ("byte-codecs", "wrong_postcondition", "wrong_postcondition"),
    ("byte-codecs", "reachable_false", "reachable_false"),
    ("comparison-ops", "wrong_equal", "wrong_equal"),
    ("chain-ops", "wrong_overflow", "wrong_overflow"),
    ("chain-ops", "wrong_split", "wrong_split"),
    ("capacity-ops", "wrong_repr", "wrong_repr"),
    ("capacity-ops", "wrong_reconstruction", "wrong_reconstruction"),
    ("capacity-ops", "lostflag", "lostflag"),
    ("capacity-ops", "wrongpos", "wrongpos"),
    ("slice-read-ops", "wrong_value", "wrong_value"),
    ("slice-read-ops", "short_consumed", "consumes_short_u16"),
    ("initialized-storage", "wrong_byte", "wrong_write_contract"),
    ("initialized-storage", "wrong_ownership", "recover_without_ownership"),
    ("uninit-ops", "wrong_init", "wrong_init"),
    ("uninit-ops", "uninitialized_recovery", "uninitialized_recovery"),
    ("wide-codecs", "wrong_wide_encode", "wrong_wide_encode_contract"),
    ("wide-codecs", "wrong_wide_decode", "wrong_wide_decode_contract"),
    ("endian-ops", "wrong_endian", "wrong_endian"),
    ("endian-ops", "wrong_signed", "wrong_signed"),
    ("region-permissions", "wrong_byte", "wrong_byte"),
    ("region-permissions", "wrong_overlap", "wrong_overlap"),
    ("slice-wide-read-ops", "wrong_value", "wrong_value"),
    ("slice-wide-read-ops", "short_consumed", "consumes_short_input"),
    ("variable-read-ops", "wrong_value", "wrong_value"),
    ("variable-read-ops", "wrong_short_consumed", "wrong_short_consumed"),
    ("signed-wide-ops", "wrong_signed", "wrong_signed_be_i64"),
)


def write_bytes_atomic(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def write_text_atomic(path: Path, text: str) -> None:
    write_bytes_atomic(path, text.encode("utf-8"))


def main() -> int:
    LOGS.mkdir(parents=True, exist_ok=True)
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    results = []

    for target, feature, goal in CASES:
        command = [str(ROOT / "scripts" / "verify-bytes.sh"), target, "--features", feature]
        run = subprocess.run(
            command,
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        log_path = LOGS / f"{target}-{feature}.log"
        write_text_atomic(log_path, run.stdout)
        intended = (
            run.returncode != 0
            and re.search(r"Goal Coma\.vc_" + re.escape(goal) + r"\b.*✘", run.stdout)
            and "unproved file" in run.stdout
            and "Compilation failed" not in run.stdout
        )
        if not intended:
            raise SystemExit(f"{target}:{feature}: expected VC rejection missing; see {log_path}")

        generated = ROOT / "verification" / "probes" / target / "verif"
        hashes = {}
        for source in generated.rglob("*.coma"):
            if not source.is_file():
                continue
            destination = EVIDENCE / f"{target}-{feature}" / source.relative_to(generated)
            data = source.read_bytes()
            write_bytes_atomic(destination, data)
            hashes[destination.relative_to(ROOT).as_posix()] = hashlib.sha256(data).hexdigest()

        results.append({
            "target": target,
            "feature": feature,
            "expected_goal": goal,
            "exit_code": run.returncode,
            "command": command,
            "log": log_path.relative_to(ROOT).as_posix(),
            "vc_hashes": hashes,
        })
        print(f"{target}:{feature}: intended VC rejected", flush=True)

    write_text_atomic(
        ARTIFACTS / "negative-results.json",
        json.dumps(results, indent=2) + "\n",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
