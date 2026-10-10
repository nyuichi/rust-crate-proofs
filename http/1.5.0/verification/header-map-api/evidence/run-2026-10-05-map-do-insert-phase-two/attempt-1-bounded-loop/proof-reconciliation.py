#!/usr/bin/env python3
"""Check the saved direct Why3 proof against the frozen direct-root print."""

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text())


def main() -> None:
    coma = ROOT / "comas/header/map/do_insert_phase_two.coma"
    proof_path = ROOT / "comas/header/map/do_insert_phase_two/proof.json"
    task_path = ROOT / "direct-task-print.json"
    status_path = ROOT / "direct-proof.exit-status"
    log_path = ROOT / "direct-proof.log"
    command_path = ROOT / "direct-proof-invocation.sh"
    preproof_path = ROOT / "independent-astra-preproof-audit/task-audit.json"
    postproof_path = ROOT / "independent-astra-postproof-audit/task-audit.json"
    postproof_report = ROOT / "independent-astra-postproof-audit/REPORT.md"

    task_data = read_json(task_path)
    task = next(
        row for row in task_data["results"]
        if row["target"] == "header/map/do_insert_phase_two"
    )
    proof_data = read_json(proof_path)
    proof_roots = proof_data["proofs"]["Coma"]
    task_roots = task["goals"]
    expected_roots = {
        "vc_do_insert_phase_two",
        "vc_is_none",
        "vc_len_Pos",
        "vc_replace_Pos",
    }

    assert task["coma"]["sha256"] == sha256(coma)
    assert task["direct_root_count"] == 4
    assert set(task_roots) == expected_roots
    assert set(proof_roots) == expected_roots
    assert all(set(leaf) == {"prover", "time"} for leaf in proof_roots.values())
    assert all(leaf["prover"] == "z3@4.15.3" for leaf in proof_roots.values())
    assert status_path.read_text().strip() == "0"
    assert "✔ (4)" in log_path.read_text(errors="replace")
    invocation = command_path.read_text()
    assert "why3find prove --root . --no-cache --show-progress always" in invocation
    assert "--preprocess" not in invocation
    assert "proofs_transform" not in invocation

    preproof = read_json(preproof_path)
    postproof = read_json(postproof_path)
    assert preproof["coma"]["sha256"] == sha256(coma)
    assert postproof["coma"]["sha256"] == sha256(coma)
    assert postproof["successful_terminal_leaf_count"] == 4
    assert postproof["actual_function_body_roots"] == 1
    assert postproof["imported_literal_true_support_roots"] == 3
    assert postproof["actual_proof_transformation_count"] == 0

    result = {
        "verdict": "PASS: the exact frozen four-root COMA has four direct successful Z3 leaves",
        "proof_tree": {
            "direct_roots": 4,
            "successful_terminal_leaves": 4,
            "actual_function_body_roots": 1,
            "imported_literal_true_support_roots": 3,
            "proof_transformations": 0,
            "all_successful_leaves_use": "z3@4.15.3",
            "proof_json": {
                "path": "comas/header/map/do_insert_phase_two/proof.json",
                "sha256": sha256(proof_path),
                "bytes": proof_path.stat().st_size,
            },
            "log": {
                "path": "direct-proof.log",
                "sha256": sha256(log_path),
                "exit_status": 0,
                "final_marker": "✔ (4)",
            },
            "invocation": {
                "path": "direct-proof-invocation.sh",
                "sha256": sha256(command_path),
                "used_split_vc": False,
            },
            "frozen_coma": {
                "path": "comas/header/map/do_insert_phase_two.coma",
                "sha256": sha256(coma),
            },
        },
        "solver_free_arity_observation": {
            "own_body_split_children": 25,
            "part_of_successful_proof_tree": False,
            "evidence": "independent-astra-arity/summary.json",
        },
        "scope": "The bounded worker body is proved modularly against its emitted support contracts; caller and rebuild closure remain unproved.",
        "independent_audit": {
            "report": "independent-astra-postproof-audit/REPORT.md",
            "report_sha256": sha256(postproof_report),
            "task_audit": "independent-astra-postproof-audit/task-audit.json",
            "task_audit_sha256": sha256(postproof_path),
        },
    }
    output = ROOT / "proof-reconciliation.json"
    output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
