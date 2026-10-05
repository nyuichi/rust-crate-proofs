#!/usr/bin/env python3
"""Archive checked proof/source trees; retain small review records beside them."""
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tarfile

run = Path(sys.argv[1])
archive = run / "artifacts.tar.gz"
if archive.exists():
    raise SystemExit(f"Refusing to overwrite {archive}")
trees = [run / name for name in ("source", "verif", "extraction/actual_split.rs")]
trees = [path for path in trees if path.exists()]
files = []
for path in trees:
    files.extend([path] if path.is_file() else [p for p in path.rglob("*") if p.is_file()])
expected = {str(p.relative_to(run)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
with tarfile.open(archive, "w:gz") as output:
    for path in trees:
        output.add(path, arcname=str(path.relative_to(run)))
with tarfile.open(archive, "r:gz") as saved:
    actual = {member.name: hashlib.sha256(saved.extractfile(member).read()).hexdigest()
              for member in saved.getmembers() if member.isfile()}
if actual != expected:
    raise SystemExit("Archive content verification failed; original trees retained")
record = {"file": archive.name, "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
          "member_files": len(actual), "members_sha256": actual}
(run / "archive.json").write_text(json.dumps(record, indent=2) + "\n")
for path in trees:
    if path.is_dir():
        shutil.rmtree(path)
    else:
        path.unlink()
print(f"{archive}: {len(actual)} files verified; {archive.stat().st_size} bytes")
