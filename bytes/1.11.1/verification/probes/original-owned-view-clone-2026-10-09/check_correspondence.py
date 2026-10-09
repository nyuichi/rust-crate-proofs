#!/usr/bin/env python3
"""AT correspondence gate: owned-View Clone over the published AS source.

The checker replays AS from its immutable canonical archive, verifies that AT
retains that complete source as a byte-exact prefix, then checks the one added
owned-view Clone extension/client against source, generated mapping, selected
native MIR, and the four actual Cargo build artifacts. This gate does not admit
the complete original crate, arbitrary Clone/From values, unwind behavior, or
concurrent/escaping owners.
"""
from __future__ import annotations

import argparse
import ast
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import sys
import tarfile
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
PROBES = ROOT.parent
CRATE_ROOT = ROOT.parents[2]
AS_ROOT = PROBES / "original-nonnull-view-boundaries-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
POSITIVE = ROOT / "generated/positive.rs"
MAPPING = ROOT / "generated/mapping.json"
CAPTURE_DIR = ROOT / "generated/compiled-inputs"
TARGET_DIR = pathlib.Path("/workspace/bytes-proof-tools/targets/bytes").resolve()
PACKAGE = "bytes-original-owned-view-clone"
RERUN_PATHS = [
    "../../../src/bytes.rs",
    "../../../src/bytes/bytes_record.rs",
    "../../../src/bytes/vtable_record.rs",
    "../../../src/bytes_mut.rs",
    "extract_public.py",
]
ARTIFACT_FIELDS = {
    "public_records.rs": "captured_input_sha256",
    "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
    "cargo-build-output.txt": "captured_build_output_sha256",
    "cargo-root-output.txt": "captured_root_output_sha256",
}

# Published AS canonical-v1 anchors. The docs were updated after publication;
# all executable/source/archive inputs remain checked against archived bytes.
AS_COMMIT = "2e3525dfa642c809c4d8612efd8a2cecb0067500"
AS_CHECKER_SHA256 = "e5ba544d0a3cdc4bbb0ef2e27a033ad4a0507a8bcedb1382c6910ad22417ce68"
AS_AUDIT_JSON_SHA256 = "d462d292dc0921975e138f7225637e7e01a96aacd011267afe68848b0b186090"
AS_AUDIT_MD_SHA256 = "f44ddf674707b2bc78b9c3aaf3c39bf4a688eacf951a289e3186158028d85d0f"
AS_ARCHIVE_SHA256 = "969a1a0406e10677c1849200284881e043aae9c9bdf29c9b15609783c91a9f74"
AS_RECEIPT_SHA256 = "6dcf08127c14b78021c1452995f04c2e0099c41144f59b0cc03e838fbe6dd2d0"
AS_AUDIT_REPLAY_SHA256 = "9e41fbe642baba65fcbb0d6936aadd5a03bf9a11b427f89cb142ffcd34aee33a"
AS_ARCHIVE_MEMBERS = 1421
AS_TARGETS = 150
AS_PROVER_LEAVES = 1328
AS_POSITIVE_SHA256 = "9fbe1698c4a1a338c7bc30eeb60af8b69bc4077304b63acab9b3be34a7650744"
AS_SOURCE_COMMIT = "8449794d8115286c26fda1ff115c7c222e62e660"

AS_MAIN_CONTROL_SHA256 = "3ca3db45d9c561468cea2bd5c65d64371b02f3adb664cc4ee0c4a2771894ca3e"
AS_MAIN_FIXTURE_SHA256 = "793bfcffa3c51f1e640127f0f772b2e62f40dfad959be0c498ff2be83cac4405"
AS_MAIN_CONTROL_RECEIPT_SHA256 = "80a5a039aea06663d3357382189cc2c3a5bb3bd7f82eae5a5f749957f4db379b"
AS_NATIVE_CHECKER_SHA256 = "a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f"
AS_NATIVE_CONTROLS_SHA256 = "6253f4e631fea8f0ab8cec3c369311dee3a824736f82cf8817a0f2a6de5528d3"
AS_NATIVE_FIXTURE_SHA256 = "e79368d7c01de089c510ec0c3bc1afb2dec846335fd9bb87fe9d050b52ccd906"
AS_NATIVE_RECEIPT_SHA256 = "ed703c3bc26743f73d8aef9f5b9fc42335a25ade73d0b8a2fb74c2bf1023ed37"

# AT source pins are provisional until the first positive semantic result.
# They are deliberately checked in addition to exact source/body/mapping
# reconstruction, never as the sole correspondence evidence.
AT_GENERATOR_SHA256 = "25d48c518d15b34e04005044e25eb1164d91f64b43ded7bc737fa43d5d71bf11"
AT_EXTENSION_SHA256 = "43be8bf642cc463c797f2ce9466246f32b297701dcb336db00761691fa463ab2"
AT_CLIENT_SHA256 = "5df8e0a9d19fb48883b44a98be7d568b60feff828ee6905c411e856da2c6e7a2"
AT_ACTIVE_SHA256 = "b5790f53182c689186442dc54428693a0c5144bb46febdb1a0824e8ccffee945"
AT_LAUNCHER_SHA256 = "8babffdcda4d4d912574d3b6aeb11fc5f3cfd416faaee9c3352cc4d851b9ce1d"
AT_NATIVE_CHECKER_SHA256 = "57a47d7304777417b2514a0b8281983513c4a1d07612aa63a7a10ae80709bf92"
AT_NATIVE_CONTROLS_SHA256 = "43c253328eaf8f79e7e1e57b24d8b996334a5575e080301d676be5ae0045e068"
AT_NATIVE_FIXTURE_SHA256 = "f6dbec119aee5491f14ab73def971e4eaee5197b13515fb36229b158a5207099"
AT_NATIVE_RECEIPT_SHA256 = "9591a9416a2a19ec23d48235c8f9a7f4acffa03086777872b351f3166a6d636a"
AT_NATIVE_CONTROL_COUNT = 45

AT_FUNCTION_TOKEN_SHA256 = {
    "singleton_replacement": "cc461f74500649219d714c29934ac1f719c139421555df57e9a20cf45b4b29cc",
    "same_allocation": "6ed0cea528ac406554b8640130250f835a3bba09132d47e6e8214d14c0c7304c",
    "has_allocation": "05745622e30d77deb96427f8d1cec505d9d95759f2383f924031ed40e057c2b7",
    "shares_view_allocation": "6fb70d387494a990da66964434175cdcdb2903deeb89bd437a61a8e91cd08dca",
    "shallow_clone_owned_view_checked": "b3725562521d310b2beb9ac3473b19c6cce6dfe42945a5c6adef54fcab0d8b12",
    "shared_owned_view_clone_checked": "da510f91d6742e16a2ea00794371576d395dcfc6ec7488a1772b9c1cfc4d684a",
    "owned_view_clone_registration": "ef38d08fe0f65e944079dc8d202c602a0681ebb0623086f8ff802dad0e4bedb4",
    "clone_owned_api": "394d4beb8b4ed9e2707af1708d2a7f3367d07397f5b64ca8b563fc21f40741f5",
}
AT_CLIENT_FUNCTION_TOKEN_SHA256 = "63d5358c0c153c20ba1b7df4dab5753d368470d0301a11b263f81a6f5240950b"
AT_FUNCTION_NAMES = list(AT_FUNCTION_TOKEN_SHA256)
AT_EXTENSION_REL = "src/owned_clone_extension.rs"
AT_HELPERS = [
    "shallow_clone_owned_view_checked",
    "shared_owned_view_clone_checked",
    "owned_view_clone_registration",
    "clone_owned_api",
]
AT_TERMINAL_HELPERS = [
    "bytes_root_detaching_terminal_drop",
    "bytes_view_terminal_drop",
    "bytes_cursor_terminal_drop",
]
AT_EXPECTED_EXCLUDED = [
    "Root and general Static Clone",
    "arbitrary escaping/concurrent ownership",
    "unwind",
    "whole crate",
]
AS_BUILD_INPUT_HASHES = {
    "Cargo.toml": "da8c39007a61878597d9fdb1bd3b4a5eecd2ba1136af03b1a33cab81e91b4414",
    "Cargo.lock": "dd77bb3b5a3b9f712ac20c7b2ae2d664fc95ca3bd6b20f67b95f9174c4236d32",
    "build.rs": "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
    "extract_public.py": "748810dd6d4961c1e729864a83daa3aff771c72e7ef678e9bb01f2cfef44e99b",
}
AT_LOCK_SHA256 = "d3308de82686dc17798b7b97128bfc96443ea4308a2127caba6267faacc0b5d3"
AT_MANIFEST_FEATURES = {
    "negative_missing_acquire": [],
    "negative_missing_payload_free": [],
    "negative_missing_control_free": [],
}
AS_ARCHIVE_DOCS_EXCLUDED_FROM_CURRENT_EQUALITY = {"README.md", "TCB.md"}


class CheckError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _read_regular(path: pathlib.Path, message: str) -> bytes:
    require(path.is_file() and not path.is_symlink(), message)
    return path.read_bytes()


def _regular_tree(directory: pathlib.Path, label: str) -> dict[str, bytes]:
    require(directory.is_dir() and not directory.is_symlink(), f"{label} source root is missing or redirected")
    result: dict[str, bytes] = {}
    for path in directory.rglob("*"):
        require(not path.is_symlink(), f"{label} contains a symlink: {path}")
        if path.is_dir():
            continue
        require(path.is_file(), f"{label} contains a nonregular entry: {path}")
        result[path.relative_to(directory).as_posix()] = path.read_bytes()
    return result


def _proof_stats(proofs: list[dict[str, Any]]) -> dict[str, int]:
    result = {"files": len(proofs), "prover": 0, "null": 0, "structural": 0}

    def visit(tree: Any) -> None:
        if tree is None:
            result["null"] += 1
        elif isinstance(tree, dict) and "children" in tree:
            children = tree["children"]
            if not children:
                result["structural"] += 1
            for child in children:
                visit(child)
        elif isinstance(tree, dict) and "prover" in tree:
            result["prover"] += 1
        else:
            raise CheckError(f"unknown archived proof-tree node: {tree!r}")

    for proof in proofs:
        coma_nodes = proof.get("proofs", {}).get("Coma")
        require(isinstance(coma_nodes, dict), "AS archived proof has no Coma proof tree")
        for node in coma_nodes.values():
            visit(node)
    return result


def assert_as_canonical_archive() -> tuple[dict[str, Any], dict[str, bytes]]:
    evidence = AS_ROOT / "evidence"
    audit_path = evidence / "AS_CANONICAL_AUDIT.json"
    audit_md = evidence / "AS_CANONICAL_AUDIT.md"
    archive_path = evidence / "as-positive-canonical-v1.tar.gz"
    receipt_path = evidence / "as-positive-canonical-v1.json"
    replay_path = evidence / "as-positive-canonical-v1-audit.json"
    pins = (
        (audit_path, AS_AUDIT_JSON_SHA256, "AS canonical audit JSON"),
        (audit_md, AS_AUDIT_MD_SHA256, "AS canonical audit report"),
        (archive_path, AS_ARCHIVE_SHA256, "AS canonical archive"),
        (receipt_path, AS_RECEIPT_SHA256, "AS canonical receipt"),
        (replay_path, AS_AUDIT_REPLAY_SHA256, "AS canonical independent replay"),
        (AS_ROOT / "check_correspondence.py", AS_CHECKER_SHA256, "published AS checker"),
    )
    for path, expected, label in pins:
        require(sha(_read_regular(path, f"{label} missing or redirected")) == expected,
                f"{label} differs from published AS")
    audit = json.loads(audit_path.read_text())
    receipt_bytes = receipt_path.read_bytes()
    receipt = json.loads(receipt_bytes)
    require(audit.get("schema") == "as-canonical-independent-audit-v1" and
            audit.get("result") == "audited_pass_selected_gate" and
            audit.get("scope", {}).get("full_original_admitted") is False and
            audit.get("archive", {}).get("sha256") == AS_ARCHIVE_SHA256 and
            audit.get("archive", {}).get("members") == AS_ARCHIVE_MEMBERS and
            audit.get("archive", {}).get("proof") == {
                "files": AS_TARGETS, "prover_leaves": AS_PROVER_LEAVES,
                "null_leaves": 0, "structural_leaves": 0,
                "every_coma_and_proof_hash_verified": True,
                "target_list_matches_all_archived_coma": True,
                "excluded": {}, "features": [],
                "correspondence_exit_status": 0, "diagnostic": False},
            "AS canonical audit is not the exact no-exclusion selected proof result")
    policy = receipt.get("target_policy", {})
    targets = receipt.get("targets")
    member_rows = receipt.get("members")
    require(receipt.get("archive") == archive_path.name and
            receipt.get("archive_sha256") == AS_ARCHIVE_SHA256 and
            receipt.get("status") == "proved" and
            receipt.get("statistics") == {
                "files": AS_TARGETS, "prover": AS_PROVER_LEAVES,
                "null": 0, "structural": 0} and
            isinstance(targets, list) and len(targets) == AS_TARGETS and
            isinstance(member_rows, list) and len(member_rows) == AS_ARCHIVE_MEMBERS and
            policy.get("excluded") == {} and policy.get("features") == [] and
            policy.get("diagnostic") is False and
            policy.get("correspondence_exit_status") == 0,
            "AS receipt does not bind every target and every successful proof")
    member_hashes = {row.get("path"): row.get("sha256") for row in member_rows if isinstance(row, dict)}
    require(len(member_hashes) == AS_ARCHIVE_MEMBERS and None not in member_hashes and
            all(isinstance(name, str) and isinstance(digest, str) for name, digest in member_hashes.items()),
            "AS receipt member table is duplicated or malformed")
    archive_files: dict[str, bytes] = {}
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        require(len(members) == AS_ARCHIVE_MEMBERS and len(set(names)) == AS_ARCHIVE_MEMBERS and
                set(names) == set(member_hashes) and all(member.isfile() for member in members),
                "AS archive is not a unique regular-file archive matching its receipt")
        for member in members:
            pure = pathlib.PurePosixPath(member.name)
            require(not pure.is_absolute() and ".." not in pure.parts,
                    f"unsafe member path in AS archive: {member.name}")
            content_file = archive.extractfile(member)
            require(content_file is not None, f"cannot read AS archive member: {member.name}")
            content = content_file.read()
            require(sha(content) == member_hashes[member.name],
                    f"AS archive member hash mismatch: {member.name}")
            archive_files[member.name] = content
    require(sha(archive_path.read_bytes()) == AS_ARCHIVE_SHA256,
            "AS canonical archive hash changed while reading")

    target_paths = set()
    proof_paths = set()
    proofs = []
    for row in targets:
        coma_path, proof_path = row.get("coma"), row.get("proof")
        require(isinstance(coma_path, str) and isinstance(proof_path, str) and
                coma_path in archive_files and proof_path in archive_files,
                "AS proof target refers to an absent archive member")
        require(row.get("coma_sha256") == sha(archive_files[coma_path]) and
                row.get("proof_sha256") == sha(archive_files[proof_path]),
                f"AS target source/proof hashes do not match archive: {coma_path}")
        target_paths.add(coma_path)
        proof_paths.add(proof_path)
        proofs.append(json.loads(archive_files[proof_path]))
    archived_coma = {name for name in archive_files if name.startswith("probe/verif/") and name.endswith(".coma")}
    archived_proofs = {name for name in archive_files if name.startswith("probe/verif/") and name.endswith("/proof.json")}
    require(target_paths == archived_coma and proof_paths == archived_proofs and
            sorted(row.get("coma", "").removeprefix("probe/") for row in targets) ==
                sorted(policy.get("included", [])) and
            _proof_stats(proofs) == {"files": AS_TARGETS, "prover": AS_PROVER_LEAVES, "null": 0, "structural": 0},
            "AS archive has an omitted, extra, excluded, null, or structural proof target")

    # Verify the current AS worktree against the immutable archive. README and
    # TCB are post-publication documentation; the audited source/checker inputs
    # and generated proof inputs remain exact.
    compared = 0
    for member_name, content in archive_files.items():
        if not member_name.startswith("probe/"):
            continue
        relative = pathlib.PurePosixPath(member_name[len("probe/"):])
        if relative.parts and relative.parts[0] == "evidence":
            continue
        if relative.as_posix() in AS_ARCHIVE_DOCS_EXCLUDED_FROM_CURRENT_EQUALITY:
            continue
        selected = AS_ROOT.joinpath(*relative.parts)
        require(selected.is_file() and not selected.is_symlink() and selected.read_bytes() == content,
                f"published AS input differs from its canonical archived byte: {relative}")
        compared += 1
    return ({"status": "pass", "archive_sha256": AS_ARCHIVE_SHA256,
        "member_count": AS_ARCHIVE_MEMBERS, "unique_regular_members": True,
        "all_member_hashes_verified": True, "proof_targets": AS_TARGETS,
        "prover_leaves": AS_PROVER_LEAVES, "null": 0, "structural": 0,
        "excluded": {}, "features": [], "diagnostic": False,
        "current_AS_probe_files_compared": compared,
        "post_publication_docs_excluded": sorted(AS_ARCHIVE_DOCS_EXCLUDED_FROM_CURRENT_EQUALITY)},
        archive_files)


def _import_published_as() -> Any:
    path = AS_ROOT / "check_correspondence.py"
    require(sha(_read_regular(path, "published AS checker is absent")) == AS_CHECKER_SHA256,
            "published AS checker changed before import")
    spec = importlib.util.spec_from_file_location("at_pinned_as_correspondence", path)
    require(spec is not None and spec.loader is not None, "cannot load pinned published AS checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def assert_published_as_ancestry(as_archive: dict[str, bytes] | None = None) -> dict[str, Any]:
    if as_archive is None:
        _archive_report, as_archive = assert_as_canonical_archive()
    # The checker and its imports are hash-pinned before code is loaded. Its
    # archive-only path deliberately avoids AS's live Cargo target directory.
    as_checker = _import_published_as()
    ar = as_checker._import_ar_checker()
    ar_report = as_checker._replay_ar_without_live_target(ar)
    aq, _lineage = ar.preflight_published_aq()
    transformation = as_checker.assert_as_transformations(ar)
    manifest = as_checker.assert_probe_manifest(environment={})
    as_mapping = json.loads(_read_regular(AS_ROOT / "generated/mapping.json", "AS mapping missing"))
    native = as_checker.audit_native(as_mapping)
    controls = as_checker.assert_as_native_controls()
    compiled = as_checker.assert_compiled_capture_only(ar)
    require(compiled.get("status") == "pass" and
            compiled.get("captured_artifact_count") == 4 and
            compiled.get("archive_capture_replayed_without_live_Cargo_target") is True,
            "AS ancestry was not replayed from its captured four artifacts")
    result = {"published_commit": AS_COMMIT,
        "canonical_archive": {"sha256": AS_ARCHIVE_SHA256, "members": AS_ARCHIVE_MEMBERS,
            "targets": AS_TARGETS, "prover_leaves": AS_PROVER_LEAVES},
        "published_AS_checker_sha256": AS_CHECKER_SHA256,
        "AS_transformations": transformation,
        "AR_lineage": ar_report,
        "AS_native_correspondence": native,
        "AS_native_controls": controls,
        "AS_captured_Cargo": compiled,
        "AS_manifest_and_build_inputs": manifest,
        "live_AS_Cargo_target_read": False}
    result["_as_checker"] = as_checker
    result["_ar_checker"] = ar
    result["_aq_ar"] = aq
    result["_ai"] = aq.AP.AO.AN.AI
    return result


def _load_generator_client(generator_source: str) -> str:
    try:
        tree = ast.parse(generator_source)
        values = []
        for item in tree.body:
            if isinstance(item, ast.Assign) and any(
                    isinstance(target, ast.Name) and target.id == "CLIENT" for target in item.targets):
                values.append(ast.literal_eval(item.value))
        require(len(values) == 1 and isinstance(values[0], str),
                "AT generator must define exactly one literal closed client source")
        return values[0]
    except (SyntaxError, ValueError) as exc:
        raise CheckError(f"AT generator client literal is not statically reconstructible: {exc}") from exc


def _extract_function(aq: Any, ar: Any, source: str, name: str) -> str:
    return ar._extract_named_function(aq, source, name)


def _function_token_hash(aq: Any, ar: Any, source: str, name: str) -> str:
    ai = aq.AP.AO.AN.AI
    tokens = ai.rust_tokens(_extract_function(aq, ar, source, name))
    return sha(" ".join(tokens).encode())


def _function_names(ai: Any, source: str) -> list[str]:
    mask = ai.mask_noncode(source)
    return re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", mask)


def _function_attributes(ai: Any, source: str, name: str) -> list[str]:
    mask = ai.mask_noncode(source)
    matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\b", mask))
    require(len(matches) == 1, f"AT attribute scan expected one function named {name}")
    cursor = matches[0].start()
    found: list[str] = []
    while cursor:
        while cursor and mask[cursor - 1].isspace():
            cursor -= 1
        if not cursor or mask[cursor - 1] != "]":
            break
        depth = 1
        pos = cursor - 2
        while pos >= 0 and depth:
            if mask[pos] == "]":
                depth += 1
            elif mask[pos] == "[":
                depth -= 1
            pos -= 1
        if depth or pos < 0 or mask[pos] != "#":
            break
        start = pos
        found.insert(0, source[start:cursor])
        cursor = start
    return found


def assert_at_source_closure(as_archive: dict[str, bytes],
                             source_tree: dict[str, bytes]) -> dict[str, Any]:
    as_sources = {
        name[len("probe/src/"):]: content for name, content in as_archive.items()
        if name.startswith("probe/src/") and name.endswith(".rs")
    }
    require(as_sources, "AS archive contains no complete Rust source tree")
    expected_names = set(as_sources) | {"owned_clone_extension.rs"}
    require(set(source_tree) == expected_names,
            f"AT Rust source inventory is not complete-AS plus one extension: missing={sorted(expected_names-set(source_tree))}, extra={sorted(set(source_tree)-expected_names)}")
    for name, content in as_sources.items():
        require(source_tree[name] == content,
                f"AT changed inherited AS Rust source outside the owned-view extension: {name}")
    require(sha(source_tree["promotion.rs"]) == AS_POSITIVE_SHA256 and
            source_tree["promotion.rs"] == as_archive["probe/generated/positive.rs"],
            "AT prefix is not the complete byte-exact AS positive source")
    require(source_tree["lib.rs"] == as_sources["lib.rs"],
            "AT changed the inherited AS module route")
    return {"complete_AS_Rust_source_byte_exact": True,
        "AS_module_count": len(as_sources),
        "only_new_Rust_module": "src/owned_clone_extension.rs",
        "promotion_prefix_sha256": AS_POSITIVE_SHA256,
        "lib_route_byte_exact": True}


def assert_probe_manifest(manifest_bytes: bytes, lock_bytes: bytes,
                          build_bytes: bytes, extractor_bytes: bytes,
                          generator_bytes: bytes, launcher_bytes: bytes,
                          environment: dict[str, str]) -> dict[str, Any]:
    try:
        manifest = tomllib.loads(manifest_bytes.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AT Cargo manifest could not be parsed: {exc}") from exc
    expected = {
        "package": {"name": PACKAGE, "version": "0.1.0", "edition": "2021", "publish": False},
        "dependencies": {"creusot-std": "=0.13.0"},
        "workspace": {},
        "features": AT_MANIFEST_FEATURES,
    }
    require(manifest == expected,
            "AT Cargo package, dependency, feature surface, or workspace route changed")
    source_inputs = {
        "build.rs": build_bytes,
        "extract_public.py": extractor_bytes,
    }
    require(all(sha(source_inputs[name]) == AS_BUILD_INPUT_HASHES[name] for name in source_inputs),
            "AT changed its inherited production extraction/build script")
    as_lock = _read_regular(AS_ROOT / "Cargo.lock", "AS lockfile missing")
    require(sha(lock_bytes) == AT_LOCK_SHA256 and
            lock_bytes.count(b'name = "bytes-original-owned-view-clone"') == 1 and
            lock_bytes.replace(b'name = "bytes-original-owned-view-clone"',
                b'name = "bytes-original-nonnull-view-boundaries"', 1) == as_lock,
            "AT lockfile differs from the pinned package identity or published AS dependency closure")
    require(sha(extractor_bytes) == AS_BUILD_INPUT_HASHES["extract_public.py"],
            "AT extractor differs from the reviewed actual-production extractor")
    require(sha(generator_bytes) == AT_GENERATOR_SHA256 and
            sha(launcher_bytes) == AT_LAUNCHER_SHA256,
            "AT generator or launcher differs from the selected source gate")
    feature_or_control_values = [
        environment.get("BYTES_DROP_FEATURE", ""),
        environment.get("BYTES_SCOPE_SOURCE_CONTROL", ""),
        environment.get("BYTES_CARGO_FEATURES", ""),
    ]
    require(not any(feature_or_control_values) and
            environment.get("BYTES_SCOPE_DIAGNOSTIC", "0") != "1" and
            environment.get("BYTES_DROP_CHECKER_SKIP", "0") != "1" and
            environment.get("BYTES_TRANSLATE_ONLY", "0") != "1",
            "AT cannot admit a feature, source-control, diagnostic, checker-skip, or translation-only run")
    return {"manifest_exact": True, "package": PACKAGE,
        "dependency_lock_matches_AS_closure_with_AT_package_identity": True,
        "feature_surface": sorted(AT_MANIFEST_FEATURES),
        "generator_and_launcher_pinned": True,
        "feature_or_diagnostic_environment": False}


def _literal_client_from_generator() -> str:
    generator_source = _read_regular(ROOT / "elaborate.py", "AT generator missing").decode("utf-8")
    require(sha(generator_source.encode()) == AT_GENERATOR_SHA256,
            "AT generator changed before client reconstruction")
    return _load_generator_client(generator_source)


def assert_at_composition(source_tree: dict[str, bytes], generator_bytes: bytes,
                          mapping: dict[str, Any], active: bytes, positive: bytes,
                          extension_generated: bytes, terminal_generated: bytes,
                          client: bytes, aq: Any, ar: Any) -> dict[str, Any]:
    ai = aq.AP.AO.AN.AI
    prefix = source_tree["promotion.rs"]
    extension = source_tree["owned_clone_extension.rs"]
    client_source = _literal_client_from_generator()
    require(generator_bytes == _read_regular(ROOT / "elaborate.py", "AT generator missing"),
            "AT generator input changed during source reconstruction")
    require(sha(extension) == AT_EXTENSION_SHA256 and
            sha(client) == AT_CLIENT_SHA256 and sha(active) == AT_ACTIVE_SHA256,
            "AT selected extension/client/active source differs from its reviewed pre-proof source snapshot")
    require(client == client_source.encode(),
            "AT elaborated client is not the exact full client source literal from the pinned generator")
    expected_active = prefix + b"\n" + extension + client
    require(active == expected_active and positive == expected_active and
            extension_generated == extension and terminal_generated == extension,
            "AT active/positive/extension/client composition does not match exact source inputs")
    require(mapping.get("feature") == "" and mapping.get("status") == "generated_unchecked" and
            mapping.get("stage") == "after-ElaborateDrops" and mapping.get("full_original_admitted") is False and
            mapping.get("ancestor_commit") == AS_COMMIT and
            mapping.get("ancestor_source") == "../original-nonnull-view-boundaries-2026-10-09/generated/positive.rs" and
            mapping.get("ancestor_sha256") == AS_POSITIVE_SHA256 and
            mapping.get("base_source") == "src/promotion.rs" and
            mapping.get("base_source_sha256") == sha(prefix) and
            mapping.get("selected_prefix_sha256") == AS_POSITIVE_SHA256 and
            mapping.get("extension_source") == AT_EXTENSION_REL and
            mapping.get("extension_sha256") == sha(extension) and
            mapping.get("client_sha256") == sha(client) and
            mapping.get("active") == "generated/active.rs" and
            mapping.get("active_sha256") == sha(active) and
            mapping.get("support_inventory") == {
                "src/" + name: sha(data) for name, data in sorted(source_tree.items())},
            "AT mapping does not bind the selected AS-prefix/extension/client composition")
    expected_map_keys = {
        "feature", "status", "stage", "full_original_admitted", "ancestor_commit",
        "ancestor_source", "ancestor_sha256", "base_source", "base_source_sha256",
        "selected_prefix_sha256", "extension_source", "extension_sha256", "client_sha256",
        "active", "active_sha256", "support_inventory", "helpers", "terminal_helpers",
        "ownership_frame", "loop_inventory", "assignment_effect", "return_evaluation",
        "excluded", "native_source", "native_source_sha256", "native_client_mir",
        "native_mir_ready", "debug_places", "normal_edges", "mir_blocks", "mir",
        "native_assignment", "native_saved_return", "logic_helpers",
    }
    require(set(mapping) == expected_map_keys,
            f"AT mapping contains unknown or omitted claims: extra={sorted(set(mapping)-expected_map_keys)}, missing={sorted(expected_map_keys-set(mapping))}")
    require(mapping.get("helpers") == AT_HELPERS and
            mapping.get("logic_helpers") == AT_FUNCTION_NAMES[:4] and
            mapping.get("terminal_helpers") == AT_TERMINAL_HELPERS and
            mapping.get("ownership_frame") ==
                "same_allocation/has_allocation/shares_view_allocation: complete allocation identity; current ticket/fraction dynamic" and
            mapping.get("loop_inventory") ==
                "cursor map exactly singleton current actual ticket id/fraction; no iteration quota" and
            mapping.get("assignment_effect") == {
                "order": ["evaluate next clone", "drop old value", "install next", "increment i"],
                "native_address_nonobserving": True} and
            mapping.get("return_evaluation") == {
                "shadow": "let saved_return=observed", "before_value_drop": True} and
            mapping.get("excluded") == AT_EXPECTED_EXCLUDED,
            "AT mapping helper, loop/effect, exclusion or return-order claims changed")
    extension_source = extension.decode("utf-8")
    names = _function_names(ai, extension_source)
    require(names == AT_FUNCTION_NAMES,
            "AT extension function surface changed or has an untracked callback")
    for name, expected_hash in AT_FUNCTION_TOKEN_SHA256.items():
        require(_function_token_hash(aq, ar, extension_source, name) == expected_hash,
                f"AT source function token body changed: {name}")
    client_source_text = client.decode("utf-8")
    require(_function_names(ai, client_source_text) == ["owned_view_clone_scope"] and
            _function_token_hash(aq, ar, client_source_text, "owned_view_clone_scope") ==
                AT_CLIENT_FUNCTION_TOKEN_SHA256,
            "AT closed native-compatible client changed its function/body surface")
    mask = ai.mask_noncode(extension_source)
    trusted_items = re.findall(r"#\s*\[\s*trusted\s*\]", mask)
    require(len(trusted_items) == 1 and
            "owned_view_clone_registration" in extension_source and
            len(re.findall(r"#\s*\[\s*check\s*\(\s*ghost\s*\)\s*\]", mask)) == 1 and
            not re.search(r"#\s*\[\s*(?:assume|axiom|extern_spec|cfg_attr)\b", mask),
            "AT extension contains an unreviewed trusted/assumed/spec/macro boundary")
    registration_attrs = [
        "#[trusted]",
        "#[check(ghost)]",
        "#[ensures(erased_call::registered3(shared_table().clone,result.inner_logic()))]",
        """#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>>
    result.inner_logic().precondition((data,ptr,len,input))==shared_owned_view_clone_checked.precondition((data,ptr,len,input)))]""",
        """#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>,output:Bytes>
    result.inner_logic().postcondition((data,ptr,len,input),output)==shared_owned_view_clone_checked.postcondition((data,ptr,len,input),output))]""",
    ]
    require([ai.rust_tokens(attr) for attr in _function_attributes(
                ai, extension_source, "owned_view_clone_registration")] ==
            [ai.rust_tokens(attr) for attr in registration_attrs],
            "AT trusted clone registration attributes or specification surface changed")
    for name in ("same_allocation", "has_allocation", "shares_view_allocation"):
        require([ai.rust_tokens(x) for x in _function_attributes(ai, extension_source, name)] ==
                [ai.rust_tokens("#[logic(prophetic)]")],
                f"AT allocation identity predicate attributes changed: {name}")
    singleton_attrs = _function_attributes(ai, extension_source, "singleton_replacement")
    require([ai.rust_tokens(x) for x in singleton_attrs] == [
                ai.rust_tokens("#[logic]"),
                ai.rust_tokens("#[requires(old_id!=new_id)]"),
                ai.rust_tokens("#[ensures(FMap::singleton(old_id,Excl(old_fraction)).insert(new_id,Excl(new_fraction)).remove(old_id)==FMap::singleton(new_id,Excl(new_fraction)))]"),
            ],
            "AT pure singleton map-algebra helper attributes changed")
    body_semantics = {
        "native_callback_count": 1,
        "fresh_ticket_registration_before_pointer_binding": True,
        "same_shared_control_identity_without_byte_data_identity_claim": True,
        "result_accepts_advanced_cursor": True,
        "result_is_owned_at_zero_length": True,
        "same_allocation_frame_is_separate_from_view_extent": True,
        "one_new_trusted_ghost_registration": True,
    }
    # These exact token phrases supplement the body pins with the ownership
    # facts that distinguish a fresh shared Clone from cursor/physical cloning.
    required_terms = [
        "field_event::increment_owned::<Shared",
        "lifecycle::State::on_register",
        "pointer_event::load_relaxed",
        "pointer_event::new_pointer",
        "pointer_event::bind_read_only",
        "OriginalSharedProof::View",
        "core:SharedCore",
        "ticket:Ghost::new(ticket.into_inner().unwrap())",
        "result.view_owned()",
        "result.has_allocation",
        "erased_call::invoke3(native",
        "owned_view_clone_registration()",
        "result.api_view_valid() && result.view_owned()",
        "result.view_bound()==input.inner_logic().1",
        "result.view_public()==input.inner_logic().0.public()",
        "result.view_content()==input.inner_logic().0.content().subsequence",
        "result.has_allocation(*input.inner_logic().0)",
        "input.inner_logic().0.accepts(^input.inner_logic().2)",
        "source.api_view_valid() && source.view_owned() && source.view_accepts",
        "result.shares_view_allocation(*source)",
        "result.view_accepts(^scope) && source.view_accepts(^scope)",
        "result.view_bound()==source.view_bound() && result.ptr==source.ptr && result.len==source.len",
    ]
    tokens = ai.rust_tokens(extension_source)
    for term in required_terms:
        expected_tokens = ai.rust_tokens(term)
        require(any(tokens[index:index+len(expected_tokens)] == expected_tokens
                    for index in range(max(0, len(tokens)-len(expected_tokens)+1))),
                f"AT extension lost reviewed Clone/effect correspondence term: {term}")
    return {"composition_exact": True,
        "AS_positive_prefix_sha256": sha(prefix),
        "extension_sha256": sha(extension), "client_sha256": sha(client),
        "active_sha256": sha(active), "mapping_support_inventory_exact": True,
        "extension_function_surface": names,
        "function_token_bodies_checked": len(AT_FUNCTION_TOKEN_SHA256),
        "client_closed_full_source_checked": True,
        "native_operation_interpretation": body_semantics,
        "native_three_argument_clone_registration": "one ghost-only exact vtable clone callback"}


def assert_native(mapping: dict[str, Any]) -> dict[str, Any]:
    path = ROOT / "check_native.py"
    checker_bytes = _read_regular(path, "AT native checker missing")
    require(AT_NATIVE_CHECKER_SHA256 and sha(checker_bytes) == AT_NATIVE_CHECKER_SHA256,
            "AT native checker differs from its frozen selected MIR checker")
    spec = importlib.util.spec_from_file_location("at_pinned_native_correspondence", path)
    require(spec is not None and spec.loader is not None, "cannot load AT native checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    bundle = module.load_bundle()
    report = module.audit_bundle(bundle)
    require(report.get("status") == "pass",
            "AT native source/MIR correspondence did not pass")
    native = report.get("native_audit", {})
    client = native.get("client", report.get("client", {}))
    selected = bundle.get("capture", {}).get("selected", [])
    expected_mir = sorted(({"path": row.get("path"), "sha256": row.get("sha256")}
        for row in selected), key=lambda row: row["path"])
    client_rows = [row for row in selected if row.get("label") == "client"]
    require(mapping.get("native_mir_ready") is True and
            mapping.get("native_source_sha256") == sha(_read_regular(ROOT / "native.rs", "AT native client missing")) and
            mapping.get("normal_edges") == client.get("normal_edges") and
            mapping.get("debug_places") == client.get("debug_places") and
            mapping.get("mir_blocks") == client.get("mir_blocks") and
            mapping.get("native_assignment") == client.get("native_assignment") and
            mapping.get("native_saved_return") == client.get("native_saved_return"),
            "AT mapping does not equal independently parsed selected native client MIR")
    require(len(client_rows) == 1 and mapping.get("native_client_mir") == client_rows[0].get("path") and
            mapping.get("mir") == expected_mir,
            "AT mapping client/MIR path inventory differs from native capture")
    controls = assert_native_controls()
    return {"status": "pass", "selected_mir_count": native.get("selected_mir_count"),
        "production_mir_count": native.get("production_mir_count"),
        "client": client, "native_report": report,
        "mapping_edges_rederived_from_MIR": True, "native_controls": controls}


def assert_native_controls() -> dict[str, Any]:
    script = _read_regular(ROOT / "native-check-controls.py", "AT native controls script missing")
    fixture_raw = _read_regular(ROOT / "fixtures/native-check-controls.json",
        "AT native controls fixture missing")
    receipt_raw = _read_regular(ROOT / "generated/native-check-controls.json",
        "AT native controls receipt missing")
    require(sha(script) == AT_NATIVE_CONTROLS_SHA256 and
            sha(fixture_raw) == AT_NATIVE_FIXTURE_SHA256 and
            sha(receipt_raw) == AT_NATIVE_RECEIPT_SHA256,
            "AT native controls source, fixture or receipt differs from the frozen MIR mutation suite")
    fixture = json.loads(fixture_raw)
    receipt = json.loads(receipt_raw)
    cases = fixture.get("cases")
    rows = receipt.get("controls")
    ids = [case.get("id") for case in cases] if isinstance(cases, list) else []
    require(fixture.get("schema") == "at-native-owned-view-clone-controls-v1" and
            receipt.get("schema") == "at-native-owned-view-clone-controls-v1" and
            len(ids) == AT_NATIVE_CONTROL_COUNT and len(set(ids)) == AT_NATIVE_CONTROL_COUNT and
            isinstance(rows, list) and [row.get("id") for row in rows] == ids and
            receipt.get("checker_sha256") == AT_NATIVE_CHECKER_SHA256 and
            receipt.get("fixture_sha256") == AT_NATIVE_FIXTURE_SHA256 and
            receipt.get("control_count") == AT_NATIVE_CONTROL_COUNT and
            receipt.get("rejected_as_expected") == AT_NATIVE_CONTROL_COUNT and
            receipt.get("accepted") == [] and receipt.get("checker_errors") == [] and
            all(row.get("status") == "rejected_as_expected" for row in rows) and
            receipt.get("mutations_in_memory_only") is True and
            receipt.get("source_mutated_on_disk") is False and
            receipt.get("cargo_or_rust_build_invoked") is False and
            receipt.get("proof_tool_or_solver_invoked") is False,
            "AT native MIR controls receipt is incomplete or admits a mutation")
    return {"status": "pass", "controls": AT_NATIVE_CONTROL_COUNT,
        "rejected_as_expected": AT_NATIVE_CONTROL_COUNT, "accepted": 0,
        "checker_errors": 0, "checker_sha256": AT_NATIVE_CHECKER_SHA256,
        "fixture_sha256": AT_NATIVE_FIXTURE_SHA256}


def _expected_public_record(as_checker: Any, ar: Any) -> tuple[bytes, dict[str, str]]:
    return as_checker._expected_public_records(ar)


def assert_compiled_capture(receipt: dict[str, Any], artifacts: dict[str, bytes],
                            stored_receipt: bytes, expected_record: bytes,
                            source_map_bytes: bytes,
                            input_hashes: dict[str, str]) -> dict[str, Any]:
    require(set(artifacts) == set(ARTIFACT_FIELDS),
            "AT compiled capture must contain exactly four actual Cargo artifacts")
    require(json.loads(stored_receipt) == receipt and
            receipt.get("schema") == "at-compiled-public-records-v1" and
            receipt.get("status") == "pass" and receipt.get("probe_package") == PACKAGE,
            "AT compiled Cargo receipt has the wrong package or schema")
    require(receipt.get("build_script_sha256") == sha(_read_regular(ROOT / "build.rs", "AT build script missing")) and
            receipt.get("extractor_sha256") == sha(_read_regular(ROOT / "extract_public.py", "AT extractor missing")),
            "AT Cargo receipt does not identify its selected build/extractor inputs")
    for name, field in ARTIFACT_FIELDS.items():
        require(receipt.get(field) == sha(artifacts[name]),
                f"AT captured Cargo artifact changed: {name}")
    fingerprint = json.loads(artifacts["cargo-run-build-fingerprint.json"])
    source_map = json.loads(source_map_bytes)
    rerun = [entry["RerunIfChanged"] for entry in fingerprint.get("local", [])
        if isinstance(entry, dict) and isinstance(entry.get("RerunIfChanged"), dict)]
    require(len(rerun) == 1 and rerun[0].get("paths") == RERUN_PATHS,
            "AT Cargo fingerprint does not bind exact production/extractor rerun inputs")
    target = pathlib.Path(receipt.get("cargo_target_dir", ""))
    output_rel = pathlib.Path(str(rerun[0].get("output", "")))
    output = pathlib.Path(receipt.get("build_output_path", ""))
    fingerprint_path = pathlib.Path(receipt.get("cargo_build_fingerprint", ""))
    require(target.resolve() == TARGET_DIR and receipt.get("cargo_target_dir") == str(TARGET_DIR) and
            not output_rel.is_absolute() and ".." not in output_rel.parts and
            output_rel.parts[:2] == ("debug", "build") and
            (target / output_rel).resolve() == output.resolve() and
            receipt.get("build_output_path") == str(output.resolve()) and
            receipt.get("cargo_build_fingerprint") == str(fingerprint_path.resolve()) and
            output.name == "output" and fingerprint_path.name == "run-build-script-build-script-build.json" and
            output.parent.name == fingerprint_path.parent.name and
            output.parent.name.startswith(PACKAGE + "-") and
            fingerprint_path.parent.parent.resolve() == (TARGET_DIR / "debug/.fingerprint").resolve() and
            output.parent.parent.resolve() == (TARGET_DIR / "debug/build").resolve(),
            "AT Cargo package, fingerprint, target and build-output paths do not join")
    directives = [
        "cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate",
    ] + ["cargo:rerun-if-changed=" + name for name in RERUN_PATHS]
    require(artifacts["cargo-build-output.txt"] == ("\n".join(directives) + "\n").encode(),
            "AT Cargo build output differs from the exact selected build script directives")
    actual_out = pathlib.Path(receipt.get("actual_out_dir", ""))
    compiled_path = pathlib.Path(receipt.get("compiled_input_path", ""))
    root_output = pathlib.Path(receipt.get("root_output_path", ""))
    expected_out = output.resolve().parent / "out"
    require(actual_out.resolve() == expected_out and compiled_path.name == "public_records.rs" and
            compiled_path.resolve() == expected_out / "public_records.rs" and
            compiled_path.parent.resolve() == expected_out and
            root_output.resolve() == expected_out.parent / "root-output" and
            artifacts["cargo-root-output.txt"] == str(expected_out).encode(),
            "AT captured Cargo root-output, OUT_DIR and compiled record paths do not resolve consistently")
    require(artifacts["public_records.rs"] == expected_record and
            receipt.get("compiled_input_sha256") == sha(expected_record) and
            receipt.get("reconstructed_generated_sha256") == sha(expected_record) and
            receipt.get("source_map_sha256") == sha(source_map_bytes) and
            source_map.get("generated/public_records.rs", {}).get("sha256") == sha(expected_record) and
            receipt.get("production_rerun_input_sha256") == input_hashes and
            receipt.get("captured_actual_Cargo_artifact_count") == 4,
            "AT compiled OUT_DIR record is not reconstructed from selected production source inputs")
    return {"status": "pass", "captured_artifact_count": 4,
        "fingerprint_inputs_exact": True, "build_output_exact": True,
        "OUT_DIR_root_output_join_exact": True, "public_records_reconstructed": True}


def _read_captured_artifacts() -> tuple[dict[str, bytes], bytes]:
    artifacts = {}
    for name in ARTIFACT_FIELDS:
        path = CAPTURE_DIR / name
        artifacts[name] = _read_regular(path, f"AT captured Cargo artifact missing: {name}")
    receipt = _read_regular(CAPTURE_DIR / "public-records-build-receipt.json",
        "AT captured Cargo build receipt missing")
    return artifacts, receipt


def assert_compiled_capture_only(as_checker: Any, ar: Any,
                                 snapshot: dict[str, Any] | None = None) -> dict[str, Any]:
    expected, input_hashes = _expected_public_record(as_checker, ar)
    if snapshot is None:
        artifacts, stored_receipt = _read_captured_artifacts()
        receipt = json.loads(stored_receipt)
        source_map = _read_regular(ROOT / "generated/source-map.json", "AT source map missing")
    else:
        artifacts = snapshot["artifacts"]
        stored_receipt = snapshot["receipt_bytes"]
        receipt = json.loads(stored_receipt)
        source_map = snapshot["source_map"]
    result = assert_compiled_capture(receipt, artifacts, stored_receipt, expected, source_map, input_hashes)
    return {**result, "capture_mode": "archived artifacts only; live Cargo target not inspected"}


def assert_live_build(as_checker: Any, ar: Any, *, capture: bool,
                      require_capture: bool) -> dict[str, Any]:
    expected, input_hashes = _expected_public_record(as_checker, ar)
    fp_path = TARGET_DIR / "debug/.fingerprint" / PACKAGE
    matches = list(TARGET_DIR.glob(f"debug/.fingerprint/{PACKAGE}-*/run-build-script-build-script-build.json"))
    require(len(matches) == 1, "AT selected Cargo package must have exactly one build-script fingerprint")
    fp_path = matches[0].resolve()
    fp_bytes = _read_regular(fp_path, "AT selected Cargo build fingerprint missing")
    fingerprint = json.loads(fp_bytes)
    rerun = [entry["RerunIfChanged"] for entry in fingerprint.get("local", [])
        if isinstance(entry, dict) and isinstance(entry.get("RerunIfChanged"), dict)]
    require(len(rerun) == 1 and rerun[0].get("paths") == RERUN_PATHS,
            "AT live Cargo fingerprint has unexpected rerun inputs")
    output_rel = pathlib.Path(str(rerun[0].get("output", "")))
    require(not output_rel.is_absolute() and ".." not in output_rel.parts and
            output_rel.parts[:2] == ("debug", "build"),
            "AT Cargo output path escapes its selected target")
    output = (TARGET_DIR / output_rel).resolve()
    require(output.is_file() and output.parent.name == fp_path.parent.name and
            fp_path.parent.parent.resolve() == (TARGET_DIR / "debug/.fingerprint").resolve() and
            output.parent.parent.resolve() == (TARGET_DIR / "debug/build").resolve(),
            "AT fingerprint does not resolve to the selected package output")
    output_bytes = _read_regular(output, "AT Cargo build output missing")
    directives = [
        "cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate",
    ] + ["cargo:rerun-if-changed=" + name for name in RERUN_PATHS]
    require(output_bytes == ("\n".join(directives) + "\n").encode(),
            "AT live build directives differ from the selected build script")
    out_dir = output.parent / "out"
    root_output = output.parent / "root-output"
    compiled = out_dir / "public_records.rs"
    require(root_output.is_file() and not root_output.is_symlink() and
            compiled.is_file() and not compiled.is_symlink() and
            root_output.read_bytes() == str(out_dir).encode() and
            compiled.read_bytes() == expected,
            "AT actual Cargo OUT_DIR/public_records differs from independent reconstruction")
    artifacts = {
        "public_records.rs": compiled.read_bytes(),
        "cargo-run-build-fingerprint.json": fp_bytes,
        "cargo-build-output.txt": output_bytes,
        "cargo-root-output.txt": root_output.read_bytes(),
    }
    source_map = _read_regular(ROOT / "generated/source-map.json", "AT generated source map missing")
    receipt = {
        "schema": "at-compiled-public-records-v1", "status": "pass", "probe_package": PACKAGE,
        "build_script_sha256": sha(_read_regular(ROOT / "build.rs", "AT build script missing")),
        "extractor_sha256": sha(_read_regular(ROOT / "extract_public.py", "AT extractor missing")),
        "cargo_target_dir": str(TARGET_DIR), "cargo_build_fingerprint": str(fp_path),
        "build_output_path": str(output), "root_output_path": str(root_output),
        "actual_out_dir": str(out_dir), "compiled_input_path": str(compiled),
        "compiled_input_sha256": sha(expected), "reconstructed_generated_sha256": sha(expected),
        "source_map_sha256": sha(source_map), "production_rerun_input_sha256": input_hashes,
        "captured_actual_Cargo_artifact_count": 4,
        "captured_input_path": "generated/compiled-inputs/public_records.rs",
        "captured_cargo_build_fingerprint_path": "generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_build_output_path": "generated/compiled-inputs/cargo-build-output.txt",
        "captured_root_output_path": "generated/compiled-inputs/cargo-root-output.txt",
    }
    for name, field in ARTIFACT_FIELDS.items():
        receipt[field] = sha(artifacts[name])
    receipt_bytes = (json.dumps(receipt, indent=2) + "\n").encode()
    if capture:
        CAPTURE_DIR.mkdir(parents=True, exist_ok=True)
        for name, content in artifacts.items():
            (CAPTURE_DIR / name).write_bytes(content)
        (CAPTURE_DIR / "public-records-build-receipt.json").write_bytes(receipt_bytes)
    if capture or require_capture:
        stored_artifacts, stored_receipt = _read_captured_artifacts()
        stored_receipt_bytes = _read_regular(
            CAPTURE_DIR / "public-records-build-receipt.json", "AT captured Cargo receipt missing")
        checked = assert_compiled_capture(json.loads(stored_receipt_bytes), stored_artifacts,
            stored_receipt_bytes, expected, source_map, input_hashes)
    else:
        checked = assert_compiled_capture(receipt, artifacts, receipt_bytes, expected, source_map, input_hashes)
    return {"status": "pass", "captured_artifact_count": 4, "snapshot_written": capture,
        "live_fingerprint_and_output_checked": True, "snapshot_matches_live_build": checked}


def _load_inputs() -> dict[str, Any]:
    src_tree = _regular_tree(ROOT / "src", "AT")
    artifacts, receipt_bytes = _read_captured_artifacts()
    return {
        "source_tree": src_tree,
        "manifest": _read_regular(ROOT / "Cargo.toml", "AT Cargo manifest missing"),
        "lock": _read_regular(ROOT / "Cargo.lock", "AT Cargo lockfile missing"),
        "build": _read_regular(ROOT / "build.rs", "AT build script missing"),
        "extractor": _read_regular(ROOT / "extract_public.py", "AT extractor missing"),
        "generator": _read_regular(ROOT / "elaborate.py", "AT generator missing"),
        "launcher": _read_regular(ROOT / "run-proof.sh", "AT proof launcher missing"),
        "mapping_bytes": _read_regular(MAPPING, "AT mapping missing"),
        "active": _read_regular(ACTIVE, "AT active shadow missing"),
        "positive": _read_regular(POSITIVE, "AT positive shadow missing"),
        "extension_generated": _read_regular(ROOT / "generated/owned-clone-extension.rs", "AT generated extension missing"),
        "terminal_generated": _read_regular(ROOT / "generated/terminal-helper.rs", "AT generated terminal helper missing"),
        "client": _read_regular(ROOT / "generated/elaborated-client.rs", "AT generated native client missing"),
        "compiled_capture": {
            "artifacts": artifacts,
            "receipt_bytes": receipt_bytes,
            "source_map": _read_regular(ROOT / "generated/source-map.json", "AT source map missing"),
        },
        "environment": dict(os.environ),
    }


def audit_inputs(inputs: dict[str, Any], *, cargo_mode: str = "captured") -> dict[str, Any]:
    archive_report, as_archive = assert_as_canonical_archive()
    ancestry = assert_published_as_ancestry(as_archive)
    source_closure = assert_at_source_closure(as_archive, inputs["source_tree"])
    mapping = json.loads(inputs["mapping_bytes"])
    composition = assert_at_composition(inputs["source_tree"], inputs["generator"], mapping,
        inputs["active"], inputs["positive"], inputs["extension_generated"],
        inputs["terminal_generated"], inputs["client"], ancestry["_aq_ar"], ancestry["_ar_checker"])
    manifest = assert_probe_manifest(inputs["manifest"], inputs["lock"], inputs["build"],
        inputs["extractor"], inputs["generator"], inputs["launcher"], inputs["environment"])
    native = assert_native(mapping)
    as_checker = ancestry["_as_checker"]
    ar = ancestry["_ar_checker"]
    if cargo_mode == "captured":
        compiled = assert_compiled_capture_only(as_checker, ar, inputs["compiled_capture"])
    elif cargo_mode == "live":
        compiled = assert_live_build(as_checker, ar, capture=False, require_capture=True)
    elif cargo_mode == "capture":
        compiled = assert_live_build(as_checker, ar, capture=True, require_capture=False)
    elif cargo_mode == "skip":
        compiled = {"status": "not_checked", "reason": "explicit ancestry/source-only mode"}
    else:
        raise CheckError(f"unknown AT Cargo audit mode: {cargo_mode}")
    return {
        "status": "pass",
        "checker_scope": "one closed owned-View Clone client over published AS",
        "full_original_admitted": False,
        "published_AS_canonical_ancestry": archive_report,
        "AS_source/native/proof_replay": {k:v for k,v in ancestry.items() if not k.startswith("_")},
        "AT_exact_source_closure": source_closure,
        "AT_source_mapping_and_closed_client": composition,
        "AT_manifest_and_build_routes": manifest,
        "AT_native_correspondence": native,
        "AT_compiled_production_input": compiled,
        "claims": {
            "selected_clone": "actual shared Bytes Clone callback on an owned View, with fresh protocol ticket",
            "owned_empty_view_clone": True,
            "source_view_extent_preserved": True,
            "same_allocation_identity": "protocol identity only; no claim that byte-data addresses are equal",
            "automatic_assignment_drop": "limited to selected normal MIR places and checked client",
            "arbitrary_clone_from_and_full_original_architecture": False,
            "concurrency_or_unwind": False,
        },
    }


def audit(*, cargo_mode: str = "captured", inputs_override: dict[str, Any] | None = None) -> dict[str, Any]:
    inputs = _load_inputs() if inputs_override is None else inputs_override
    return audit_inputs(inputs, cargo_mode=cargo_mode)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", type=pathlib.Path, default=ACTIVE)
    parser.add_argument("--mapping", type=pathlib.Path, default=MAPPING)
    parser.add_argument("--ancestry-only", action="store_true")
    parser.add_argument("--capture-compiled-inputs", action="store_true")
    parser.add_argument("--audit-compiled-capture-only", action="store_true")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        require(args.shadow.resolve() == ACTIVE.resolve(), "shadow must select AT generated/active.rs")
        require(args.mapping.resolve() == MAPPING.resolve(), "mapping must select AT generated/mapping.json")
        if args.ancestry_only:
            archive, files = assert_as_canonical_archive()
            ancestry = assert_published_as_ancestry(files)
            report = {"status": "pass", "checker_scope": "published AS ancestry only; no AT admission",
                "AS_archive": archive, "AS_replay": {k:v for k,v in ancestry.items() if not k.startswith("_")}}
        elif args.audit_compiled_capture_only:
            inputs = _load_inputs()
            archive, files = assert_as_canonical_archive()
            ancestry = assert_published_as_ancestry(files)
            source_closure = assert_at_source_closure(files, inputs["source_tree"])
            mapping = json.loads(inputs["mapping_bytes"])
            composition = assert_at_composition(inputs["source_tree"], inputs["generator"], mapping,
                inputs["active"], inputs["positive"], inputs["extension_generated"],
                inputs["terminal_generated"], inputs["client"], ancestry["_aq_ar"], ancestry["_ar_checker"])
            report = {"status": "pass", "checker_scope": "AT source and archived Cargo capture; no live Cargo target",
                "AS_archive": archive, "AS_replay": {k:v for k,v in ancestry.items() if not k.startswith("_")},
                "AT_source_closure": source_closure, "AT_source_composition": composition,
                "AT_compiled_production_input": assert_compiled_capture_only(
                    ancestry["_as_checker"], ancestry["_ar_checker"])}
        elif args.capture_compiled_inputs:
            inputs = _load_inputs()
            # Check source/mapping before reading Cargo output; this mode writes
            # only the four explicitly captured compiler artifacts and receipt.
            archive, files = assert_as_canonical_archive()
            ancestry = assert_published_as_ancestry(files)
            source_closure = assert_at_source_closure(files, inputs["source_tree"])
            mapping = json.loads(inputs["mapping_bytes"])
            composition = assert_at_composition(inputs["source_tree"], inputs["generator"], mapping,
                inputs["active"], inputs["positive"], inputs["extension_generated"],
                inputs["terminal_generated"], inputs["client"], ancestry["_aq_ar"], ancestry["_ar_checker"])
            report = {"status": "pass", "checker_scope": "AT source and actual four-artifact Cargo capture",
                "AS_archive": archive, "AT_source_closure": source_closure,
                "AT_source_composition": composition,
                "AT_compiled_production_input": assert_live_build(
                    ancestry["_as_checker"], ancestry["_ar_checker"], capture=True, require_capture=False),
                "proof_result_not_claimed": True}
        else:
            report = audit(cargo_mode="live" if args.audit_compiled_capture_only else "captured")
    except Exception as exc:
        report = {"status": "reject", "checker_scope": "AT selected source/native/Cargo correspondence",
            "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if report.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
