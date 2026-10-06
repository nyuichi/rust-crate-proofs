#!/usr/bin/env python3
"""Archive a completed run; negative capture requires its pre-exported Why3 tasks."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile

probe = Path(__file__).resolve().parent
crate = probe.parents[2]
label, source_dir, verdict, log_name = sys.argv[1:]
run = (probe / source_dir).resolve()
sha = lambda data: hashlib.sha256(data).hexdigest()
inputs = json.loads((probe / "evidence/input-hashes.json").read_text())
for relative, expected in inputs.items():
    assert sha((crate / relative).read_bytes()) == expected, relative

members = {}
for name in ["Cargo.toml", "Cargo.lock", "why3find.json", "run-proof.sh", "EXPERIMENT.md"]:
    path = run / name
    if not path.exists():
        path = probe / name
    members["probe/" + name] = path.read_bytes()
for folder in ["src", "verif", "tasks"]:
    for path in sorted((run / folder).rglob("*")):
        if path.is_file() and (folder != "verif" or path.suffix == ".coma" or path.name == "proof.json"):
            members["run/" + str(path.relative_to(run))] = path.read_bytes()
if label == "negative-no-acquire":
    assert (probe / "evidence/negative-tasks/retire-Coma-vc_retire_T8.why").exists(), "Export the negative Why3 tasks before capture"
    for path in sorted((probe / "evidence/negative-tasks").glob("*.why")):
        members["diagnostics/" + path.name] = path.read_bytes()
    members["diagnostics/task-export.log"] = (probe / "evidence/negative-task-export.log").read_bytes()
log_names = [log_name, "native.log", "task-export.log"]
if source_dir != ".":
    log_names.append("translation.log")
for name in log_names:
    path = run / name
    if not path.exists():
        path = run / "evidence" / name
    if path.exists():
        members["run/logs/" + name] = path.read_bytes()
members["probe/input-hashes.json"] = (probe / "evidence/input-hashes.json").read_bytes()
for relative in inputs:
    members["crate/" + relative] = (crate / relative).read_bytes()
tool = Path("/workspace/bytes-proof-tools/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/creusot-std-0.13.0")
for relative in ["src/ghost.rs", "src/snapshot.rs", "src/ghost/invariant.rs", "src/std/thread.rs", "src/std/sync/view.rs"]:
    members["creusot-std-0.13.0/" + relative] = (tool / relative).read_bytes()
members["tool/creusot_why3.conf"] = Path("/workspace/bytes-proof-tools/creusot-data/creusot_why3.conf").read_bytes()

def null_paths(value, path=()):
    if value is None:
        yield list(path)
    elif isinstance(value, dict):
        for key, item in value.items():
            yield from null_paths(item, path + (key,))
    elif isinstance(value, list):
        for key, item in enumerate(value):
            yield from null_paths(item, path + (key,))

proofs = {}
for name, data in members.items():
    if name.endswith("/proof.json"):
        proofs[name] = list(null_paths(json.loads(data)["proofs"]))
if verdict == "PROVED_37_FILES":
    assert len(proofs) == 37 and not any(proofs.values())
if verdict == "EXPECTED_MISSING_ACQUIRE_REJECTION":
    assert len(proofs) == 37 and sum(map(len, proofs.values())) == 1
archive = probe / "evidence" / (label + ".tar.gz")
with tarfile.open(archive, "w:gz") as out:
    for name, data in sorted(members.items()):
        info = tarfile.TarInfo(name)
        info.size = len(data)
        out.addfile(info, io.BytesIO(data))
record = {
    "experiment": "T02 objective physical thread transport",
    "verdict": verdict,
    "scope": "Concrete RetiredPart scoped-thread transport with conditional parent cleanup; not architecture admission, actual Bytes API, automatic Drop, or eventual exactly-one completion",
    "native_scope": "84 physical threaded cases plus unchanged primitive race test; native assertions are not formal completion claims",
    "native_configuration": "Default features; the native log is not a negative-feature execution",
    "proof_features": ["negative_no_acquire"] if label == "negative-no-acquire" else [],
    "new_trusted_items": 0,
    "primitive_dependencies_unchanged": True,
    "newly_exercised_standard_assumptions": ["Creusot stock scoped thread scope/spawn contracts", "Creusot stock join_unwrap contract"],
    "proof_files": len(proofs),
    "null_leaves": sum(map(len, proofs.values())),
    "failed_files": {name: paths for name, paths in proofs.items() if paths},
    "archive": {"file": archive.name, "sha256": sha(archive.read_bytes()), "bytes": archive.stat().st_size},
    "members": {name: {"sha256": sha(data), "bytes": len(data)} for name, data in sorted(members.items())},
}
if label == "negative-no-acquire":
    record["failed_goal"] = {
        "function": "SharedRetirement::retire",
        "body_result": "11/12",
        "obligation": "AtView::sync requires peer.view() <= current_view; missing Acquire provides no such witness",
        "task": "diagnostics/retire-Coma-vc_retire_T8.why",
    }
manifest = archive.with_suffix("").with_suffix(".json")
manifest.write_text(json.dumps(record, indent=2) + "\n")
saved = json.loads(manifest.read_text())
with tarfile.open(archive, "r:gz") as check:
    names = {member.name for member in check.getmembers()}
    assert names == set(saved["members"])
    for name, expected in saved["members"].items():
        data = check.extractfile(name).read()
        assert len(data) == expected["bytes"] and sha(data) == expected["sha256"], name
assert sha(archive.read_bytes()) == saved["archive"]["sha256"]
(probe / "evidence" / (label + "-audit.json")).write_text(json.dumps({
    "archive": saved["archive"], "members_checked": len(saved["members"]),
    "all_hashes_match": True, "primitive_dependencies_unchanged": True,
    "proof_files": saved["proof_files"], "null_leaves": saved["null_leaves"],
}, indent=2) + "\n")
print(json.dumps({key: saved[key] for key in ["verdict", "proof_files", "null_leaves", "archive"]}))
