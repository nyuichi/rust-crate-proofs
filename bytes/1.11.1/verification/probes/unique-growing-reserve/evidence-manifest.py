#!/usr/bin/env python3
"""Capture this probe's sources, extracted fragment hashes, and proof trees."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

probe = Path(__file__).resolve().parent
crate = probe.parents[2]
phase, output = sys.argv[1:]
output = Path(output)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_hashes():
    files = list((crate / "src").rglob("*.rs"))
    files += [probe / p for p in ("Cargo.toml", "Cargo.lock", "build.rs", "src/lib.rs", "why3find.json")]
    return {str(p.relative_to(crate)): digest(p) for p in sorted(files)}


if phase == "before":
    shutil.copytree(crate / "src", output / "source/runtime-src")
    for name in ("Cargo.toml", "Cargo.lock", "build.rs", "src/lib.rs", "why3find.json"):
        dest = output / "source/probe" / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(probe / name, dest)
    (output / "sources-before.json").write_text(json.dumps(source_hashes(), indent=2) + "\n")
elif phase == "after":
    if (probe / "verif").exists():
        shutil.copytree(probe / "verif", output / "verif")
    target = Path(os.environ["CARGO_TARGET_DIR"])
    generated = list(target.glob("debug/build/bytes-unique-growing-reserve-*/out/actual_split.rs"))
    if generated:
        latest = max(generated, key=lambda path: path.stat().st_mtime_ns)
        (output / "extraction").mkdir()
        for name in ("actual_split.rs", "source_fragments.txt"):
            shutil.copy2(latest.parent / name, output / "extraction" / name)
    after = source_hashes()
    before = json.loads((output / "sources-before.json").read_text())
    (output / "sources-after.json").write_text(json.dumps(after, indent=2) + "\n")
    changed = sorted(key for key in before.keys() | after.keys() if before.get(key) != after.get(key))
    snapshot_mismatches = []
    for name, expected in before.items():
        if name.startswith("src/"):
            saved = output / "source/runtime-src" / name.removeprefix("src/")
        else:
            saved = output / "source/probe" / name.removeprefix("verification/probes/unique-growing-reserve/")
        if not saved.exists() or digest(saved) != expected:
            snapshot_mismatches.append(name)
    fragment_mismatches = []
    fragments = output / "extraction/source_fragments.txt"
    snapshot_source = (output / "source/runtime-src/bytes_mut.rs").read_bytes()
    if fragments.exists():
        for line in fragments.read_text().splitlines():
            if line.startswith("adaptation:"):
                continue
            name, start, end, expected = line.split()
            value = 0xcbf29ce484222325
            for byte in snapshot_source[int(start):int(end)]:
                value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
            if f"{value:016x}" != expected:
                fragment_mismatches.append(name)
    unproved = []
    def scan(value, location):
        if value is None:
            unproved.append(location)
        elif isinstance(value, dict):
            for key, child in value.items():
                scan(child, location + "/" + key)
        elif isinstance(value, list):
            for index, child in enumerate(value):
                scan(child, location + "/" + str(index))
    proofs = sorted((output / "verif").rglob("proof.json"))
    for path in proofs:
        scan(json.loads(path.read_text()).get("proofs", {}), str(path.relative_to(output)))
    summary = {"proof_files": len(proofs), "null_proof_leaves": len(unproved),
               "unproved_locations": unproved, "source_files_changed_during_run": changed,
               "snapshot_files_mismatching_before_hashes": snapshot_mismatches,
               "fragments_mismatching_source_snapshot": fragment_mismatches,
               "exit_status": int((output / "exit-status.txt").read_text())}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    manifest = {str(p.relative_to(output)): digest(p) for p in sorted(output.rglob("*"))
                if p.is_file() and p.name != "sha256.json"}
    (output / "sha256.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(summary))
else:
    raise SystemExit("phase must be before or after")
