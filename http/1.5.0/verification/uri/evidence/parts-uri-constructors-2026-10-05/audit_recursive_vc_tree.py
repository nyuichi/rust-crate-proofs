#!/usr/bin/env python3
"""Recompute recorded URI proof tree arities from immutable archived JSON.

This does not independently establish that the JSON contains every split_vc
child expected from the archived COMA; that completeness audit is pending.
"""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
MANIFEST = json.loads((HERE / "manifest.json").read_text())

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
    "method": "Recursive walk over archived proof JSON child arrays; this independently recomputes recorded tree counts, not expected arity from COMA. No solver or proof cache is invoked.",
    "source": "Each proof JSON and corresponding COMA file is hash-checked against the manifest before counting. The COMA hash pins identity but this script does not transform COMA into expected proof tasks.",
    "targets": {},
}
for target in MANIFEST["proof_batch"]["target_counts"]:
    target_path = target["target"].replace("/", "__")
    coma = HERE / "comas" / f"{target_path}.coma"
    proof = HERE / "proofs" / f"{target_path}.json"
    if sha(coma) != target["coma_sha256"] or sha(proof) != target["proof_json_sha256"]:
        raise SystemExit(f"hash mismatch: {target['target']}")
    data = json.loads(proof.read_text())["proofs"]["Coma"]
    obligations = {name: audit(node, [name]) for name, node in data.items()}
    leaves = sum(x["terminal_leaves"] for x in obligations.values())
    passed = sum(x["passed"] for x in obligations.values())
    nulls = sum(x["null"] for x in obligations.values())
    expected = (target["own_terminal_leaves"], target["passed"], target["null"])
    actual = (leaves, passed, nulls)
    if expected != actual:
        raise SystemExit(f"tree count mismatch for {target['target']}: {actual} != {expected}")
    result["targets"][target["target"]] = {
        "coma_sha256": sha(coma),
        "proof_json_sha256": sha(proof),
        "top_level_obligation_count": len(obligations),
        "own_terminal_leaves": leaves,
        "passed": passed,
        "null": nulls,
        "obligations": obligations,
    }
summary = {
    "targets": len(result["targets"]),
    "own_terminal_leaves": sum(x["own_terminal_leaves"] for x in result["targets"].values()),
    "passed": sum(x["passed"] for x in result["targets"].values()),
    "null": sum(x["null"] for x in result["targets"].values()),
}
expected_summary = {k: MANIFEST["proof_batch"][k] for k in ("own_terminal_leaves", "passed", "null")}
if {k: summary[k] for k in expected_summary} != expected_summary:
    raise SystemExit(f"batch count mismatch: {summary}")
result["summary"] = summary
result["completeness_limit"] = "The JSON child-array arity is internally consistent with the manifest counts. Independent expected split_vc child arity from the archived COMA has not been established; do not treat these totals as a completeness claim."
out = HERE / "recursive-vc-arity-audit.json"
out.write_text(json.dumps(result, indent=2) + "\n")
print(out)
print(json.dumps(summary, sort_keys=True))
print(sha(out))
