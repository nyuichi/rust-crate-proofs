#!/usr/bin/env python3
"""Freeze the actual Bytes record/data-pointer constructor proof state."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tarfile
from datetime import datetime, timezone
from pathlib import Path


PROBE = Path(__file__).resolve().parent
CRATE = PROBE.parents[2]
NAME = "bytes-record-data-field-2026-10-08"
OUT = PROBE / "evidence" / NAME
ARCHIVE = PROBE / "evidence" / f"{NAME}.tar.gz"

PROBE_INPUTS = [
    ".gitignore",
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "run-proof.sh",
    "snapshot_bytes_record.py",
    "source_map.py",
    "source_map.json",
    "why3find.json",
    "src/lib.rs",
    "src/field_event.rs",
    "src/pointer_event.rs",
    "src/source_adapter.rs",
]
CRATE_INPUTS = [
    "src/bytes.rs",
    "src/loom.rs",
    "src/provenance_specs.rs",
    "src/allocation_ops.rs",
    "src/ownership_proof/owned_region.rs",
    "src/ownership_proof/raw_vec.rs",
    "src/ownership_proof/bound_ptr.rs",
    "src/ownership_proof/boxed_alignment.rs",
]
TARGET_JSON = Path(
    "verif/bytes_original_shared_lifecycle_rlib/source_adapter/"
    "from_vec_spare_capacity/proof.json"
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def proof_summary() -> dict[str, object]:
    records = sorted((PROBE / "verif").rglob("proof.json"))
    vcs: list[tuple[str, str | None]] = []

    def walk(value: object, file_name: str, where: str = "") -> None:
        if isinstance(value, dict):
            if "prover" in value and "time" in value:
                prover = value.get("prover")
                vcs.append((f"{file_name}{where}", prover if isinstance(prover, str) else None))
                return
            for name, child in value.items():
                walk(child, file_name, f"{where}/{name}")
        elif isinstance(value, list):
            for index, child in enumerate(value):
                walk(child, file_name, f"{where}/{index}")

    for path in records:
        walk(json.loads(path.read_text()), path.relative_to(PROBE).as_posix())
    target_path = PROBE / TARGET_JSON
    target_doc = json.loads(target_path.read_text())
    target_vcs: list[tuple[str, str | None]] = []

    def target_walk(value: object, where: str = "") -> None:
        if isinstance(value, dict):
            if "prover" in value and "time" in value:
                prover = value.get("prover")
                target_vcs.append((where, prover if isinstance(prover, str) else None))
                return
            for name, child in value.items():
                target_walk(child, f"{where}/{name}")
        elif isinstance(value, list):
            for index, child in enumerate(value):
                target_walk(child, f"{where}/{index}")

    target_walk(target_doc)
    nulls = [name for name, prover in vcs if prover is None]
    target_nulls = [name for name, prover in target_vcs if prover is None]
    return {
        "proof_json_files": len(records),
        "translation_coma_files": len(list((PROBE / "verif").rglob("*.coma"))),
        "vc_leaves": len(vcs),
        "null_vcs": nulls,
        "selected_constructor_vcs": len(target_vcs),
        "selected_constructor_null_vcs": target_nulls,
        "selected_constructor_proof_json": TARGET_JSON.as_posix(),
    }


def run_text(command: list[str], *, cwd: Path) -> str:
    result = subprocess.run(command, cwd=cwd, check=True, text=True, capture_output=True)
    return result.stdout + result.stderr


def main() -> None:
    subprocess.run(["python3", str(PROBE / "source_map.py")], check=True, cwd=PROBE)
    summary = proof_summary()
    if summary["null_vcs"] or summary["selected_constructor_null_vcs"]:
        raise SystemExit(f"refusing to archive null proof leaves: {summary}")

    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)
    input_hashes: dict[str, str] = {}
    for name in PROBE_INPUTS:
        source = PROBE / name
        target = OUT / "probe" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        input_hashes[f"probe/{name}"] = sha256(target)
    for name in CRATE_INPUTS:
        source = CRATE / name
        target = OUT / "crate" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        input_hashes[f"crate/{name}"] = sha256(target)

    outputs = OUT / "outputs"
    for source in sorted((PROBE / "verif").rglob("*.proof.json")) + sorted(
        (PROBE / "verif").rglob("proof.json")
    ):
        rel = source.relative_to(PROBE / "verif")
        target = outputs / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    for source in sorted((PROBE / "verif").rglob("*.coma")):
        rel = source.relative_to(PROBE / "verif")
        target = OUT / "translation" / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    config = PROBE / ".proof-config/creusot/why3.conf"
    if config.exists():
        shutil.copy2(config, OUT / "why3.conf")
    proof_log = PROBE / "evidence" / f"{NAME}-proof.log"
    if proof_log.exists():
        shutil.copy2(proof_log, OUT / "proof.log")

    feature_tree = run_text(
        ["bash", "-lc", "source /workspace/bytes-proof-tools/activate.sh && cargo tree --locked -e features --edges normal,build"],
        cwd=PROBE,
    )
    (OUT / "feature-tree.txt").write_text(feature_tree)
    toolchain = run_text(
        ["bash", "-lc", "source /workspace/bytes-proof-tools/activate.sh && rustc -Vv && cargo -V"],
        cwd=PROBE,
    )
    (OUT / "toolchain.txt").write_text(toolchain)
    git_head = run_text(["git", "rev-parse", "HEAD"], cwd=CRATE).strip()

    all_files = sorted(path for path in OUT.rglob("*") if path.is_file())
    manifest = {
        "name": "original bytes::Shared actual Bytes record and data-pointer constructor leaf",
        "snapshot_utc": datetime.now(timezone.utc).isoformat(),
        "git_head": git_head,
        "scope": "selected len < cap From<Vec<u8>> path; actual Shared.ref_cnt and Bytes.data fields; no clone, release, vtable identity/dispatch, or automatic Drop claim",
        "configuration": {
            "target": "x86_64-unknown-linux-gnu",
            "production_atomic_alias": "loom::sync::atomic::{AtomicUsize,AtomicPtr} -> core::sync::atomic::{AtomicUsize,AtomicPtr}",
            "loom": False,
            "extra_platforms": False,
            "sc_drf": False,
            "proof_budget": "one prover, 1024 MiB Why3 memory limit, serialized shared lock",
        },
        "proof_summary": summary,
        "trusted_generic_boundaries": [
            "field_event::new constructs the actual CoreAtomicUsize and its initial ModelAtomic permission/history; association follows an ordinary move into Shared",
            "pointer_event::new_pointer constructs the actual CoreAtomicPtr<()> and its initial ModelAtomicPtr permission/history; association follows an ordinary move into Bytes",
            "pointer_event::bind_read_only consumes the sole pointer permission and binds the unchanged-history read model to the actual Bytes.data field for this selected no-store leaf",
            "boxed_alignment::into_raw_aligned preserves the Box allocation's pointer permission/alignment",
        ],
        "unproved_relations": [
            "the vtable reference is an explicit constructor input; identity with SHARED_VTABLE and dynamic callback behavior are outside this leaf",
            "no shared clone/refcount transition, read lifetime, release/recovery, or deallocation is included",
        ],
        "source_map_sha256": sha256(OUT / "probe/source_map.json"),
        "input_sha256": input_hashes,
        "archived_file_sha256": {
            path.relative_to(OUT).as_posix(): sha256(path) for path in all_files
        },
    }
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    with tarfile.open(ARCHIVE, "w:gz") as tar:
        tar.add(OUT, arcname=OUT.name)
    (ARCHIVE.with_suffix(ARCHIVE.suffix + ".sha256")).write_text(
        f"{sha256(ARCHIVE)}  {ARCHIVE.name}\n"
    )
    print(json.dumps({"snapshot": str(OUT), "archive": str(ARCHIVE), "summary": summary}, indent=2))


if __name__ == "__main__":
    main()
