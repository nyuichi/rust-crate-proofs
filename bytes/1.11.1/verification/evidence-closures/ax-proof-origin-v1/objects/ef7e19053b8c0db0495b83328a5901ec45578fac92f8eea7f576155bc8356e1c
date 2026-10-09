#!/usr/bin/env python3
"""Content-addressed evidence closure validation and safe hydration.

The manifest digest is an input trust anchor.  Digests embedded in the
manifest identify content; they do not authenticate the manifest itself.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import pathlib
import shutil
import stat
import sys
import tarfile
import tempfile
from typing import Any, Callable


SCHEMA = "evidence-closure-v1"
VERSION = 1
ROLES = frozenset({
    "proof_origin",
    "canonical_admission",
    "reused_ancestor",
    "diagnostic_control",
})
HEX = frozenset("0123456789abcdef")
CHUNK = 1024 * 1024


class ClosureError(ValueError):
    """Invalid or incomplete evidence closure."""


def _fail(message: str) -> None:
    raise ClosureError(message)


def sha256_file(path: pathlib.Path) -> tuple[str, int]:
    h = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        while True:
            block = stream.read(CHUNK)
            if not block:
                break
            h.update(block)
            size += len(block)
    return h.hexdigest(), size


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _is_digest(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(c in HEX for c in value)


def _safe_rel(value: Any, what: str) -> str:
    if not isinstance(value, str) or not value or "\\" in value or "\x00" in value:
        _fail(f"{what} must be a nonempty POSIX relative path")
    p = pathlib.PurePosixPath(value)
    if p.is_absolute() or p.as_posix() != value or any(part in {"", ".", ".."} for part in p.parts):
        _fail(f"unsafe {what}: {value!r}")
    # Reject drive-like prefixes even on POSIX so a restored tree stays portable.
    if p.parts and ":" in p.parts[0]:
        _fail(f"drive-like {what}: {value!r}")
    return value


def _exact_keys(obj: Any, required: set[str], optional: set[str], what: str) -> None:
    if not isinstance(obj, dict):
        _fail(f"{what} must be an object")
    keys = set(obj)
    if not required <= keys or keys - required - optional:
        _fail(f"{what} keys mismatch: missing={sorted(required-keys)}, extra={sorted(keys-required-optional)}")


def validate_manifest_shape(manifest: Any) -> dict[str, Any]:
    """Validate the complete versioned schema without touching transports."""
    _exact_keys(manifest, {"schema", "version", "root_id", "required_roles", "objects", "mounts", "files", "provenance"}, set(), "manifest")
    if manifest["schema"] != SCHEMA or manifest["version"] != VERSION:
        _fail("unknown evidence closure schema/version")
    root_id = manifest["root_id"]
    if not isinstance(root_id, str) or not root_id or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for c in root_id):
        _fail("invalid root_id")
    roles = manifest["required_roles"]
    if (not isinstance(roles, list) or not roles or
            any(not isinstance(role, str) for role in roles) or
            len(set(roles)) != len(roles) or any(role not in ROLES for role in roles)):
        _fail("required_roles contains a duplicate or unknown role")

    objects = manifest["objects"]
    if not isinstance(objects, list) or not objects:
        _fail("objects must be a nonempty list")
    object_by_sha: dict[str, dict[str, Any]] = {}
    part_declarations: dict[str, tuple[str, int]] = {}
    for row in objects:
        _exact_keys(row, {"sha256", "size", "media", "transport"}, set(), "object")
        digest, size, media, transport = row["sha256"], row["size"], row["media"], row["transport"]
        if not _is_digest(digest) or type(size) is not int or size < 0:
            _fail("invalid object identity")
        if not isinstance(media, str) or media not in {"tar", "blob"}:
            _fail(f"unknown object media: {media!r}")
        if digest in object_by_sha:
            if object_by_sha[digest]["size"] != size or object_by_sha[digest]["media"] != media:
                _fail("conflicting size/media declarations for one digest")
            _fail("duplicate object declaration")
        if not isinstance(transport, dict) or not isinstance(transport.get("kind"), str):
            _fail("object transport missing kind")
        kind = transport["kind"]
        if kind == "published_parts":
            _exact_keys(transport, {"kind", "parts"}, set(), "published_parts transport")
            parts = transport["parts"]
            if not isinstance(parts, list) or not parts:
                _fail("published_parts must contain an ordered nonempty list")
            local_part_paths: set[str] = set()
            for part in parts:
                _exact_keys(part, {"path", "sha256", "size"}, set(), "part")
                _safe_rel(part["path"], "part path")
                if not _is_digest(part["sha256"]) or type(part["size"]) is not int or part["size"] < 0:
                    _fail("invalid part identity")
                if part["path"] in local_part_paths:
                    _fail("duplicate part path within one ordered transport")
                local_part_paths.add(part["path"])
                identity = (part["sha256"], part["size"])
                old = part_declarations.get(part["path"])
                if old is not None and old != identity:
                    _fail("conflicting declarations for one published part path")
                part_declarations[part["path"]] = identity
        elif kind == "archive_member":
            _exact_keys(transport, {"kind", "parent_sha256", "member_path"}, set(), "archive_member transport")
            if not _is_digest(transport["parent_sha256"]):
                _fail("invalid parent object digest")
            _safe_rel(transport["member_path"], "archive member path")
            if media != "tar":
                _fail("archive_member object must be a tar archive")
        elif kind == "cas":
            _exact_keys(transport, {"kind", "path"}, set(), "cas transport")
            _safe_rel(transport["path"], "CAS object path")
            if media != "blob":
                _fail("local CAS transport is restricted to opaque blobs")
        else:
            _fail(f"unknown transport kind: {kind!r}")
        object_by_sha[digest] = row

    # All dependencies must be explicit, with no cycles or unreachable objects.
    for row in objects:
        tr = row["transport"]
        if tr["kind"] == "archive_member" and tr["parent_sha256"] not in object_by_sha:
            _fail("archive_member references an undeclared parent object")
        if tr["kind"] == "archive_member" and object_by_sha[tr["parent_sha256"]]["media"] != "tar":
            _fail("archive_member parent must be a tar object")
    color: dict[str, int] = {}
    def visit(digest: str) -> None:
        state = color.get(digest, 0)
        if state == 1:
            _fail("object dependency cycle")
        if state == 2:
            return
        color[digest] = 1
        tr = object_by_sha[digest]["transport"]
        if tr["kind"] == "archive_member":
            visit(tr["parent_sha256"])
        color[digest] = 2
    for digest in object_by_sha:
        visit(digest)

    mounts = manifest["mounts"]
    if not isinstance(mounts, list) or not mounts:
        _fail("mounts must be a nonempty list")
    mount_ids: set[str] = set()
    for mount in mounts:
        _exact_keys(mount, {"id", "object_sha256", "format", "path_prefix", "member_prefix", "strip_prefix", "provenance_id"}, set(), "mount")
        ident = mount["id"]
        if not isinstance(ident, str) or not ident or ident in mount_ids:
            _fail("duplicate or invalid mount id")
        mount_ids.add(ident)
        if not _is_digest(mount["object_sha256"]) or mount["object_sha256"] not in object_by_sha:
            _fail("mount references an undeclared object")
        if not isinstance(mount["format"], str):
            _fail("mount format must be a string")
        if mount["format"] != object_by_sha[mount["object_sha256"]]["media"]:
            _fail("mount format does not match object media")
        if mount["format"] not in {"tar", "blob"}:
            _fail("unknown mount format")
        _safe_rel(mount["path_prefix"], "mount path prefix")
        if mount["format"] == "tar":
            mp = mount["member_prefix"]
            sp = mount["strip_prefix"]
            if mp is not None:
                if not isinstance(mp, str) or not isinstance(sp, str):
                    _fail("member_prefix and strip_prefix must be strings")
                _safe_rel(mp.rstrip("/"), "member prefix")
                if not mp.endswith("/"):
                    _fail("member_prefix must end with '/'")
                if not sp.endswith("/"):
                    _fail("strip_prefix must end with '/' when member_prefix is set")
                _safe_rel(sp.rstrip("/"), "strip prefix")
                if sp != mp:
                    _fail("v1 only permits identical member and strip prefixes")
            elif sp is not None:
                _fail("strip_prefix without member_prefix")
        elif mount["member_prefix"] is not None or mount["strip_prefix"] is not None:
            _fail("blob mount cannot have tar member prefixes")
        if not isinstance(mount["provenance_id"], str) or not mount["provenance_id"]:
            _fail("mount requires a provenance edge")

    edges = manifest["provenance"]
    if not isinstance(edges, list) or not edges:
        _fail("provenance must be a nonempty list")
    edge_ids: set[str] = set()
    edge_by_id: dict[str, dict[str, Any]] = {}
    for edge in edges:
        _exact_keys(edge, {"id", "role", "object_sha256", "object_size", "bindings"}, set(), "provenance edge")
        ident = edge["id"]
        if not isinstance(ident, str) or not ident or ident in edge_ids:
            _fail("duplicate or invalid provenance id")
        edge_ids.add(ident); edge_by_id[ident] = edge
        if not isinstance(edge["role"], str) or edge["role"] not in ROLES:
            _fail(f"unknown provenance role: {edge['role']!r}")
        if not _is_digest(edge["object_sha256"]):
            _fail("invalid provenance object digest")
        obj = object_by_sha.get(edge["object_sha256"])
        if obj is None or type(edge["object_size"]) is not int or obj["size"] != edge["object_size"]:
            _fail("provenance edge does not bind declared object digest and size")
        bindings = edge["bindings"]
        if not isinstance(bindings, list) or not bindings:
            _fail("provenance edge must bind at least one logical file")
        for binding in bindings:
            _exact_keys(binding, {"logical_path"}, {"member_path", "object_file"}, "provenance binding")
            _safe_rel(binding["logical_path"], "provenance logical path")
            has_member = "member_path" in binding
            has_object = binding.get("object_file") is True
            if has_member == has_object:
                _fail("provenance binding must use exactly one of member_path or object_file=true")
            if "object_file" in binding and binding["object_file"] is not True:
                _fail("object_file provenance binding must be true")
            if has_member:
                _safe_rel(binding["member_path"], "provenance source member")
            elif binding["object_file"] is not True:
                _fail("object_file provenance binding must be true")

    for role in roles:
        if not any(edge["role"] == role for edge in edges):
            _fail(f"required provenance role is absent: {role}")
    for mount in mounts:
        edge = edge_by_id.get(mount["provenance_id"])
        if edge is None or edge["object_sha256"] != mount["object_sha256"]:
            _fail("mount provenance edge must identify the same object")

    files = manifest["files"]
    if not isinstance(files, list) or not files:
        _fail("files must be a nonempty inventory")
    seen: set[str] = set()
    previous = ""
    for row in files:
        _exact_keys(row, {"path", "sha256", "size", "mount_id", "member_path"}, set(), "inventory row")
        path = _safe_rel(row["path"], "inventory path")
        if path in seen:
            _fail(f"duplicate logical path: {path}")
        if previous and path <= previous:
            _fail("logical inventory must be strictly sorted by path")
        previous = path; seen.add(path)
        if not _is_digest(row["sha256"]) or type(row["size"]) is not int or row["size"] < 0:
            _fail("invalid inventory identity")
        if not isinstance(row["mount_id"], str) or row["mount_id"] not in mount_ids:
            _fail("inventory row references undeclared mount")
        if row["member_path"] is not None:
            _safe_rel(row["member_path"], "inventory member path")
    for path in seen:
        parts = pathlib.PurePosixPath(path).parts
        for i in range(1, len(parts)):
            if "/".join(parts[:i]) in seen:
                _fail(f"logical file is parent of another path: {'/'.join(parts[:i])}")

    # Every inventory row must be covered by its mount's provenance edge, and
    # every edge binding must name an inventory item.
    row_by_path = {row["path"]: row for row in files}
    mount_by_id = {m["id"]: m for m in mounts}
    edge_bindings: dict[str, set[tuple[str, str]]] = {e["id"]: set() for e in edges}
    for edge in edges:
        for binding in edge["bindings"]:
            if binding["logical_path"] not in row_by_path:
                _fail("provenance binding references undeclared logical path")
            key = (binding["logical_path"], binding.get("member_path", "@object"))
            if key in edge_bindings[edge["id"]]:
                _fail("duplicate provenance binding")
            edge_bindings[edge["id"]].add(key)
    for row in files:
        mount = mount_by_id[row["mount_id"]]
        if mount["format"] == "blob":
            if row["member_path"] is not None or row["path"] != mount["path_prefix"]:
                _fail("blob inventory row must map the entire object to its path")
        else:
            prefix = mount["path_prefix"] + "/"
            if not row["path"].startswith(prefix):
                _fail("inventory path is outside its mount prefix")
            member_projected = row["path"][len(prefix):]
            mp = mount["member_prefix"] or ""
            if mp:
                member_projected = mp + member_projected
            if row["member_path"] != member_projected:
                _fail("inventory member path does not match mount projection")
        mount_edge = mount["provenance_id"]
        expected_binding = (row["path"], row["member_path"] if row["member_path"] is not None else "@object")
        if expected_binding not in edge_bindings[mount_edge]:
            _fail("mount provenance edge does not cover its full projected inventory")
    return {"objects": object_by_sha, "mounts": mount_by_id, "edges": edge_by_id, "files": row_by_path}


def _regular_file(root: pathlib.Path, rel: str, what: str) -> pathlib.Path:
    rel = _safe_rel(rel, what)
    cur = root
    for i, part in enumerate(pathlib.PurePosixPath(rel).parts):
        cur = cur / part
        try:
            st = cur.lstat()
        except FileNotFoundError:
            _fail(f"missing {what}: {rel}")
        if stat.S_ISLNK(st.st_mode):
            _fail(f"symlink in {what}: {rel}")
        if i + 1 < len(pathlib.PurePosixPath(rel).parts):
            if not stat.S_ISDIR(st.st_mode):
                _fail(f"non-directory path component in {what}: {rel}")
        elif not stat.S_ISREG(st.st_mode):
            _fail(f"nonregular {what}: {rel}")
    return cur


def _write_stream(path: pathlib.Path, source, expected_size: int | None = None) -> tuple[str, int]:
    path.parent.mkdir(parents=True, exist_ok=True)
    h = hashlib.sha256(); size = 0
    with path.open("xb") as out:
        while True:
            block = source.read(CHUNK)
            if not block:
                break
            out.write(block); h.update(block); size += len(block)
    if expected_size is not None and size != expected_size:
        path.unlink(missing_ok=True)
        _fail(f"object size mismatch while writing cache: {path.name}")
    return h.hexdigest(), size


def _safe_tar_members(path: pathlib.Path) -> dict[str, tuple[str, int]]:
    result: dict[str, tuple[str, int]] = {}
    try:
        with tarfile.open(path, mode="r:*") as archive:
            for member in archive.getmembers():
                name = _safe_rel(member.name, "tar member path")
                if name in result:
                    _fail(f"duplicate tar member: {name}")
                if not member.isfile():
                    _fail(f"nonregular tar member: {name}")
                stream = archive.extractfile(member)
                if stream is None:
                    _fail(f"unreadable tar member: {name}")
                h = hashlib.sha256(); size = 0
                with stream:
                    while True:
                        block = stream.read(CHUNK)
                        if not block:
                            break
                        h.update(block); size += len(block)
                if size != member.size:
                    _fail(f"tar member size mismatch: {name}")
                result[name] = (h.hexdigest(), size)
    except (tarfile.TarError, OSError) as exc:
        _fail(f"invalid tar object {path.name}: {exc}")
    return result


def _safe_extract_member(parent_archive: tarfile.TarFile, validated_members: dict[str, tuple[str, int]],
                         member_name: str, out_path: pathlib.Path, expected_size: int) -> tuple[str, int]:
    _safe_rel(member_name, "archive member path")
    try:
        # The caller has already verified every parent member, including path
        # uniqueness, regular-file type, size, and digest. Reuse one open tar
        # handle and its parsed member table for all child objects.
        if member_name not in validated_members:
            _fail("archive_member is absent from its parent tar")
        try:
            member = parent_archive.getmember(member_name)
        except KeyError:
            _fail("archive_member disappeared from its validated parent tar")
        if not member.isfile() or member.size != expected_size:
            _fail("archive_member must identify one regular member of the declared size")
        stream = parent_archive.extractfile(member)
        if stream is None:
            _fail("archive_member cannot be read")
        with stream:
            return _write_stream(out_path, stream, expected_size)
    except (tarfile.TarError, OSError) as exc:
        _fail(f"cannot resolve archive_member: {exc}")


def _unique_json_pairs(pairs):
    out = {}
    for key, value in pairs:
        if key in out:
            raise ClosureError(f"duplicate JSON key: {key}")
        out[key] = value
    return out


def load_pinned_manifest(path: pathlib.Path, expected_root_sha256: str) -> dict[str, Any]:
    if not _is_digest(expected_root_sha256):
        _fail("an independently supplied expected root SHA-256 is required")
    if path.is_symlink() or not path.is_file():
        _fail("manifest must be a regular non-symlink file")
    raw = path.read_bytes()
    actual = sha256_bytes(raw)
    if actual != expected_root_sha256:
        _fail("root manifest does not match independently supplied digest")
    try:
        manifest = json.loads(raw, object_pairs_hook=_unique_json_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        _fail(f"manifest is not valid JSON: {exc}")
    validate_manifest_shape(manifest)
    if any(row["sha256"] == actual for row in manifest["objects"]):
        _fail("root manifest cannot include itself as a content object")
    return manifest


def write_manifest(path: pathlib.Path, manifest: dict[str, Any]) -> str:
    """Write deterministic JSON and return its detached root SHA-256."""
    validate_manifest_shape(manifest)
    raw = (json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode()
    digest = sha256_bytes(raw)
    if any(row["sha256"] == digest for row in manifest["objects"]):
        _fail("refusing manifest self-inclusion")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() or path.is_symlink():
        _fail("refusing to overwrite an immutable closure manifest")
    tmp = path.with_name(path.name + ".tmp")
    if tmp.exists() or tmp.is_symlink():
        _fail("manifest temporary path already exists")
    with tmp.open("xb") as stream:
        stream.write(raw)
    os.replace(tmp, path)
    return digest


def resolve_objects(manifest: dict[str, Any], *, repository_root: pathlib.Path, cas_root: pathlib.Path,
                    cache_root: pathlib.Path) -> dict[str, pathlib.Path]:
    shaped = validate_manifest_shape(manifest)
    repository_root = repository_root.resolve(strict=True)
    cas_root = cas_root.resolve(strict=True)
    if cache_root.is_symlink():
        _fail("cache root must be a real directory")
    cache_root.mkdir(parents=True, exist_ok=True)
    if cache_root.is_symlink() or not cache_root.is_dir():
        _fail("cache root must be a real directory")
    result: dict[str, pathlib.Path] = {}
    in_progress: set[str] = set()
    parent_tar_handles: dict[str, tarfile.TarFile] = {}
    parent_member_tables: dict[str, dict[str, tuple[str, int]]] = {}

    def resolve(digest: str) -> pathlib.Path:
        if digest in result:
            return result[digest]
        if digest in in_progress:
            _fail("object dependency cycle during resolution")
        in_progress.add(digest)
        row = shaped["objects"][digest]
        dest = cache_root / digest
        if dest.exists():
            if dest.is_symlink() or not dest.is_file():
                _fail("unsafe cache object")
            got, size = sha256_file(dest)
            if got == digest and size == row["size"]:
                result[digest] = dest; in_progress.remove(digest); return dest
            dest.unlink()
        tmp = cache_root / (digest + ".tmp")
        if tmp.exists():
            if tmp.is_symlink() or not tmp.is_file():
                _fail("unsafe stale cache temporary")
            tmp.unlink()
        tr = row["transport"]
        if tr["kind"] == "published_parts":
            h = hashlib.sha256(); size = 0
            with tmp.open("xb") as output:
                for part in tr["parts"]:
                    p = _regular_file(repository_root, part["path"], "published transport part")
                    part_hash, part_size = sha256_file(p)
                    if part_hash != part["sha256"] or part_size != part["size"]:
                        _fail(f"published part identity mismatch: {part['path']}")
                    with p.open("rb") as source:
                        while True:
                            block = source.read(CHUNK)
                            if not block:
                                break
                            output.write(block); h.update(block); size += len(block)
            got = h.hexdigest()
        elif tr["kind"] == "archive_member":
            parent = resolve(tr["parent_sha256"])
            parent_row = shaped["objects"][tr["parent_sha256"]]
            if parent_row["media"] != "tar":
                _fail("archive_member parent is not a tar object")
            parent_sha = tr["parent_sha256"]
            if parent_sha not in parent_member_tables:
                # Validate the complete parent before making any member visible.
                parent_member_tables[parent_sha] = _safe_tar_members(parent)
                parent_tar_handles[parent_sha] = tarfile.open(parent, mode="r:*")
            got, size = _safe_extract_member(parent_tar_handles[parent_sha],
                parent_member_tables[parent_sha], tr["member_path"], tmp, row["size"])
        else:
            p = _regular_file(cas_root, tr["path"], "local CAS object")
            if tr["path"] != digest:
                _fail("CAS path must be the object digest")
            with p.open("rb") as source:
                got, size = _write_stream(tmp, source, row["size"])
        if got != digest or size != row["size"]:
            tmp.unlink(missing_ok=True)
            _fail(f"object digest/size mismatch: {digest}")
        os.replace(tmp, dest)
        result[digest] = dest; in_progress.remove(digest)
        return dest

    try:
        for digest in shaped["objects"]:
            resolve(digest)
    finally:
        for archive in parent_tar_handles.values():
            archive.close()
    return result


def validate_content(manifest: dict[str, Any], objects: dict[str, pathlib.Path]) -> dict[str, Any]:
    shaped = validate_manifest_shape(manifest)
    member_tables: dict[str, dict[str, tuple[str, int]]] = {}
    for digest, row in shaped["objects"].items():
        path = objects[digest]
        got, size = sha256_file(path)
        if got != digest or size != row["size"]:
            _fail(f"resolved object changed after verification: {digest}")
        if row["media"] == "tar":
            member_tables[digest] = _safe_tar_members(path)

    expected: dict[str, tuple[str, int, str, str | None]] = {}
    mount_by_id = shaped["mounts"]
    for mount in manifest["mounts"]:
        obj = mount["object_sha256"]
        prefix = mount["path_prefix"] + "/"
        member_prefix = mount["member_prefix"] or ""
        strip = mount["strip_prefix"] or ""
        if mount["format"] == "blob":
            logical = mount["path_prefix"]
            identity = (obj, shaped["objects"][obj]["size"])
            if logical in expected:
                _fail("duplicate logical path across mounts")
            expected[logical] = (identity[0], identity[1], obj, None)
            continue
        for member_path, (digest, size) in member_tables[obj].items():
            if member_prefix and not member_path.startswith(member_prefix):
                continue
            suffix = member_path[len(strip):] if strip else member_path
            if not suffix:
                _fail("mount prefix projects a file to the namespace root")
            logical = prefix + suffix
            if logical in expected:
                _fail(f"duplicate logical path across mounts: {logical}")
            expected[logical] = (digest, size, obj, member_path)
    actual: dict[str, tuple[str, int, str, str | None]] = {}
    for row in manifest["files"]:
        actual[row["path"]] = (row["sha256"], row["size"], row["source_object_sha256"] if "source_object_sha256" in row else mount_by_id[row["mount_id"]]["object_sha256"], row["member_path"])
    # Current schema source is derived from the owning mount.  Refuse any unexpected row keys
    # before comparing; this also prevents hidden alternate sources.
    if expected != actual:
        missing = sorted(set(expected) - set(actual))[:3]
        extra = sorted(set(actual) - set(expected))[:3]
        _fail(f"logical inventory differs from extracted mounts (missing={missing}, extra={extra})")

    rows_by_path = {r["path"]: r for r in manifest["files"]}
    edge_by_id = shaped["edges"]
    for edge in manifest["provenance"]:
        edge_obj = edge["object_sha256"]
        for binding in edge["bindings"]:
            row = rows_by_path[binding["logical_path"]]
            if binding.get("object_file") is True:
                if row["sha256"] != edge_obj or row["size"] != edge["object_size"]:
                    _fail("provenance object-file binding identity mismatch")
            else:
                member = binding["member_path"]
                actual_member = member_tables.get(edge_obj, {}).get(member)
                if actual_member is None or actual_member != (row["sha256"], row["size"]):
                    _fail("provenance archive-member binding identity mismatch")
    for mount in manifest["mounts"]:
        edge = edge_by_id[mount["provenance_id"]]
        if mount["format"] == "tar":
            got = {(b["logical_path"], b["member_path"]) for b in edge["bindings"] if "member_path" in b}
            expected_bindings = {(r["path"], r["member_path"]) for r in manifest["files"] if r["mount_id"] == mount["id"]}
        else:
            got = {(b["logical_path"], "@object") for b in edge["bindings"] if b.get("object_file") is True}
            expected_bindings = {(r["path"], "@object") for r in manifest["files"] if r["mount_id"] == mount["id"]}
        if not expected_bindings <= got:
            _fail("mount's required provenance edge omits inventory members")
    referenced = set()
    for mount in manifest["mounts"]:
        referenced.add(mount["object_sha256"])
    for edge in manifest["provenance"]:
        referenced.add(edge["object_sha256"])
    for row in manifest["objects"]:
        if row["transport"]["kind"] == "archive_member":
            referenced.add(row["transport"]["parent_sha256"])
    if referenced != set(shaped["objects"]):
        _fail("manifest contains unreachable or undeclared objects")

    logical_bytes = sum(row["size"] for row in manifest["files"])
    unique_object_bytes = sum(row["size"] for row in manifest["objects"])
    published_parts = {}
    for row in manifest["objects"]:
        for part in row["transport"].get("parts", []):
            published_parts[part["path"]] = part["size"]
    published_transport_bytes = sum(published_parts.values())
    return {"status": "pass", "root_id": manifest["root_id"],
            "object_count": len(manifest["objects"]), "file_count": len(manifest["files"]),
            "logical_expanded_bytes": logical_bytes,
            "unique_object_bytes": unique_object_bytes,
            "referenced_published_transport_bytes": published_transport_bytes,
            "newly_published_bytes": sum(row["size"] for row in manifest["objects"] if row["transport"]["kind"] == "cas"),
            "namespace_member_counts": {m["id"]: sum(r["mount_id"] == m["id"] for r in manifest["files"]) for m in manifest["mounts"]}}


def audit_closure(manifest_path: pathlib.Path, expected_root_sha256: str, *, repository_root: pathlib.Path,
                  cas_root: pathlib.Path, cache_root: pathlib.Path) -> tuple[dict[str, Any], dict[str, Any], dict[str, pathlib.Path]]:
    manifest = load_pinned_manifest(manifest_path, expected_root_sha256)
    objects = resolve_objects(manifest, repository_root=repository_root, cas_root=cas_root, cache_root=cache_root)
    report = validate_content(manifest, objects)
    return manifest, report, objects


def hydrate(manifest_path: pathlib.Path, expected_root_sha256: str, *, repository_root: pathlib.Path,
            cas_root: pathlib.Path, cache_root: pathlib.Path, work_root: pathlib.Path,
            destination: pathlib.Path) -> dict[str, Any]:
    manifest, report, objects = audit_closure(manifest_path, expected_root_sha256,
        repository_root=repository_root, cas_root=cas_root, cache_root=cache_root)
    work_root = work_root.resolve(strict=True)
    destination = destination.absolute()
    resolved_parent = destination.parent.resolve(strict=False)
    if resolved_parent != work_root and work_root not in resolved_parent.parents:
        _fail("destination parent resolves outside the disposable work root")
    if destination.exists() or destination.is_symlink():
        _fail("destination must not exist; hydration never overwrites a tree")
    if work_root not in destination.parents:
        _fail("destination must be inside the disposable work root")
    destination.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix="evidence-closure-", dir=destination.parent))
    try:
        mount_by_id = {m["id"]: m for m in manifest["mounts"]}
        rows_by_mount: dict[str, list[dict[str, Any]]] = {}
        for row in manifest["files"]:
            rows_by_mount.setdefault(row["mount_id"], []).append(row)
        for mount_id, rows in rows_by_mount.items():
            mount = mount_by_id[mount_id]
            source_obj = mount["object_sha256"]
            if mount["format"] == "blob":
                row = rows[0]
                out = stage.joinpath(*pathlib.PurePosixPath(row["path"]).parts)
                out.parent.mkdir(parents=True, exist_ok=True)
                with objects[source_obj].open("rb") as source:
                    got, size = _write_stream(out, source, row["size"])
                if got != row["sha256"]:
                    _fail("hydrated blob differs from validated inventory")
                continue
            expected_by_member = {row["member_path"]: row for row in rows}
            copied: set[str] = set()
            try:
                with tarfile.open(objects[source_obj], mode="r:*") as archive:
                    members = archive.getmembers()
                    for member in members:
                        row = expected_by_member.get(member.name)
                        if row is None:
                            continue
                        if member.name in copied or not member.isfile() or member.size != row["size"]:
                            _fail("source archive member changed after audit")
                        out = stage.joinpath(*pathlib.PurePosixPath(row["path"]).parts)
                        out.parent.mkdir(parents=True, exist_ok=True)
                        stream = archive.extractfile(member)
                        if stream is None:
                            _fail("source archive member cannot be reopened")
                        with stream:
                            got, size = _write_stream(out, stream, row["size"])
                        if got != row["sha256"] or size != row["size"]:
                            _fail("hydrated member differs from validated inventory")
                        copied.add(member.name)
            except (tarfile.TarError, OSError) as exc:
                _fail(f"cannot materialize audited members: {exc}")
            if copied != set(expected_by_member):
                _fail("not every validated member was materialized")
        # Publish only after every object and every hydrated path has been validated.
        os.replace(stage, destination)
    except Exception:
        shutil.rmtree(stage, ignore_errors=True)
        raise
    report = dict(report); report["hydrated_path"] = str(destination)
    return report


def pack_regular_files(root_id: str, source_root: pathlib.Path, relative_paths: list[str], cas_root: pathlib.Path,
                       role: str) -> dict[str, Any]:
    """Create a small-file CAS closure; large existing archives should use transports, not this helper."""
    if role not in ROLES:
        _fail("unknown provenance role")
    source_root = source_root.resolve(strict=True)
    cas_root.mkdir(parents=True, exist_ok=True)
    if cas_root.is_symlink() or not cas_root.is_dir():
        _fail("CAS root must be a real directory")
    objects: dict[str, dict[str, Any]] = {}
    files = []
    bindings_by_digest: dict[str, list[dict[str, Any]]] = {}
    seen = set()
    for rel in sorted(relative_paths):
        rel = _safe_rel(rel, "packed logical path")
        if rel in seen:
            _fail("duplicate packed path")
        seen.add(rel)
        source = _regular_file(source_root, rel, "pack source")
        digest, size = sha256_file(source)
        dest = cas_root / digest
        if dest.exists():
            if dest.is_symlink() or not dest.is_file() or sha256_file(dest) != (digest, size):
                _fail("conflicting CAS object")
        else:
            tmp = cas_root / (digest + ".tmp")
            shutil.copyfile(source, tmp)
            if sha256_file(tmp) != (digest, size):
                tmp.unlink(missing_ok=True); _fail("CAS copy verification failed")
            os.replace(tmp, dest)
        objects[digest] = {"sha256": digest, "size": size, "media": "blob", "transport": {"kind": "cas", "path": digest}}
        files.append({"path": rel, "sha256": digest, "size": size, "mount_id": "", "member_path": None})
        bindings_by_digest.setdefault(digest, []).append({"logical_path": rel, "object_file": True})
    if not objects:
        _fail("cannot pack an empty file set")
    mounts = []
    edges = []
    mount_index = 0
    for digest, bindings in sorted(bindings_by_digest.items()):
        edge_id = f"packed-{digest[:16]}"
        edges.append({"id": edge_id, "role": role, "object_sha256": digest,
                      "object_size": objects[digest]["size"], "bindings": bindings})
        for binding in bindings:
            mount_id = f"packed-{mount_index:06d}"
            mounts.append({"id": mount_id, "object_sha256": digest, "format": "blob",
                           "path_prefix": binding["logical_path"], "member_prefix": None,
                           "strip_prefix": None, "provenance_id": edge_id})
            next(row for row in files if row["path"] == binding["logical_path"])["mount_id"] = mount_id
            mount_index += 1
    result = {"schema": SCHEMA, "version": VERSION, "root_id": root_id,
              "required_roles": [role], "objects": sorted(objects.values(), key=lambda x: x["sha256"]),
              "mounts": mounts, "files": sorted(files, key=lambda x: x["path"]), "provenance": edges}
    validate_manifest_shape(result)
    return result


def _parser_checks() -> dict[str, Any]:
    base = {
        "schema": SCHEMA, "version": 1, "root_id": "unit-test", "required_roles": ["canonical_admission"],
        "objects": [{"sha256": "a" * 64, "size": 1, "media": "blob", "transport": {"kind": "cas", "path": "a" * 64}}],
        "mounts": [{"id": "one", "object_sha256": "a" * 64, "format": "blob", "path_prefix": "one", "member_prefix": None, "strip_prefix": None, "provenance_id": "edge"}],
        "files": [{"path": "one", "sha256": "a" * 64, "size": 1, "mount_id": "one", "member_path": None}],
        "provenance": [{"id": "edge", "role": "canonical_admission", "object_sha256": "a" * 64, "object_size": 1, "bindings": [{"logical_path": "one", "object_file": True}]}],
    }
    validate_manifest_shape(base)
    cases: list[tuple[str, Callable[[dict[str, Any]], None]]] = []
    cases.append(("unknown-version", lambda m: m.update(version=2)))
    cases.append(("unknown-role", lambda m: m["provenance"][0].update(role="invented")))
    cases.append(("absolute-path", lambda m: m["files"][0].update(path="/tmp/escape")))
    cases.append(("traversal", lambda m: m["files"][0].update(path="one/../escape")))
    cases.append(("duplicate-logical-path", lambda m: m["files"].append(dict(m["files"][0]))))
    cases.append(("duplicate-object", lambda m: m["objects"].append(dict(m["objects"][0]))))
    cases.append(("dependency-cycle", lambda m: (m["objects"][0].update(media="tar", transport={"kind":"archive_member","parent_sha256":"a"*64,"member_path":"x"}), m["mounts"][0].update(format="tar"))))
    cases.append(("unhashable-required-role", lambda m: m.update(required_roles=[[]])))
    cases.append(("unhashable-mount-object", lambda m: m["mounts"][0].update(object_sha256=[])))
    cases.append(("unhashable-mount-edge", lambda m: m["mounts"][0].update(provenance_id=[])))
    cases.append(("unhashable-edge-role", lambda m: m["provenance"][0].update(role=[])))
    cases.append(("unhashable-edge-object", lambda m: m["provenance"][0].update(object_sha256=[])))
    cases.append(("unhashable-file-mount", lambda m: m["files"][0].update(mount_id=[])))
    cases.append(("unhashable-object-media", lambda m: m["objects"][0].update(media=[])))
    cases.append(("unhashable-mount-format", lambda m: m["mounts"][0].update(format=[])))
    cases.append(("nonstring-member-prefix", lambda m: m["mounts"][0].update(format="tar", member_prefix=[], strip_prefix=[])))
    cases.append(("nonstring-strip-prefix", lambda m: m["mounts"][0].update(format="tar", member_prefix="x/", strip_prefix=[])))
    cases.append(("unhashable-binding-path", lambda m: m["provenance"][0]["bindings"][0].update(logical_path=[])))
    cases.append(("unhashable-binding-member", lambda m: m["provenance"][0]["bindings"][0].update(member_path=[])))
    cases.append(("unhashable-provenance-id", lambda m: m["provenance"][0].update(id=[])))
    cases.append(("unhashable-mount-id", lambda m: m["mounts"][0].update(id=[])))
    passed = []
    for name, mutate in cases:
        m = json.loads(json.dumps(base)); mutate(m)
        try:
            validate_manifest_shape(m)
        except ClosureError:
            passed.append(name)
        else:
            _fail(f"parser control unexpectedly accepted: {name}")
    try:
        json.loads('{"schema":"evidence-closure-v1","schema":"evidence-closure-v1"}', object_pairs_hook=_unique_json_pairs)
    except ClosureError:
        passed.append("duplicate-json-key")
    else:
        _fail("parser control unexpectedly accepted duplicate JSON keys")
    return {"status": "pass", "accepted_baseline": True, "rejected_controls": passed, "count": len(passed)}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("validate")
    hydrate_p = sub.add_parser("hydrate")
    for p in (check, hydrate_p):
        p.add_argument("--manifest", type=pathlib.Path, required=True)
        p.add_argument("--expected-root-sha256", required=True)
        p.add_argument("--repository-root", type=pathlib.Path, required=True)
        p.add_argument("--cas-root", type=pathlib.Path, required=True)
        p.add_argument("--cache-root", type=pathlib.Path, required=True)
        p.add_argument("--report", type=pathlib.Path)
    if hydrate_p:
        hydrate_p.add_argument("--work-root", type=pathlib.Path, required=True)
        hydrate_p.add_argument("--destination", type=pathlib.Path, required=True)
    sub.add_parser("parser-checks")
    args = parser.parse_args(argv)
    try:
        if args.command == "parser-checks":
            result = _parser_checks()
        elif args.command == "validate":
            _manifest, result, _objects = audit_closure(args.manifest, args.expected_root_sha256,
                repository_root=args.repository_root, cas_root=args.cas_root, cache_root=args.cache_root)
        else:
            result = hydrate(args.manifest, args.expected_root_sha256, repository_root=args.repository_root,
                cas_root=args.cas_root, cache_root=args.cache_root, work_root=args.work_root, destination=args.destination)
        payload = json.dumps(result, sort_keys=True, indent=2) + "\n"
        if args.command != "parser-checks" and args.report:
            args.report.write_text(payload)
        sys.stdout.write(payload)
        return 0
    except ClosureError as exc:
        sys.stderr.write(f"evidence-closure: {exc}\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
