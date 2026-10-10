#!/usr/bin/env python3
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

probe = pathlib.Path(__file__).resolve().parent
crate = probe.parents[2]
out = pathlib.Path(sys.argv[1]).resolve()
evidence = probe / "evidence"
source_map = json.loads((out / "source-map.json").read_text())
sha_bytes = lambda data: hashlib.sha256(data).hexdigest()
sha_file = lambda path: sha_bytes(path.read_bytes())

# Re-extract from current source independently of Cargo's possibly cached OUT_DIR.
with tempfile.TemporaryDirectory(prefix="bytesmut-len-api-source-") as temp:
    fresh_dir = pathlib.Path(temp)
    env = os.environ.copy()
    env["OUT_DIR"] = str(fresh_dir)
    subprocess.run(
        [sys.executable, str(probe / "extract_bytes_mut.py")],
        cwd=probe,
        env=env,
        check=True,
    )
    fresh_map = json.loads((fresh_dir / "source-map.json").read_text())
    if fresh_map != source_map:
        raise SystemExit("Cargo OUT_DIR source map does not match fresh source extraction")
    if (fresh_dir / "bytes_mut_api.rs").read_bytes() != (out / "bytes_mut_api.rs").read_bytes():
        raise SystemExit("Cargo OUT_DIR Rust source does not match fresh exact extraction")
    if source_map["full_source_sha256"] != sha_file(crate / source_map["source"]):
        raise SystemExit("source map is stale for src/bytes_mut.rs")

(evidence / "source").mkdir(parents=True, exist_ok=True)
(evidence / "coma").mkdir(parents=True, exist_ok=True)
(evidence / "proof-results").mkdir(parents=True, exist_ok=True)
shutil.copyfile(out / "bytes_mut_api.rs", evidence / "source/bytes_mut_api.rs")

coma_root = probe / "verif/bytesmut_len_api_proof_rlib"
target_paths = {
    "len": ("impl_BytesMut_0/len.coma", "impl_BytesMut_0/len/proof.json"),
    "truncate": ("impl_BytesMut_0/truncate.coma", "impl_BytesMut_0/truncate/proof.json"),
    "set_len": ("impl_BytesMut_0/set_len.coma", "impl_BytesMut_0/set_len/proof.json"),
    "split_view_region": ("view_region/split_view_region.coma", "view_region/split_view_region/proof.json"),
    "advance_within": ("raw_vec/impl_BoundPtr/advance_within.coma", "raw_vec/impl_BoundPtr/advance_within/proof.json"),
    "physical_region_split_at": ("raw_vec/impl_PhysicalRegion/split_at.coma", "raw_vec/impl_PhysicalRegion/split_at/proof.json"),
    "owned_region_split_at": ("owned_region/impl_OwnedRegion/split_at.coma", "owned_region/impl_OwnedRegion/split_at/proof.json"),
}


def has_null(value):
    if value is None:
        return True
    if isinstance(value, dict):
        return any(has_null(key) or has_null(item) for key, item in value.items())
    if isinstance(value, list):
        return any(has_null(item) for item in value)
    return False


def prover_leaves(value):
    if isinstance(value, dict):
        if "prover" in value:
            if not isinstance(value["prover"], str) or not value["prover"]:
                raise SystemExit("target proof record has an invalid prover leaf")
            if not isinstance(value.get("time"), (int, float)):
                raise SystemExit("target proof record has a prover leaf without time")
            return 1
        if "children" in value and not value["children"]:
            raise SystemExit("target proof record contains an empty proof tree")
        return sum(prover_leaves(item) for item in value.values())
    if isinstance(value, list):
        return sum(prover_leaves(item) for item in value)
    return 0


sha = lambda path: sha_file(path)
target_receipts = {}
for name, (coma_relative, proof_relative) in target_paths.items():
    coma_source = coma_root / coma_relative
    proof_source = coma_root / proof_relative
    if not coma_source.is_file() or not proof_source.is_file():
        raise SystemExit(f"missing actual COMA or prover result for target {name}")
    target_proof = json.loads(proof_source.read_text())
    if has_null(target_proof) or not target_proof.get("proofs"):
        raise SystemExit(f"target {name} has an empty or invalid prover result")
    if prover_leaves(target_proof["proofs"]) == 0:
        raise SystemExit(f"target {name} has no successful prover leaves")
    coma_saved = evidence / f"coma/{name}.coma"
    proof_saved = evidence / f"proof-results/{name}.json"
    shutil.copyfile(coma_source, coma_saved)
    shutil.copyfile(proof_source, proof_saved)
    target_receipts[name] = {
        "coma": coma_saved.relative_to(probe).as_posix(),
        "coma_sha256": sha(coma_saved),
        "proof_results": proof_saved.relative_to(probe).as_posix(),
        "proof_results_sha256": sha(proof_saved),
        "result": "proved",
    }

inputs = [
    "src/bytes_mut.rs",
    "src/ownership_proof/raw_vec.rs",
    "src/ownership_proof/owned_region.rs",
    "src/ownership_proof/view_region.rs",
    "src/provenance_specs.rs",
    "verification/probes/bytesmut-len-api/Cargo.toml",
    "verification/probes/bytesmut-len-api/Cargo.lock",
    "verification/probes/bytesmut-len-api/build.rs",
    "verification/probes/bytesmut-len-api/extract_bytes_mut.py",
    "verification/probes/bytesmut-len-api/src/lib.rs",
    "verification/probes/bytesmut-len-api/verify.sh",
    "verification/probes/bytesmut-len-api/save_evidence.py",
    "verification/probes/bytesmut-len-api/README.md",
    "verification/probes/bytesmut-len-api/why3find.json",
    ".cargo/config.toml",
    "scripts/prepare-proof-std.py",
    "verification/std-support/manifest.json",
    "verification/std-support/alloc-capacity.patch",
    "verification/std-support/address-model.patch",
    "verification/std-support/pointer-model.patch",
]

tool_root = pathlib.Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools"))
tool_manifest_path = tool_root / "installation-manifest.json"
tool_manifest = json.loads(tool_manifest_path.read_text())
for name in ("creusot-rustc", "cargo-creusot", "why3", "why3find", "alt-ergo", "z3", "cvc4", "cvc5"):
    binary = tool_manifest["binaries"][name]
    if sha_file(pathlib.Path(binary["path"])) != binary["sha256"]:
        raise SystemExit(f"pinned tool binary changed: {name}")
std_manifest_path = crate / "verification/std-support/manifest.json"
std_manifest = json.loads(std_manifest_path.read_text())

proof = {
    "status": "proved",
    "scope": "actual extracted BytesMut::len, truncate, and set_len method bodies under the existing full-allocation, offset-zero Vec invariant, plus a spatial view-split helper and body-proved split dependencies",
    "api_complete": False,
    "representation": "BytesMut unique_proof; no BytesMut or Shared runtime field changes",
    "source_correspondence": {
        "kind": "exact source extraction",
        "source": source_map["source"],
        "full_source_sha256": source_map["full_source_sha256"],
        "extracted_components": source_map["included_exactly"],
        "component_sha256": source_map["sha256"],
        "generated_rust_sha256": sha(evidence / "source/bytes_mut_api.rs"),
    },
    "targets": target_receipts,
    "established_guarantees": [
        "len returns the handle's stored length",
        "truncate sets length to min(requested, old length), leaves every capacity slot unchanged, and preserves pointer, capacity, tag, allocation namespace, recovery namespace, and physical resource identity",
        "set_len supports shrinking and growing when target length is within capacity and each newly exposed slot is Known; it preserves every capacity slot and the same allocation identities",
        "the method bodies have no panic path under their stated contracts, including set_len's bounds assertion",
        "split_view_region partitions one valid PhysicalRegion at an in-bounds relative cut, returns shifted pointer metadata, and preserves spatial resource identity and slot maps",
        "BoundPtr::advance_within, PhysicalRegion::split_at, and OwnedRegion::split_at actual bodies are separately selected proof targets",
    ],
    "solver": "Why3, one worker, no proof cache",
    "toolchain": {
        "rust_toolchain": "nightly-2026-06-22",
        "creusot_sha": tool_manifest["creusot_sha"],
        "why3_sha": tool_manifest["why3_sha"],
        "why3find_sha": tool_manifest["why3find_sha"],
        "installation_manifest_sha256": sha(tool_manifest_path),
        "binary_sha256": {
            name: tool_manifest["binaries"][name]["sha256"]
            for name in ("creusot-rustc", "cargo-creusot", "why3", "why3find", "alt-ergo", "z3", "cvc4", "cvc5")
        },
        "std_overlay_tree_sha256": std_manifest["candidate_tree"]["sha256"],
        "std_overlay_manifest_sha256": sha(std_manifest_path),
    },
    "verified_inputs_sha256": {name: sha(crate / name) for name in inputs},
    "limitations": [
        "unique_valid supports only the original whole-allocation Vec representation at offset zero",
        "the constructor/transition that establishes this invariant is not body-proved here",
        "Shared-backed, split, nonzero-offset, and Bytes-owned states are not covered by the length-method result",
        "the spatial helper establishes interval partitioning only; it does not prove Shared lifetime, refcount, or immutable-overlap authority",
        "the crate's retained current.json/current.zip snapshot is separate and stale for changed inputs",
    ],
}
(evidence / "proof.json").write_text(json.dumps(proof, indent=2) + "\n")
