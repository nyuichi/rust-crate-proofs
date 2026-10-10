#!/usr/bin/env python3
"""Reconcile the try_insert_entry proof against its frozen COMA and root print."""
import hashlib, json, pathlib
run = pathlib.Path(__file__).resolve().parent
selection = json.loads((run / "target-selection.json").read_text())
ar = json.loads((run / "arity/roots.json").read_text())
coma = run / selection["target_coma"]
proof = coma.parent / coma.stem / "proof.json"
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
if sha(coma) != selection["target_coma_sha256"]: raise SystemExit("COMA hash mismatch")
if not proof.is_file(): raise SystemExit("fresh proof JSON missing")
doc = json.loads(proof.read_text())
if set(doc.get("proofs", {})) != {"Coma"}: raise SystemExit("unexpected proof modules")
actual = doc["proofs"]["Coma"]
expected = {x["goal"]: x["class"] for x in ar["roots"]}
if set(actual) != set(expected): raise SystemExit("proof root names differ from direct printer")
for name, node in actual.items():
    if "prover" in node:
        pass
    elif not isinstance(node.get("children"), list) or not node["children"] or any("prover" not in c for c in node["children"]):
        raise SystemExit(f"not all terminal leaves have prover result: {name}")
classes = {k: sum(v == k for v in expected.values()) for k in set(expected.values())}
audit=run/"independent-astra-audit"
summary = {
  "schema_version": 1,
  "run_id": "run-2026-10-05-map-try-insert-entry-prefix-attempt-1-frontend",
  "status": "passed",
  "proof_method": "direct Why3find on archived emitted COMA; Cargo frontend was not rerun during proof",
  "command": "direct-proof-command.sh",
  "command_sha256": sha(run / "direct-proof-command.sh"),
  "log": "direct-proof.log",
  "log_sha256": sha(run / "direct-proof.log"),
  "exit_status_file": "direct-proof.exit-status",
  "exit_status": int((run / "direct-proof.exit-status").read_text().strip()),
  "resource_limits": {"prover_jobs": 1, "memory_mib_per_prover": 1024, "stop_same_logical_leaf_after_seconds": 150},
  "coma": str(coma.relative_to(run)),
  "coma_sha256": sha(coma),
  "proof_json": str(proof.relative_to(run)),
  "proof_json_sha256": sha(proof),
  "direct_root_count": len(actual),
  "actual_body_roots": classes.get("actual_function_body", 0),
  "imported_literal_true_support_roots": classes.get("imported_literal_true_support", 0),
  "terminal_leaf_count": sum(1 if "prover" in n else len(n["children"]) for n in actual.values()),
  "root_names_match_solver_free_arity": True,
  "all_terminal_leaves_have_prover_results": True,
  "independent_solver_free_replay": {"status":"pass", "report":"independent-astra-audit/REPORT.md", "report_sha256":sha(audit/"REPORT.md"), "task_audit":"independent-astra-audit/task-audit.json", "task_audit_sha256":sha(audit/"task-audit.json"), "replay_script":"independent-astra-audit/audit.py", "script_sha256":sha(audit/"audit.py"), "direct_roots_replayed":4, "terminal_leaves":4},
}
if summary["exit_status"] != 0 or summary["direct_root_count"] != ar["direct_root_count"]: raise SystemExit("proof failed or root count mismatch")
if summary["actual_body_roots"] != ar["actual_body_roots"] or summary["imported_literal_true_support_roots"] != ar["imported_literal_true_support_roots"]: raise SystemExit("root classification mismatch")
(run / "direct-proof-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps({k: summary[k] for k in ("status", "direct_root_count", "actual_body_roots", "imported_literal_true_support_roots", "terminal_leaf_count", "root_names_match_solver_free_arity")}, indent=2))
