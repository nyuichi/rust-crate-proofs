#!/usr/bin/env python3
"""Reconcile frozen COMAs, complete printed tasks and saved proof trees; no solver."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ATTEMPT = HERE.parent
PRIOR = ATTEMPT / "independent-astra-audit"
sha = lambda data: hashlib.sha256(data).hexdigest()

def record(path):
    data = path.read_bytes()
    return {"path": os.path.relpath(path, HERE), "sha256": sha(data), "bytes": len(data)}

def packed(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(gzip.compress(data, mtime=0))
    return dict(record(path), raw_sha256=sha(data), raw_bytes=len(data))

def run_command(command, output):
    run = subprocess.run(command, capture_output=True)
    result = {"command": command, "returncode": run.returncode,
              "stdout": packed(output / "stdout.gz", run.stdout),
              "stderr": packed(output / "stderr.gz", run.stderr)}
    assert run.returncode == 0, run.stderr.decode(errors="replace")
    return run, result

freeze = json.loads((ATTEMPT / "source-freeze.json").read_text())
for row in freeze["sources"]:
    data = (ATTEMPT / "sources" / row["path"]).read_bytes()
    assert sha(data) == row["sha256"] and len(data) == row["bytes"]
selection = json.loads((ATTEMPT / "target-selection.json").read_text())
prior_report = json.loads((PRIOR / "task-audit.json").read_text())
prior_rows = {row["target"]: row for row in prior_report["results"]}

def inspect(node, path, tactics, terminals):
    assert node is not None and isinstance(node, dict), (path, node)
    if "tactic" in node:
        assert node["tactic"] == "split_vc" and node["children"], (path, node)
        tactics.append({"path": path, "tactic": node["tactic"], "arity": len(node["children"])})
        for i, child in enumerate(node["children"]):
            inspect(child, path + [i], tactics, terminals)
    else:
        assert "prover" in node and isinstance(node.get("time"), (int, float)), (path, node)
        assert node["prover"] in ["z3@4.15.3", "cvc5@1.3.1"], node
        terminals.append(dict(node, path=path))

results = []
with tempfile.TemporaryDirectory(prefix="map7-astra-api-") as temp:
    build = Path(temp)
    shutil.copyfile(HERE / "replay.ml", build / "replay.ml")
    env = dict(os.environ)
    env["PATH"] = "/workspace/proof-tools/creusot-data/_opam/bin:" + env["PATH"]
    compiler = ["/workspace/proof-tools/creusot-data/_opam/bin/ocamlfind", "ocamlopt", "-package", "why3", "-linkpkg", "-o", "replay", "replay.ml"]
    compiled = subprocess.run(compiler, cwd=build, env=env, capture_output=True)
    compile_record = {"command": compiler, "cwd": "temporary build directory containing replay.ml", "returncode": compiled.returncode,
                      "stdout": packed(HERE / "compile.stdout.gz", compiled.stdout),
                      "stderr": packed(HERE / "compile.stderr.gz", compiled.stderr)}
    assert compiled.returncode == 0, compiled.stderr.decode()
    for i, item in enumerate(selection["comas"]):
        coma = ATTEMPT / item["archive"]
        assert sha(coma.read_bytes()) == item["sha256"]
        proof = coma.with_suffix("") / "proof.json"
        proof_data = proof.read_bytes()
        parsed = json.loads(proof_data)
        assert set(parsed["proofs"]) == {"Coma"}
        output = HERE / f"target-{i:02}"
        output.mkdir(exist_ok=True)
        (output / "proof.json").write_bytes(proof_data)
        command = ["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3", str(coma)]
        printed, printed_record = run_command(command, output)
        prior = prior_rows[item["target"]]
        prior_stream = gzip.decompress((PRIOR / prior["stdout"]["path"]).read_bytes())
        assert sha(prior_stream) == prior["stdout"]["raw_sha256"]
        assert printed.stdout == prior_stream
        chunks = re.split(r"^goal ([^ ]+) :", printed.stdout.decode(), flags=re.M)
        roots = []
        for j in range(1, len(chunks), 2):
            goal, tail = chunks[j:j+2]
            formula = tail.split("\nend", 1)[0].strip()
            literal_true = formula in ("true", "[@coma:solid] true")
            kind = "imported_literal_true_support" if literal_true else "trait_refinement" if goal == "refines" else "actual_function_body"
            node = parsed["proofs"]["Coma"][goal]
            tactics, terminals = [], []
            inspect(node, [], tactics, terminals)
            for k, tactic in enumerate(tactics):
                nested = output / f"nested-{j // 2:02}-{k:02}"
                nested.mkdir(exist_ok=True)
                path = ",".join(map(str, tactic["path"])) or "-"
                command = [str(build / "replay"), str(coma), goal, path, str(nested)]
                replayed, replay_record = run_command(command, nested)
                arities = [int(x) for x in re.findall(rb"arity=(\d+)", replayed.stdout)]
                assert len(arities) == len(tactic["path"]) + 1
                assert arities[-1] == tactic["arity"], (goal, tactic, arities)
                rendered = []
                for task in sorted(nested.glob("*.why")):
                    rendered.append(packed(task.with_suffix(".why.gz"), task.read_bytes()))
                    task.unlink()
                tactic.update(independent_arities=arities, replay=replay_record, full_tasks=rendered)
            roots.append({"goal": goal, "class": kind, "literal_true": literal_true,
                          "proof_complete": True, "terminal_count": len(terminals),
                          "terminals": terminals, "transformations": tactics})
        assert set(parsed["proofs"]["Coma"]) == {root["goal"] for root in roots}
        assert [(r["goal"], r["class"]) for r in roots] == [(r["goal"], r["class"]) for r in prior["roots"]]
        assert sha(proof.read_bytes()) == sha(proof_data), "proof changed during audit"
        results.append({"target": item["target"], "coma": record(coma), "source_proof_json": record(proof),
                        "archived_proof_json": record(output / "proof.json"), "printing": printed_record,
                        "complete_stdout_identical_to_prior_audit": True, "roots": roots})

roots = [root for result in results for root in result["roots"]]
counts = {kind: sum(root["class"] == kind for root in roots) for kind in sorted({root["class"] for root in roots})}
terminals = sum(root["terminal_count"] for root in roots)
transforms = sum(len(root["transformations"]) for root in roots)
terminal_classes = {kind: sum(root["terminal_count"] for root in roots if root["class"] == kind) for kind in counts}
terminal_provers = {prover: sum(leaf["prover"] == prover for root in roots for leaf in root["terminals"]) for prover in ["z3@4.15.3", "cvc5@1.3.1"]}
assert len(results) == 13 and len(roots) == 30 and terminals == 43 and transforms == 1
assert counts == {"actual_function_body": 8, "imported_literal_true_support": 17, "trait_refinement": 5}
assert (ATTEMPT / "direct-proof.exit-status").read_text().strip() == "0"
batch_log = (ATTEMPT / "direct-proof.log").read_text()
assert len(re.findall(r"^Library .*: ✔ \(\d+\)$", batch_log, re.M)) == 13
report = {"verdict": "ACCEPT: frozen selected contracts have complete matching proof trees; all transformation arities independently replayed from original COMA",
          "solver_invoked_by_audit": False, "frontend_invoked_by_audit": False, "source_or_coma_modified": False,
          "source_files_hash_verified": len(freeze["sources"]), "source_freeze": record(ATTEMPT / "source-freeze.json"),
          "source_selection": record(ATTEMPT / "source-selection.json"), "target_selection": record(ATTEMPT / "target-selection.json"),
          "prior_contract_audit": record(PRIOR / "REPORT.md"), "prior_task_audit": record(PRIOR / "task-audit.json"),
          "selected_coma_count": len(results), "direct_root_count": len(roots), "class_counts": counts,
          "terminal_leaf_count": terminals, "unproved_leaf_count": 0, "independently_replayed_transformation_nodes": transforms,
          "terminal_class_counts": terminal_classes, "terminal_prover_counts": terminal_provers,
          "get_mut_actual_body_split_arity": 14, "get_mut_coma_terminal_total_including_3_support": 17,
          "prior_complete_stdout_identical_count": len(results), "compiler": compile_record, "replay_source": record(HERE / "replay.ml"),
          "proof_evidence": [record(ATTEMPT / name) for name in ["direct-proof-command.sh", "direct-proof.log", "direct-proof.exit-status", "direct-proof-started-at.txt", "direct-pilot-proof.log", "direct-pilot-proof.exit-status"]],
          "proof_freshness_note": "The same frozen get.coma already had a successful direct pilot before the full batch; the batch progress starts with !2/?28. Both pilot and batch have exit status zero. This audit does not claim that every saved terminal prover result originated in the full batch rather than that same-input pilot.",
          "scope": "Conditional modular getter contracts, including get_mut pre/future value relation, frame and readiness; no negative lookup completeness, whole-crate closure or unselected mutation proof.",
          "results": results}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ["selected_coma_count", "direct_root_count", "class_counts", "terminal_leaf_count", "unproved_leaf_count", "independently_replayed_transformation_nodes"]}))
