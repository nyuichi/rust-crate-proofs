#!/usr/bin/env python3
"""Check only the Rust move errors in the isolated affine compile-fail fixtures."""
from __future__ import annotations

import fcntl
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / "verification/probes/raw-vec-negative/affine-fixtures"
CASES = ("duplicate_descriptor", "use_after_transfer")
LOCK = Path(os.environ.get("BYTES_PROOF_LOCK", "/tmp/itoa-creusot-proof.lock"))


def write_text_atomic(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def main() -> int:
    tool_root = Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools"))
    activate = tool_root / "activate.sh"
    if not activate.is_file():
        print(f"missing pinned tool environment: {activate}", file=sys.stderr)
        return 2

    lock_parent = LOCK.parent
    lock_parent.mkdir(parents=True, exist_ok=True)
    failures = []
    with LOCK.open("a+") as lock_stream:
        fcntl.flock(lock_stream, fcntl.LOCK_EX)
        for feature in CASES:
            shell = (
                'set -euo pipefail; source "$1/activate.sh"; '
                'export CARGO_NET_OFFLINE=true; rustc --version; '
                'cargo check --offline --locked --manifest-path "$2" '
                '--no-default-features --features "$3"'
            )
            command = [
                "bash",
                "-c",
                shell,
                "affine-check",
                str(tool_root),
                str(FIXTURES / "Cargo.toml"),
                feature,
            ]
            run = subprocess.run(
                command,
                cwd=ROOT,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            output = run.stdout
            log_path = FIXTURES / "logs" / f"{feature}.log"
            write_text_atomic(
                log_path,
                "$ " + " ".join(command) + "\n" + output + f"\nexit status: {run.returncode}\n",
            )
            errors = re.findall(r"(?m)^error\[E(\d+)\]:", output)
            valid = run.returncode != 0 and errors == ["0382"] and "could not compile" in output
            label = "expected E0382 move error" if valid else f"unexpected compile result (E-codes={errors})"
            print(f"{feature}: {label}; log {log_path}", flush=True)
            if not valid:
                failures.append(feature)

    if failures:
        print("Unexpected result for: " + ", ".join(failures), file=sys.stderr)
        return 1
    print(f"Validated {len(CASES)} Rust compile-fail fixture(s); no program was executed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
