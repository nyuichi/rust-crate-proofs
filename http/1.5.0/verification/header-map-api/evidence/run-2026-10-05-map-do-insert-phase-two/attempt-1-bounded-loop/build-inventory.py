#!/usr/bin/env python3
"""Regenerate the deterministic hash inventory for this evidence run."""

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
OUTPUT = ROOT / "inventory.json"


def main() -> None:
    files = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path in {OUTPUT, ROOT / "manifest.json"}:
            continue
        data = path.read_bytes()
        files.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    inventory = {
        "record_type": "bounded-phase-two-proof-evidence-inventory",
        "file_count": len(files),
        "files": files,
    }
    OUTPUT.write_text(json.dumps(inventory, indent=2) + "\n")
    print(f"inventory files: {len(files)}")
    print(f"inventory sha256: {hashlib.sha256(OUTPUT.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
