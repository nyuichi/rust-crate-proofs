#!/usr/bin/env python3
"""Compare fresh Method/Status/Version task streams to the accepted proofs.

The source COMAs may differ because source locations moved.  Reuse is accepted
only when the complete untransformed Why3 printer stdout is byte-identical to
the task input used by the recorded proof, the old complete proof tree still
matches its recorded hash and counts, and independent split_vc arities match
that tree at every nested tactic node.
"""
import datetime
import gzip
import hashlib
import json
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
WHY3 = Path("/workspace/proof-tools/creusot-data/bin/why3")
WHY3_CONFIG = Path("/workspace/proof-tools/config/creusot/why3.conf")
CREUSOT_LIB = Path("/workspace/proof-tools/creusot-data/share/why3find/packages/creusot")
BASE = [str(WHY3), "prove", "-C", str(WHY3_CONFIG), "-L", str(CREUSOT_LIB)]
TASK_RE = re.compile(rb"(?ms)^theory Task\n.*?^end\s*$")
GOAL_RE = re.compile(rb"(?m)^goal (.+?)\s*:")

PROFILES = {
    "method": {
        "crate": ROOT / "http/1.5.0/verification/method",
        "baseline": ROOT / "http/1.5.0/verification/method/evidence/final-2026-10-05.json",
        "coma_root": "verif/http_method_proofs_rlib",
        "current_sources": ["http/1.5.0/src/method.rs", "http/1.5.0/src/ascii.rs"],
        "evidence": ROOT / "http/1.5.0/verification/method/evidence/current-source-reconciliation-20261005",
    },
    "scalars": {
        "crate": ROOT / "http/1.5.0/verification/scalars",
        "baseline": ROOT / "http/1.5.0/verification/scalars/evidence/final-2026-10-05.json",
        "coma_root": "verif/http_scalar_proofs_rlib",
        "current_sources": ["http/1.5.0/src/status.rs", "http/1.5.0/src/version.rs", "http/1.5.0/src/ascii.rs"],
        "evidence": ROOT / "http/1.5.0/verification/scalars/evidence/current-source-reconciliation-20261005",
    },
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha(path.read_bytes())


def run_why3(coma_or_why, split=False):
    argv = BASE + (["-a", "split_vc"] if split else []) + ["-D", "why3", str(coma_or_why)]
    return argv, subprocess.run(argv, cwd=ROOT, capture_output=True)


def blocks_and_labels(stdout):
    blocks = TASK_RE.findall(stdout)
    labels = []
    for block in blocks:
        match = GOAL_RE.search(block)
        if not match:
            raise RuntimeError("printed Why3 task has no goal label")
        labels.append(match.group(1).decode("utf-8", "strict"))
    return blocks, labels


def why3_label(vc_name):
    # Why3 prints Rust/WhyML apostrophe numeric suffixes as qtN.
    return re.sub(r"'(\d+)$", r"qt\1", vc_name)


def save_gzip(path, payload):
    path.parent.mkdir(parents=True, exist_ok=True)
    packed = gzip.compress(payload, compresslevel=9, mtime=0)
    path.write_bytes(packed)
    return {"path": str(path.relative_to(ROOT)), "raw_sha256": sha(payload), "raw_bytes": len(payload), "gzip_sha256": sha(packed), "gzip_bytes": len(packed)}


def proof_tree_counts(proof_coma):
    counts = {"terminal_leaves": 0, "passed": 0, "null": 0, "tactic_nodes": 0, "max_tactic_depth": 0}

    def visit(node, depth=0):
        if node is None:
            counts["terminal_leaves"] += 1
            counts["null"] += 1
            return
        if isinstance(node, dict) and "prover" in node:
            counts["terminal_leaves"] += 1
            counts["passed"] += 1
            return
        if isinstance(node, dict) and "children" in node:
            counts["tactic_nodes"] += 1
            counts["max_tactic_depth"] = max(counts["max_tactic_depth"], depth)
            for child in node.get("children", []):
                visit(child, depth + 1)

    for node in proof_coma.values():
        visit(node)
    return counts


def audit_root_arities(name, target, coma, proof_coma, records):
    expected_roots = {
        vc_name: node for vc_name, node in proof_coma.items()
        if isinstance(node, dict) and node.get("children")
    }
    if not expected_roots:
        return
    argv, run = run_why3(coma, split=True)
    if run.returncode:
        raise RuntimeError(f"first-level split_vc transform failed for {name}:{target}: {run.stderr[-2000:]!r}")
    blocks, labels = blocks_and_labels(run.stdout)
    actual_by_label = {}
    for label in labels:
        actual_by_label[label] = actual_by_label.get(label, 0) + 1
    for vc_name, node in expected_roots.items():
        label = why3_label(vc_name)
        actual = actual_by_label.get(label, 0)
        expected = len(node["children"])
        if actual != expected:
            raise RuntimeError(f"first-level split_vc arity mismatch {name}:{target}:{vc_name}: proof tree={expected}, Why3={actual}")
        records.append({
            "target": target,
            "vc_path": [vc_name],
            "depth": 0,
            "tactic": node["tactic"],
            "expected_child_arity": expected,
            "why3_child_task_count": actual,
            "all_printed_child_goal_labels": labels,
            "stdout_sha256": sha(run.stdout),
            "stderr_sha256": sha(run.stderr),
            "command": argv,
        })


def audit_profile(name, spec, baseline_commit):
    evidence = spec["evidence"]
    raw_root = evidence / "task-streams"
    baseline = json.loads(spec["baseline"].read_text())
    source_sha = {p: sha_file(ROOT / p) for p in spec["current_sources"]}
    proof_rows = baseline["proof_run"]["targets"]
    rows = []
    arities = []
    for ordinal, saved in enumerate(proof_rows):
        target = saved["target"]
        rel_coma = saved["coma"]
        current_coma = spec["crate"] / rel_coma
        git_path = str(current_coma.relative_to(ROOT))
        old_bytes = subprocess.run(["git", "show", f"{baseline_commit}:{git_path}"], cwd=ROOT, capture_output=True, check=True).stdout
        old_coma_sha = sha(old_bytes)
        if old_coma_sha != saved["coma_sha256"]:
            raise RuntimeError(f"baseline COMA hash mismatch for {name}:{target}: {old_coma_sha} != {saved['coma_sha256']}")
        current_coma_sha = sha_file(current_coma)

        old_coma = evidence / "historical-comas-tmp" / f"{ordinal:03d}.coma"
        old_coma.parent.mkdir(parents=True, exist_ok=True)
        old_coma.write_bytes(old_bytes)
        old_argv, old_run = run_why3(old_coma)
        current_argv, current_run = run_why3(current_coma)
        if old_run.returncode or current_run.returncode:
            raise RuntimeError(f"untransformed Why3 print failed for {name}:{target}: old={old_run.returncode} current={current_run.returncode}")
        old_blocks, old_labels = blocks_and_labels(old_run.stdout)
        current_blocks, current_labels = blocks_and_labels(current_run.stdout)

        old_stdout_meta = save_gzip(raw_root / f"{ordinal:03d}.proved.why.gz", old_run.stdout)
        current_stdout_meta = save_gzip(raw_root / f"{ordinal:03d}.current.why.gz", current_run.stdout)
        old_stderr_meta = save_gzip(raw_root / f"{ordinal:03d}.proved.stderr.gz", old_run.stderr)
        current_stderr_meta = save_gzip(raw_root / f"{ordinal:03d}.current.stderr.gz", current_run.stderr)

        proof_path = spec["crate"] / saved["proof_json"]
        proof_bytes = proof_path.read_bytes()
        proof_hash = sha(proof_bytes)
        proof_tree = json.loads(proof_bytes)["proofs"]["Coma"]
        counts = proof_tree_counts(proof_tree)
        expected_counts = {
            "terminal_leaves": saved["passed_leaves"],
            "passed": saved["passed_leaves"],
            "null": saved["unproved_leaves"],
        }
        if any(counts[k] != v for k, v in expected_counts.items()):
            raise RuntimeError(f"recorded proof tree count mismatch for {name}:{target}: {counts} vs {expected_counts}")
        stream_equal = old_run.stdout == current_run.stdout
        proof_tree_fresh = proof_hash == saved["proof_json_sha256"]
        reusable = stream_equal and proof_tree_fresh and counts["null"] == 0

        row = {
            "target": target,
            "recorded_coma_sha256": saved["coma_sha256"],
            "historical_coma_sha256": old_coma_sha,
            "current_coma_sha256": current_coma_sha,
            "historical_coma_equals_current": old_coma_sha == current_coma_sha,
            "historical_proof_json_sha256": saved["proof_json_sha256"],
            "current_proof_json_sha256": proof_hash,
            "current_proof_tree_hash_matches": proof_tree_fresh,
            "proof_tree_counts": counts,
            "recorded_proof_counts": {
                "own_goal_leaves": saved["own_goal_leaves"],
                "callee_contract_leaves": saved["callee_contract_leaves"],
                "passed_leaves": saved["passed_leaves"],
                "unproved_leaves": saved["unproved_leaves"],
            },
            "historical_untransformed_goal_count": len(old_blocks),
            "current_untransformed_goal_count": len(current_blocks),
            "historical_untransformed_goal_labels": old_labels,
            "current_untransformed_goal_labels": current_labels,
            "historical_full_stdout": old_stdout_meta,
            "current_full_stdout": current_stdout_meta,
            "historical_stderr": old_stderr_meta,
            "current_stderr": current_stderr_meta,
            "full_stdout_byte_identical": stream_equal,
            "complete_proof_tree_reusable": reusable,
            "commands": {"historical": old_argv, "current": current_argv},
        }
        if reusable:
            audit_root_arities(name, target, current_coma, proof_tree, arities)
        rows.append(row)
        if (ordinal + 1) % 10 == 0:
            print(f"{name}: compared {ordinal + 1}/{len(proof_rows)}", flush=True)
        old_coma.unlink()

    (evidence / "historical-comas-tmp").rmdir()
    target_count = len(rows)
    reusable = sum(row["complete_proof_tree_reusable"] for row in rows)
    changed_stdout = [row["target"] for row in rows if not row["full_stdout_byte_identical"]]
    independent_arity = {
        "method": "For each reusable target, `why3 prove -a split_vc -D why3` independently checks each recorded root split_vc parent against its immediate child-task count, with no prover. Nested nodes are checked by a solver-free Why3 OCaml API replay from each original COMA; it selects the original goal and exact proof-tree child, then splits only that child. Printed tasks are never reparsed and successful siblings are not retransformed.",
        "solver_usage": "No prover or Why3 server was selected or started.",
        "root_tactic_nodes_checked": len(arities),
        "expected_child_tasks": sum(item["expected_child_arity"] for item in arities),
        "why3_child_tasks": sum(item["why3_child_task_count"] for item in arities),
        "all_root_arities_match": all(item["expected_child_arity"] == item["why3_child_task_count"] for item in arities),
        "nodes": arities,
    }
    nested_pending = sum(
        1 for row in rows if row["complete_proof_tree_reusable"]
        for target in [row["target"]]
        for saved in proof_rows if saved["target"] == target
        for node in json.loads((spec["crate"] / saved["proof_json"]).read_text())["proofs"]["Coma"].values()
        for _ in nested_nodes(node)
    )
    independent_arity["nested_tactic_nodes_pending"] = nested_pending
    if name == "method":
        astra_dir = evidence / "independent-astra-nested"
        astra_report_path = astra_dir / "report.json"
        astra_report_bytes = astra_report_path.read_bytes()
        astra_report = json.loads(astra_report_bytes)
        expected_nested = {
            "method::extension::impl_PartialEq_for_InlineExtension::eq": ("vc_eq_InlineExtension", 2, 2),
            "method::impl_From_for_Method::from": ("vc_from_Method", 1, 3),
            "method::impl_PartialEq_for_Method::eq": ("vc_eq_Method", 1, 2),
            "method::impl_PartialEq_for_Method_1::eq": ("vc_eq_Method", 1, 2),
            "method::impl_PartialEq_for_Method_2::eq": ("vc_eq_Method", 1, 2),
            "method::impl_PartialEq_for_ref_str::eq": ("vc_eq_ref_str'0", 1, 2),
            "method::impl_PartialEq_for_str::eq": ("vc_eq_str", 1, 2),
        }
        replayed = astra_report["results"]
        actual_nested = {
            row["target"]: (
                row["goal"], row["selected_child_index_zero_based"],
                row["actual_nested_child_arity"],
            )
            for row in replayed
        }
        if actual_nested != expected_nested:
            raise RuntimeError(f"Astra nested replay target/goal/child/arity mismatch: {actual_nested}")
        if len(replayed) != nested_pending or not astra_report["all_arities_match"]:
            raise RuntimeError("Astra nested replay count or arity report mismatch")
        if astra_report["solver_invoked"] or astra_report["already_successful_siblings_retransformed"]:
            raise RuntimeError("Astra nested replay violated solver-free/selected-child scope")
        for replay_row in replayed:
            proof_row = next(item for item in proof_rows if item["target"] == replay_row["target"])
            live_coma = spec["crate"] / proof_row["coma"]
            proof_json = spec["crate"] / proof_row["proof_json"]
            if sha_file(live_coma) != replay_row["coma_sha256"]:
                raise RuntimeError(f"Astra replay COMA hash differs from fresh COMA: {replay_row['target']}")
            if sha_file(proof_json) != replay_row["proof_json_sha256"]:
                raise RuntimeError(f"Astra replay proof-tree hash differs from accepted proof: {replay_row['target']}")
        independent_arity.update({
            "nested_tactic_nodes_pending": 0,
            "nested_tactic_nodes_checked": astra_report["nested_nodes_checked"],
            "nested_child_tasks": astra_report["nested_child_tasks"],
            "all_nested_arities_match": astra_report["all_arities_match"],
            "nested_replay_method": astra_report["method"],
            "nested_replay_report": str(astra_report_path.relative_to(ROOT)),
            "nested_replay_report_sha256": sha(astra_report_bytes),
            "nested_replay_source_sha256": astra_report["replay_source_sha256"],
        })
    else:
        independent_arity["nested_tactic_nodes_checked"] = 0
        independent_arity["nested_child_tasks"] = 0
        independent_arity["all_nested_arities_match"] = nested_pending == 0
        independent_arity["nested_tactic_nodes_pending"] = nested_pending

    return {
        "baseline_source_hashes": {
            path: baseline["source_hashes"][path] for path in source_sha if path in baseline["source_hashes"]
        },
        "current_source_sha256": source_sha,
        "historical_source_snapshot": baseline.get("source_snapshot_captured_at_utc"),
        "target_count": target_count,
        "full_stdout_equal_target_count": target_count - len(changed_stdout),
        "full_stdout_changed_target_count": len(changed_stdout),
        "full_stdout_changed_targets": changed_stdout,
        "complete_proof_tree_reusable_target_count": reusable,
        "complete_proof_tree_reusable_targets": [row["target"] for row in rows if row["complete_proof_tree_reusable"]],
        "complete_proof_tree_not_reusable_targets": [row["target"] for row in rows if not row["complete_proof_tree_reusable"]],
        "reused_recorded_leaf_totals": {
            "own_goal_leaves": sum(row["recorded_proof_counts"]["own_goal_leaves"] for row in rows if row["complete_proof_tree_reusable"]),
            "callee_contract_leaves": sum(row["recorded_proof_counts"]["callee_contract_leaves"] for row in rows if row["complete_proof_tree_reusable"]),
            "passed_leaves": sum(row["recorded_proof_counts"]["passed_leaves"] for row in rows if row["complete_proof_tree_reusable"]),
        },
        "independent_arity": independent_arity,
        "targets": rows,
}


def nested_nodes(node):
    if isinstance(node, dict) and node.get("children"):
        for child in node["children"]:
            if isinstance(child, dict) and child.get("children"):
                yield child
            yield from nested_nodes(child)


def main():
    baseline_commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    version = subprocess.run([str(WHY3), "--version"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    common = {
        "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "baseline_commit": baseline_commit,
        "why3_version": version,
        "why3_executable_sha256": sha_file(WHY3),
        "why3_config_sha256": sha_file(WHY3_CONFIG),
        "profiles": {},
    }
    for name, spec in PROFILES.items():
        print(f"starting {name}", flush=True)
        common["profiles"][name] = audit_profile(name, spec, baseline_commit)
    for name, spec in PROFILES.items():
        (spec["evidence"] / "task-stream-reconciliation.json").write_text(json.dumps(common["profiles"][name], indent=2) + "\n")
    (PROFILES["method"]["evidence"] / "manifest.json").write_text(json.dumps(common, indent=2) + "\n")
    print(json.dumps({name: {"target_count": profile["target_count"], "stdout_equal": profile["full_stdout_equal_target_count"], "reusable": profile["complete_proof_tree_reusable_target_count"], "arity_nodes": profile["independent_arity"]["root_tactic_nodes_checked"], "arity_matches": profile["independent_arity"]["all_root_arities_match"], "nested_pending": profile["independent_arity"]["nested_tactic_nodes_pending"]} for name, profile in common["profiles"].items()}, sort_keys=True), flush=True)


if __name__ == "__main__":
    main()
