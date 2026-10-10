#!/usr/bin/env python3
"""Reproduce the solver-free phase-two context and original-root split audit."""
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
RUN = HERE.parent
sha = lambda data: hashlib.sha256(data).hexdigest()

def record(path):
    data = path.read_bytes()
    return {"path": os.path.relpath(path, HERE), "sha256": sha(data), "bytes": len(data)}

def packed(path, data):
    path.write_bytes(gzip.compress(data, mtime=0))
    return dict(record(path), raw_sha256=sha(data), raw_bytes=len(data))

freeze = json.loads((RUN / "source-freeze.json").read_text())
for row in freeze["sources"]:
    data = (RUN / "sources" / row["path"]).read_bytes()
    assert sha(data) == row["sha256"] and len(data) == row["bytes"]
selection = json.loads((RUN / "target-selection.json").read_text())
item = next(row for row in selection["comas"] if row["target"] == "header/map/do_insert_phase_two")
coma = RUN / item["archive"]
assert sha(coma.read_bytes()) == item["sha256"] == "97d23992c74b1c4829532c962744b60336afe094ba3a02ebe5711b3fcf76304d"
commands = []
with tempfile.TemporaryDirectory(prefix="phase-two-astra-reproduce-") as temp:
    build = Path(temp)
    shutil.copyfile(HERE / "replay.ml", build / "replay.ml")
    env = dict(os.environ)
    env["PATH"] = "/workspace/proof-tools/creusot-data/_opam/bin:" + env["PATH"]
    def invoke(argv, label):
        result = subprocess.run(argv, cwd=build, env=env, capture_output=True)
        commands.append({"argv": argv, "cwd": str(build), "returncode": result.returncode,
                         "stdout": packed(HERE / (label + ".stdout.gz"), result.stdout),
                         "stderr": packed(HERE / (label + ".stderr.gz"), result.stderr)})
        assert result.returncode == 0, result.stderr.decode()
        return result.stdout
    invoke(["/workspace/proof-tools/creusot-data/_opam/bin/ocamlfind", "ocamlopt", "-package", "why3", "-linkpkg", "-o", "replay", "replay.ml"], "compile")
    full = invoke(["/workspace/proof-tools/creusot-data/bin/why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf", "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot", "-D", "why3", str(coma)], "full")
    output = build / "nested"
    output.mkdir()
    split = invoke([str(build / "replay"), str(coma), "vc_do_insert_phase_two", "-", str(output)], "split")
    assert split == b"depth=0 arity=25\n"
    (HERE / "nested").mkdir(exist_ok=True)
    rendered = []
    for path in sorted(output.glob("*.why")):
        content = path.read_bytes()
        goal = content.decode().rsplit("\ngoal ", 1)[1].split("\nend", 1)[0]
        rendered.append(dict(packed(HERE / "nested" / (path.name + ".gz"), content), goal_formula=goal))

parts = re.split(r"^goal ([^ ]+) :", full.decode(), flags=re.M)
roots = []
for j in range(1, len(parts), 2):
    goal, tail = parts[j:j+2]
    formula = tail.split("\nend", 1)[0].strip()
    literal_true = formula in ("true", "[@coma:solid] true")
    roots.append({"goal": goal, "literal_true": literal_true,
                  "class": "imported_literal_true_support" if literal_true else "actual_function_body",
                  "literal_support_formula": formula if literal_true else None})
assert {r["goal"] for r in roots} == {"vc_len_Pos", "vc_is_none", "vc_replace_Pos", "vc_do_insert_phase_two"}
assert sum(r["literal_true"] for r in roots) == 3
absent = ["RandomState", "Hasher", "hash_elem", "build_hasher", "finish", "as_str", "precondition"]
assert all(token not in full.decode() for token in absent)
assert re.search(r"\bfalse\b", full.decode()) is None
report = {"verdict": "Proof-ready conditional worker contract; no propagated hash/false-precondition blocker found. No solver result claimed.",
          "solver_invoked": False, "frontend_invoked": False, "source_or_coma_modified": False,
          "source_files_hash_verified": len(freeze["sources"]), "source_freeze": record(RUN / "source-freeze.json"),
          "frozen_map": record(RUN / "sources/http/1.5.0/src/header/map.rs"),
          "frozen_name": record(RUN / "sources/http/1.5.0/src/header/name.rs"),
          "target_selection": record(RUN / "target-selection.json"), "coma": record(coma),
          "replay_source": record(HERE / "replay.ml"), "commands": commands, "direct_roots": roots,
          "direct_root_count": 4, "actual_body_roots": 1, "literal_true_support_roots": 3,
          "own_root": "vc_do_insert_phase_two", "independent_split_vc_arity": 25,
          "whole_coma_terminal_candidates_if_own_root_is_split_once": 28,
          "full_context_absent_symbols": absent, "lowercase_false_formula_present": False,
          "nested_renderings": rendered, "proof_json_reconciled": False,
          "scope": "Conditional one-ring termination, first-empty displacement count, exact index/hash shift and untouched suffix. Caller preconditions and capacity/occupancy closure are separate."}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ["source_files_hash_verified", "direct_root_count", "actual_body_roots", "literal_true_support_roots", "independent_split_vc_arity"]}))
