#!/usr/bin/env python3
"""Freeze the current gate's tasks, exact generated code, configs and hashes."""
from pathlib import Path
import hashlib
import json
import re
import shutil
import sys

root = Path(__file__).resolve().parent
name = sys.argv[1]
source_root = root / "iterator" if len(sys.argv) > 2 and sys.argv[2] == "iterator" else root
destination = source_root / "evidence" / name
destination.mkdir(parents=True, exist_ok=False)
shutil.copytree(source_root / "verif", destination / "verif")
for item in ("Cargo.toml", "Cargo.lock", "why3find.json", "build.rs", "run.sh", "src"):
    source = source_root / item
    if source.is_dir():
        shutil.copytree(source, destination / item)
    else:
        shutil.copy2(source, destination / item)
for item in ("build_iter.rs", "build_readonly.rs"):
    shutil.copy2(root / item, destination / item)
generated = set()
for coma in (source_root / "verif").rglob("*.coma"):
    generated.update(re.findall(r'"([^"\n]+/out/actual_(?:traits|iterator)\.rs)"', coma.read_text()))
for index, filename in enumerate(sorted(generated)):
    directory = Path(filename).parent
    shutil.copytree(directory, destination / f"generated-{index}")
files = {str(p.relative_to(destination)): hashlib.sha256(p.read_bytes()).hexdigest()
         for p in destination.rglob("*") if p.is_file()}
(destination / "manifest.json").write_text(json.dumps({
    "scope": "Exact generated-source gate snapshot, not a whole-crate proof",
    "files": files,
}, indent=2) + "\n")
print(destination)
