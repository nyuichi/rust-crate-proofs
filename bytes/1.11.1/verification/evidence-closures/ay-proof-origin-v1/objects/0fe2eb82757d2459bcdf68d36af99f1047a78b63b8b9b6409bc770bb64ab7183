#!/usr/bin/env python3
"""Small offline schema/transport controls for evidence_closure.py."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import pathlib
import sys
import tempfile


HERE = pathlib.Path(__file__).resolve().parent
FIXTURE = HERE.parent / "evidence-closures" / "av-admitted-v1" / "fixtures" / "corrupt-required-object.json"
SPEC = importlib.util.spec_from_file_location("evidence_closure", HERE / "evidence_closure.py")
ec = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(ec)
sys.modules["evidence_closure"] = ec

BOOTSTRAP_DIR = HERE.parent / "evidence-closures" / "av-admitted-v1"
sys.path.insert(0, str(BOOTSTRAP_DIR))
BOOTSTRAP_SPEC = importlib.util.spec_from_file_location("bootstrap_av", BOOTSTRAP_DIR / "bootstrap_av.py")
bootstrap_av = importlib.util.module_from_spec(BOOTSTRAP_SPEC)
assert BOOTSTRAP_SPEC.loader is not None
BOOTSTRAP_SPEC.loader.exec_module(bootstrap_av)


def check_corrupt_required_object() -> dict:
    fixture = json.loads(FIXTURE.read_text())
    assert fixture["schema"] == "evidence-closure-corruption-fixture-v1"
    assert fixture["mutation"] == "replace-required-cas-bytes-keep-manifest"
    with tempfile.TemporaryDirectory(prefix="closure-corrupt-", dir="/workspace/work") as raw:
        tmp = pathlib.Path(raw)
        source = tmp / "source"
        source.mkdir()
        (source / "required.json").write_bytes(b'{"verified":true}\n')
        cas = tmp / "cas"
        manifest = ec.pack_regular_files("corruption-fixture", source, ["required.json"], cas, "diagnostic_control")
        manifest_path = tmp / "closure.json"
        root_sha = ec.write_manifest(manifest_path, manifest)
        expected_object = manifest["objects"][0]["sha256"]
        (cas / expected_object).write_bytes(b"corrupted")
        called = False
        destination = tmp / "hydrated"
        try:
            _m, _r, _o = ec.audit_closure(manifest_path, root_sha, repository_root=tmp,
                cas_root=cas, cache_root=tmp / "cache")
            # This callback stands for the evidence checker; it is reached only
            # after the complete object graph and inventory pass validation.
            called = True
        except ec.ClosureError as exc:
            assert "mismatch" in str(exc)
        else:
            raise AssertionError("corrupted required object was accepted")
        assert not called
        assert not destination.exists()
    return {"id": "corrupted-required-object", "rejected_before_checker": True,
            "corrupt_object_sha256": expected_object}


def check_valid_roundtrip() -> dict:
    with tempfile.TemporaryDirectory(prefix="closure-roundtrip-", dir="/workspace/work") as raw:
        tmp = pathlib.Path(raw)
        source = tmp / "source"; source.mkdir()
        (source / "a.txt").write_bytes(b"alpha\n")
        (source / "sub").mkdir()
        (source / "sub/b.txt").write_bytes(b"beta\n")
        cas = tmp / "cas"
        manifest = ec.pack_regular_files("roundtrip", source, ["a.txt", "sub/b.txt"], cas, "canonical_admission")
        path = tmp / "manifest.json"
        root_sha = ec.write_manifest(path, manifest)
        report = ec.hydrate(path, root_sha, repository_root=tmp, cas_root=cas,
            cache_root=tmp / "cache", work_root=tmp, destination=tmp / "hydrated")
        assert report["status"] == "pass" and report["file_count"] == 2
        assert (tmp / "hydrated/a.txt").read_bytes() == b"alpha\n"
        assert (tmp / "hydrated/sub/b.txt").read_bytes() == b"beta\n"
        return {"status": "pass", "file_count": report["file_count"],
                "logical_expanded_bytes": report["logical_expanded_bytes"]}


def check_existing_hydration_destination_rejected() -> dict:
    with tempfile.TemporaryDirectory(prefix="closure-stale-tree-", dir="/workspace/work") as raw:
        stale = pathlib.Path(raw) / "hydrated"
        stale.mkdir()
        (stale / "tampered.txt").write_text("unverified content\n")
        checker_called = False
        try:
            bootstrap_av.require_fresh_hydration_destination(stale)
            checker_called = True
        except ec.ClosureError as exc:
            assert "must not exist" in str(exc)
        else:
            raise AssertionError("bootstrap accepted a preexisting hydration tree")
        assert not checker_called
    return {"id": "existing-hydration-tree", "rejected_before_checker": True}


def main() -> int:
    parser = ec._parser_checks()
    roundtrip = check_valid_roundtrip()
    corruption = check_corrupt_required_object()
    stale_tree = check_existing_hydration_destination_rejected()
    print(json.dumps({"status": "pass", "parser": parser, "roundtrip": roundtrip,
                      "corruption": corruption, "stale_tree": stale_tree}, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
