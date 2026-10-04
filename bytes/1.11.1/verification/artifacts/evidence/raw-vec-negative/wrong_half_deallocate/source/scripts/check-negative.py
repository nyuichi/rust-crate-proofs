#!/usr/bin/env python3
"""Replay the isolated B2/B3/B4 negative VCs and preserve per-run evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
PROBE = ROOT / "verification/probes/raw-vec-negative"
EVIDENCE = ROOT / "verification/artifacts/evidence/raw-vec-negative"
CASES = (
    ("wrong_half_region", "wrong_half_region_recovery"),
    ("wrong_unknown_prefix", "wrong_unknown_prefix_resume"),
    ("wrong_namespace", "wrong_namespace_resume"),
    ("wrong_unknown_borrow", "wrong_unknown_borrow"),
    ("wrong_out_of_region_borrow", "wrong_out_of_region_borrow"),
    ("wrong_stale_value_after_mutation", "wrong_stale_value_after_mutation"),
    ("wrong_half_deallocate", "wrong_half_deallocate"),
    ("wrong_namespace_deallocate", "wrong_namespace_deallocate"),
)
SOURCE_FILES = {
    "raw_vec.rs": ROOT / "src/ownership_proof/raw_vec.rs",
    "owned_region.rs": ROOT / "src/ownership_proof/owned_region.rs",
    "probe-lib.rs": PROBE / "src/lib.rs",
    "Cargo.toml": PROBE / "Cargo.toml",
    "Cargo.lock": PROBE / "Cargo.lock",
    "why3find.json": PROBE / "why3find.json",
    "scripts/check-negative.py": PROBE / "scripts/check-negative.py",
    "scripts/check-affine.py": PROBE / "scripts/check-affine.py",
}


def write_bytes_atomic(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def replace_proof_artifacts(source: Path, destination: Path) -> bool:
    destination.parent.mkdir(parents=True, exist_ok=True)
    staging_parent = Path(tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent))
    staging = staging_parent / "tree"
    backup = destination.with_name(f".{destination.name}.old")
    try:
        staging.mkdir()
        copied = 0
        for source_file in source.rglob("*"):
            if not source_file.is_file() or not (source_file.suffix == ".coma" or source_file.name == "proof.json"):
                continue
            target_file = staging / source_file.relative_to(source)
            target_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source_file, target_file)
            copied += 1
        if copied == 0:
            return False
        if backup.exists():
            shutil.rmtree(backup)
        if destination.exists():
            os.replace(destination, backup)
        try:
            os.replace(staging, destination)
        except BaseException:
            if backup.exists() and not destination.exists():
                os.replace(backup, destination)
            raise
        if backup.exists():
            shutil.rmtree(backup)
        return True
    finally:
        if staging_parent.exists():
            shutil.rmtree(staging_parent)


def snapshot_sources(destination: Path) -> None:
    hashes = {}
    destination.mkdir(parents=True, exist_ok=True)
    source_root = destination / "source"
    staging_parent = Path(tempfile.mkdtemp(prefix=".source-snapshot.", dir=destination))
    staging = staging_parent / "source"
    staging.mkdir()
    backup = destination / ".source.old"
    try:
        for relative, source in SOURCE_FILES.items():
            data = source.read_bytes()
            write_bytes_atomic(staging / relative, data)
            hashes[relative] = hashlib.sha256(data).hexdigest()
        if backup.exists():
            shutil.rmtree(backup)
        if source_root.exists():
            os.replace(source_root, backup)
        try:
            os.replace(staging, source_root)
        except BaseException:
            if backup.exists() and not source_root.exists():
                os.replace(backup, source_root)
            raise
        if backup.exists():
            shutil.rmtree(backup)
    finally:
        if staging_parent.exists():
            shutil.rmtree(staging_parent)
    write_bytes_atomic(
        destination / "source-hashes.json",
        (json.dumps({"files": hashes}, indent=2) + "\n").encode(),
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("features", nargs="*", help="subset of features to replay; default: all eight")
    args = parser.parse_args()
    selected = set(args.features)
    known = {feature for feature, _ in CASES}
    unknown = selected - known
    if unknown:
        parser.error("unknown feature(s): " + ", ".join(sorted(unknown)))
    cases = tuple(case for case in CASES if not selected or case[0] in selected)

    failures = []
    attempted = []
    log_dir = PROBE / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    for index, (feature, function) in enumerate(cases):
        attempted.append(feature)
        command = [
            str(ROOT / "scripts/verify-bytes.sh"),
            "raw-vec-negative",
            "--no-default-features",
            "--features",
            feature,
        ]
        evidence = EVIDENCE / feature
        snapshot_sources(evidence)
        run = subprocess.run(
            command,
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        output = run.stdout
        log_path = log_dir / f"{feature}.log"
        write_bytes_atomic(
            log_path,
            ("$ " + " ".join(command) + "\n" + output + f"\nexit status: {run.returncode}\n").encode(),
        )

        generated = PROBE / "verif"
        proof_archive = evidence / "proof-artifacts"
        archived_fresh_proof = generated.is_dir() and replace_proof_artifacts(
            generated, proof_archive
        )
        if not archived_fresh_proof and proof_archive.exists():
            shutil.rmtree(proof_archive)

        expected_goal = f"Coma.vc_{function}"
        failed_goals = re.findall(
            r"(?m)^Goal (Coma\.vc_[A-Za-z0-9_]+): ✘(?: \([^\n]*\))?$", output
        )
        has_compiler_error = bool(
            re.search(r"(?m)^error(?::|\[E\d+\])", output)
            or "Compilation failed" in output
            or "could not compile" in output
        )
        expected_coma = (
            list(proof_archive.rglob(f"{function}.coma"))
            if archived_fresh_proof
            else []
        )
        expected_proof = (
            list(proof_archive.rglob(f"{function}/proof.json"))
            if archived_fresh_proof
            else []
        )
        valid = (
            run.returncode != 0
            and failed_goals == [expected_goal]
            and "unproved file" in output
            and not has_compiler_error
            and len(expected_coma) == 1
            and len(expected_proof) == 1
        )
        label = "intended VC rejected" if valid else "UNEXPECTED COMPILER/PROOF RESULT"
        print(f"{feature}: {label}; failed goals={failed_goals}; log {log_path}", flush=True)
        if not valid:
            failures.append(feature)
        if has_compiler_error:
            first_error = next(
                (
                    line.strip()
                    for line in output.splitlines()
                    if re.match(r"(?:error(?::|\[E\d+\])|Error: Compilation failed)", line.strip())
                ),
                "compiler/translation failure (see full log)",
            )
            print(f"Stopping after shared compile failure: {first_error}", file=sys.stderr)
            unattempted = [case[0] for case in cases[index + 1 :]]
            if unattempted:
                print("Not attempted: " + ", ".join(unattempted), file=sys.stderr)
            break

    if failures:
        print("Unexpected result for: " + ", ".join(failures), file=sys.stderr)
        return 1
    print(f"Validated {len(attempted)} isolated negative feature(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
