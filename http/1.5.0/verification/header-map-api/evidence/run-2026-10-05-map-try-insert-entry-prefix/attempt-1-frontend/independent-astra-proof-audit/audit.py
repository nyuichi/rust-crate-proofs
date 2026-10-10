#!/usr/bin/env python3
"""Reconcile try_insert_entry saved proof with exact frozen tasks; no prover."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ATTEMPT = HERE.parent
PRIOR = ATTEMPT / "independent-astra-audit"
sha = lambda data: hashlib.sha256(data).hexdigest()

def record(path):
    data = path.read_bytes()
    return {"path": os.path.relpath(path, HERE), "sha256": sha(data), "bytes": len(data)}

def packed(path, data):
    path.write_bytes(gzip.compress(data, mtime=0))
    return dict(record(path), raw_sha256=sha(data), raw_bytes=len(data))

freeze = json.loads((ATTEMPT / "source-freeze.json").read_text())
for row in freeze["sources"]:
    data = (ATTEMPT / "sources" / row["path"]).read_bytes()
    assert sha(data) == row["sha256"] and len(data) == row["bytes"]
selection = json.loads((ATTEMPT / "target-selection.json").read_text())
coma = ATTEMPT / selection["target_coma"]
assert sha(coma.read_bytes()) == selection["target_coma_sha256"]
prior = json.loads((PRIOR / "task-audit.json").read_text())
proof = coma.with_suffix("") / "proof.json"
proof_bytes = proof.read_bytes()
(HERE / "proof.json").write_bytes(proof_bytes)
parsed = json.loads(proof_bytes)
assert set(parsed["proofs"]) == {"Coma"}
base = ["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3"]
command = base + [str(coma)]
run = subprocess.run(command, capture_output=True)
assert run.returncode == 0, run.stderr.decode()
previous_stream = gzip.decompress((PRIOR / "full.stdout.gz").read_bytes())
assert sha(previous_stream) == prior["streams"][0]["raw_sha256"]
assert run.stdout == previous_stream
streams = [packed(HERE / "full.stdout.gz", run.stdout), packed(HERE / "full.stderr.gz", run.stderr)]
tasks = HERE / "tasks"
tasks.mkdir(exist_ok=True)
files_command = base + ["-o", str(tasks), str(coma)]
files = subprocess.run(files_command, capture_output=True)
assert files.returncode == 0, files.stderr.decode()
streams += [packed(HERE / "files.stdout.gz", files.stdout), packed(HERE / "files.stderr.gz", files.stderr)]
roots = []
for row in prior["roots"]:
    printed = HERE / row["task_file"]
    content = printed.read_bytes()
    assert sha(content) == row["task_file_sha256"]
    assert content == (PRIOR / row["task_file"]).read_bytes()
    match = re.search(r"^goal ([^ ]+) :", content.decode(), re.M)
    assert match and match[1] == row["goal"]
    formula = content.decode()[match.end():].split("\nend", 1)[0].strip()
    literal_true = formula in ("true", "[@coma:solid] true")
    assert literal_true == row["literal_true"]
    node = parsed["proofs"]["Coma"][row["goal"]]
    assert isinstance(node, dict) and node.get("prover") == "z3@4.15.3"
    assert set(node) == {"prover", "time"} and isinstance(node["time"], (int, float))
    roots.append({"goal": row["goal"], "class": row["class"], "literal_true": literal_true,
                  "literal_support_formula": formula if literal_true else None,
                  "complete_task": record(printed), "raw_context_identical_to_prior": True,
                  "proof_leaf": node, "terminal_count": 1, "transformation_count": 0})
root_names = {row["goal"] for row in roots}
assert set(parsed["proofs"]["Coma"]) == root_names
assert set(re.findall(r"^goal ([^ ]+) :", run.stdout.decode(), re.M)) == root_names
assert root_names == {"vc_try_insert_entry_T", "vc_len_Bucket_T", "vc_push_Bucket_T", "vc_new"}
assert (ATTEMPT / "direct-proof.exit-status").read_text().strip() == "0"
assert "✔ (4)" in (ATTEMPT / "direct-proof.log").read_text()
assert proof.read_bytes() == proof_bytes
report = {"verdict": "ACCEPT: exact frozen try_insert_entry contract and body have a complete matching saved proof",
          "solver_invoked_by_audit": False, "frontend_invoked_by_audit": False, "source_or_coma_modified": False,
          "source_files_hash_verified": len(freeze["sources"]), "source_freeze": record(ATTEMPT / "source-freeze.json"),
          "map_source_sha256": freeze["map_sha256"], "name_source_sha256": freeze["name_sha256"],
          "coma": record(coma), "proof_source": record(proof), "proof_copy": record(HERE / "proof.json"),
          "prior_contract_report": record(PRIOR / "REPORT.md"), "prior_task_audit": record(PRIOR / "task-audit.json"),
          "full_stream_command": command, "full_stream_returncode": run.returncode,
          "individual_task_command": files_command, "individual_task_returncode": files.returncode,
          "streams": streams, "complete_stdout_raw_identical_to_prior": True, "individual_tasks_raw_identical_to_prior_count": 4,
          "direct_root_count": 4, "actual_body_roots": 1, "imported_literal_true_roots": 3,
          "successful_terminal_count": 4, "unproved_terminal_count": 0, "transformation_node_count": 0,
          "proof_evidence": [record(ATTEMPT / name) for name in ["direct-proof-command.sh", "direct-proof.log", "direct-proof.exit-status", "direct-proof-started-at.txt", "direct-proof-completed-at.txt"]],
          "scope": "Local append-last-bucket plus old-entry component/key-model prefix and external-field frame; no table-index insertion or global readiness proof.",
          "roots": roots}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ["direct_root_count", "actual_body_roots", "imported_literal_true_roots", "successful_terminal_count", "transformation_node_count", "source_files_hash_verified"]}))
