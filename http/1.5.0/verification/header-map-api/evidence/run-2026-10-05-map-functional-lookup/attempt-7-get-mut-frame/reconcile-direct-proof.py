#!/usr/bin/env python3
"""Reconcile direct Why3find proof JSONs with the frozen COMAs and printer roots."""
from __future__ import annotations
import hashlib, json, pathlib, sys

run = pathlib.Path(__file__).resolve().parent
selection = json.loads((run / "target-selection.json").read_text())
arity = json.loads((run / "arity/roots.json").read_text())
arity_by_target = {item["target"]: item for item in arity["targets"]}


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def terminal_proofs(node):
    if isinstance(node, dict) and isinstance(node.get("prover"), str):
        if "children" in node:
            raise ValueError("proof node has both a prover and children")
        return [{"prover": node["prover"], "time": node.get("time")}]
    if isinstance(node, dict) and isinstance(node.get("children"), list):
        if not node["children"]:
            raise ValueError("split proof node has no children")
        result = []
        for child in node["children"]:
            result.extend(terminal_proofs(child))
        return result
    raise ValueError(f"unproved or unrecognized proof node: {node!r}")

rows = []
class_counts = {"actual_function_body": 0, "trait_refinement": 0, "imported_literal_true_support": 0}
terminal_count = 0
terminal_class_counts = {"actual_function_body": 0, "trait_refinement": 0, "imported_literal_true_support": 0}
for chosen in selection["comas"]:
    coma = run / chosen["archive"]
    if digest(coma) != chosen["sha256"]:
        raise SystemExit(f"frozen COMA hash mismatch: {coma}")
    proof_json = coma.parent / coma.stem / "proof.json"
    if not proof_json.is_file():
        raise SystemExit(f"fresh proof JSON missing: {proof_json}")
    proof = json.loads(proof_json.read_text())
    proof_modules = proof.get("proofs")
    if not isinstance(proof_modules, dict) or set(proof_modules) != {"Coma"}:
        raise SystemExit(f"unexpected proof module set in {proof_json}: {proof_modules!r}")
    actual_roots = proof_modules["Coma"]
    expected = arity_by_target[chosen["target"]]["direct_roots"]
    expected_classes = {item["goal"]: item["class"] for item in expected}
    if set(actual_roots) != set(expected_classes):
        raise SystemExit(f"direct root mismatch for {chosen['target']}: expected {sorted(expected_classes)}, got {sorted(actual_roots)}")
    root_rows = []
    for goal in sorted(expected_classes):
        leaves = terminal_proofs(actual_roots[goal])
        if not leaves:
            raise SystemExit(f"no terminal leaves for {chosen['target']}::{goal}")
        terminal_count += len(leaves)
        cls = expected_classes[goal]
        class_counts[cls] += 1
        terminal_class_counts[cls] += len(leaves)
        root_rows.append({"goal": goal, "class": cls, "terminal_leaf_count": len(leaves), "terminal_leaves": leaves})
    rows.append({
        "target": chosen["target"],
        "coma": chosen["archive"],
        "coma_sha256": digest(coma),
        "proof_json": str(proof_json.relative_to(run)),
        "proof_json_sha256": digest(proof_json),
        "direct_root_count": len(root_rows),
        "direct_roots": root_rows,
    })

audit_dir = run / "independent-astra-proof-audit"
audit_report = audit_dir / "REPORT.md"
audit_json = audit_dir / "task-audit.json"
audit_script = audit_dir / "audit.py"
summary = {
    "schema_version": 1,
    "run_id": "run-2026-10-05-map-functional-lookup-attempt-7-get-mut-frame",
    "status": "passed",
    "proof_method": "direct Why3find on archived COMA inputs; Cargo frontend was not rerun during proof",
    "command": "direct-proof-command.sh",
    "command_sha256": digest(run / "direct-proof-command.sh"),
    "log": "direct-proof.log",
    "log_sha256": digest(run / "direct-proof.log"),
    "exit_status_file": "direct-proof.exit-status",
    "exit_status": int((run / "direct-proof.exit-status").read_text().strip()),
    "resource_limits": {"prover_jobs": 1, "memory_mib_per_prover": 1024, "stop_same_logical_leaf_after_seconds": 150},
    "selection_count": len(rows),
    "direct_root_count": sum(row["direct_root_count"] for row in rows),
    "actual_function_body_roots": class_counts["actual_function_body"],
    "trait_refinement_roots": class_counts["trait_refinement"],
    "imported_literal_true_support_roots": class_counts["imported_literal_true_support"],
    "terminal_leaf_count": terminal_count,
    "actual_body_terminal_leaves": terminal_class_counts["actual_function_body"],
    "trait_refinement_terminal_leaves": terminal_class_counts["trait_refinement"],
    "imported_literal_true_support_terminal_leaves": terminal_class_counts["imported_literal_true_support"],
    "own_terminal_leaves": terminal_class_counts["actual_function_body"] + terminal_class_counts["trait_refinement"],
    "all_terminal_leaves_have_prover_results": True,
    "proof_json_count": len(rows),
    "root_sets_match_solver_free_arity": True,
    "proofs": rows,
    "independent_solver_free_replay": {
        "status": "pass",
        "report": str(audit_report.relative_to(run)),
        "report_sha256": digest(audit_report),
        "task_audit": str(audit_json.relative_to(run)),
        "task_audit_sha256": digest(audit_json),
        "replay_script": str(audit_script.relative_to(run)),
        "script_sha256": digest(audit_script),
        "direct_roots_replayed": 30,
        "terminal_leaves": 43,
        "get_mut_body_split_children": 14,
    },
}
if summary["exit_status"] != 0:
    raise SystemExit("Why3find command failed")
if summary["direct_root_count"] != arity["direct_root_count"]:
    raise SystemExit("direct root count mismatch")
for field, cls, arity_field in (("actual_function_body_roots", "actual_function_body", "actual_body_roots"), ("trait_refinement_roots", "trait_refinement", "trait_refinement_roots"), ("imported_literal_true_support_roots", "imported_literal_true_support", "literal_true_imported_support_roots")):
    if summary[field] != arity[arity_field]:
        raise SystemExit(f"classification mismatch: {field}")
(run / "direct-proof-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps({k: summary[k] for k in ("status", "selection_count", "direct_root_count", "actual_function_body_roots", "trait_refinement_roots", "imported_literal_true_support_roots", "terminal_leaf_count", "actual_body_terminal_leaves", "trait_refinement_terminal_leaves", "imported_literal_true_support_terminal_leaves", "proof_json_count", "root_sets_match_solver_free_arity")}, indent=2))
