#!/usr/bin/env python3
"""Audit recorded URI proof trees and first-level Why3 split arities.

The JSON recursion recomputes recorded proof-tree counts. The direct Why3
transformation independently checks the first split level against COMA.
Nested child split arity is deliberately not claimed here; it requires replaying
only those individual nested nodes, not a global second split.
"""
import hashlib
import json
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
MANIFEST = json.loads((HERE / "manifest.json").read_text())
WHY3 = Path("/workspace/proof-tools/creusot-data/bin/why3")
WHY3_CONFIG = Path("/workspace/proof-tools/config/creusot/why3.conf")
CREUSOT_LIB = Path("/workspace/proof-tools/creusot-data/share/why3find/packages/creusot")
WHY3_COMMAND = [
    str(WHY3), "prove", "-C", str(WHY3_CONFIG), "-L", str(CREUSOT_LIB),
    "-a", "split_vc", "-D", "why3",
]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def audit(node, path):
    if node is None:
        return {"shape": "null", "null_paths": [path], "terminal_leaves": 1, "passed": 0, "null": 1}
    if "prover" in node:
        return {
            "shape": {"prover": node["prover"]},
            "null_paths": [], "terminal_leaves": 1, "passed": 1, "null": 0,
        }
    children = node.get("children", [])
    audited = [audit(child, path + [i]) for i, child in enumerate(children)]
    return {
        "shape": {"children": [x["shape"] for x in audited]},
        "child_arity": len(children),
        "null_paths": [p for x in audited for p in x["null_paths"]],
        "terminal_leaves": sum(x["terminal_leaves"] for x in audited),
        "passed": sum(x["passed"] for x in audited),
        "null": sum(x["null"] for x in audited),
    }

result = {
    "method": "Recompute archived JSON tree counts, then use Why3 split_vc with the printer driver and no prover to independently verify first-level child arities against each archived COMA.",
    "solver_usage": "No prover, solver server, or proof cache was invoked.",
    "targets": {},
}
recorded_summary = {"targets": 0, "own_terminal_leaves": 0, "passed": 0, "null": 0}
first_level_summary = {"expected_child_tasks": 0, "why3_child_tasks": 0}
for target in MANIFEST["proof_batch"]["target_counts"]:
    name = target["target"]
    archive_name = name.replace("/", "__")
    coma = HERE / "comas" / f"{archive_name}.coma"
    proof = HERE / "proofs" / f"{archive_name}.json"
    if sha(coma) != target["coma_sha256"] or sha(proof) != target["proof_json_sha256"]:
        raise SystemExit(f"archive hash mismatch: {name}")
    proof_data = json.loads(proof.read_text())["proofs"]["Coma"]
    obligations = {key: audit(node, [key]) for key, node in proof_data.items()}
    leaves = sum(x["terminal_leaves"] for x in obligations.values())
    passed = sum(x["passed"] for x in obligations.values())
    nulls = sum(x["null"] for x in obligations.values())
    expected = (target["own_terminal_leaves"], target["passed"], target["null"])
    if (leaves, passed, nulls) != expected:
        raise SystemExit(f"recorded JSON tree mismatch for {name}: {(leaves, passed, nulls)} != {expected}")

    roots = [(key, node) for key, node in proof_data.items() if node.get("children")]
    run = subprocess.run(WHY3_COMMAND + [str(coma)], capture_output=True, text=True)
    if run.returncode:
        raise SystemExit(f"Why3 transform failed for {name}: {run.stderr[-1000:]}")
    task_blocks = re.findall(r"(?ms)^theory Task\n.*?^end\s*$", run.stdout)
    labels = []
    for block in task_blocks:
        match = re.search(r"^goal (.+?)\s*:", block, re.M)
        if not match:
            raise SystemExit(f"unlabeled transformed task for {name}")
        labels.append(match.group(1).strip())
    root_counts = {}
    for key, node in roots:
        expected_arity = len(node["children"])
        actual_arity = sum(label == key for label in labels)
        if expected_arity != actual_arity:
            raise SystemExit(f"first-level arity mismatch {name}:{key}: JSON={expected_arity}, Why3={actual_arity}")
        root_counts[key] = {"json_child_arity": expected_arity, "why3_task_count": actual_arity}
        first_level_summary["expected_child_tasks"] += expected_arity
        first_level_summary["why3_child_tasks"] += actual_arity
    if len(roots) != 1:
        raise SystemExit(f"expected one selected own root in {name}, got {len(roots)}")

    result["targets"][name] = {
        "coma_sha256": sha(coma),
        "proof_json_sha256": sha(proof),
        "recorded_top_level_obligation_count": len(obligations),
        "recorded_own_terminal_leaves": leaves,
        "recorded_passed": passed,
        "recorded_null": nulls,
        "recorded_null_paths": {key: value["null_paths"] for key, value in obligations.items() if value["null_paths"]},
        "first_level_coma_split": {
            "command": "why3 prove -C <why3.conf> -L <creusot package> -a split_vc -D why3 <archived COMA>",
            "stdout_sha256": hashlib.sha256(run.stdout.encode()).hexdigest(),
            "stderr_sha256": hashlib.sha256(run.stderr.encode()).hexdigest(),
            "task_count": len(task_blocks),
            "root_arities": root_counts,
        },
        "obligations": obligations,
    }
    recorded_summary["targets"] += 1
    recorded_summary["own_terminal_leaves"] += leaves
    recorded_summary["passed"] += passed
    recorded_summary["null"] += nulls

expected_summary = {key: MANIFEST["proof_batch"][key] for key in ("own_terminal_leaves", "passed", "null")}
if {key: recorded_summary[key] for key in expected_summary} != expected_summary:
    raise SystemExit(f"recorded batch count mismatch: {recorded_summary}")
if first_level_summary["expected_child_tasks"] != first_level_summary["why3_child_tasks"]:
    raise SystemExit(f"first-level total mismatch: {first_level_summary}")
result["recorded_json_tree_summary"] = recorded_summary
result["independently_checked_first_level_summary"] = first_level_summary
result["nested_arity_limit"] = "Nested children recorded in proof JSON are not independently checked by this script. A global second split would also split sibling goals; replay only the individual nested parent tasks before claiming their expected arity."
result["why3_version"] = subprocess.run([str(WHY3), "--version"], capture_output=True, text=True, check=True).stdout.strip()
result["why3_executable_sha256"] = sha(WHY3)
result["why3_config_sha256"] = sha(WHY3_CONFIG)
out = HERE / "recursive-vc-arity-audit.json"
out.write_text(json.dumps(result, indent=2) + "\n")
print(out)
print(json.dumps({"recorded_json_tree": recorded_summary, "first_level_coma_split": first_level_summary}, sort_keys=True))
print(sha(out))
