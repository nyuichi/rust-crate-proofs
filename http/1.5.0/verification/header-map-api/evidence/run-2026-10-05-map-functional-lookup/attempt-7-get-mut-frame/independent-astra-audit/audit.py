#!/usr/bin/env python3
"""Archive complete solver-free task streams for frozen Map getter contracts."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ATTEMPT = HERE.parent
PRIOR = ATTEMPT.parent / "attempt-6-name-eq-open/independent-astra-functional-audit"
sha = lambda data: hashlib.sha256(data).hexdigest()

def packed(path, data):
    content = gzip.compress(data, mtime=0)
    path.write_bytes(content)
    return {"path": str(path.relative_to(HERE)), "raw_sha256": sha(data), "raw_bytes": len(data), "gzip_sha256": sha(content)}

freeze = json.loads((ATTEMPT / "source-freeze.json").read_text())
for row in freeze["sources"]:
    content = (ATTEMPT / "sources" / row["path"]).read_bytes()
    assert sha(content) == row["sha256"] and len(content) == row["bytes"]
selection = json.loads((ATTEMPT / "target-selection.json").read_text())
source_selection = json.loads((ATTEMPT / "source-selection.json").read_text())
prior = {row["target"]: row for row in json.loads((PRIOR / "report.json").read_text())["results"]}
results = []
for i, row in enumerate(selection["comas"]):
    coma = ATTEMPT / row["archive"]
    assert sha(coma.read_bytes()) == row["sha256"]
    output = HERE / f"target-{i:02}"
    output.mkdir(exist_ok=True)
    command = ["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3", str(coma)]
    run = subprocess.run(command, capture_output=True)
    assert run.returncode == 0, run.stderr.decode()
    parts = re.split(r"^goal ([^ ]+) :", run.stdout.decode(), flags=re.M)
    roots = []
    for j in range(1, len(parts), 2):
        goal, tail = parts[j:j+2]
        literal_true = tail.split("\nend", 1)[0].strip() == "[@coma:solid] true"
        kind = "imported_literal_true_support" if literal_true else "trait_refinement" if goal == "refines" else "actual_function_body"
        roots.append({"goal": goal, "class": kind, "literal_true": literal_true})
    old = prior[row["target"]]
    old_stream = gzip.decompress((PRIOR / old["stdout"]["path"]).read_bytes())
    assert sha(old_stream) == old["stdout"]["raw_sha256"]
    results.append({"target": row["target"], "frozen_coma": row["archive"], "coma_sha256": row["sha256"],
                    "command": command, "returncode": run.returncode, "prover_selected": False,
                    "stdout": packed(output / "stdout.gz", run.stdout), "stderr": packed(output / "stderr.gz", run.stderr),
                    "roots": roots, "direct_root_count": len(roots),
                    "attempt6_stdout_reference": str((PRIOR / old["stdout"]["path"]).relative_to(ATTEMPT.parent)),
                    "attempt6_stdout_raw_sha256": sha(old_stream),
                    "raw_complete_stdout_identical_to_attempt6": run.stdout == old_stream})

roots = [root for row in results for root in row["roots"]]
counts = {kind: sum(root["class"] == kind for root in roots) for kind in sorted({root["class"] for root in roots})}
assert len(results) == 13 and len(roots) == 30
assert counts == {"actual_function_body": 8, "imported_literal_true_support": 17, "trait_refinement": 5}
changed = [row["target"] for row in results if not row["raw_complete_stdout_identical_to_attempt6"]]
assert set(changed) == {"header/map/impl_HeaderMap_T/get.coma", "header/map/impl_HeaderMap_T/get_mut.coma"}
report = {"scope": "Frozen contract-shape, non-vacuity, task-context and direct-arity audit only; no solver result is claimed.",
          "solver_invoked": False, "source_or_coma_modified": False,
          "source_files_hash_verified": len(freeze["sources"]),
          "source_freeze_sha256": sha((ATTEMPT / "source-freeze.json").read_bytes()),
          "source_selection_sha256": sha((ATTEMPT / "source-selection.json").read_bytes()),
          "target_selection_sha256": sha((ATTEMPT / "target-selection.json").read_bytes()),
          "map_source_sha256": source_selection["map_source_sha256"],
          "name_source_sha256": source_selection["name_source_sha256"],
          "selected_coma_count": len(results), "total_direct_root_count": len(roots), "class_counts": counts,
          "unchanged_raw_complete_stdout_count_against_attempt6": len(results) - len(changed),
          "changed_raw_complete_stdout_targets": changed,
          "contract_shape_verdict": "No soundness or impossible-precondition blocker found. get_mut exposes a pre/future value relation, nonselected value and structure frame, and independent future readiness.",
          "contract_strength_limit": "HeaderName keys are preserved by deep-model equality; no physical address or exact runtime-key identity is asserted. None remains unconstrained about key absence. Core lookup/parser/hash implementation closure is separate.",
          "proof_json_reconciled": False,
          "results": results}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ["selected_coma_count", "total_direct_root_count", "class_counts", "unchanged_raw_complete_stdout_count_against_attempt6"]}))
