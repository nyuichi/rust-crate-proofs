#!/usr/bin/env python3
"""Independent, solver-free current Map batch task/context capture."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
RUN = HERE.parent
sha = lambda data: hashlib.sha256(data).hexdigest()
results = []
source_manifest = json.loads((RUN / "source-freeze.json").read_text())
source_results = []
for source in source_manifest["sources"]:
    path = RUN / "sources" / source["path"]
    actual = sha(path.read_bytes())
    assert actual == source["sha256"], str(path)
    source_results.append({"path": source["path"], "sha256": actual})
for i, rel in enumerate((RUN / "target-list.txt").read_text().splitlines()):
    coma = RUN / "artifacts" / rel
    cmd = ["why3", "prove", "-C", "/workspace/proof-tools/config/creusot/why3.conf",
           "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot",
           "-D", "why3", str(coma)]
    proc = subprocess.run(cmd, capture_output=True)
    stem = f"{i:02d}-" + str(Path(rel).with_suffix("")).replace("/", "_")
    (HERE / (stem + ".stdout")).write_bytes(proc.stdout)
    (HERE / (stem + ".stderr")).write_bytes(proc.stderr)
    assert proc.returncode == 0, proc.stderr.decode()
    goals = re.findall(rb"(?m)^goal\s+([^\s:]+)\s*:", proc.stdout)
    trivial = re.findall(rb"(?m)^goal\s+([^\s:]+)\s*:\s*\[@coma:solid\]\s*true\s*\n\s*end", proc.stdout)
    row = {"target": rel, "command": cmd, "exit_code": proc.returncode,
           "coma_sha256": sha(coma.read_bytes()),
           "stdout_file": stem + ".stdout", "stdout_sha256": sha(proc.stdout),
           "stderr_file": stem + ".stderr", "stderr_sha256": sha(proc.stderr),
           "printed_goals": [g.decode() for g in goals], "printed_root_count": len(goals),
           "trivial_imported_stub_roots": [g.decode() for g in trivial],
           "own_roots": [g.decode() for g in goals if g not in trivial]}
    proof = coma.with_suffix("") / "proof.json"
    assert proof.exists(), str(proof)
    if proof.exists():
        tree = json.loads(proof.read_text())["proofs"]["Coma"]
        row["proof_sha256"] = sha(proof.read_bytes())
        row["proof_roots"] = sorted(tree)
        row["proof_roots_match"] = sorted(tree) == sorted(row["printed_goals"])
        row["all_roots_direct_success"] = all(isinstance(n, dict) and n.get("prover") and "children" not in n for n in tree.values())
        assert row["proof_roots_match"] and row["all_roots_direct_success"], rel
    results.append(row)
report = {"method": "why3 prove -D why3, no solver, no preprocessing, complete stdout/stderr preserved losslessly.",
          "solver_invoked": False, "source_files_verified": len(source_results),
          "source_freeze_sha256": sha((RUN / "source-freeze.json").read_bytes()),
          "selected_targets": len(results), "total_roots": sum(r["printed_root_count"] for r in results),
          "actual_function_body_roots": sum(len(r["own_roots"]) for r in results if not r["target"].endswith("__refines.coma")),
          "trait_refinement_roots": sum(len(r["own_roots"]) for r in results if r["target"].endswith("__refines.coma")),
          "trivial_imported_stub_roots": sum(len(r["trivial_imported_stub_roots"]) for r in results),
          "results": results}
(HERE / "task-audit.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({k:v for k,v in report.items() if k != "results"}, indent=2))
