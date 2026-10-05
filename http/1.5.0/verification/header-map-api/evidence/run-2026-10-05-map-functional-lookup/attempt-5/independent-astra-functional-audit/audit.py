#!/usr/bin/env python3
"""Independent frozen Map task audit, including exact nested path; no solver."""
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
ROOT = next(p for p in HERE.parents if (p / "AGENTS.md").is_file())
LIVE = ROOT / "http/1.5.0/verification/header-map-api/verif/http_header_map_api_proof_rlib"
WHY3 = "/workspace/proof-tools/creusot-data/bin/why3"
CONFIG = "/workspace/proof-tools/config/creusot/why3.conf"
LIBRARY = "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot"
sha = lambda data: hashlib.sha256(data).hexdigest()

def stream(path, data):
    packed = gzip.compress(data, mtime=0)
    path.write_bytes(packed)
    return {"path": str(path.relative_to(HERE)), "raw_sha256": sha(data),
            "raw_bytes": len(data), "gzip_sha256": sha(packed)}

def complete(node):
    if isinstance(node, dict) and node.get("prover"):
        return not node.get("children")
    return isinstance(node, dict) and node.get("tactic") == "split_vc" and bool(node.get("children")) and all(map(complete, node["children"]))

def terminal_counts(node):
    if isinstance(node, dict) and node.get("children"):
        children = [terminal_counts(c) for c in node["children"]]
        return sum(c[0] for c in children), sum(c[1] for c in children)
    return 1, int(isinstance(node, dict) and bool(node.get("prover")))

freeze = json.loads((ATTEMPT / "source-freeze.json").read_text())
for item in freeze["sources"]:
    data = (ATTEMPT / "sources" / item["path"]).read_bytes()
    assert sha(data) == item["sha256"] and len(data) == item["bytes"]
manifest = json.loads((ATTEMPT / "manifest.json").read_text())
arity = json.loads((ATTEMPT / "arity/independent-direct-root-audit.json").read_text())
classifications = {row["target"]: {r["goal"]: r["class"] for r in row["roots"]} for row in arity["results"]}
rows = []
for i, item in enumerate(manifest["fresh_comas"]):
    coma = ATTEMPT / item["archive"]
    rel = coma.relative_to(ATTEMPT / "comas")
    assert sha(coma.read_bytes()) == item["sha256"]
    directory = HERE / f"target-{i:02}"
    directory.mkdir(exist_ok=True)
    proof = directory / "proof.json"
    if not proof.exists():
        live_coma = LIVE / rel
        assert live_coma.read_bytes() == coma.read_bytes(), f"live source drift: {rel}"
        shutil.copyfile(live_coma.with_suffix("") / "proof.json", proof)
    tree = json.loads(proof.read_text())["proofs"]["Coma"]
    command = [WHY3, "prove", "-C", CONFIG, "-L", LIBRARY, "-D", "why3", str(coma)]
    result = subprocess.run(command, capture_output=True)
    assert result.returncode == 0, result.stderr.decode()
    text = result.stdout.decode()
    printed_roots = re.findall(r"^goal ([^ ]+) :", text, flags=re.M)
    assert set(printed_roots) == set(tree)
    definitions = re.split(r"^goal ([^ ]+) :", text, flags=re.M)
    root_rows = []
    for j in range(1, len(definitions), 2):
        goal, goal_tail = definitions[j:j+2]
        formula = goal_tail.split("\nend", 1)[0].strip()
        trivial = formula == "[@coma:solid] true"
        kind = classifications[str(rel)][goal]
        assert trivial == (kind == "imported_trivial_support_stub")
        leaves, proved = terminal_counts(tree[goal])
        root_rows.append({"goal": goal, "class": kind, "literal_true": trivial, "complete": complete(tree[goal]), "terminal_leaves": leaves, "proved_terminal_leaves": proved})
    rows.append({"target": str(rel), "coma_sha256": sha(coma.read_bytes()),
                 "proof_json": str(proof.relative_to(HERE)), "proof_json_sha256": sha(proof.read_bytes()),
                 "command": command, "returncode": result.returncode,
                 "stdout": stream(directory / "stdout.gz", result.stdout),
                 "stderr": stream(directory / "stderr.gz", result.stderr),
                 "exact_root_set_match": True, "roots": root_rows})

target = "header/map/as_header_name/impl_Sealed_for_ref_str/find.coma"
coma = ATTEMPT / "comas" / target
selected = next(row for row in rows if row["target"] == target)
tree = json.loads((HERE / selected["proof_json"]).read_text())["proofs"]["Coma"]["vc_find_ref_str"]
assert tree["tactic"] == "split_vc" and len(tree["children"]) == 3
assert tree["children"][2]["tactic"] == "split_vc"
assert len(tree["children"][2]["children"]) == 2
assert tree["children"][2]["children"][1] is None
api_dir = HERE / "nested-ref-str"
api_dir.mkdir(exist_ok=True)
api_source = ROOT / "http/1.5.0/verification/method/evidence/current-source-reconciliation-20261005/independent-astra-nested/replay.ml"
shutil.copyfile(api_source, HERE / "replay.ml")
env = dict(os.environ)
env["PATH"] = "/workspace/proof-tools/creusot-data/_opam/bin:" + env["PATH"]
with tempfile.TemporaryDirectory(prefix="map-functional-arity-") as tmp:
    build = Path(tmp)
    shutil.copyfile(HERE / "replay.ml", build / "replay.ml")
    compiled = subprocess.run(["ocamlfind", "ocamlopt", "-package", "why3", "-linkpkg", "-o", str(build / "replay"), str(build / "replay.ml")], env=env, capture_output=True)
    assert compiled.returncode == 0, compiled.stderr.decode()
    (HERE / "compile.stdout").write_bytes(compiled.stdout)
    (HERE / "compile.stderr").write_bytes(compiled.stderr)
    command = [str(build / "replay"), str(coma), "vc_find_ref_str", "2", str(api_dir)]
    run = subprocess.run(command, capture_output=True, env=env)
    assert run.returncode == 0, run.stderr.decode()
    assert run.stdout == b"depth=0 arity=3\ndepth=1 arity=2\n"
    cli = subprocess.run([WHY3, "prove", "-C", CONFIG, "-L", LIBRARY, "-D", "why3", str(coma), "-T", "Coma", "-G", "vc_find_ref_str"], capture_output=True)
    assert cli.returncode == 0
    assert cli.stdout == (api_dir / "root.why").read_bytes()
    assert (api_dir / "parent-1.why").read_bytes() == (api_dir / "depth-0-child-2.why").read_bytes()
    api_streams = [stream(api_dir / "api.stdout.gz", run.stdout), stream(api_dir / "api.stderr.gz", run.stderr), stream(api_dir / "cli-root.stdout.gz", cli.stdout), stream(api_dir / "cli-root.stderr.gz", cli.stderr)]
    for path in sorted(api_dir.glob("*.why")):
        api_streams.append(stream(path.with_suffix(".why.gz"), path.read_bytes()))
        path.unlink()

all_roots = [r for row in rows for r in row["roots"]]
report = {"solver_invoked": False, "source_files_hash_verified": len(freeze["sources"]),
          "source_freeze_sha256": sha((ATTEMPT / "source-freeze.json").read_bytes()),
          "focus_source_sha256": manifest["focus_source_sha256"],
          "target_count": len(rows), "root_count": len(all_roots),
          "complete_root_count": sum(r["complete"] for r in all_roots),
          "terminal_leaf_count": sum(r["terminal_leaves"] for r in all_roots),
          "proved_terminal_leaf_count": sum(r["proved_terminal_leaves"] for r in all_roots),
          "class_counts": {kind: sum(r["class"] == kind for r in all_roots) for kind in sorted({r["class"] for r in all_roots})},
          "results": rows,
          "nested": {"target": target, "goal": "vc_find_ref_str", "root_arity": 3,
                     "selected_child_zero_based": 2, "nested_arity": 2, "null_path": [2, 1],
                     "api_root_identical_to_cli": True, "nested_parent_identical_to_selected_child": True,
                     "successful_siblings_retransformed": False, "streams": api_streams},
          "verdict": "Partial modular evidence only. 36 complete roots; one own &str functional root open due missing logical equality bridge. Public get/get_mut have no functional ensures in this frozen source."}
(HERE / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ["target_count", "root_count", "complete_root_count", "class_counts", "source_files_hash_verified"]}))
