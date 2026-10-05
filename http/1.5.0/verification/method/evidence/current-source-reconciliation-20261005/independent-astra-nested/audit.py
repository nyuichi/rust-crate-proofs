#!/usr/bin/env python3
"""Replay only the seven recorded nested Method split_vc parents; no solver."""
import gzip
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / "AGENTS.md").is_file())
CRATE = ROOT / "http/1.5.0/verification/method"
BIN = Path("/workspace/proof-tools/creusot-data/_opam/bin")
CONFIG = "/workspace/proof-tools/config/creusot/why3.conf"
LIBRARY = "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot"
WHY3 = "/workspace/proof-tools/creusot-data/bin/why3"
TARGETS = [
    ("inline-eq", "extension/impl_PartialEq_for_InlineExtension/eq", "vc_eq_InlineExtension", 2, 2),
    ("method-from", "impl_From_for_Method/from", "vc_from_Method", 1, 3),
    ("method-eq", "impl_PartialEq_for_Method/eq", "vc_eq_Method", 1, 2),
    ("method-eq1", "impl_PartialEq_for_Method_1/eq", "vc_eq_Method", 1, 2),
    ("method-eq2", "impl_PartialEq_for_Method_2/eq", "vc_eq_Method", 1, 2),
    ("ref-str-eq", "impl_PartialEq_for_ref_str/eq", "vc_eq_ref_str'0", 1, 2),
    ("str-eq", "impl_PartialEq_for_str/eq", "vc_eq_str", 1, 2),
]
sha = lambda data: hashlib.sha256(data).hexdigest()

def save_gzip(path, data):
    packed = gzip.compress(data, mtime=0)
    path.write_bytes(packed)
    return {"path": str(path.relative_to(HERE)), "raw_sha256": sha(data),
            "raw_bytes": len(data), "gzip_sha256": sha(packed)}

def successful(node):
    if isinstance(node, dict) and node.get("prover"):
        return "children" not in node
    return isinstance(node, dict) and node.get("tactic") == "split_vc" and bool(node.get("children")) and all(successful(n) for n in node["children"])

results = []
with tempfile.TemporaryDirectory(prefix="http-method-arity-api-") as temporary:
    build = Path(temporary)
    shutil.copyfile(HERE / "replay.ml", build / "replay.ml")
    env = dict(os.environ)
    env["PATH"] = str(BIN) + ":" + env["PATH"]
    compile_command = [str(BIN / "ocamlfind"), "ocamlopt", "-package", "why3", "-linkpkg", "-o", str(build / "replay"), str(build / "replay.ml")]
    compiled = subprocess.run(compile_command, capture_output=True, env=env)
    assert compiled.returncode == 0, compiled.stderr.decode()
    (HERE / "compile.stdout").write_bytes(compiled.stdout)
    (HERE / "compile.stderr").write_bytes(compiled.stderr)
    for name, rel, goal, index, expected_nested in TARGETS:
        output = HERE / name
        output.mkdir(exist_ok=True)
        live = CRATE / "verif/http_method_proofs_rlib/method" / (rel + ".coma")
        coma = output / "input.coma"
        proof = output / "proof.json"
        shutil.copyfile(live, coma)
        shutil.copyfile(live.with_suffix("") / "proof.json", proof)
        tree = json.loads(proof.read_text())["proofs"]["Coma"][goal]
        assert tree["tactic"] == "split_vc" and successful(tree)
        nested = tree["children"][index]
        assert nested["tactic"] == "split_vc" and len(nested["children"]) == expected_nested
        command = [str(build / "replay"), str(coma), goal, str(index), str(output)]
        run = subprocess.run(command, capture_output=True, env=env)
        (output / "api.stdout").write_bytes(run.stdout)
        (output / "api.stderr").write_bytes(run.stderr)
        assert run.returncode == 0, run.stderr.decode()
        arities = [(int(d), int(n)) for d, n in re.findall(rb"depth=(\d+) arity=(\d+)", run.stdout)]
        assert arities == [(0, len(tree["children"])), (1, expected_nested)], arities
        cli_command = [WHY3, "prove", "-C", CONFIG, "-L", LIBRARY, "-D", "why3", str(coma), "-T", "Coma", "-G", goal]
        cli = subprocess.run(cli_command, capture_output=True, env=env)
        assert cli.returncode == 0, cli.stderr.decode()
        root_bytes = (output / "root.why").read_bytes()
        parent_bytes = (output / "parent-1.why").read_bytes()
        assert root_bytes == cli.stdout, "API root differs from CLI full task"
        assert parent_bytes == (output / f"depth-0-child-{index}.why").read_bytes()
        streams = [save_gzip(output / "cli-root.stdout.gz", cli.stdout), save_gzip(output / "cli-root.stderr.gz", cli.stderr)]
        for path in sorted(output.glob("*.why")):
            streams.append(save_gzip(path.with_suffix(".why.gz"), path.read_bytes()))
            path.unlink()
        row = {"target": "method::" + rel.replace("/", "::"), "goal": goal,
               "coma_sha256": sha(coma.read_bytes()), "proof_json_sha256": sha(proof.read_bytes()),
               "source_coma": str(live.relative_to(ROOT)), "frozen_coma": str(coma.relative_to(HERE)),
               "initial_split_child_arity": len(tree["children"]), "selected_child_index_zero_based": index,
               "expected_nested_child_arity": expected_nested, "actual_nested_child_arity": arities[1][1],
               "api_root_byte_identical_to_cli_selected_original_task": True,
               "nested_parent_byte_identical_to_selected_initial_child": True,
               "already_successful_siblings_retransformed": False, "all_proof_tree_leaves_successful": True,
               "api_command": command, "cli_root_command": cli_command,
               "api_stdout_sha256": sha(run.stdout), "api_stderr_sha256": sha(run.stderr), "streams": streams}
        results.append(row)
        print(name, "root", len(tree["children"]), "child", index, "->", expected_nested, flush=True)
report = {"method": "Why3 OCaml API: parse original COMA; select original goal; split_vc; select exactly the recorded child; split_vc only that child. No printed task is reparsed.",
          "solver_invoked": False, "already_successful_siblings_retransformed": False,
          "nested_nodes_checked": len(results), "nested_child_tasks": sum(r["actual_nested_child_arity"] for r in results),
          "all_arities_match": True, "replay_source_sha256": sha((HERE / "replay.ml").read_bytes()),
          "compile_command_template": "ocamlfind ocamlopt -package why3 -linkpkg -o <temporary>/replay <temporary>/replay.ml",
          "results": results}
(HERE / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print("PASS", report["nested_nodes_checked"], "nested nodes", report["nested_child_tasks"], "nested children")
