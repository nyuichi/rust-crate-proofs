#!/usr/bin/env python3
"""Drop only the named Power_sum premise from the saved actual SMT task."""

from pathlib import Path
import gzip
import os
import resource
import subprocess
import sys
import tempfile


here = Path(__file__).resolve().parent
task = here.parent / "step3-why3-pruning/mulhi-product-split-before.smt2.gz"
z3 = os.environ.get("Z3_BIN", "/tmp/creusot-data/bin/z3")
raw = gzip.decompress(task.read_bytes()).decode()

start = raw.index(';; "Power_sum"')
end = raw.index(';; "pow2pos"', start)
removed = raw[start:end]
assert removed.count("(assert") == 1 and "(forall" in removed

stronger_task = raw[:start] + raw[end:]


def limits():
    resource.setrlimit(resource.RLIMIT_AS, (1024 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_CPU, (12, 12))


with tempfile.TemporaryDirectory(prefix="mulhi-power-sum-") as temporary:
    output = Path(temporary) / "before_without_Power_sum.smt2"
    output.write_text(stronger_task)
    command = [
        z3,
        "-smt2",
        "-T:10",
        "sat.random_seed=42",
        "nlsat.randomize=false",
        "smt.random_seed=42",
        "-st",
        str(output),
    ]
    result = subprocess.run(
        command, capture_output=True, text=True, timeout=15, preexec_fn=limits
    )

transcript = result.stdout + result.stderr
print(transcript, end="" if transcript.endswith("\n") else "\n")
status = next((line.strip() for line in transcript.splitlines() if line.strip()), "")
if result.returncode != 0 or status != "unsat":
    raise SystemExit(result.returncode or 1)
