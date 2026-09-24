#!/usr/bin/env python3
"""Verify the no-default-features codec without a Why3 socket server.

Requires Creusot 0.13.0, its Why3 revision and prelude, and Z3 4.15.3.
Set WHY3, Z3, CREUSOT_PRELUDE and WHY3_Z3_DRIVER if not in standard locations.
"""
import concurrent.futures
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import tempfile


def main():
    os.chdir(Path(__file__).resolve().parent)
    why3 = os.environ.get("WHY3", "why3")
    z3 = os.environ.get("Z3", "z3")
    data = Path(os.environ.get("CREUSOT_DATA_HOME", Path.home() / ".local/share/creusot"))
    prelude = os.environ.get("CREUSOT_PRELUDE", str(data / "share/why3find/packages/creusot"))
    driver = os.environ.get("WHY3_Z3_DRIVER", "z3_4_12")
    # These are generated files only. A fresh build prevents stale goals/results.
    shutil.rmtree("verif/hex_rlib", ignore_errors=True)
    with tempfile.TemporaryDirectory(prefix="hex-core-build-") as build:
        env = dict(os.environ, CARGO_TARGET_DIR=build)
        subprocess.run(["cargo", "creusot", "--only", "coma", "--", "--locked", "--no-default-features"], env=env, check=True)
    sources = sorted(Path("verif/hex_rlib").rglob("*.coma"))
    if not sources:
        raise RuntimeError("No generated proof files")
    out = Path("target/core-proof")
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    tasks = []
    exported = []
    for i, source in enumerate(sources):
        dest = out / str(i)
        dest.mkdir()
        run = subprocess.run([why3, "prove", "-L", prelude, "-L", "verif", "-a", "split_vc", "-D", driver, "-o", str(dest), str(source)], text=True, capture_output=True)
        (dest / "export.log").write_text(run.stdout + run.stderr)
        run.check_returncode()
        goals = sorted(dest.glob("*.smt2"))
        exported.append({"source": str(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest(), "goals": len(goals)})
        tasks.extend((source, goal) for goal in goals)
    if not tasks:
        raise RuntimeError("No proof obligations exported")

    def solve(task):
        source, goal = task
        cmd = [z3, "-smt2", "-T:10", "sat.random_seed=42", "nlsat.randomize=false", "smt.random_seed=42", str(goal)]
        try:
            run = subprocess.run(cmd, text=True, capture_output=True, timeout=15)
            result = {"exit_code": run.returncode, "stdout": run.stdout, "stderr": run.stderr}
            # Z3 may discard unsupported quantifier triggers; this changes
            # search heuristics, not the asserted formula. Preserve warnings.
            warnings_only = all(re.fullmatch(r"WARNING: \(\d+,\d+\): '(?:if|and|or)' cannot be used in patterns\.", line) for line in run.stderr.splitlines())
            passed = run.returncode == 0 and run.stdout.strip() == "unsat" and warnings_only
        except subprocess.TimeoutExpired:
            result, passed = {"stdout": "timeout"}, False
        return {"source": str(source), "goal": str(goal), "sha256": hashlib.sha256(goal.read_bytes()).hexdigest(), "command": cmd, "passed": passed, **result}

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(solve, tasks))
    summary = {"method": "Why3 split_vc export, direct Z3 execution; no why3find session", "files": exported, "results": results}
    (out / "results.json").write_text(json.dumps(summary, indent=2) + "\n")
    failed = [r for r in results if not r["passed"]]
    print(f"Proved {len(results)-len(failed)}/{len(results)} obligations from {len(sources)} Coma files", flush=True)
    for r in failed:
        print(r)
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
