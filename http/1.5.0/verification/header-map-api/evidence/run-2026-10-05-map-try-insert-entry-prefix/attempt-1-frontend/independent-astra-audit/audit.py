#!/usr/bin/env python3
"""Independent source/contract/direct-root audit; printing only, no prover."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ATTEMPT = HERE.parent
sha = lambda data: hashlib.sha256(data).hexdigest()

def packed(path, data):
    compressed = gzip.compress(data, mtime=0)
    path.write_bytes(compressed)
    return {"path": str(path.relative_to(HERE)), "raw_sha256": sha(data), "raw_bytes": len(data), "gzip_sha256": sha(compressed)}

freeze = json.loads((ATTEMPT / "source-freeze.json").read_text())
for row in freeze["sources"]:
    data = (ATTEMPT / "sources" / row["path"]).read_bytes()
    assert sha(data) == row["sha256"] and len(data) == row["bytes"]
selection = json.loads((ATTEMPT / "target-selection.json").read_text())
coma = ATTEMPT / selection["target_coma"]
data = coma.read_bytes()
assert sha(data) == selection["target_coma_sha256"]
text = data.decode()
assert "[@expl:try_insert_entry ensures #5]" in text
own = text[text.index("let rec try_insert_entry_T"):]
assert "(! bb0" in own
assert "forall i: int. 0 <= i" in own
assert "i < header_map_keys_len_T self.current" in own
assert "deep_model_HeaderName (Seq.get (view_Vec_Bucket_T_Global self.final.entries) i).key'0" in own

base = ["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3"]
command = base + [str(coma)]
run = subprocess.run(command, capture_output=True)
assert run.returncode == 0, run.stderr.decode()
streams = [packed(HERE / "full.stdout.gz", run.stdout), packed(HERE / "full.stderr.gz", run.stderr)]
roots_from_stdout = re.findall(r"^goal ([^ ]+) :", run.stdout.decode(), flags=re.M)
output = HERE / "tasks"
output.mkdir(exist_ok=True)
files_command = base + ["-o", str(output), str(coma)]
files = subprocess.run(files_command, capture_output=True)
assert files.returncode == 0, files.stderr.decode()
streams += [packed(HERE / "files.stdout.gz", files.stdout), packed(HERE / "files.stderr.gz", files.stderr)]
roots = []
for path in sorted(output.glob("*.why")):
    content = path.read_bytes()
    matches = list(re.finditer(r"^goal ([^ ]+) :", content.decode(), flags=re.M))
    assert len(matches) == 1
    match = matches[0]
    name = match[1]
    formula = content.decode()[match.end():].split("\nend", 1)[0].strip()
    literal_true = formula in ("true", "[@coma:solid] true")
    assert literal_true == (name != "vc_try_insert_entry_T")
    roots.append({"goal": name, "class": "imported_literal_true_support" if literal_true else "actual_function_body", "literal_true": literal_true,
                  "literal_support_formula": formula if literal_true else None,
                  "task_file": str(path.relative_to(HERE)), "task_file_sha256": sha(content), "task_file_bytes": len(content)})
assert set(roots_from_stdout) == {row["goal"] for row in roots}
assert len(roots) == 4
expected = {"vc_try_insert_entry_T", "vc_len_Bucket_T", "vc_push_Bucket_T", "vc_new"}
assert set(roots_from_stdout) == expected
report = {"scope": "Frozen frontend contract-shape and direct-root audit, not solver proof evidence.",
          "verdict": "Sound conditional prefix/frame contract; no impossible precondition or empty-range vacuity defect found.",
          "solver_invoked": False, "source_or_coma_modified": False, "proof_json_reconciled": False,
          "source_files_hash_verified": len(freeze["sources"]),
          "source_freeze_sha256": sha((ATTEMPT / "source-freeze.json").read_bytes()),
          "map_source_sha256": freeze["map_sha256"], "name_source_sha256": freeze["name_sha256"],
          "coma_path": selection["target_coma"], "coma_sha256": sha(data),
          "full_stream_command": command, "task_files_command": files_command,
          "full_stream_returncode": run.returncode, "task_files_returncode": files.returncode, "streams": streams,
          "direct_root_count": len(roots), "actual_function_body_roots": 1, "imported_literal_true_support_roots": 3,
          "stdout_roots_exactly_match_individually_printed_task_roots": True, "roots": roots,
          "contract_limits": "Old keys are preserved by deep-model equality. Err preserves all indexed entry observations and external map fields, but the explicit contract does not state whole runtime-key, entries-vector, or map equality. No global insertion invariant is claimed."}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ["direct_root_count", "actual_function_body_roots", "imported_literal_true_support_roots", "source_files_hash_verified"]}))
