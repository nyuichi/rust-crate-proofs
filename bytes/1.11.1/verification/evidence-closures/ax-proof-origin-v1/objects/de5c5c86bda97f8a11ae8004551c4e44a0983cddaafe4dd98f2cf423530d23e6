#!/usr/bin/env python3
"""Offline parser, import, transport, and corruption controls for evidence_capture.py."""
from __future__ import annotations

import hashlib
import io
import json
import pathlib
import sys
import tarfile
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import evidence_closure as ec
import evidence_capture as capture


def pinned_recipe(path: pathlib.Path, recipe: dict) -> str:
    raw = (json.dumps(recipe, sort_keys=True, indent=2) + "\n").encode()
    path.write_bytes(raw)
    return hashlib.sha256(raw).hexdigest()


def base_recipe() -> dict:
    return {
        "schema": capture.SCHEMA,
        "version": capture.VERSION,
        "root_id": "capture-test",
        "required_roles": ["proof_origin"],
        "sources": [{"id": "src", "kind": "local_file", "path": "input.txt"}],
        "imports": [],
        "mounts": [{"id": "input", "source_id": "src", "path_prefix": "proof/input.txt", "role": "proof_origin"}],
    }


def parser_controls() -> dict:
    cases = []
    bad = base_recipe(); bad["version"] = True; cases.append(("boolean-version", bad))
    bad = base_recipe(); bad["required_roles"] = [[]]; cases.append(("unhashable-role", bad))
    bad = base_recipe(); bad["sources"][0]["path"] = "../escape"; cases.append(("source-traversal", bad))
    bad = base_recipe(); bad["mounts"][0]["source_id"] = []; cases.append(("unhashable-source-reference", bad))
    bad = base_recipe(); bad["mounts"][0]["role"] = "invented"; cases.append(("unknown-role", bad))
    bad = base_recipe(); bad["sources"].append(dict(bad["sources"][0])); cases.append(("duplicate-source-id", bad))
    bad = base_recipe(); bad["mounts"].append(dict(bad["mounts"][0])); cases.append(("duplicate-mount-id", bad))
    bad = base_recipe(); bad["imports"] = [{"id": "old", "manifest_path": "old.json", "manifest_sha256": "bad",
        "cas_root": "old-cas", "mounts": []}]; cases.append(("empty-import-selection", bad))
    rejected = []
    for name, recipe in cases:
        try:
            capture._validate_recipe_shape(recipe)
        except ec.ClosureError:
            rejected.append(name)
        else:
            raise AssertionError(f"recipe parser accepted control: {name}")
    try:
        json.loads('{"schema":"evidence-capture-recipe-v1","schema":"evidence-capture-recipe-v1"}',
            object_pairs_hook=ec._unique_json_pairs)
    except ec.ClosureError:
        rejected.append("duplicate-json-key")
    else:
        raise AssertionError("recipe parser accepted duplicate JSON keys")
    return {"status": "pass", "rejected": rejected, "count": len(rejected)}


def create_tar(path: pathlib.Path, files: dict[str, bytes]) -> None:
    with tarfile.open(path, "w") as archive:
        for name, payload in files.items():
            info = tarfile.TarInfo(name)
            info.size = len(payload)
            archive.addfile(info, io.BytesIO(payload))


def roundtrip_import_and_archive_member() -> dict:
    with tempfile.TemporaryDirectory(prefix="evidence-capture-roundtrip-", dir="/tmp") as raw:
        root = pathlib.Path(raw)
        repo = root / "repo"; repo.mkdir()
        source = root / "source"; source.mkdir()
        (source / "input.txt").write_bytes(b"current source\n")
        nested = repo / "nested.tar"
        create_tar(nested, {"inside.txt": b"nested archive member\n"})
        outer = repo / "outer.tar"
        create_tar(outer, {"nested.tar": nested.read_bytes(), "outer.txt": b"outer member\n"})

        # Build a small prior closure with one CAS object; the child must pin,
        # audit, and import its selected mount without trusting the recipe's
        # claim about the old file bytes.
        old_source = root / "old-source"; old_source.mkdir()
        (old_source / "old.txt").write_bytes(b"old ancestry\n")
        old_cas = repo / "old-cas"
        old = ec.pack_regular_files("old-closure", old_source, ["old.txt"], old_cas, "proof_origin")
        old_dir = repo / "prior"; old_dir.mkdir()
        old_path = old_dir / "manifest.json"
        old_sha = ec.write_manifest(old_path, old)

        recipe = {
            "schema": capture.SCHEMA, "version": capture.VERSION, "root_id": "roundtrip-import",
            "required_roles": ["proof_origin", "reused_ancestor"],
            "sources": [
                {"id": "current", "kind": "local_file", "path": "input.txt"},
                {"id": "outer", "kind": "published_parts", "media": "tar", "parts": ["outer.tar"]},
                {"id": "nested", "kind": "archive_member", "parent": "outer", "member_path": "nested.tar"},
            ],
            "imports": [{"id": "prior", "manifest_path": "prior/manifest.json", "manifest_sha256": old_sha,
                "cas_root": "old-cas", "mounts": [{"source_mount_id": old["mounts"][0]["id"],
                    "path_prefix": "ancestry/old.txt", "role": "proof_origin"}]}],
            "mounts": [
                {"id": "current-file", "source_id": "current", "path_prefix": "proof/input.txt", "role": "proof_origin"},
                {"id": "outer-tar", "source_id": "outer", "path_prefix": "archive/outer", "role": "proof_origin"},
                {"id": "nested-tar", "source_id": "nested", "path_prefix": "archive/nested", "role": "proof_origin"},
            ],
        }
        recipe_path = root / "recipe.json"
        recipe_sha = pinned_recipe(recipe_path, recipe)
        cas = root / "new-cas"; cache = root / "cache"
        output = root / "output" / "manifest.json"
        report_path = root / "output" / "capture.json"
        report = capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
            cas_root=cas, cache_root=cache, output_manifest=output, report_path=report_path)
        manifest, audit, _objects = ec.audit_closure(output, report["manifest_sha256"],
            repository_root=repo, cas_root=cas, cache_root=root / "replay-cache")
        paths = {row["path"] for row in manifest["files"]}
        assert paths == {"proof/input.txt", "archive/outer/nested.tar", "archive/outer/outer.txt",
            "archive/nested/inside.txt", "ancestry/old.txt", "lineage/capture-recipe.json",
            "lineage/imports/prior/manifest.json", "lineage/imports/prior/selection.json"}
        assert audit["file_count"] == 8
        by_path = {row["path"]: row for row in manifest["files"]}
        recipe_lineage = cas / by_path["lineage/capture-recipe.json"]["sha256"]
        assert recipe_lineage.read_bytes() == recipe_path.read_bytes()
        object_by_digest = {row["sha256"]: row for row in manifest["objects"]}
        raw_manifest_digest = by_path["lineage/imports/prior/manifest.json"]["sha256"]
        raw_manifest_object = object_by_digest[raw_manifest_digest]
        assert raw_manifest_object["transport"] == {"kind": "published_parts", "parts": [{
            "path": "prior/manifest.json", "sha256": old_sha, "size": old_path.stat().st_size}]}
        assert not (cas / raw_manifest_digest).exists()
        assert (repo / "prior/manifest.json").read_bytes() == old_path.read_bytes()
        imported_object_digest = old["objects"][0]["sha256"]
        imported_object = object_by_digest[imported_object_digest]
        assert imported_object["transport"] == {"kind": "published_parts", "parts": [{
            "path": "old-cas/" + imported_object_digest, "sha256": imported_object_digest,
            "size": old["objects"][0]["size"]}]}
        assert not (cas / imported_object_digest).exists()
        selection_lineage = cas / by_path["lineage/imports/prior/selection.json"]["sha256"]
        selection = json.loads(selection_lineage.read_bytes())
        assert selection["source_manifest_sha256"] == old_sha
        assert selection["source_manifest_size"] == old_path.stat().st_size
        assert selection["selections"] == [{"output_mount_id": "prior-mount-" +
            hashlib.sha256(old["mounts"][0]["id"].encode()).hexdigest()[:16],
            "output_path_prefix": "ancestry/old.txt", "output_role": "proof_origin",
            "source_format": "blob", "source_member_prefix": None, "source_mount_id": old["mounts"][0]["id"],
            "source_object_sha256": old["mounts"][0]["object_sha256"],
            "source_path_prefix": "old.txt", "source_provenance_id": old["mounts"][0]["provenance_id"],
            "source_role": "proof_origin", "source_strip_prefix": None}]
        assert report["imported_manifests"][0]["manifest_sha256"] == old_sha
        return {"status": "pass", "file_count": audit["file_count"], "lineage_file_count": 3,
            "manifest_sha256": report["manifest_sha256"], "imported_manifest_sha256": old_sha}


def imported_role_escalation_rejected() -> dict:
    with tempfile.TemporaryDirectory(prefix="evidence-capture-role-", dir="/tmp") as raw:
        root = pathlib.Path(raw)
        repo = root / "repo"; repo.mkdir()
        source = root / "source"; source.mkdir(); (source / "input.txt").write_bytes(b"current\n")
        old_source = root / "old-source"; old_source.mkdir(); (old_source / "old.txt").write_bytes(b"ancestor\n")
        old_cas = repo / "old-cas"
        old = ec.pack_regular_files("old-closure", old_source, ["old.txt"], old_cas, "proof_origin")
        prior = repo / "prior"; prior.mkdir()
        old_manifest_path = prior / "manifest.json"
        old_sha = ec.write_manifest(old_manifest_path, old)
        recipe = base_recipe()
        recipe["root_id"] = "role-control"
        recipe["required_roles"] = ["proof_origin", "reused_ancestor"]
        recipe["imports"] = [{"id": "prior", "manifest_path": "prior/manifest.json",
            "manifest_sha256": old_sha, "cas_root": "old-cas", "mounts": [
                {"source_mount_id": old["mounts"][0]["id"], "path_prefix": "ancestor/old.txt",
                 "role": "reused_ancestor"}]}]
        recipe_path = root / "recipe.json"; recipe_sha = pinned_recipe(recipe_path, recipe)
        output = root / "out" / "manifest.json"; report = root / "out" / "report.json"
        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=root / "cas", cache_root=root / "cache", output_manifest=output, report_path=report)
        except ec.ClosureError as exc:
            assert "cannot change" in str(exc)
            assert not output.exists() and not report.exists()
            return {"status": "pass", "rejected_role": "reused_ancestor", "source_role": "proof_origin"}
        raise AssertionError("imported source provenance role was reassigned")


def filesystem_preflight_controls() -> dict:
    with tempfile.TemporaryDirectory(prefix="evidence-capture-paths-", dir="/tmp") as raw:
        root = pathlib.Path(raw)
        repo = root / "repo"; repo.mkdir()
        source = root / "source"; source.mkdir(); (source / "input.txt").write_bytes(b"path check\n")
        recipe = base_recipe(); recipe_path = root / "recipe.json"; recipe_sha = pinned_recipe(recipe_path, recipe)
        real = root / "real"; real.mkdir()
        link = root / "symlink-parent"
        try:
            link.symlink_to(real, target_is_directory=True)
        except (OSError, NotImplementedError):
            return {"status": "skip", "reason": "symlink creation unavailable"}

        source_link = root / "source-link"
        source_link.symlink_to(source, target_is_directory=True)
        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source_link,
                cas_root=root / "source-link-cas", cache_root=root / "source-link-cache",
                output_manifest=root / "source-link-out" / "manifest.json",
                report_path=root / "source-link-out" / "report.json")
        except ec.ClosureError as exc:
            assert "symlink" in str(exc)
            assert not (root / "source-link-cas").exists()
        else:
            raise AssertionError("source-root symlink was accepted")

        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=link / "cas", cache_root=root / "cache", output_manifest=root / "out" / "manifest.json",
                report_path=root / "out" / "report.json")
        except ec.ClosureError as exc:
            assert "symlink" in str(exc)
            assert not (real / "cas").exists()
        else:
            raise AssertionError("CAS symlink ancestor was accepted")

        output_link = root / "output-link"
        output_link.symlink_to(real, target_is_directory=True)
        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=root / "cas", cache_root=root / "cache", output_manifest=output_link / "manifest.json",
                report_path=output_link / "report.json")
        except ec.ClosureError as exc:
            assert "symlink" in str(exc)
            assert not (root / "cas").exists()
        else:
            raise AssertionError("output symlink ancestor was accepted")

        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=root / "same", cache_root=root / "same", output_manifest=root / "manifest.json",
                report_path=root / "report.json")
        except ec.ClosureError as exc:
            assert "disjoint" in str(exc)
            assert not (root / "same").exists()
        else:
            raise AssertionError("canonical CAS/cache alias was accepted")

        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=source / "writable-cas", cache_root=root / "separate-cache",
                output_manifest=root / "source-cas-out" / "manifest.json",
                report_path=root / "source-cas-out" / "report.json")
        except ec.ClosureError as exc:
            assert "source input tree" in str(exc)
            assert not (source / "writable-cas").exists()
        else:
            raise AssertionError("CAS root inside source input tree was accepted")

        repository_cas = repo / "writable-cas"
        repository_out = root / "repository-cas-out"
        report = capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
            cas_root=repository_cas, cache_root=root / "repository-cas-cache",
            output_manifest=repository_out / "manifest.json", report_path=repository_out / "report.json")
        assert report["status"] == "pass" and repository_cas.is_dir()

        nested_output = root / "nested-output"
        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=root / "nested-cas", cache_root=root / "nested-cache",
                output_manifest=nested_output, report_path=nested_output / "report.json")
        except ec.ClosureError as exc:
            assert "disjoint paths" in str(exc)
            assert not nested_output.exists() and not (root / "nested-cas").exists()
        else:
            raise AssertionError("ancestor/descendant output path alias was accepted")
        return {"status": "pass", "rejected": ["source-root symlink", "CAS symlink ancestor",
            "output symlink ancestor", "canonical CAS/cache alias", "CAS inside source tree",
            "manifest/report ancestor alias"], "accepted": ["CAS inside repository locator"]}


def unique_copy_temporary_control() -> dict:
    with tempfile.TemporaryDirectory(prefix="evidence-capture-temp-", dir="/tmp") as raw:
        root = pathlib.Path(raw)
        source = root / "source"; source.write_bytes(b"safe copied bytes")
        cas = root / "cas"; cas.mkdir()
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        destination = cas / digest
        sentinel = root / "sentinel"; sentinel.write_bytes(b"must stay unchanged")
        stale = destination.with_name(destination.name + ".tmp")
        try:
            stale.symlink_to(sentinel)
        except (OSError, NotImplementedError):
            return {"status": "skip", "reason": "symlink creation unavailable"}
        capture._copy_verified(source, destination, digest, source.stat().st_size)
        assert destination.read_bytes() == source.read_bytes()
        assert sentinel.read_bytes() == b"must stay unchanged"
        assert stale.is_symlink()

        linked_destination = cas / ("a" * 64)
        linked_destination.symlink_to(sentinel)
        try:
            capture._copy_verified(source, linked_destination, "a" * 64, source.stat().st_size)
        except ec.ClosureError as exc:
            assert "unsafe existing" in str(exc)
        else:
            raise AssertionError("symlink CAS destination was accepted")
        return {"status": "pass", "random_exclusive_temp_ignores_stale_fixed_name": True,
            "existing_destination_symlink_rejected": True}


def reserved_lineage_collision_rejected() -> dict:
    with tempfile.TemporaryDirectory(prefix="evidence-capture-lineage-", dir="/tmp") as raw:
        root = pathlib.Path(raw)
        repo = root / "repo"; repo.mkdir()
        source = root / "source"; source.mkdir(); (source / "input.txt").write_bytes(b"content\n")
        recipe = base_recipe()
        recipe["mounts"][0]["path_prefix"] = "lineage/capture-recipe.json"
        recipe_path = root / "recipe.json"; recipe_sha = pinned_recipe(recipe_path, recipe)
        output = root / "out" / "manifest.json"; report = root / "out" / "report.json"
        try:
            capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
                cas_root=root / "cas", cache_root=root / "cache", output_manifest=output, report_path=report)
        except ec.ClosureError as exc:
            assert "lineage path" in str(exc)
            assert not output.exists() and not report.exists()
            return {"status": "pass", "reserved_path": "lineage/capture-recipe.json"}
        raise AssertionError("user mount shadowed the reserved recipe-lineage path")


def corrupt_object_rejected() -> dict:
    with tempfile.TemporaryDirectory(prefix="evidence-capture-corrupt-", dir="/tmp") as raw:
        root = pathlib.Path(raw)
        repo = root / "repo"; repo.mkdir()
        source = root / "source"; source.mkdir(); (source / "input.txt").write_bytes(b"must remain exact\n")
        recipe = base_recipe(); recipe_path = root / "recipe.json"; recipe_sha = pinned_recipe(recipe_path, recipe)
        cas = root / "cas"; output = root / "manifest.json"; report_path = root / "capture.json"
        report = capture.compose(recipe_path, recipe_sha, repository_root=repo, source_root=source,
            cas_root=cas, cache_root=root / "cache", output_manifest=output, report_path=report_path)
        digest = ec.load_pinned_manifest(output, report["manifest_sha256"])["objects"][0]["sha256"]
        (cas / digest).write_bytes(b"tampered required bytes")
        try:
            ec.audit_closure(output, report["manifest_sha256"], repository_root=repo,
                cas_root=cas, cache_root=root / "replay-cache")
        except ec.ClosureError as exc:
            return {"status": "pass", "rejected_before_audit": True, "reason": str(exc)}
        raise AssertionError("tampered required object passed closure audit")


def main() -> int:
    result = {"status": "pass", "parser": parser_controls(),
        "roundtrip_import_archive_member": roundtrip_import_and_archive_member(),
        "imported_role_escalation": imported_role_escalation_rejected(),
        "reserved_lineage_collision": reserved_lineage_collision_rejected(),
        "filesystem_preflight": filesystem_preflight_controls(),
        "unique_copy_temporary": unique_copy_temporary_control(),
        "corrupt_required_object": corrupt_object_rejected()}
    print(json.dumps(result, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
