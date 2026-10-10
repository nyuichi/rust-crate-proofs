#!/usr/bin/env python3
"""Read frozen inputs and print complete Why3 tasks; never select a solver."""
import difflib
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
RUN = HERE.parent
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((RUN / "manifest.json").read_text())
reference = json.loads((RUN / "source-reference.json").read_text())

def packed(path, data):
    content = gzip.compress(data, mtime=0)
    path.write_bytes(content)
    return {"path": path.name, "raw_sha256": sha(data), "raw_bytes": len(data), "gzip_sha256": sha(content)}

identities = []
for row in reference["copied_source_files"]:
    path = RUN / row["archive"]
    assert sha(path.read_bytes()) == row["sha256"]
    original = RUN / manifest["source_provenance"]["attempt6"] / "sources" / row["path"]
    assert original.read_bytes() == path.read_bytes()
    identities.append({"path": row["archive"], "sha256": row["sha256"]})
for key in ("map", "name"):
    row = manifest["execution"]["invocation_context_sources"][key]
    assert sha((RUN / row["path"]).read_bytes()) == row["sha256"]
    identities.append({"path": row["path"], "sha256": row["sha256"]})
    before = (RUN / "sources/http/1.5.0/src/header" / (key + ".rs")).read_text()
    after = (RUN / row["path"]).read_text()
    diff = "".join(difflib.unified_diff(before.splitlines(keepends=True), after.splitlines(keepends=True), fromfile="archived " + key, tofile="invocation " + key))
    (HERE / (key + ".rs.diff")).write_text(diff)
    if key == "map":
        start = "#[inline]\n#[cfg_attr(creusot, requires(match entry.links {"
        stop = "// ===== impl Iter ====="
        old_item = before[before.index(start):before.index(stop, before.index(start))]
        new_item = after[after.index(start):after.index(stop, after.index(start))]
        assert old_item == new_item
        append_item_sha256 = sha(old_item.encode())

freeze_path = RUN / reference["source_freeze"]
assert sha(freeze_path.read_bytes()) == reference["source_freeze_sha256"]
frozen = json.loads(freeze_path.read_text())
for row in frozen["sources"]:
    assert sha((freeze_path.parent / "sources" / row["path"]).read_bytes()) == row["sha256"]

paths = [RUN / manifest["input"]["archived_coma"], RUN / manifest["live_reemission_comparison"]["live_coma"]]
assert sha(paths[0].read_bytes()) == manifest["input"]["archived_coma_sha256"]
assert sha(paths[1].read_bytes()) == manifest["live_reemission_comparison"]["live_coma_sha256"]
streams = []
for label, path in zip(("archived", "invocation"), paths):
    command = ["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3", str(path)]
    result = subprocess.run(command, capture_output=True)
    assert result.returncode == 0, result.stderr.decode()
    streams.append({"label": label, "coma_sha256": sha(path.read_bytes()), "command": command,
                    "stdout": packed(HERE / (label + ".stdout.gz"), result.stdout),
                    "stderr": packed(HERE / (label + ".stderr.gz"), result.stderr)})
    if label == "archived":
        archived_stdout = result.stdout
    else:
        assert result.stdout == archived_stdout, "complete task stream changed"

proof_bytes = (RUN / manifest["execution"]["proof_json"]).read_bytes()
assert sha(proof_bytes) == manifest["execution"]["proof_json_sha256"]
assert sha((RUN / "proof.log").read_bytes()) == manifest["execution"]["proof_log_sha256"]
assert (RUN / "proof.exit-status").read_text().strip() == "0"
tree = json.loads(proof_bytes)["proofs"]["Coma"]
roots = re.findall(r"^goal ([^ ]+) :", archived_stdout.decode(), flags=re.M)
assert set(roots) == set(tree) and len(roots) == 5
formulas = re.split(r"^goal ([^ ]+) :", archived_stdout.decode(), flags=re.M)
rows = []
for i in range(1, len(formulas), 2):
    goal, tail = formulas[i:i+2]
    literal_true = tail.split("\nend", 1)[0].strip() == "[@coma:solid] true"
    assert literal_true == (goal != "vc_append_value_T")
    assert tree[goal].get("prover") and not tree[goal].get("children")
    rows.append({"goal": goal, "class": "imported_literal_true_support" if literal_true else "actual_function_body", "complete_direct_prover_leaf": True})

report = {"verdict": "Accepted conditional modular proof of the actual append_value body, with four imported trivial support roots.",
          "solver_invoked": False, "source_files_in_inherited_freeze_hash_verified": len(frozen["sources"]),
          "source_freeze_sha256": reference["source_freeze_sha256"], "source_identities": identities,
          "append_contract_and_body_byte_identical_between_archived_and_invocation_sources": True,
          "append_contract_and_body_sha256": append_item_sha256,
          "complete_raw_task_streams_byte_identical_without_normalization": True,
          "streams": streams, "direct_root_count": 5, "terminal_leaf_count": 5,
          "own_body_root_count": 1, "imported_literal_true_support_root_count": 4,
          "proof_json_sha256": sha(proof_bytes), "proof_log_sha256": manifest["execution"]["proof_log_sha256"],
          "exact_proof_json_root_set_match": True, "all_terminal_leaves_successful": True, "roots": rows}
(HERE / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({k: report[k] for k in ["direct_root_count", "own_body_root_count", "imported_literal_true_support_root_count", "complete_raw_task_streams_byte_identical_without_normalization"]}))
