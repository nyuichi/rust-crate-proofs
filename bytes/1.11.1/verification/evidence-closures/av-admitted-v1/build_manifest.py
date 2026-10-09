#!/usr/bin/env python3
"""Build the AV bootstrap closure from already-published immutable archives."""
from __future__ import annotations

import hashlib
import json
import pathlib
import sys
import tarfile
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[3]
REPOSITORY = ROOT.parents[1]
PROBE = ROOT / "verification/probes/original-promotable-suffix-promotion-2026-10-09"
OUT = pathlib.Path(__file__).resolve().parent / "manifest.json"
CANONICAL_SHA = "a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489"
ORIGIN_SHA = "a28ac41b5d7d447f0324b37ade94c9cbbd22ff3f3d7aa95effeee5764e82cb47"
ANCESTORS = {
    "au": ("0924cfd015be3e7c7284c94b88dc4921ccdbf6e0c9821172774568ebe86ef701",
           "inputs/repository/bytes/1.11.1/verification/probes/original-owned-view-call-summaries-2026-10-09/evidence/au-positive-reuse-canonical-v1.tar.gz"),
    "at": ("958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d",
           "inputs/repository/bytes/1.11.1/verification/probes/original-owned-view-clone-2026-10-09/evidence/at-positive-canonical-v2.tar.gz"),
    "ar": ("b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7",
           "inputs/repository/bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09/evidence/ar-positive-canonical-v2.tar.gz"),
    "as": ("969a1a0406e10677c1849200284881e043aae9c9bdf29c9b15609783c91a9f74",
           "inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/as-positive-canonical-v1.tar.gz"),
    "ap": ("a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1",
           "inputs/repository/bytes/1.11.1/verification/probes/original-shared-finite-owners-2026-10-09/evidence/ap-positive-canonical-v1.tar.gz"),
    "aq": ("41e8e9c164a1110c7f611bb1726f490e111c6b10a77af6b57bfc707ce49ceba5",
           "inputs/repository/bytes/1.11.1/verification/probes/original-shared-slice-views-2026-10-09/evidence/aq-positive-canonical-v1.tar.gz"),
}
TOOL_PATH = ROOT / "verification/tools/evidence_closure.py"
sys.path.insert(0, str(TOOL_PATH.parent))
import evidence_closure as ec  # noqa: E402


def _part_transport(stem: str, expected_sha: str) -> dict[str, Any]:
    sidecar = PROBE / "evidence" / f"{stem}.tar.gz.parts.json"
    meta = json.loads(sidecar.read_text())
    if meta.get("sha256") != expected_sha or meta.get("archive") != f"{stem}.tar.gz":
        raise ec.ClosureError(f"published parts sidecar does not identify {stem}")
    return {
        "kind": "published_parts",
        "parts": [{
            "path": (PROBE / "evidence" / row["file"]).relative_to(REPOSITORY).as_posix(),
            "sha256": row["sha256"],
            "size": row["bytes"],
        } for row in meta["parts"]],
    }, meta["bytes"]


def _tar_table(path: pathlib.Path) -> dict[str, tuple[str, int]]:
    table = ec._safe_tar_members(path)
    return table


def _reconstruct_parts(transport: dict[str, Any], expected_sha: str, expected_size: int,
                       output: pathlib.Path, label: str) -> pathlib.Path:
    if output.exists() and ec.sha256_file(output) == (expected_sha, expected_size):
        return output
    output.parent.mkdir(parents=True, exist_ok=True)
    tmp = output.with_suffix(".build-tmp")
    tmp.unlink(missing_ok=True)
    h = hashlib.sha256(); size = 0
    with tmp.open("xb") as dest:
        for part in transport["parts"]:
            source = ec._regular_file(ROOT, part["path"], f"published {label} part")
            if ec.sha256_file(source) != (part["sha256"], part["size"]):
                raise ec.ClosureError(f"published part mismatch: {part['path']}")
            with source.open("rb") as stream:
                while block := stream.read(ec.CHUNK):
                    dest.write(block); h.update(block); size += len(block)
    if (h.hexdigest(), size) != (expected_sha, expected_size):
        tmp.unlink(missing_ok=True)
        raise ec.ClosureError(f"published {label} parts do not reconstruct the pinned archive")
    tmp.replace(output)
    return output


def _find_ancestor_members(canonical_path: pathlib.Path) -> dict[str, tuple[str, int]]:
    wanted = {digest: name for name, (digest, _path) in ANCESTORS.items()}
    found: dict[str, tuple[str, int]] = {}
    with tarfile.open(canonical_path, mode="r:*") as tar:
        for member in tar.getmembers():
            if member.name not in {p for _d, p in ANCESTORS.values()}:
                continue
            if not member.isfile():
                raise ec.ClosureError(f"ancestor archive member is not regular: {member.name}")
            stream = tar.extractfile(member)
            if stream is None:
                raise ec.ClosureError(f"cannot read ancestor archive member: {member.name}")
            h = hashlib.sha256()
            size = 0
            with stream:
                while True:
                    block = stream.read(ec.CHUNK)
                    if not block:
                        break
                    h.update(block)
                    size += len(block)
            digest = h.hexdigest()
            if digest not in wanted:
                raise ec.ClosureError(f"unexpected nested ancestor digest at {member.name}: {digest}")
            if digest in found:
                raise ec.ClosureError(f"duplicate nested ancestor object: {digest}")
            found[digest] = (member.name, size)
    if set(found) != set(wanted):
        raise ec.ClosureError("canonical AV archive does not contain exactly the six pinned ancestor archives")
    return found


def build_manifest() -> tuple[dict[str, Any], str]:
    canonical_transport, canonical_size = _part_transport("av-positive-reuse-canonical-v1", CANONICAL_SHA)
    origin_transport, origin_size = _part_transport("av-full-diagnostic-v1", ORIGIN_SHA)
    canonical_row = {"sha256": CANONICAL_SHA, "size": canonical_size, "media": "tar", "transport": canonical_transport}
    origin_row = {"sha256": ORIGIN_SHA, "size": origin_size, "media": "tar", "transport": origin_transport}

    # Reconstruct the canonical object only in the disposable work cache; this
    # reads the exact published parts and does not create a second repo archive.
    scratch = pathlib.Path("/workspace/work/evidence-closure-cache")
    scratch.mkdir(parents=True, exist_ok=True)
    canonical_path = scratch / CANONICAL_SHA
    canonical_path = _reconstruct_parts(canonical_transport, CANONICAL_SHA, canonical_size,
                                        canonical_path, "AV canonical")
    origin_path = _reconstruct_parts(origin_transport, ORIGIN_SHA, origin_size,
                                     scratch / ORIGIN_SHA, "AV proof origin")

    canonical_table = _tar_table(canonical_path)
    if len(canonical_table) != 2164:
        raise ec.ClosureError(f"AV canonical archive member count changed: {len(canonical_table)}")
    canonical_receipt_path = PROBE / "evidence/av-positive-reuse-canonical-v1.json"
    canonical_receipt = json.loads(canonical_receipt_path.read_text())
    canonical_receipt_members = {row.get("path"): row.get("sha256") for row in canonical_receipt.get("members", [])}
    if canonical_receipt.get("archive_sha256") != CANONICAL_SHA or \
            len(canonical_receipt_members) != len(canonical_table) or \
            canonical_receipt_members != {name: value[0] for name, value in canonical_table.items()}:
        raise ec.ClosureError("published AV canonical receipt does not match the exact archive member inventory")
    nested = _find_ancestor_members(canonical_path)
    objects = [canonical_row, origin_row]
    child_path_by_sha: dict[str, str] = {}
    for slug, (digest, expected_member) in ANCESTORS.items():
        member_name, size = nested[digest]
        if member_name != expected_member:
            raise ec.ClosureError(f"nested {slug.upper()} archive is at an unexpected path: {member_name}")
        objects.append({"sha256": digest, "size": size, "media": "tar",
                        "transport": {"kind": "archive_member", "parent_sha256": CANONICAL_SHA,
                                      "member_path": member_name}})
        child_path_by_sha[digest] = member_name

    mounts: list[dict[str, Any]] = []
    files: list[dict[str, Any]] = []
    edges: list[dict[str, Any]] = []
    member_tables: dict[str, dict[str, tuple[str, int]]] = {CANONICAL_SHA: canonical_table}
    member_tables[ORIGIN_SHA] = _tar_table(origin_path)
    origin_receipt_path = PROBE / "evidence/av-full-diagnostic-v1.json"
    origin_receipt = json.loads(origin_receipt_path.read_text())
    origin_receipt_members = {row.get("path"): row.get("sha256") for row in origin_receipt.get("members", [])}
    if origin_receipt.get("archive_sha256") != ORIGIN_SHA or \
            len(origin_receipt_members) != len(member_tables[ORIGIN_SHA]) or \
            origin_receipt_members != {name: value[0] for name, value in member_tables[ORIGIN_SHA].items()}:
        raise ec.ClosureError("published AV origin receipt does not match the exact archive member inventory")
    if canonical_receipt.get("targets") != origin_receipt.get("targets"):
        raise ec.ClosureError("AV canonical target list differs from the proof origin")
    target_files: dict[str, str] = {}
    for target in canonical_receipt.get("targets", []):
        for path_field, hash_field in (("coma", "coma_sha256"), ("proof", "proof_sha256")):
            path = target.get(path_field); digest = target.get(hash_field)
            if not isinstance(path, str) or not isinstance(digest, str) or path in target_files:
                raise ec.ClosureError("invalid or duplicate AV target file in published target list")
            target_files[path] = digest
    if any(member_tables[ORIGIN_SHA].get(name, (None, None))[0] != digest
           for name, digest in target_files.items()):
        raise ec.ClosureError("proof-origin target bytes differ from the published target list")
    for digest, member_name in child_path_by_sha.items():
        # The resolver will later prove this object from the parent member.
        # For deterministic manifest construction the exact bytes are already
        # present as named members in the disposable canonical extraction.
        child_scratch = scratch / digest
        if not child_scratch.exists() or ec.sha256_file(child_scratch) != (digest, next(o["size"] for o in objects if o["sha256"] == digest)):
            with tarfile.open(canonical_path, mode="r:*") as tar:
                member = tar.getmember(member_name)
                stream = tar.extractfile(member)
                if stream is None:
                    raise ec.ClosureError(f"cannot materialize nested ancestor for inventory: {member_name}")
                tmp = child_scratch.with_suffix(".build-tmp")
                tmp.unlink(missing_ok=True)
                digest_now, size_now = ec._write_stream(tmp, stream, member.size)
                if digest_now != digest:
                    raise ec.ClosureError(f"nested ancestor hash mismatch: {member_name}")
                tmp.replace(child_scratch)
        member_tables[digest] = _tar_table(child_scratch)

    # The AV archive itself does not contain its post-capture audit records.
    # Preserve these small original records as content-addressed blobs so the
    # admission can be audited offline without copying any large archive.
    record_roles = {
        "av-full-diagnostic-v1.json": "proof_origin",
        "av-full-diagnostic-v1-audit.json": "proof_origin",
        "av-full-diagnostic-v1.tar.gz.parts.json": "proof_origin",
        "AV_ORIGIN_AUDIT.json": "proof_origin",
        "AV_ORIGIN_AUDIT.md": "proof_origin",
        "AV_PROOF_REUSE_IDENTITY.json": "proof_origin",
        "av-positive-reuse-canonical-v1.json": "canonical_admission",
        "av-positive-reuse-canonical-v1-audit.json": "canonical_admission",
        "av-positive-reuse-canonical-v1.tar.gz.parts.json": "canonical_admission",
        "AV_CANONICAL_AUDIT.json": "canonical_admission",
        "AV_CANONICAL_AUDIT.md": "canonical_admission",
        "AV_CAPACITY_CONTROL_AUDIT.json": "diagnostic_control",
        "AV_CAPACITY_CONTROL_AUDIT.md": "diagnostic_control",
        "av-new-bodies-diagnostic-v3.json": "diagnostic_control",
        "PUBLISHED_ARCHIVES.json": "reused_ancestor",
    }
    record_sources = sorted(path for path in (PROBE / "evidence").iterdir()
                            if path.is_file() and path.name in record_roles)
    if {path.name for path in record_sources} != set(record_roles):
        raise ec.ClosureError("required AV admission/origin/control sidecar set is incomplete")
    cas_root = pathlib.Path(__file__).resolve().parent / "objects"
    cas_root.mkdir(parents=True, exist_ok=True)
    record_edge_bindings: dict[tuple[str, str], list[dict[str, Any]]] = {}
    record_edge_objects: dict[str, str] = {}
    for source in record_sources:
        digest, size = ec.sha256_file(source)
        cas_path = cas_root / digest
        if cas_path.exists() or cas_path.is_symlink():
            if cas_path.is_symlink() or not cas_path.is_file() or ec.sha256_file(cas_path) != (digest, size):
                raise ec.ClosureError(f"existing bootstrap CAS object conflicts: {digest}")
        else:
            cas_path.write_bytes(source.read_bytes())
            if ec.sha256_file(cas_path) != (digest, size):
                raise ec.ClosureError(f"new bootstrap CAS object failed verification: {digest}")
        if digest not in {row["sha256"] for row in objects}:
            objects.append({"sha256": digest, "size": size, "media": "blob",
                            "transport": {"kind": "cas", "path": digest}})
        role = record_roles[source.name]
        edge_id = "record-" + role + "-" + digest[:16]
        mount_id = "record-" + hashlib.sha256(source.name.encode()).hexdigest()[:16]
        logical = "source-records/" + source.name
        files.append({"path": logical, "sha256": digest, "size": size,
                      "mount_id": mount_id, "member_path": None})
        mounts.append({"id": mount_id, "object_sha256": digest, "format": "blob", "path_prefix": logical,
                       "member_prefix": None, "strip_prefix": None, "provenance_id": edge_id})
        record_edge_bindings.setdefault((edge_id, role), []).append({"logical_path": logical, "object_file": True})
        record_edge_objects[edge_id] = digest

    def add_mount(mount_id: str, digest: str, prefix: str, provenance_id: str, *,
                  role: str, extra_bindings: list[dict[str, Any]] | None = None) -> None:
        mount = {"id": mount_id, "object_sha256": digest, "format": "tar", "path_prefix": prefix,
                 "member_prefix": None, "strip_prefix": None, "provenance_id": provenance_id}
        mounts.append(mount)
        bindings = []
        for member_name, (member_sha, member_size) in member_tables[digest].items():
            logical = prefix + "/" + member_name
            files.append({"path": logical, "sha256": member_sha, "size": member_size,
                          "mount_id": mount_id, "member_path": member_name})
            bindings.append({"logical_path": logical, "member_path": member_name})
        if extra_bindings:
            bindings.extend(extra_bindings)
        size = next(obj["size"] for obj in objects if obj["sha256"] == digest)
        edges.append({"id": provenance_id, "role": role, "object_sha256": digest,
                      "object_size": size, "bindings": sorted(bindings, key=lambda b: b["logical_path"])})

    canonical_id = "canonical-admission"
    add_mount("canonical", CANONICAL_SHA, "canonical", canonical_id, role="canonical_admission")
    origin_object_binding = [{"logical_path": "canonical/inputs/av-proof-origin/av-full-diagnostic-v1.tar.gz",
                              "object_file": True}]
    add_mount("proof-origin", ORIGIN_SHA, "proof-origin", "proof-origin", role="proof_origin",
              extra_bindings=origin_object_binding)
    for slug, (digest, member_path) in ANCESTORS.items():
        add_mount("ancestor-" + slug, digest, "ancestors/" + slug, "ancestor-" + slug,
                  role="reused_ancestor",
                  extra_bindings=[{"logical_path": "canonical/" + member_path, "object_file": True}])

    diagnostic_members = sorted(name for name in canonical_table
        if name.startswith("admission/av-capacity-diagnostic-v1/") or
           name in {"probe/evidence/AV_CAPACITY_CONTROL_AUDIT.json",
                    "probe/evidence/AV_CAPACITY_CONTROL_AUDIT.md"})
    if not diagnostic_members:
        raise ec.ClosureError("canonical archive is missing the reviewed capacity diagnostic control")
    diagnostic_bindings = [{"logical_path": "canonical/" + name, "member_path": name}
                           for name in diagnostic_members]
    edges.append({"id": "diagnostic-capacity-control", "role": "diagnostic_control",
                  "object_sha256": CANONICAL_SHA, "object_size": canonical_size,
                  "bindings": diagnostic_bindings})
    for (edge_id, role), bindings in record_edge_bindings.items():
        digest = record_edge_objects[edge_id]
        size = next(obj["size"] for obj in objects if obj["sha256"] == digest)
        edges.append({"id": edge_id, "role": role, "object_sha256": digest,
                      "object_size": size, "bindings": sorted(bindings, key=lambda b: b["logical_path"])})

    manifest = {"schema": ec.SCHEMA, "version": ec.VERSION, "root_id": "av-admitted-v1",
                "required_roles": ["proof_origin", "canonical_admission", "reused_ancestor", "diagnostic_control"],
                "objects": sorted(objects, key=lambda x: x["sha256"]),
                "mounts": sorted(mounts, key=lambda x: x["id"]),
                "files": sorted(files, key=lambda x: x["path"]),
                "provenance": sorted(edges, key=lambda x: x["id"])}
    ec.validate_manifest_shape(manifest)
    root_sha = hashlib.sha256((json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode()).hexdigest()
    return manifest, root_sha


def main() -> int:
    try:
        manifest, root_sha = build_manifest()
        if OUT.exists() or OUT.is_symlink():
            raise ec.ClosureError("refusing to overwrite the immutable AV bootstrap manifest")
        actual = ec.write_manifest(OUT, manifest)
        if actual != root_sha:
            raise ec.ClosureError("deterministic manifest digest changed between preparation and write")
        print(json.dumps({"status": "pass", "manifest": str(OUT), "expected_root_sha256": root_sha,
                          "objects": len(manifest["objects"]), "mounts": len(manifest["mounts"]),
                          "logical_files": len(manifest["files"]),
                          "canonical_members": sum(row["mount_id"] == "canonical" for row in manifest["files"])},
                         indent=2, sort_keys=True))
        return 0
    except ec.ClosureError as exc:
        print(json.dumps({"status": "error", "error": str(exc)}, indent=2), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
