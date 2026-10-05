#!/usr/bin/env python3
"""Prepare a throwaway runtime diagnostic; never changes the actual crate.

The three frontier modes TEMPORARILY trust Bytes Send/Sync. skip-deref also
trusts the three Deref bodies. sequential-accessors instead omits the unsafe
Send/Sync impls from Creusot and adds no trusted annotation. These scaffolds
only expose translation errors: no mode constitutes an ownership,
thread-safety, or full-crate proof.
"""
import argparse
from pathlib import Path
import re
import shutil

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("mode", choices=("ghost-deref", "ghost-accessors", "skip-deref", "sequential-accessors"))
parser.add_argument("destination", type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
destination = args.destination.resolve()
if destination.exists():
    parser.error("destination must be new; existing files are never removed")
destination.mkdir(parents=True)
for name in ("src", "scripts"):
    shutil.copytree(root / name, destination / name)
for name in ("Cargo.toml", "Cargo.lock", "why3find.json"):
    shutil.copy2(root / name, destination / name)
for name in ("bytes.rs", "bytes_mut.rs"):
    path = destination / "src" / name
    source = path.read_text()
    if name == "bytes.rs":
        for trait in ("Send", "Sync"):
            item = f"unsafe impl {trait} for Bytes {{}}"
            assert source.count(item) == 1
            annotation = ("#[cfg(not(creusot))]\n" if args.mode == "sequential-accessors"
                else "#[cfg_attr(creusot, creusot_std::prelude::trusted)]\n")
            source = source.replace(item, annotation + item)
    names = "deref|deref_mut"
    if args.mode in ("ghost-accessors", "sequential-accessors"):
        names += "|as_slice|as_slice_mut|as_ref|as_mut"
    annotation = "    #[cfg_attr(creusot, creusot_std::prelude::check(ghost))]\n"
    if args.mode == "skip-deref":
        annotation = "    #[cfg_attr(creusot, creusot_std::prelude::trusted)]\n" + annotation
    source = re.sub(
        rf"^(    )(?=fn (?:{names})\((?:&self|&mut self)\) -> &(?:mut )?\[u8\] \{{)",
        lambda match: annotation + match[1], source, flags=re.M,
    )
    path.write_text(source)
print(f"BYTES_TRANSLATE_ONLY=1 {destination}/scripts/verify-bytes.sh runtime")
