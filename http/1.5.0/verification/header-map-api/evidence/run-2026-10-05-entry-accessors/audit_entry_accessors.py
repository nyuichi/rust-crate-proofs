#!/usr/bin/env python3
"""Check Entry accessor proof JSON against solver-free COMA printer tasks."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
MANIFEST = HERE / "manifest.json"
ARTIFACTS = HERE / "artifacts/verif/http_header_map_api_proof_rlib"
WHY3_CONFIG = "/workspace/proof-tools/config/creusot/why3.conf"
CREUSOT_LIB = "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot"

manifest = json.loads(MANIFEST.read_text())
results = []
for target in manifest["proof_run"]["targets"]:
    rel = target["relative_target"]
    coma = ARTIFACTS / (rel + ".coma")
    proof_json = ARTIFACTS / rel / "proof.json"
    command = [
        "why3", "prove", "-C", WHY3_CONFIG, "-L", CREUSOT_LIB,
        "-D", "why3", str(coma),
    ]
    run = subprocess.run(command, text=True, capture_output=True, check=False)
    if run.returncode != 0:
        raise SystemExit(f"solver-free COMA print failed for {rel}: {run.returncode}\n{run.stderr[-2000:]}")
    goals = re.findall(r"(?m)^goal\s+([^\s:]+)\s*:", run.stdout)
    data = json.loads(proof_json.read_text())["proofs"]["Coma"]
    expected = {target["own_task_key"], *target["support_task_keys"]}
    if set(goals) != expected:
        raise SystemExit(f"{rel}: printed task names {goals}, proof JSON keys {sorted(data)}, expected {sorted(expected)}")
    if len(goals) != len(data):
        raise SystemExit(f"{rel}: printed {len(goals)} tasks but JSON has {len(data)} roots")
    for key, node in data.items():
        if not isinstance(node, dict) or not node.get("prover") or "children" in node:
            raise SystemExit(f"{rel}:{key}: unresolved or nested proof node {node!r}")
    results.append({
        "target": rel,
        "command": "why3 prove -C <why3.conf> -L <creusot package> -D why3 <archived COMA>; no prover selected",
        "solver_invoked": False,
        "coma_sha256": hashlib.sha256(coma.read_bytes()).hexdigest(),
        "proof_json_sha256": hashlib.sha256(proof_json.read_bytes()).hexdigest(),
        "printer_stdout_sha256": hashlib.sha256(run.stdout.encode()).hexdigest(),
        "printer_stderr_sha256": hashlib.sha256(run.stderr.encode()).hexdigest(),
        "printed_task_names": goals,
        "printed_task_count": len(goals),
        "json_root_names": sorted(data),
        "json_root_count": len(data),
        "all_roots_direct_success_leaves": True,
    })
audit = {
    "method": "For each archived COMA, print the untransformed Why3 goals with no prover (matching the proof run, which used no split_vc preprocessing); compare every printed goal name against the archived proof JSON root keys and reject nested/null leaves.",
    "solver_usage": "No solver, proof server, or proof cache was used.",
    "selected_targets": len(results),
    "own_leaf_count": sum(t["own_successful_leaves"] for t in manifest["proof_run"]["targets"]),
    "support_leaf_count": sum(t["support_successful_leaves"] for t in manifest["proof_run"]["targets"]),
    "total_leaf_count": sum(t["total_successful_leaves"] for t in manifest["proof_run"]["targets"]),
    "results": results,
}
(HERE / "recursive-vc-arity-audit.json").write_text(json.dumps(audit, indent=2, sort_keys=True) + "\n")
manifest["independent_arity_audit"] = {
    "command": "python3 audit_entry_accessors.py (after source /workspace/proof-tools/activate.sh)",
    "solver_used": False,
    "result": "PASS: every COMA printer task name and count exactly matches its proof JSON roots; all roots are direct successful prover leaves.",
    "audit_sha256": hashlib.sha256((HERE / "recursive-vc-arity-audit.json").read_bytes()).hexdigest(),
}
MANIFEST.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
print(f"PASS targets={len(results)} own={audit['own_leaf_count']} support={audit['support_leaf_count']} total={audit['total_leaf_count']}")
for result in results:
    print(f"{result['target']}: {result['printed_task_count']} tasks; {', '.join(result['printed_task_names'])}")
