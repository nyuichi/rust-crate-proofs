#!/usr/bin/env python3
"""AV-specific inventory checks and archived audit replay for the generic closure."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
TOOLS = HERE.parents[1] / "tools"
sys.path.insert(0, str(TOOLS))
import evidence_closure as ec  # noqa: E402
from build_manifest import ANCESTORS, CANONICAL_SHA, ORIGIN_SHA  # noqa: E402

CANONICAL_MEMBERS = 2164
ORIGIN_TARGETS = 167
PROVER_LEAVES = 1683
AV_SLUG = "original-promotable-suffix-promotion-2026-10-09"
ANCESTOR_PROBES = {
    "au": "original-owned-view-call-summaries-2026-10-09",
    "at": "original-owned-view-clone-2026-10-09",
    "ar": "original-bytes-cursor-closure-2026-10-09",
    "as": "original-nonnull-view-boundaries-2026-10-09",
    "ap": "original-shared-finite-owners-2026-10-09",
    "aq": "original-shared-slice-views-2026-10-09",
}
ORIGIN_MEMBER = "inputs/av-proof-origin/av-full-diagnostic-v1.tar.gz"
ORIGIN_RECEIPT_MEMBER = "inputs/av-proof-origin/av-full-diagnostic-v1.json"
ORIGIN_LOG_MEMBER = "inputs/av-proof-origin/av-full-diagnostic-v1.log"


def _sha(path: pathlib.Path) -> tuple[str, int]:
    return ec.sha256_file(path)


def _read_json(path: pathlib.Path, label: str) -> dict[str, Any]:
    if not path.is_file() or path.is_symlink():
        raise ec.ClosureError(f"missing regular {label}: {path}")
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ec.ClosureError(f"{label} must be a JSON object")
    return value


def require_fresh_hydration_destination(destination: pathlib.Path) -> None:
    """Fail before any audit/checker use if a destination already exists."""
    if destination.exists() or destination.is_symlink():
        raise ec.ClosureError("bootstrap hydration destination must not exist; refusing an unverified tree")


def verify_av_inventory(manifest: dict[str, Any], objects: dict[str, pathlib.Path],
                        hydrated: pathlib.Path | None = None) -> dict[str, Any]:
    rows = {row["path"]: row for row in manifest["files"]}
    canonical_rows = {path.removeprefix("canonical/"): row for path, row in rows.items()
                      if row["mount_id"] == "canonical"}
    origin_rows = {path.removeprefix("proof-origin/"): row for path, row in rows.items()
                   if row["mount_id"] == "proof-origin"}
    if len(canonical_rows) != CANONICAL_MEMBERS:
        raise ec.ClosureError(f"canonical mount has {len(canonical_rows)} members, expected {CANONICAL_MEMBERS}")
    canonical_object = objects[CANONICAL_SHA]
    origin_object = objects[ORIGIN_SHA]
    canonical_table = ec._safe_tar_members(canonical_object)
    origin_table = ec._safe_tar_members(origin_object)
    if set(canonical_table) != set(canonical_rows) or set(origin_table) != set(origin_rows):
        raise ec.ClosureError("hydrated canonical/origin mount paths do not equal their exact tar member tables")

    base = hydrated
    if base is not None:
        canonical_receipt_path = base / "source-records/av-positive-reuse-canonical-v1.json"
        origin_receipt_path = base / "source-records/av-full-diagnostic-v1.json"
        canonical_audit_path = base / "source-records/AV_CANONICAL_AUDIT.json"
        canonical_receipt = _read_json(canonical_receipt_path, "AV canonical receipt")
        origin_receipt = _read_json(origin_receipt_path, "AV origin receipt")
        canonical_audit = _read_json(canonical_audit_path, "AV canonical audit")
        receipt_table = {row.get("path"): row.get("sha256") for row in canonical_receipt.get("members", [])
                         if isinstance(row, dict)}
        origin_receipt_table = {row.get("path"): row.get("sha256") for row in origin_receipt.get("members", [])
                                if isinstance(row, dict)}
        if (canonical_receipt.get("archive_sha256") != CANONICAL_SHA or
                receipt_table != {name: identity[0] for name, identity in canonical_table.items()}):
            raise ec.ClosureError("published AV canonical receipt does not match every hydrated archive member")
        if (origin_receipt.get("archive_sha256") != ORIGIN_SHA or
                origin_receipt_table != {name: identity[0] for name, identity in origin_table.items()}):
            raise ec.ClosureError("published AV origin receipt does not match every hydrated origin member")
        if canonical_audit.get("status") != "pass" or \
                canonical_audit.get("archive_sha256") != CANONICAL_SHA or \
                canonical_audit.get("archive_members") != CANONICAL_MEMBERS or \
                canonical_audit.get("duplicate_unsafe_or_nonregular_members") != 0:
            raise ec.ClosureError("independent AV canonical audit does not bind the published archive")
        if canonical_receipt.get("status") != "admitted_reuse" or \
                canonical_receipt.get("full_original_admitted") is not False or \
                canonical_receipt.get("statistics") != {"files": ORIGIN_TARGETS, "prover": PROVER_LEAVES,
                                                         "null": 0, "structural": 0}:
            raise ec.ClosureError("AV canonical receipt does not record the reviewed proof-reuse result")
        if origin_receipt.get("status") != "diagnostic" or \
                origin_receipt.get("statistics") != {"files": ORIGIN_TARGETS, "prover": PROVER_LEAVES,
                                                      "null": 0, "structural": 0}:
            raise ec.ClosureError("AV proof-origin receipt is not the complete diagnostic proof run")
        if canonical_receipt.get("targets") != origin_receipt.get("targets"):
            raise ec.ClosureError("canonical proof target table differs from the proof origin")
        target_rows = canonical_receipt.get("targets")
        if not isinstance(target_rows, list) or len(target_rows) != ORIGIN_TARGETS:
            raise ec.ClosureError("AV target table has the wrong target count")
        target_hashes: dict[str, str] = {}
        for target in target_rows:
            for path_field, digest_field in (("coma", "coma_sha256"), ("proof", "proof_sha256")):
                logical = target.get(path_field); digest = target.get(digest_field)
                if not isinstance(logical, str) or not isinstance(digest, str) or logical in target_hashes:
                    raise ec.ClosureError("AV target receipt has malformed or duplicate target paths")
                target_hashes[logical] = digest
        for name, digest in target_hashes.items():
            if origin_table.get(name, (None, None))[0] != digest:
                raise ec.ClosureError(f"proof-origin target differs from receipt: {name}")
        if len(target_hashes) != ORIGIN_TARGETS * 2:
            raise ec.ClosureError("proof origin does not contain 334 unique Coma/proof paths")

    ancestor_rows: dict[str, dict[str, Any]] = {}
    for slug, (digest, member_name) in ANCESTORS.items():
        parent_binding = canonical_table.get(member_name)
        if parent_binding is None or parent_binding[0] != digest:
            raise ec.ClosureError(f"canonical archive does not bind the exact {slug.upper()} archive")
        if not any(mount["object_sha256"] == digest and mount["id"] == "ancestor-" + slug
                   for mount in manifest["mounts"]):
            raise ec.ClosureError(f"logical closure omits the mounted {slug.upper()} ancestor")
        ancestor_rows[slug] = {"sha256": digest, "size": parent_binding[1],
                               "member_count": sum(row["mount_id"] == "ancestor-" + slug
                                                   for row in manifest["files"])}
    return {"canonical_sha256": CANONICAL_SHA, "canonical_members": len(canonical_table),
            "origin_sha256": ORIGIN_SHA, "origin_members": len(origin_table),
            "origin_targets": ORIGIN_TARGETS, "origin_coma_and_proof_files": ORIGIN_TARGETS * 2,
            "proof_statistics": {"files": ORIGIN_TARGETS, "prover": PROVER_LEAVES,
                                 "null": 0, "structural": 0},
            "reused_ancestors": ancestor_rows}


def _copy_regular(src: pathlib.Path, dst: pathlib.Path) -> None:
    if not src.is_file() or src.is_symlink():
        raise ec.ClosureError(f"scratch restore source is not a regular file: {src}")
    if dst.exists() or dst.is_symlink():
        if dst.is_symlink() or not dst.is_file() or _sha(src) != _sha(dst):
            raise ec.ClosureError(f"scratch restore found a conflicting existing file: {dst}")
        return
    dst.parent.mkdir(parents=True, exist_ok=True)
    try:
        os.link(src, dst)
    except OSError:
        shutil.copyfile(src, dst)
    if _sha(src) != _sha(dst):
        raise ec.ClosureError(f"scratch restore copy failed verification: {dst}")


def restore_av_checkout(hydrated: pathlib.Path, checkout: pathlib.Path, cache_root: pathlib.Path) -> pathlib.Path:
    hydrated = hydrated.resolve(strict=True)
    checkout = checkout.absolute()
    if checkout.exists() or checkout.is_symlink():
        raise ec.ClosureError("scratch checkout destination already exists")
    repo_bytes = checkout / "bytes"
    archived_repo = hydrated / "canonical/inputs/repository/bytes"
    if not archived_repo.is_dir() or archived_repo.is_symlink():
        raise ec.ClosureError("AV archive omits its repository snapshot")
    shutil.copytree(archived_repo, repo_bytes, symlinks=False)
    av_probe = repo_bytes / "1.11.1/verification/probes" / AV_SLUG
    av_probe.mkdir(parents=True, exist_ok=True)
    canonical_probe = hydrated / "canonical/probe"
    for source in canonical_probe.rglob("*"):
        if source.is_symlink():
            raise ec.ClosureError("hydrated AV canonical probe unexpectedly contains a symlink")
        if source.is_dir():
            continue
        relative = source.relative_to(canonical_probe)
        _copy_regular(source, av_probe / relative)
    evidence = av_probe / "evidence"
    for source in (hydrated / "source-records").iterdir():
        if source.is_file():
            _copy_regular(source, evidence / source.name)
    for name in ("av-full-diagnostic-v1.tar.gz", "av-full-diagnostic-v1.json", "av-full-diagnostic-v1.log"):
        source = hydrated / "canonical" / "inputs/av-proof-origin" / name
        _copy_regular(source, evidence / name)
    canonical_archive = cache_root / CANONICAL_SHA
    if not canonical_archive.is_file() or _sha(canonical_archive)[0] != CANONICAL_SHA:
        raise ec.ClosureError("verified AV canonical object is missing from the offline cache")
    _copy_regular(canonical_archive, evidence / "av-positive-reuse-canonical-v1.tar.gz")

    # The selected repository snapshot intentionally does not duplicate every
    # published ancestor probe tree. Reconstruct each sibling from its mounted
    # archive object so imported auditors see the exact published files rather
    # than a partial repository snapshot or live workspace.
    probes_root = repo_bytes / "1.11.1/verification/probes"
    for slug, folder in ANCESTOR_PROBES.items():
        source = hydrated / "ancestors" / slug / "probe"
        if not source.is_dir() or source.is_symlink():
            raise ec.ClosureError(f"hydrated {slug.upper()} ancestor probe is missing or redirected")
        destination = probes_root / folder
        if destination.is_symlink():
            raise ec.ClosureError(f"scratch {slug.upper()} ancestor destination is redirected")
        destination.mkdir(parents=True, exist_ok=True)
        for path in source.rglob("*"):
            if path.is_symlink() or (not path.is_dir() and not path.is_file()):
                raise ec.ClosureError(f"hydrated {slug.upper()} ancestor contains a nonregular path")
            if path.is_dir():
                continue
            relative = path.relative_to(source)
            # The captured probe source omits its post-capture evidence
            # directory. Keep the independently archived repository snapshot's
            # corresponding evidence records, then add all source-archive files.
            target = destination / relative
            if target.exists() or target.is_symlink():
                if target.is_symlink() or not target.is_file():
                    raise ec.ClosureError(f"scratch {slug.upper()} path is redirected: {target}")
                if _sha(path) != _sha(target):
                    if relative.as_posix() in {"README.md", "TCB.md"}:
                        continue
                    raise ec.ClosureError(f"ancestor snapshot conflicts with repository input: {slug}/{relative}")
            else:
                _copy_regular(path, target)
    # AU's completed proof-origin archive is a separate typed input mount in
    # the published AU closure, not a member of the canonical repository tree.
    # Recreate the two exact files at the sibling paths consumed by AU's
    # offline proof-reuse checker.
    au_root = probes_root / ANCESTOR_PROBES["au"]
    au_evidence = au_root / "evidence"
    for name in ("au-full-diagnostic-v1.tar.gz", "au-full-diagnostic-v1.json"):
        source = hydrated / "ancestors/au/inputs/au-proof-origin" / name
        _copy_regular(source, au_evidence / name)
    return av_probe


def replay_archived_audits(av_probe: pathlib.Path, work_root: pathlib.Path) -> dict[str, Any]:
    env = dict(os.environ)
    for name in ("BYTES_DROP_FEATURE", "BYTES_SCOPE_SOURCE_CONTROL", "BYTES_CARGO_FEATURES",
                 "BYTES_SCOPE_DIAGNOSTIC", "BYTES_DROP_CHECKER_SKIP", "BYTES_TRANSLATE_ONLY"):
        env.pop(name, None)
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    evidence_log = work_root / "av-offline-evidence-audit.json"
    checker_report = work_root / "av-offline-checker-report.json"
    evidence_run = subprocess.run([sys.executable, "evidence.py", "audit", "av-positive-reuse-canonical-v1"],
        cwd=av_probe, env=env, text=True, capture_output=True, check=False)
    evidence_log.write_text(evidence_run.stdout + evidence_run.stderr)
    if evidence_run.returncode != 0:
        raise ec.ClosureError(f"archived AV evidence audit rejected ({evidence_run.returncode}); see {evidence_log}")
    checker_run = subprocess.run([sys.executable, "check_correspondence.py", "--shadow", "generated/active.rs",
        "--mapping", "generated/mapping.json", "--audit-compiled-capture-only", "--output", str(checker_report)],
        cwd=av_probe, env=env, text=True, capture_output=True, check=False)
    if checker_run.returncode != 0:
        (work_root / "av-offline-checker.log").write_text(checker_run.stdout + checker_run.stderr)
        raise ec.ClosureError(f"archived AV correspondence checker rejected ({checker_run.returncode}); see {checker_report}")
    report = _read_json(checker_report, "offline AV checker report")
    if report.get("status") != "pass" or report.get("full_original_admitted") is not False:
        raise ec.ClosureError("offline AV checker did not reproduce the bounded admitted-reuse result")
    target = report.get("AV_proof_inventory", {})
    if target.get("target_count") != ORIGIN_TARGETS or target.get("prover_leaves") != PROVER_LEAVES or \
            target.get("null") != 0 or target.get("structural") != 0:
        raise ec.ClosureError("offline AV checker proof inventory differs from its published result")
    return {"status": "pass", "evidence_audit_returncode": evidence_run.returncode,
            "correspondence_returncode": checker_run.returncode,
            "evidence_audit_log": str(evidence_log), "correspondence_report": str(checker_report),
            "proof_inventory": target, "full_original_admitted": False,
            "solver_or_cargo_invoked": False}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--expected-root-sha256", required=True)
    parser.add_argument("--repository-root", type=pathlib.Path, required=True)
    parser.add_argument("--cas-root", type=pathlib.Path, required=True)
    parser.add_argument("--cache-root", type=pathlib.Path, required=True)
    parser.add_argument("--work-root", type=pathlib.Path, required=True)
    parser.add_argument("--hydrate-to", type=pathlib.Path, required=True)
    parser.add_argument("--checkout-to", type=pathlib.Path, required=True)
    args = parser.parse_args()
    try:
        require_fresh_hydration_destination(args.hydrate_to)
        manifest, closure_report, objects = ec.audit_closure(args.manifest, args.expected_root_sha256,
            repository_root=args.repository_root, cas_root=args.cas_root, cache_root=args.cache_root)
        hydrated = args.hydrate_to
        closure_report = ec.hydrate(args.manifest, args.expected_root_sha256,
            repository_root=args.repository_root, cas_root=args.cas_root, cache_root=args.cache_root,
            work_root=args.work_root, destination=hydrated)
        av_inventory = verify_av_inventory(manifest, objects, hydrated)
        av_probe = restore_av_checkout(hydrated, args.checkout_to, args.cache_root)
        replay = replay_archived_audits(av_probe, args.work_root.resolve(strict=True))
        result = {"status": "pass", "root_manifest_sha256": args.expected_root_sha256,
                  "closure": closure_report, "av_inventory": av_inventory, "offline_replay": replay}
        print(json.dumps(result, sort_keys=True, indent=2))
        return 0
    except ec.ClosureError as exc:
        print(json.dumps({"status": "reject", "error": str(exc)}, sort_keys=True, indent=2), file=sys.stderr)
        return 2
    except Exception as exc:
        print(json.dumps({"status": "error", "error": f"{type(exc).__name__}: {exc}"},
                         sort_keys=True, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
