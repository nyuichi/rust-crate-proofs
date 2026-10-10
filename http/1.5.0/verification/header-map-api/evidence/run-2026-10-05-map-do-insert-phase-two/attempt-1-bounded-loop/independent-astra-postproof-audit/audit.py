#!/usr/bin/env python3
"""Reconcile the saved unsplit phase-two proof; printing only, never a solver."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
RUN = HERE.parent
PRIOR = RUN / "independent-astra-preproof-audit"
sha = lambda data: hashlib.sha256(data).hexdigest()

def record(path):
    data = path.read_bytes()
    return {"path": os.path.relpath(path, HERE), "sha256": sha(data), "bytes": len(data)}

def packed(path, data):
    path.write_bytes(gzip.compress(data, mtime=0))
    return dict(record(path), raw_sha256=sha(data), raw_bytes=len(data))

freeze = json.loads((RUN / "source-freeze.json").read_text())
after = json.loads((RUN / "source-after-emission.json").read_text())
for row in freeze["sources"]:
    content = (RUN / "sources" / row["path"]).read_bytes()
    assert sha(content) == row["sha256"] and len(content) == row["bytes"]
assert len(freeze["sources"]) == 177 and after["changed_count"] == 0
assert {r["path"]: r["sha256"] for r in freeze["sources"]} == {r["path"]: r["sha256"] for r in after["sources"]}
prior = json.loads((PRIOR / "task-audit.json").read_text())
coma = RUN / "comas/header/map/do_insert_phase_two.coma"
assert sha(coma.read_bytes()) == prior["coma"]["sha256"] == "97d23992c74b1c4829532c962744b60336afe094ba3a02ebe5711b3fcf76304d"
proof = coma.with_suffix("") / "proof.json"
proof_bytes = proof.read_bytes()
(HERE / "proof.json").write_bytes(proof_bytes)
parsed = json.loads(proof_bytes)
assert set(parsed["proofs"]) == {"Coma"}
base = ["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3"]
command = base + [str(coma)]
printed = subprocess.run(command, capture_output=True)
assert printed.returncode == 0, printed.stderr.decode()
previous = gzip.decompress((PRIOR / "full.stdout.gz").read_bytes())
assert sha(previous) == prior["commands"][1]["stdout"]["raw_sha256"]
assert printed.stdout == previous
owner_print = json.loads((RUN / "direct-task-print.json").read_text())
owner_worker = next(row for row in owner_print["results"] if row["target"] == "header/map/do_insert_phase_two")
assert owner_worker["coma"]["sha256"] == sha(coma.read_bytes())
assert owner_worker["streams"][0]["raw_sha256"] == sha(printed.stdout)
assert gzip.decompress((RUN / owner_worker["streams"][0]["path"]).read_bytes()) == printed.stdout
streams = [packed(HERE / "full.stdout.gz", printed.stdout), packed(HERE / "full.stderr.gz", printed.stderr)]
tasks = HERE / "tasks"
tasks.mkdir(exist_ok=True)
files_command = base + ["-o", str(tasks), str(coma)]
files = subprocess.run(files_command, capture_output=True)
assert files.returncode == 0, files.stderr.decode()
for row in owner_worker["tasks"]:
    owned_task = RUN / row["path"]
    audit_task = tasks / owned_task.name
    assert sha(owned_task.read_bytes()) == row["sha256"]
    assert audit_task.read_bytes() == owned_task.read_bytes()
streams += [packed(HERE / "files.stdout.gz", files.stdout), packed(HERE / "files.stderr.gz", files.stderr)]
roots = []
for task in sorted(tasks.glob("*.why")):
    text = task.read_text()
    matches = list(re.finditer(r"^goal ([^ ]+) :", text, flags=re.M))
    assert len(matches) == 1
    match = matches[0]
    goal = match[1]
    formula = text[match.end():].split("\nend", 1)[0].strip()
    literal_true = formula in ("true", "[@coma:solid] true")
    assert literal_true == (goal != "vc_do_insert_phase_two")
    leaf = parsed["proofs"]["Coma"][goal]
    assert isinstance(leaf, dict) and set(leaf) == {"prover", "time"}
    assert leaf["prover"] == "z3@4.15.3" and isinstance(leaf["time"], (int, float))
    roots.append({"goal": goal, "class": "imported_literal_true_support" if literal_true else "actual_function_body",
                  "literal_true": literal_true, "literal_support_formula": formula if literal_true else None,
                  "complete_task": record(task), "proof_leaf": leaf,
                  "terminal_leaf_count": 1, "transformation_node_count": 0})
names = {r["goal"] for r in roots}
assert names == {"vc_len_Pos", "vc_is_none", "vc_replace_Pos", "vc_do_insert_phase_two"}
assert names == {r["goal"] for r in prior["direct_roots"]} == set(parsed["proofs"]["Coma"])
assert names == set(re.findall(r"^goal ([^ ]+) :", printed.stdout.decode(), re.M))
assert (RUN / "direct-proof.exit-status").read_text().strip() == "0"
assert "✔ (4)" in (RUN / "direct-proof.log").read_text()
assert "--no-cache" in (RUN / "direct-proof-invocation.sh").read_text()
assert "--preprocess" not in (RUN / "direct-proof-invocation.sh").read_text()
assert proof.read_bytes() == proof_bytes
assert sha(coma.read_bytes()) == prior["coma"]["sha256"]
report = {"verdict": "ACCEPT: original four-root context exactly matches four saved direct Z3 successes; no proof transformation was used",
          "solver_invoked_by_audit": False, "frontend_invoked_by_audit": False, "source_or_coma_modified": False,
          "source_files_hash_verified": 177, "before_after_emission_sources_equal": True,
          "source_freeze": record(RUN / "source-freeze.json"), "source_after_emission": record(RUN / "source-after-emission.json"),
          "frozen_map": record(RUN / "sources/http/1.5.0/src/header/map.rs"),
          "frozen_name": record(RUN / "sources/http/1.5.0/src/header/name.rs"),
          "coma": record(coma), "proof_source": record(proof), "proof_copy": record(HERE / "proof.json"),
          "preproof_report": record(PRIOR / "REPORT.md"), "preproof_task_audit": record(PRIOR / "task-audit.json"),
          "owner_direct_task_print": record(RUN / "direct-task-print.json"),
          "worker_full_stream_and_all_4_tasks_raw_identical_to_owner_print": True,
          "complete_stdout_raw_identical_to_preproof": True, "full_stream_command": command,
          "individual_tasks_command": files_command, "print_returncodes": [printed.returncode, files.returncode],
          "streams": streams, "direct_root_count": 4, "actual_function_body_roots": 1,
          "imported_literal_true_support_roots": 3, "successful_terminal_leaf_count": 4,
          "unproved_leaf_count": 0, "actual_proof_transformation_count": 0,
          "preproof_own_root_split_candidate_arity": 25,
          "candidate_split_is_part_of_successful_proof_tree": False,
          "proof_evidence": [record(RUN / name) for name in ["direct-proof-invocation.sh", "direct-proof.log", "direct-proof.exit-status", "direct-proof-started-at.txt", "direct-proof-completed-at.txt"]],
          "scope": "Conditional bounded one-ring worker body, including variant, first-empty displacement count, exact index/hash shift and untouched suffix; no caller/capacity/readiness closure.",
          "roots": roots}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({k: report[k] for k in ["direct_root_count", "actual_function_body_roots", "imported_literal_true_support_roots", "successful_terminal_leaf_count", "actual_proof_transformation_count", "before_after_emission_sources_equal"]}))
