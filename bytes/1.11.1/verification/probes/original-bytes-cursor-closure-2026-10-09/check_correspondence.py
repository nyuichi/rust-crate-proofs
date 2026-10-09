#!/usr/bin/env python3
"""AR closed-source/native correspondence for the selected Bytes cursor client.

The report binds transformed proof source, selected API/frame helpers, the
generic pointer TCB, actual native source, and the selected normal MIR trace.
Proof results, unwind behavior, and whole-crate admission are separate evidence.
"""
from __future__ import annotations

import argparse
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
AQ_ROOT = PROBES / "original-shared-slice-views-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
MAPPING = ROOT / "generated/mapping.json"
AR_PREFIX = ROOT / "src/promotion.rs"
AR_EXTENSION = ROOT / "src/cursor_extension.rs"
AR_POINTER = ROOT / "src/cursor_pointer.rs"
AR_CLIENT = ROOT / "generated/elaborated-client.rs"
AR_CARGO_TARGET_DIR = pathlib.Path("/workspace/bytes-proof-tools/targets/bytes").resolve()
AR_COMPILED_CAPTURE = ROOT / "generated/compiled-inputs"
AR_BUILD_RERUN_PATHS = ["../../../src/bytes.rs", "../../../src/bytes/bytes_record.rs",
    "../../../src/bytes/vtable_record.rs", "../../../src/bytes_mut.rs", "extract_public.py"]
AR_COMPILED_ARTIFACT_FIELDS = {
    "public_records.rs": "captured_input_sha256",
    "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
    "cargo-build-output.txt": "captured_build_output_sha256",
    "cargo-root-output.txt": "captured_root_output_sha256",
}

AR_SLICE_IS_EMPTY_SPEC = """creusot_std::macros::extern_spec! {
    impl<T> [T] {
        #[check(ghost)]
        #[ensures(result == (self@.len() == 0))]
        fn is_empty(&self) -> bool;
    }
}"""

# AQ is published commit 8449794d8115286c26fda1ff115c7c222e62e660.
# Check every executable/checker receipt and the complete AQ archive before
# importing its Python checker. The archive is then compared to current AQ and
# production source bytes at their original paths.
AQ_COMMIT = "8449794d8115286c26fda1ff115c7c222e62e660"
AQ_MAIN_CHECKER_SHA256 = "b93167002855ffde5c17765c508e3306e55804e44fe10387d71eb915efbbeb3d"
AQ_NATIVE_CHECKER_SHA256 = "0c5099600ba7e9bc4ce18f52d1926f9df3c079b5da91809517f492d533a43eb3"
AQ_MAIN_CONTROLS_SHA256 = "34b5bcc76e472d1483516e32302ea9059e8f5175726dd26e0b9258de14464698"
AQ_MAIN_FIXTURE_SHA256 = "81fff40a6d6dcd1d11375911956551fbeeb091fadd74e4760a5b91813d85a215"
AQ_MAIN_RECEIPT_SHA256 = "010d9b04599354d9e3ed6872f1bfed9ff46ef1e3bef5ab6fbd90769a0d1cbb77"
AQ_NATIVE_CONTROLS_SHA256 = "bc88dcc475c491e9c777b8c81eaff4b76fdb2b7e0aba25e794b25e6881934d85"
AQ_NATIVE_FIXTURE_SHA256 = "2971473668c46810ddd72c52d978d77524ca1b9d2294aee702e511803cd0d3bc"
AQ_NATIVE_RECEIPT_SHA256 = "e3d10feecfd34a16b138e71f5efa21dc9f731d54a1755364f133d173c67924c8"
AQ_CORRESPONDENCE_SHA256 = "fb1632771ddfcfc0f9008e820e99be03c22e096f52fd96dbaad89362b0d46500"
AQ_ACTIVE_SHA256 = "167f08c84980ff5ab80c7db50909a99879294c98ed4bc11ff7453d05c267e42b"
AQ_AUDIT_JSON_SHA256 = "a22f141a73ab45e441b74ff8b224cdf58325e3aa9c7985c787f49ff9b11f5260"
AQ_AUDIT_MD_SHA256 = "40b8b7845e980994e835b731be58a4fcd67e1930daa7b4f6b922c3de01fd88e4"
AQ_ARCHIVE_SHA256 = "41e8e9c164a1110c7f611bb1726f490e111c6b10a77af6b57bfc707ce49ceba5"
AQ_ARCHIVE_RECEIPT_SHA256 = "69723dab5c79d46d6b69c53d23b4a08a934f4920121964fff0280a66d5d50969"
AQ_NATIVE_ARCHIVE_MEMBERS = 1178
AQ_PROOF_TARGETS = 139
AQ_PROVER_LEAVES = 1202
AR_NATIVE_CHECKER_SHA256 = "a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f"
AR_NATIVE_CONTROLS_SHA256 = "6253f4e631fea8f0ab8cec3c369311dee3a824736f82cf8817a0f2a6de5528d3"
AR_NATIVE_FIXTURE_SHA256 = "e79368d7c01de089c510ec0c3bc1afb2dec846335fd9bb87fe9d050b52ccd906"
AR_NATIVE_RECEIPT_SHA256 = "ed703c3bc26743f73d8aef9f5b9fc42335a25ade73d0b8a2fb74c2bf1023ed37"
AR_CAPTURE_SHA256 = "bd5d5254c0329d29b906fa02c07528e91be383334929fb455cb00d5194e814ad"
AR_GENERATOR_SHA256 = "a63317a440aae0c8f12364a60e91ff12ba1f2eece6db3e20ee97719d9ab19b0f"
AR_NATIVE_SOURCE_SHA256 = "b0b2519d79479374eb31ab954180f11bd1030931d5057627c95e3e45a5b5c371"
AR_EXPECTED_ALPHA_RENAMING = {
    "inc_start": {"by": "cursor_by"},
    "cursor_scope": {"by": "cursor_by"},
    "slice_cursor_entry": {"begin": "view_begin"},
}
AR_EXPECTED_NORMAL_EDGES = [
    {"block":"bb2","place":"_7","owner":"original","successor":"bb3","unwind":"bb34","repeated":False,"scope":"scope"},
    {"block":"bb4","place":"_6","owner":"owner","successor":"bb5","unwind":"bb34","repeated":False,"scope":"detached"},
    {"block":"bb26","place":"_5","owner":"value","successor":"bb27","unwind":"bb34","repeated":False,"scope":"detached"},
]

# Frozen positive AR source inputs. Proof status is recorded by the separate
# proof/archive policy; these pins bind the translation source to the native
# and source correspondence report.
AR_SOURCE_PINS: dict[str, str] = {
    "src/promotion.rs": "92f6115507d4648dca6507737b120a8bea48cc42ed7fa334897fda7bda420fb9",
    "src/cursor_extension.rs": "fc4a8c4db529f2100c61faabb82d6863d5c40048c49cf39e8816bc40465b8ab0",
    "src/cursor_pointer.rs": "c943b74f3c5b5e96f896fdc27634d8e7ca2699d6150ecf27ed99ecce5fffd376",
    "generated/elaborated-client.rs": "9bb89c5d481fa21863c0564cf0d3759dc0b5c4f5bc48b7a6134c30e9d5b0243b",
    "generated/active.rs": "b30f0128f9b6a2ec935359930af4c8f835d24ab1c031f3618f5829f5f504c113",
    "generated/positive.rs": "b30f0128f9b6a2ec935359930af4c8f835d24ab1c031f3618f5829f5f504c113",
    "src/lib.rs": "e16f1e271a7061d3d9c0d7e8e1256fd3604d3ac4caa7a9e7d8cf014f9f19c2c7",
    "elaborate.py": AR_GENERATOR_SHA256,
    "Cargo.toml": "2e1bb0ef8df56804bd11952197e83a6ccdd303953084e5f2485b86c3604efa1b",
    "Cargo.lock": "2531a543d3c8315fa63c6bd6291272ebb3ecf93a7506eecd8f6e109b6f88a11b",
    "build.rs": "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
    "extract_public.py": "748810dd6d4961c1e729864a83daa3aff771c72e7ef678e9bb01f2cfef44e99b",
}


class CheckError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _read_pinned(path: pathlib.Path, expected: str, label: str) -> bytes:
    require(path.is_file() and not path.is_symlink(), f"pinned {label} is missing or redirected: {path}")
    data = path.read_bytes()
    require(sha(data) == expected, f"pinned {label} changed: {path}")
    return data


def _verify_aq_archive_current_files(receipt: dict[str, Any], archive_path: pathlib.Path) -> dict[str, Any]:
    rows = receipt.get("members")
    require(isinstance(rows, list) and len(rows) == AQ_NATIVE_ARCHIVE_MEMBERS,
            "AQ canonical receipt has an incomplete member inventory")
    recorded = {row.get("path"): row.get("sha256") for row in rows if isinstance(row, dict)}
    require(len(recorded) == AQ_NATIVE_ARCHIVE_MEMBERS and None not in recorded and
            all(isinstance(k, str) and isinstance(v, str) for k, v in recorded.items()),
            "AQ canonical member receipt has duplicate or malformed paths")
    names: list[str] = []
    checked_probe = 0
    checked_production = 0
    skipped_unfrozen_docs = 0
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) == AQ_NATIVE_ARCHIVE_MEMBERS and all(m.isfile() for m in members),
                "AQ canonical archive has duplicate/nonregular members or a changed size")
        names = [m.name for m in members]
        require(len(set(names)) == len(names) and set(names) == set(recorded),
                "AQ canonical archive member names differ from its exact receipt")
        for member in members:
            relative: pathlib.Path | None = None
            base: pathlib.Path | None = None
            if member.name.startswith("probe/"):
                base = AQ_ROOT
                relative = pathlib.Path(member.name[len("probe/"):])
                # Explanatory docs/reports can be appended after AQ's archive.
                # Executable inputs, source, generated files, MIR and proof
                # artifacts remain compared at their original paths.
                if relative.parts[:1] == ("evidence",) or relative.as_posix() in {"README.md", "TCB.md"}:
                    skipped_unfrozen_docs += 1
                    base = None
                    relative = None
            elif member.name.startswith("inputs/repository/bytes/1.11.1/"):
                base = CRATE_ROOT
                relative = pathlib.Path(member.name[len("inputs/repository/bytes/1.11.1/"):])
            handle = archive.extractfile(member)
            require(handle is not None, f"AQ archive member cannot be read: {member.name}")
            archived_bytes = handle.read()
            require(sha(archived_bytes) == recorded[member.name],
                    f"AQ archive member hash differs from its receipt: {member.name}")
            if base is None or relative is None:
                continue
            require(not relative.is_absolute() and ".." not in relative.parts,
                    f"AQ archive member has an unsafe relative path: {member.name}")
            selected = base / relative
            require(selected.is_file() and not selected.is_symlink() and
                    selected.resolve().is_relative_to(base.resolve()),
                    f"AQ archive member is absent or redirected from its original path: {member.name}")
            require(selected.read_bytes() == archived_bytes,
                    f"published AQ/production worktree bytes differ from canonical archive: {member.name}")
            if base == AQ_ROOT:
                checked_probe += 1
            else:
                checked_production += 1
    return {"member_count": len(names), "unique_regular_members": True,
        "probe_files_compared_at_original_paths": checked_probe,
        "production_files_compared_at_original_paths": checked_production,
        "post_archive_explanatory_docs_and_reports_excluded_from_worktree_equality": skipped_unfrozen_docs,
        "archive_member_hashes_verified": True}


def _import_pinned_aq(path: pathlib.Path, expected: str) -> Any:
    # This function is called only after every ancestor script/manifest/receipt
    # and the complete canonical AQ archive have been checked.
    require(sha(path.read_bytes()) == expected, "AQ checker changed before import")
    spec = importlib.util.spec_from_file_location("ar_pinned_aq_correspondence", path)
    require(spec is not None and spec.loader is not None, "cannot import the pinned AQ checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def audit_native_capture() -> dict[str, Any]:
    """Replay the frozen actual-source/MIR checker and its separate 76 controls."""
    checker_path = ROOT / "check_native.py"
    controls_path = ROOT / "native-check-controls.py"
    fixture_path = ROOT / "fixtures/native-check-controls.json"
    receipt_path = ROOT / "generated/native-check-controls.json"
    for path, expected, label in (
        (checker_path, AR_NATIVE_CHECKER_SHA256, "native checker"),
        (controls_path, AR_NATIVE_CONTROLS_SHA256, "native controls"),
        (fixture_path, AR_NATIVE_FIXTURE_SHA256, "native control fixture"),
        (receipt_path, AR_NATIVE_RECEIPT_SHA256, "native control receipt"),
        (ROOT / "native-mir/capture.json", AR_CAPTURE_SHA256, "native MIR capture"),
    ):
        _read_pinned(path, expected, label)
    control_receipt = json.loads(receipt_path.read_text())
    require(control_receipt.get("schema") == "ar-native-cursor-source-mir-controls-v1" and
            control_receipt.get("control_count") == 76 and control_receipt.get("rejected_as_expected") == 76 and
            control_receipt.get("accepted") == [] and control_receipt.get("checker_errors") == [] and
            control_receipt.get("checker_sha256") == AR_NATIVE_CHECKER_SHA256,
            "AR native source/MIR controls are incomplete")
    spec = importlib.util.spec_from_file_location("ar_pinned_native_correspondence", checker_path)
    require(spec is not None and spec.loader is not None, "cannot import the pinned AR native checker")
    native_module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = native_module
    spec.loader.exec_module(native_module)
    bundle = native_module.load_bundle()
    report = native_module.audit_bundle(bundle)
    require(report.get("status") == "pass" and
            report.get("native_mir", {}).get("selected_mir_count") == 30 and
            report.get("native_mir", {}).get("selected_production_mir_count") == 29 and
            report.get("native_mir", {}).get("client_mir_count") == 1 and
            report.get("native_mir", {}).get("inherited_aq_production_mirs_byte_exact") == 24 and
            report.get("client", {}).get("normal_edges") == [
                {"block":"bb2","place":"_7","owner":"original","successor":"bb3","unwind":"bb34","repeated":False,"scope":"scope"},
                {"block":"bb4","place":"_6","owner":"owner","successor":"bb5","unwind":"bb34","repeated":False,"scope":"detached"},
                {"block":"bb26","place":"_5","owner":"value","successor":"bb27","unwind":"bb34","repeated":False,"scope":"detached"}],
            "AR frozen native cursor/MIR correspondence gate failed")
    return report


def preflight_published_aq() -> tuple[Any, dict[str, Any]]:
    evidence = AQ_ROOT / "evidence"
    source_paths = {
        "checker": (AQ_ROOT / "check_correspondence.py", AQ_MAIN_CHECKER_SHA256),
        "native_checker": (AQ_ROOT / "check_native.py", AQ_NATIVE_CHECKER_SHA256),
        "main_controls": (AQ_ROOT / "check_checker_controls.py", AQ_MAIN_CONTROLS_SHA256),
        "main_fixture": (AQ_ROOT / "fixtures/checker-controls.json", AQ_MAIN_FIXTURE_SHA256),
        "main_receipt": (AQ_ROOT / "generated/checker-controls-receipt.json", AQ_MAIN_RECEIPT_SHA256),
        "native_controls": (AQ_ROOT / "native-check-controls.py", AQ_NATIVE_CONTROLS_SHA256),
        "native_fixture": (AQ_ROOT / "fixtures/native-check-controls.json", AQ_NATIVE_FIXTURE_SHA256),
        "native_receipt": (AQ_ROOT / "generated/native-check-controls.json", AQ_NATIVE_RECEIPT_SHA256),
        "correspondence": (AQ_ROOT / "generated/correspondence.json", AQ_CORRESPONDENCE_SHA256),
        "audit_json": (evidence / "AQ_CANONICAL_AUDIT.json", AQ_AUDIT_JSON_SHA256),
        "audit_markdown": (evidence / "AQ_CANONICAL_AUDIT.md", AQ_AUDIT_MD_SHA256),
        "archive": (evidence / "aq-positive-canonical-v1.tar.gz", AQ_ARCHIVE_SHA256),
        "archive_receipt": (evidence / "aq-positive-canonical-v1.json", AQ_ARCHIVE_RECEIPT_SHA256),
    }
    for label, (path, digest) in source_paths.items():
        _read_pinned(path, digest, label)

    audit = json.loads(source_paths["audit_json"][0].read_text())
    receipt = json.loads(source_paths["archive_receipt"][0].read_text())
    correspondence = json.loads(source_paths["correspondence"][0].read_text())
    controls = json.loads(source_paths["main_receipt"][0].read_text())
    native_controls = json.loads(source_paths["native_receipt"][0].read_text())
    require(audit.get("schema") == "aq-canonical-independent-audit-v1" and
            audit.get("archive", {}).get("sha256") == AQ_ARCHIVE_SHA256 and
            audit.get("archive", {}).get("members") == AQ_NATIVE_ARCHIVE_MEMBERS and
            audit.get("archive", {}).get("target_count") == AQ_PROOF_TARGETS and
            audit.get("archive", {}).get("status") == "proved" and
            audit.get("archive", {}).get("proof_leaves") == {
                "prover": AQ_PROVER_LEAVES, "null": 0, "structural": 0, "unknown": 0} and
            audit.get("archive", {}).get("policy", {}).get("excluded_targets") == 0 and
            audit.get("archive", {}).get("policy", {}).get("features") == [] and
            audit.get("archive", {}).get("policy", {}).get("diagnostic") is False,
            "AQ canonical audit does not record its full positive proof/source gate")
    require(receipt.get("status") == "proved" and receipt.get("archive_sha256") == AQ_ARCHIVE_SHA256 and
            receipt.get("statistics", {}).get("files") == AQ_PROOF_TARGETS and
            receipt.get("statistics", {}).get("prover") == AQ_PROVER_LEAVES and
            receipt.get("statistics", {}).get("null") == 0 and
            receipt.get("statistics", {}).get("structural") == 0 and
            receipt.get("target_policy", {}).get("excluded") == {} and
            receipt.get("target_policy", {}).get("features") == [] and
            receipt.get("target_policy", {}).get("diagnostic") is False,
            "AQ canonical archive receipt is not a full positive no-exclusion proof archive")
    require(correspondence.get("status") == "pass" and
            correspondence.get("active_composition", {}).get("active_composition_exact") is True and
            controls.get("status") == "pass" and controls.get("control_count") == 76 and
            controls.get("rejected_as_expected") == 76 and controls.get("cargo_or_rust_build_invoked") is False and
            controls.get("solver_invoked") is False and
            native_controls.get("schema") == "aq-native-source-mir-controls-v1" and
            native_controls.get("control_count") == 47 and native_controls.get("rejected") == 47 and
            native_controls.get("accepted") == [],
            "AQ source/native/control receipts are not all independently admitted")
    archive_check = _verify_aq_archive_current_files(receipt, source_paths["archive"][0])
    aq = _import_pinned_aq(source_paths["checker"][0], AQ_MAIN_CHECKER_SHA256)
    # AQ's own pre-import AP gate also checks its complete published ancestor.
    _, lineage = aq.preflight_published_ap()
    return aq, {"published_commit": AQ_COMMIT, "aq_checker_sha256": AQ_MAIN_CHECKER_SHA256,
        "aq_native_checker_sha256": AQ_NATIVE_CHECKER_SHA256,
        "aq_archive_sha256": AQ_ARCHIVE_SHA256, "aq_archive_current_worktree": archive_check,
        "aq_proof_targets": AQ_PROOF_TARGETS, "aq_prover_leaves": AQ_PROVER_LEAVES,
        "aq_controls": {"main": 76, "native": 47}, "ap_lineage": lineage}


def _aq_rust_tokens(aq: Any, source: str) -> list[str]:
    return aq.AP.AO.AN.AI.rust_tokens(source)


def assert_vacant_enum_transformation(aq: Any, ancestor: str, selected: str,
                                      mapping: dict[str, Any]) -> dict[str, Any]:
    """Require AR's only prefix edit to add unit Vacant to the exact AQ enum."""
    ancestor_start, ancestor_end, ancestor_enum = aq.rust_item_span(ancestor, "enum", "OriginalSharedProof")
    selected_start, selected_end, selected_enum = aq.rust_item_span(selected, "enum", "OriginalSharedProof")
    old_rows = aq.enum_variant_rows(ancestor_enum)
    new_rows = aq.enum_variant_rows(selected_enum)
    require([name for name, _ in old_rows] == ["Root", "Child", "View", "Empty"] and
            [name for name, _ in new_rows] == ["Root", "Child", "View", "Empty", "Vacant"],
            "AR must add exactly the unit Vacant case to the complete AQ proof sum")
    for name, old_tokens in old_rows:
        new_tokens = next(tokens for variant, tokens in new_rows if variant == name)
        require(new_tokens == old_tokens, f"AR changed inherited AQ {name} proof payload")
    require(next(tokens for name, tokens in new_rows if name == "Vacant") == _aq_rust_tokens(aq, "Vacant"),
            "AR Vacant proof placeholder has a payload or resource")
    expected_new = (mapping.get("source_transform", {}).get("new"))
    expected_old = (mapping.get("source_transform", {}).get("old"))
    require(isinstance(expected_new, str) and isinstance(expected_old, str) and
            _aq_rust_tokens(aq, expected_old) == _aq_rust_tokens(aq, ancestor_enum) and
            _aq_rust_tokens(aq, expected_new) == _aq_rust_tokens(aq, selected_enum),
            "AR mapping old/new enum literals do not identify the reviewed single Vacant edit")
    reconstructed = ancestor[:ancestor_start] + selected_enum + ancestor[ancestor_end:]
    require(selected == reconstructed,
            "AR promotion source changes AQ bytes outside the single proof-only enum declaration")
    return {"aq_active_sha256": sha(ancestor.encode()), "ar_prefix_sha256": sha(selected.encode()),
        "aq_variants_preserved": ["Root", "Child", "View", "Empty"],
        "added_variant": "Vacant", "only_declaration_changed": True,
        "not_byte_exact_to_AQ": True}


def assert_lib_route(aq: Any, selected_lib: str) -> dict[str, Any]:
    ancestor_lib = (AQ_ROOT / "src/lib.rs").read_text()
    expected = ancestor_lib.rstrip("\n") + "\n\n#[cfg(creusot)] mod cursor_pointer;\n"
    require(selected_lib == expected,
            "AR lib.rs changed AQ routes beyond the one generic cursor-pointer module")
    return {"aq_lib_route_preserved": True, "added_route": "#[cfg(creusot)] mod cursor_pointer;"}


def _regular_tree_files(directory: pathlib.Path, label: str) -> dict[str, bytes]:
    require(directory.is_dir() and not directory.is_symlink(), f"{label} source directory is missing or redirected")
    files: dict[str, bytes] = {}
    for path in directory.rglob("*"):
        require(not path.is_symlink(), f"{label} source tree contains a symlink: {path}")
        if path.is_dir():
            continue
        require(path.is_file(), f"{label} source tree contains a nonregular entry: {path}")
        relative = path.relative_to(directory).as_posix()
        files[relative] = path.read_bytes()
    return files


def assert_ar_support_closure(aq: Any, selected_lib: str | None = None,
                              ar_tree_override: dict[str, bytes] | None = None) -> dict[str, Any]:
    """Bind every inherited proof source and the complete selected module tree."""
    aq_files = _regular_tree_files(AQ_ROOT / "src", "AQ")
    ar_files = ar_tree_override if ar_tree_override is not None else _regular_tree_files(ROOT / "src", "AR")
    expected_names = set(aq_files) | {"cursor_extension.rs", "cursor_pointer.rs"}
    require(set(ar_files) == expected_names,
            "AR Rust source inventory is not AQ's complete module tree plus exactly the cursor extension and pointer module")
    changed: list[str] = []
    for relative, original in aq_files.items():
        if relative in {"lib.rs", "promotion.rs"}:
            continue
        require(ar_files[relative] == original,
                f"AR inherited AQ support/source module changed: {relative}")
        changed.append(relative)
    try:
        chosen_lib = selected_lib if selected_lib is not None else ar_files["lib.rs"].decode()
    except (KeyError, UnicodeDecodeError) as exc:
        raise CheckError(f"AR lib.rs is absent or invalid text: {exc}") from exc
    route = assert_lib_route(aq, chosen_lib)
    require("cursor_extension.rs" in ar_files and "cursor_pointer.rs" in ar_files,
            "AR cursor source modules are not regular selected source files")
    return {"aq_source_file_count": len(aq_files), "ar_source_file_count": len(ar_files),
        "all_inherited_modules_byte_exact_except_transformed_promotion_and_lib": True,
        "inherited_modules_compared": sorted(changed),
        "only_added_modules": ["cursor_extension.rs", "cursor_pointer.rs"],
        "no_symlinks_or_external_source_tree_redirects": True, "lib_route": route}


def assert_slice_is_empty_spec(aq: Any, extension: str) -> dict[str, Any]:
    """Pin the sole probe-local generic Std fact needed by the client.

    This mirrors the native slice contract `is_empty() == (len() == 0)` and
    introduces no Bytes-specific representation, ownership, or Drop property.
    Keep it exact so a caller cannot add a precondition, weaken the post, or
    replace the selected extern-spec macro while preserving nearby tokens.
    """
    expected_tokens = _aq_rust_tokens(aq, AR_SLICE_IS_EMPTY_SPEC)
    actual_tokens = _aq_rust_tokens(aq, extension)
    require(actual_tokens[:len(expected_tokens)] == expected_tokens,
            "AR generic slice is_empty extern_spec contract changed")
    mask = aq.AP.AO.AN.AI.mask_noncode(extension)
    require(len(re.findall(r"\bcreusot_std\s*::\s*macros\s*::\s*extern_spec\s*!", mask)) == 1 and
            len(re.findall(r"\bfn\s+is_empty\b", mask)) == 1,
            "AR generic slice is_empty extern_spec route is missing or duplicated")
    return {"path": "creusot_std::macros::extern_spec! impl<T> [T]::is_empty",
        "contract": "result == (self@.len() == 0)", "exact_checked_ghost_spec": True,
        "bytes_specific_ownership_facts": False}


def _extract_named_function(aq: Any, source: str, name: str) -> str:
    """Extract one executable fn item without comments/strings steering spans."""
    ai = aq.AP.AO.AN.AI
    masked = ai.mask_noncode(source)
    matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\b", masked))
    require(len(matches) == 1, f"expected one selected AR function {name}")
    start = matches[0].start()
    opening_paren = masked.find("(", matches[0].end())
    require(opening_paren >= 0, f"AR function {name} has no parameter list")
    depth = 0
    close = None
    for pos in range(opening_paren, len(masked)):
        if masked[pos] == "(":
            depth += 1
        elif masked[pos] == ")":
            depth -= 1
            if depth == 0:
                close = pos
                break
    require(close is not None, f"AR function {name} has an unterminated parameter list")
    opening = masked.find("{", close + 1)
    require(opening >= 0, f"AR function {name} has no executable body")
    depth = 0
    for pos in range(opening, len(masked)):
        if masked[pos] == "{":
            depth += 1
        elif masked[pos] == "}":
            depth -= 1
            if depth == 0:
                return source[start:pos + 1]
    raise CheckError(f"AR function {name} body is unclosed")


def _contains_sequence(tokens: list[str], phrase_tokens: list[str]) -> bool:
    if not phrase_tokens or len(phrase_tokens) > len(tokens):
        return False
    return any(tokens[i:i + len(phrase_tokens)] == phrase_tokens
               for i in range(len(tokens) - len(phrase_tokens) + 1))


def _require_order(aq: Any, tokens: list[str], phrases: list[str], label: str) -> None:
    ai = aq.AP.AO.AN.AI
    positions: list[int] = []
    for phrase in phrases:
        needle = ai.rust_tokens(phrase)
        matches = [i for i in range(len(tokens) - len(needle) + 1)
                   if tokens[i:i + len(needle)] == needle]
        require(bool(matches), f"{label} is missing operation `{phrase}`")
        previous = positions[-1] if positions else -1
        next_match = next((i for i in matches if i > previous), None)
        require(next_match is not None, f"{label} operation order changed at `{phrase}`")
        positions.append(next_match)
    require(positions == sorted(positions), f"{label} operation order changed")


def assert_ar_shadow_surface(aq: Any, extension: str, pointer: str,
                             client: str, mapping: dict[str, Any]) -> dict[str, Any]:
    """Check exact selected API/frame surfaces and body-derived call order."""
    require(sha(extension.encode()) == AR_SOURCE_PINS["src/cursor_extension.rs"] and
            sha(pointer.encode()) == AR_SOURCE_PINS["src/cursor_pointer.rs"] and
            sha(client.encode()) == AR_SOURCE_PINS["generated/elaborated-client.rs"],
            "AR selected API/frame/client source differs from its reviewed whole-file spans")
    ai = aq.AP.AO.AN.AI
    extension_names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", ai.mask_noncode(extension))
    expected_extension_names = [
        "is_empty", "api_view_valid", "api_owned", "api_id", "api_public",
        "shared_drop_input", "static_drop_input", "cursor_shared_drop_checked",
        "cursor_static_drop_checked", "cursor_shared_drop_registration",
        "cursor_static_drop_registration", "bytes_cursor_terminal_drop",
        "same_api_owner", "inc_start_api", "advance_api", "remaining_api",
        "read_api", "chunk_api", "consumed", "slice_cursor_entry",
    ]
    require(extension_names == expected_extension_names,
            "AR cursor extension added, removed or reordered a declaration")

    trusted_names = [name for name in extension_names
        if any("trusted" in attr for attr in aq.rust_outer_attributes(extension, name))]
    require(trusted_names == ["cursor_shared_drop_registration", "cursor_static_drop_registration"],
            "AR extension changed its trusted ghost callback-registration boundary")
    for name in ("api_view_valid", "api_owned", "api_id", "api_public",
                 "shared_drop_input", "static_drop_input", "cursor_shared_drop_checked",
                 "cursor_static_drop_checked", "bytes_cursor_terminal_drop", "same_api_owner",
                 "inc_start_api", "advance_api", "remaining_api", "read_api", "chunk_api",
                 "consumed", "slice_cursor_entry"):
        attrs = aq.rust_outer_attributes(extension, name)
        require(all("trusted" not in attr for attr in attrs),
                f"body-proved AR API/frame item gained trust: {name}")

    pointer_names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", ai.mask_noncode(pointer))
    require(pointer_names == ["add"], "AR generic pointer module added a callable route")
    pointer_attrs = aq.rust_outer_attributes(pointer, "add")
    require(pointer_attrs == [ai.rust_tokens("#[trusted]"),
            ai.rust_tokens("#[requires(bound.inner_logic().invariant())]"),
            ai.rust_tokens("#[requires(!pointer.is_null_logic())]"),
            ai.rust_tokens("#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]"),
            ai.rust_tokens("#[requires(match lease.inner_logic() { AdvanceLease::Live(region)=>bound.inner_logic()@!=None && region.invariant() && bound.inner_logic()@.unwrap_logic().0==region.namespace() && bound.inner_logic()@.unwrap_logic().1==region.capacity() && region.lo()<=bound.inner_logic()@.unwrap_logic().2 && bound.inner_logic()@.unwrap_logic().2+count@<=region.hi(), AdvanceLease::Zero=>count==0usize && bound.inner_logic()@==None, })]"),
            ai.rust_tokens("#[ensures(result.0==result.1.inner_logic().raw_pointer() as *const u8)]"),
            ai.rust_tokens("#[ensures(result.0.addr_logic()@==result.1.inner_logic().current_address())]"),
            ai.rust_tokens("#[ensures(crate::view_pointer::shifted(*bound.inner_logic(),result.1.inner_logic(),count@))]"),
            ai.rust_tokens("#[ensures(count==0usize ==> result.0==pointer)]"),
            ai.rust_tokens("#[ensures(!result.0.is_null_logic())]")],
            "AR trusted pointer-add contract is not the reviewed live/zero address-only interface")

    def tokens(name: str) -> list[str]:
        return ai.rust_tokens(_extract_named_function(aq, extension, name))

    frame = tokens("api_view_valid")
    for term in ("! self . ptr . is_null_logic ( )", "OriginalSharedProof :: Child", "self . child_valid ( )",
                 "bounded_view", "shared_table ( )", "pointer_event :: pointer_model", "OriginalSharedProof :: Empty",
                 "self . view_valid ( )"):
        require(_contains_sequence(frame, ai.rust_tokens(term)),
                f"AR owner/view frame lost selected validity fact `{term}`")
    owner = tokens("same_api_owner")
    for term in ("OriginalSharedProof :: Child", "OriginalSharedProof :: View", "OriginalSharedProof :: Empty",
                 "a == b"):
        require(_contains_sequence(owner, ai.rust_tokens(term)),
                f"AR API owner frame lost selected identity case `{term}`")

    inc = tokens("inc_start_api")
    _require_order(aq, inc, ["debug_assert!", "value.len-=cursor_by", "cursor_pointer::add", "value.ptr=ptr"],
                   "AR inc_start_api")
    for term in ("OriginalSharedProof :: Child", "OriginalSharedProof :: View", "OriginalSharedProof :: Empty",
                 "AdvanceLease :: Zero", "AdvanceLease :: Live", "value . original_shared"):
        require(_contains_sequence(inc, ai.rust_tokens(term)),
                f"AR inc_start_api lost selected cursor/pointer route `{term}`")
    advance = tokens("advance_api")
    _require_order(aq, advance, ["assert!", "inc_start_api"], "AR advance_api")
    require(_contains_sequence(advance, ai.rust_tokens("count<=value.len")),
            "AR advance_api no longer guards the native offset by remaining length")
    remaining = tokens("remaining_api")
    require(_contains_sequence(remaining, ai.rust_tokens("value.len")),
            "AR remaining_api is no longer the selected len projection")
    read = tokens("read_api")
    for term in ("if value.len==0", "physical_projection::borrow_empty", "PhysicalRegion",
                 "physical_projection::borrow"):
        require(_contains_sequence(read, ai.rust_tokens(term)),
                f"AR read_api lost selected zero/live physical read branch `{term}`")
    require(_contains_sequence(tokens("chunk_api"), ai.rust_tokens("read_api(value)")),
            "AR chunk_api no longer selects the checked ordinary slice-read adapter")

    sliced = tokens("slice_cursor_entry")
    require("begin" not in sliced and "view_begin" in sliced and sliced.count("checked_add") == 2 and
            all(case in sliced for case in ("Included", "Excluded", "Unbounded")),
            "AR slice_cursor_entry Range cases or native `begin` alpha rename changed")
    _require_order(aq, sliced, ["start_bound", "checked_add", "end_bound", "assert", "wrapping_bounded",
                                  "new_empty_view", "clone_shared_view", "add_live"],
                   "AR slice_cursor_entry")

    drop = tokens("bytes_cursor_terminal_drop")
    for term in ("value . vtable . drop", "cursor_shared_drop_registration", "cursor_static_drop_registration",
                 "erased_call :: invoke3"):
        require(_contains_sequence(drop, ai.rust_tokens(term)),
                f"AR terminal cursor Drop lost native callback route `{term}`")
    shared_drop = tokens("cursor_shared_drop_checked")
    require(_contains_sequence(shared_drop, ai.rust_tokens("child_drop_checked")) and
            all(_contains_sequence(shared_drop, ai.rust_tokens(term)) for term in
                ("OriginalSharedProof :: Child", "OriginalSharedProof :: View")),
            "AR Shared terminal callback no longer delegates to the inherited child-drop helper")
    static_drop = tokens("cursor_static_drop_checked")
    require(_contains_sequence(static_drop, ai.rust_tokens("Some ( ViewEffect :: Static )")),
            "AR ticket-free Empty callback no longer emits a Static view effect")
    for name, table in (("cursor_shared_drop_registration", "shared_table"),
                        ("cursor_static_drop_registration", "static_view_table")):
        reg = tokens(name) + [token for attr in aq.rust_outer_attributes(extension, name) for token in attr]
        require(all(_contains_sequence(reg, ai.rust_tokens(term)) for term in
                    ("registered3", table, "drop", "precondition", "postcondition")),
                f"AR callback {name} no longer binds the exact 3-argument native drop table")

    fold = tokens("consumed")
    for term in ("previous + steps [ count - 1 ]@ <= capacity", "previous + steps [ count - 1 ]@", "capacity"):
        require(_contains_sequence(fold, ai.rust_tokens(term)),
                f"AR consumed fold lost bounded prefix rule `{term}`")
    client_names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", ai.mask_noncode(client))
    require(client_names == ["cursor_scope"], "AR generated client added or removed an executable entry")
    client_tokens = ai.rust_tokens(_extract_named_function(aq, client, "cursor_scope"))
    client_attrs = aq.rust_outer_attributes(client, "cursor_scope")
    require(client_attrs == [
        ai.rust_tokens("#[requires(input@.len()>0)]"),
        ai.rust_tokens("#[requires(a<=b && b@<=input@.len())]"),
        ai.rust_tokens("#[ensures(result@==input@.subsequence(a@+consumed(steps@,steps@.len(),b@-a@),b@))]"),
    ], "AR client domain or suffix postcondition changed")
    _require_order(aq, client_tokens, ["from_box_scoped", "clone_root", "bytes_root_detaching_terminal_drop",
        "slice_cursor_entry", "bytes_view_terminal_drop", "while", "chunk_api", "to_vec",
        "advance_api", "saved_return", "bytes_cursor_terminal_drop", "saved_return"], "AR client cursor_scope")
    for term in ("consumed", "steps.len()", "value.view_owned()==(a<b)",
                 "assert_eq!(remaining_api(&value),0)", "assert!(chunk_api(&value).is_empty())"):
        require(_contains_sequence(client_tokens, ai.rust_tokens(term)),
                f"AR generated cursor client lost selected pre/post/invariant assertion `{term}`")

    alpha = mapping.get("native_alpha_renaming")
    require(alpha == AR_EXPECTED_ALPHA_RENAMING,
            "AR native/proof alpha-renaming map changed")
    require(mapping.get("ownership_frame") ==
            "same_api_owner; original data/vtable and actual ticket are preserved independently of byte length" and
            mapping.get("fold") == "consumed(steps,count,capacity): resource-free bounded prefix fold, no unrolling or quota" and
            mapping.get("return_evaluation") == {"shadow":"let saved_return=observed","after_final_drain":True,"before_value_drop":True},
            "AR API ownership frame, finite fold or pre-Drop result evaluation changed")
    require(mapping.get("excluded") == ["Root cursor mutation", "arbitrary concurrent or escaping ownership",
            "unwind", "whole crate"],
            "AR correspondence mapping removed an explicit scope exclusion")
    expected_methods = ["slice_cursor_entry", "inc_start_api", "advance_api", "remaining_api", "read_api", "chunk_api"]
    require(mapping.get("cursor_methods") == expected_methods and
            mapping.get("helpers") == ["bytes_root_detaching_terminal_drop", "bytes_view_terminal_drop", "bytes_cursor_terminal_drop"] and
            mapping.get("callbacks") == ["cursor_shared_drop_checked", "cursor_static_drop_checked"],
            "AR mapped API methods, terminal helpers or callbacks changed")
    body_fingerprints = {name:sha(" ".join(tokens(name)).encode()) for name in expected_extension_names
                         if name != "is_empty"}
    body_fingerprints["cursor_pointer_add"] = sha(" ".join(
        ai.rust_tokens(_extract_named_function(aq, pointer, "add"))).encode())
    body_fingerprints["client_cursor_scope"] = sha(" ".join(client_tokens).encode())
    return {"extension_functions": extension_names, "trusted_ghost_callback_registrations": trusted_names,
        "generic_pointer_trust_surface": ["add"],
        "selected_function_signature_body_token_sha256":body_fingerprints,
        "ordinary_api_method_surface": expected_methods,
        "api_frame": {"nonnull":True,"same_owner_ticket_binding":True,"data_vtable_preserved":True,
            "owned_zero_length_remains_owned":True,"ticket_free_empty_view_remains_static":True},
        "cursor_fold": "resource-free bounded prefix fold for arbitrary finite advance steps",
        "native_callback_dispatch": "selected stored vtable drop with exact ghost registration",
        "slice_empty_generic_contract": "checked Std is_empty iff slice length is zero",
        "client_trace_source_sha256": sha(client.encode()),
        "client_trace_order_checked": True}


def assert_ar_composition(aq: Any, mapping: dict[str, Any], prefix: str,
                          extension: str, client: str, active: str,
                          positive: str, pointer: str) -> dict[str, Any]:
    """Check the generator's exact selected proof-source composition."""
    is_empty_spec = assert_slice_is_empty_spec(aq, extension)
    aq_positive = (AQ_ROOT / "generated/positive.rs").read_text()
    transform = mapping.get("source_transform", {})
    require(transform.get("ancestor_source") == "../original-shared-slice-views-2026-10-09/generated/positive.rs" and
            (ROOT / transform.get("ancestor_source", "")).resolve() ==
                (AQ_ROOT / "generated/positive.rs").resolve() and
            transform.get("ancestor_sha256") == AQ_ACTIVE_SHA256 and
            transform.get("transformed_sha256") == sha(prefix.encode()) and
            mapping.get("base_source") == "src/promotion.rs" and
            mapping.get("base_source_sha256") == sha(prefix.encode()) and
            mapping.get("extension_source") == "src/cursor_extension.rs" and
            mapping.get("extension_sha256") == sha(extension.encode()) and
            mapping.get("terminal_helpers_sha256") == sha(extension.encode()) and
            mapping.get("client_sha256") == sha(client.encode()) and
            mapping.get("cursor_pointer_support") == {"path":"src/cursor_pointer.rs", "sha256":sha(pointer.encode())} and
            mapping.get("active") == "generated/active.rs" and
            mapping.get("active_sha256") == sha(active.encode()) and
            mapping.get("selected_prefix_sha256") == sha(prefix.encode()) and
            mapping.get("feature") == "" and mapping.get("status") == "generated_unchecked",
            "AR mapping does not bind the selected AQ-prefix / helper / client composition")
    expected_active = prefix + "\n" + extension + client
    require(active == expected_active and positive == expected_active and
            (ROOT / "generated/cursor-extension.rs").read_text() == extension and
            (ROOT / "generated/terminal-helper.rs").read_text() == extension and
            (ROOT / "generated/elaborated-client.rs").read_text() == client,
            "AR active, positive, extension or client file composition changed")
    return {"composition_exact": True, "aq_ancestor_source": "AQ generated/positive.rs",
        "generic_slice_is_empty_spec": is_empty_spec,
        "prefix_sha256": sha(prefix.encode()), "extension_sha256": sha(extension.encode()),
        "client_sha256": sha(client.encode()), "active_sha256": sha(active.encode())}


def ancestry_audit() -> dict[str, Any]:
    aq, lineage = preflight_published_aq()
    native_report = audit_native_capture()
    required = (MAPPING.is_file() and AR_PREFIX.is_file() and AR_EXTENSION.is_file() and
                AR_POINTER.is_file() and AR_CLIENT.is_file() and ACTIVE.is_file() and
                (ROOT / "generated/positive.rs").is_file())
    require(required, "AR source composition is incomplete")
    mapping = json.loads(MAPPING.read_text())
    prefix, extension = AR_PREFIX.read_text(), AR_EXTENSION.read_text()
    pointer, client = AR_POINTER.read_text(), AR_CLIENT.read_text()
    active = ACTIVE.read_text()
    positive = (ROOT / "generated/positive.rs").read_text()
    enum_transform = assert_vacant_enum_transformation(
        aq, (AQ_ROOT / "generated/positive.rs").read_text(), prefix, mapping)
    route = assert_lib_route(aq, (ROOT / "src/lib.rs").read_text())
    support_closure = assert_ar_support_closure(aq, (ROOT / "src/lib.rs").read_text())
    composition = assert_ar_composition(aq, mapping, prefix, extension, client, active, positive, pointer)
    shadow = assert_ar_shadow_surface(aq, extension, pointer, client, mapping)
    native_summary = assert_native_mapping(mapping, native_report)
    compiled_build = assert_ar_live_build(aq, require_capture=True)
    return {"status":"pass", "checker_scope":"AR ancestry and closed proof/API-source/native correspondence; proof results separate",
        "aq_ancestry":lineage, "enum_transformation":enum_transform,
        "lib_route":route, "support_source_closure":support_closure,
        "source_composition":composition,
        "api_and_frame_source_surface":shadow,
        "compiled_production_input":compiled_build,
        "native_capture": {"summary":native_summary,"status":native_report["status"], "selected_mir_count":30,
            "production_mir_count":29, "normal_edges":native_report["client"]["normal_edges"],
            "source_and_callback_checks":native_report["production_callbacks"],
            "actual_source_methods":native_report["cursor_source"],
            "selected_client_cfg_and_drop_edges":native_report["client"],
            "selected_api_mir":native_report["cursor_mir"]},
        "current_stage":mapping.get("stage"), "full_original_admitted":mapping.get("full_original_admitted")}


def assert_native_mapping(mapping: dict[str, Any], native: dict[str, Any]) -> dict[str, Any]:
    """Join the generator receipt to paths/edges independently checked from MIR."""
    capture = native.get("capture", {})
    client = native.get("client", {})
    cursor_source = native.get("cursor_source", {})
    cursor_mir = native.get("cursor_mir", {})
    expected_cursor_hashes = {
        "inc_start":"59117581e5d70be68feae2252aee80cc7cf18cbf3da1a6acb01e6155f98f744b",
        "remaining":"65f4b1a4e34688e0c1faae50e741e759bcfdad5485bcbee98b089c54075fd315",
        "chunk":"1b97deed01a0f490a65483d48891121d397399c293b3e7b953fe1ee426aa6af8",
        "advance":"1306de3fe45d81505ea044c468ff0537ac4d2ae919655c2901766f1c4188aa19",
        "len":"87a65c27502d58e14017cc5d4fc84e528bbdf8b7d8530a710794f100cba1d304",
    }
    require(cursor_source.get("source_body_names") == ["inc_start","remaining","chunk","advance","len"] and
            cursor_source.get("source_body_hashes") == expected_cursor_hashes and
            cursor_source.get("buf_routes") == {"advance":"assert cnt <= len; inc_start(cnt)",
                "remaining":"Bytes::len","chunk":"Bytes::as_slice","len":"Bytes.len field"} and
            cursor_source.get("default_native_inc_start", {}).get("selected_by_native_mir") is True and
            cursor_source.get("default_native_inc_start", {}).get("len_subtraction_precedes_ptr_add") is True,
            "AR actual native Buf/Bytes cursor method source is not pinned to the selected operation route")
    require(mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == AR_NATIVE_SOURCE_SHA256 and
            mapping.get("native_client_mir") ==
                "native-mir/bytes_cursor_closure_native.cursor_scope.2-2-004.ElaborateDrops.after.mir" and
            mapping.get("native_mir_ready") is True and
            mapping.get("normal_edges") == AR_EXPECTED_NORMAL_EDGES == client.get("normal_edges") and
            mapping.get("debug_places") == {
                "input":"_1","a":"_2","b":"_3","steps":"_4","value":"_5","i":"_14",
                "by":"_21","observed":"_35","rest":"_39","left_val":"_51","right_val":"_52",
                "kind":"_57","owner":"_6","original":"_7"} and
            capture.get("stage") == "2-2-004.ElaborateDrops.after.mir" and
            capture.get("selected_mir_count") == 30 and capture.get("production_mir_count") == 29 and
            capture.get("client_mir_count") == 1,
            "AR mapping does not identify the exact selected native cursor client/MIR edges")
    require(native.get("production_callbacks", {}).get("constructor_clone_cleanup_read_source_exact") is True and
            native.get("production_callbacks", {}).get("even_odd_clone_and_drop_sources_exact") is True and
            native.get("production_callbacks", {}).get("shared_arc_promotion_and_cleanup_sources_exact") is True and
            native.get("production_callbacks", {}).get("promotable_parity_vtables_bind_clone_and_drop_callbacks") is True and
            native.get("production_callbacks", {}).get("shared_vtable_binds_existing_arc_clone_and_drop_callbacks") is True and
            native.get("production_callbacks", {}).get("shared_fields_and_drop_guard_checked") is True and
            native.get("production_callbacks", {}).get("native_atomic_mut_and_refcount_increment_sources_exact") is True,
            "AR inherited actual Bytes constructor/clone/drop callback source gate failed")
    require(native.get("terminal_field_profile", {}).get("exact_native_Bytes_field_profile_checked") is True and
            native.get("terminal_field_profile", {}).get("compile_time_no_independent_field_drop_glue") is True and
            native.get("terminal_field_profile", {}).get("cfg_only_proof_Ghost_fields_erased") is True,
            "AR native field Drop/erasure profile is not checked")
    require(client.get("normal_cfg_exact") is True and
            client.get("normal_completion_only") is True and client.get("unwind_claim") is False and
            client.get("suffix_and_drain", {}).get("result_move_precedes_drop") is True and
            client.get("loop", {}).get("arbitrary_finite_steps_no_fixed_quota") is True and
            cursor_mir.get("inc_start", {}).get("normal_cfg_exact") is True and
            cursor_mir.get("buf_advance", {}).get("success_calls_inc_start") is True and
            cursor_mir.get("buf_remaining_calls_len") is True and
            cursor_mir.get("buf_chunk_calls_as_slice") is True and
            cursor_mir.get("bytes_drop", {}).get("single_stored_vtable_drop_call") is True,
            "AR selected API/MIR client and native callback routing are not exact")
    return {"mapping_edges_derive_from_selected_mir":True,
        "normal_completion_only":True,"unwind_included":False,
        "native_drop_dispatch":"one actual stored-vtable callback with data/ptr/len",
        "actual_constructor_clone_and_callback_sources_checked":True,
        "default_native_field_profile_checked":True}


def assert_probe_inputs(manifest_text: str | None = None, lock_bytes: bytes | None = None,
                        build_bytes: bytes | None = None,
                        environment: dict[str, str] | None = None) -> dict[str, Any]:
    expected_manifest = {
        "package": {"name":"bytes-original-bytes-cursor-closure","version":"0.1.0",
                     "edition":"2021","publish":False},
        "dependencies": {"creusot-std":"=0.13.0"},
        "workspace": {},
        "features": {"negative_missing_acquire":[],"negative_missing_payload_free":[],
                     "negative_missing_control_free":[]},
    }
    manifest_path = ROOT / "Cargo.toml"
    lock_path = ROOT / "Cargo.lock"
    build_path = ROOT / "build.rs"
    try:
        manifest = tomllib.loads(manifest_path.read_text() if manifest_text is None else manifest_text)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AR probe manifest could not be parsed: {exc}") from exc
    selected_lock = lock_path.read_bytes() if lock_bytes is None else lock_bytes
    selected_build = build_path.read_bytes() if build_bytes is None else build_bytes
    require(manifest == expected_manifest and
            sha(selected_lock) == "2531a543d3c8315fa63c6bd6291272ebb3ecf93a7506eecd8f6e109b6f88a11b" and
            sha(selected_build) == "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
            "AR Cargo dependency/build route or feature surface changed")
    selected_env = dict(os.environ) if environment is None else environment
    require(not selected_env.get("BYTES_DROP_FEATURE") and not selected_env.get("BYTES_SCOPE_SOURCE_CONTROL") and
            selected_env.get("BYTES_SCOPE_DIAGNOSTIC", "0") != "1" and
            selected_env.get("BYTES_DROP_CHECKER_SKIP", "0") != "1",
            "AR source checker cannot admit a feature, source-control or diagnostic run")
    return {"manifest_exact":True,"lock_hash_pinned":True,"build_source_route_pinned":True,
        "active_negative_or_diagnostic_environment":False}


def assert_ar_compiled_capture(receipt: dict[str, Any],
                               artifacts_override: dict[str, bytes] | None = None,
                               stored_receipt_bytes: bytes | None = None,
                               expected_record: bytes | None = None,
                               source_map_bytes: bytes | None = None,
                               expected_input_hashes: dict[str, str] | None = None) -> dict[str, Any]:
    """Validate the four archived Cargo artifacts and bind their paths to AR."""
    receipt_path = AR_COMPILED_CAPTURE / "public-records-build-receipt.json"
    if artifacts_override is not None:
        require(set(artifacts_override) == set(AR_COMPILED_ARTIFACT_FIELDS),
                "AR compiled-input replay override has an incomplete artifact set")
    if stored_receipt_bytes is None:
        require(receipt_path.is_file() and not receipt_path.is_symlink(),
                "AR captured Cargo build receipt is missing or redirected")
        stored_receipt_bytes = receipt_path.read_bytes()
    try:
        require(json.loads(stored_receipt_bytes) == receipt,
                "AR compiled-input receipt bytes differ from the receipt being checked")
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise CheckError(f"AR compiled-input receipt is not valid JSON: {exc}") from exc
    require(receipt.get("schema") == "ar-compiled-public-records-v1" and
            receipt.get("status") == "pass" and
            receipt.get("probe_package") == "bytes-original-bytes-cursor-closure" and
            receipt.get("build_script_sha256") == AR_SOURCE_PINS["build.rs"] and
            receipt.get("extractor_sha256") == AR_SOURCE_PINS["extract_public.py"],
            "AR compiled-input receipt identifies a changed Cargo package/build extractor")
    expected_capture_paths = {
        "captured_input_path": "generated/compiled-inputs/public_records.rs",
        "captured_cargo_build_fingerprint_path": "generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_build_output_path": "generated/compiled-inputs/cargo-build-output.txt",
        "captured_root_output_path": "generated/compiled-inputs/cargo-root-output.txt",
    }
    require(all(receipt.get(field) == value for field, value in expected_capture_paths.items()) and
            receipt.get("captured_actual_Cargo_artifact_count") == 4,
            "AR compiled-input receipt redirects or omits a captured Cargo artifact")
    if artifacts_override is None:
        require(AR_COMPILED_CAPTURE.is_dir() and not AR_COMPILED_CAPTURE.is_symlink(),
                "AR compiled-input capture directory is missing or redirected")
        expected_names = set(AR_COMPILED_ARTIFACT_FIELDS) | {"public-records-build-receipt.json"}
        actual_names = {entry.name for entry in AR_COMPILED_CAPTURE.iterdir()}
        require(actual_names == expected_names and all((AR_COMPILED_CAPTURE / name).is_file() and
                not (AR_COMPILED_CAPTURE / name).is_symlink() for name in expected_names),
                "AR compiled-input capture inventory has missing, extra or redirected files")
    captured: dict[str, bytes] = {}
    for name, field in AR_COMPILED_ARTIFACT_FIELDS.items():
        if artifacts_override is not None:
            data = artifacts_override[name]
        else:
            path = AR_COMPILED_CAPTURE / name
            require(path.is_file() and not path.is_symlink(), f"AR captured Cargo artifact is missing: {name}")
            data = path.read_bytes()
        require(receipt.get(field) == sha(data), f"AR captured Cargo artifact hash changed: {name}")
        captured[name] = data

    try:
        fingerprint = json.loads(captured["cargo-run-build-fingerprint.json"])
        source_map = json.loads(source_map_bytes if source_map_bytes is not None else
                                (ROOT / "generated/source-map.json").read_bytes())
    except (UnicodeDecodeError, json.JSONDecodeError, OSError) as exc:
        raise CheckError(f"AR captured fingerprint/source map cannot be parsed: {exc}") from exc
    rerun_rows = [row["RerunIfChanged"] for row in fingerprint.get("local", [])
                  if isinstance(row, dict) and isinstance(row.get("RerunIfChanged"), dict)]
    require(len(rerun_rows) == 1 and rerun_rows[0].get("paths") == AR_BUILD_RERUN_PATHS,
            "AR captured Cargo fingerprint does not record the exact production/extractor inputs")
    output_rel = pathlib.Path(str(rerun_rows[0].get("output", "")))
    target = pathlib.Path(str(receipt.get("cargo_target_dir", "")))
    output_path = pathlib.Path(str(receipt.get("build_output_path", "")))
    fingerprint_path = pathlib.Path(str(receipt.get("cargo_build_fingerprint", "")))
    target_resolved = target.resolve()
    output_resolved = output_path.resolve()
    fingerprint_resolved = fingerprint_path.resolve()
    require(not output_rel.is_absolute() and ".." not in output_rel.parts and
            output_rel.parts[:2] == ("debug", "build") and
            target_resolved == AR_CARGO_TARGET_DIR and
            receipt.get("cargo_target_dir") == str(AR_CARGO_TARGET_DIR) and
            (target / output_rel).resolve() == output_resolved and
            receipt.get("build_output_path") == str(output_resolved) and
            receipt.get("cargo_build_fingerprint") == str(fingerprint_resolved) and
            output_path.name == "output" and
            fingerprint_path.name == "run-build-script-build-script-build.json" and
            fingerprint_resolved.parent.parent == (target / "debug/.fingerprint").resolve() and
            output_resolved.parent.parent == (target / "debug/build").resolve() and
            fingerprint_resolved.parent.name == output_resolved.parent.name and
            fingerprint_resolved.parent.name.startswith("bytes-original-bytes-cursor-closure-") and
            receipt.get("cargo_build_fingerprint_sha256") == sha(captured["cargo-run-build-fingerprint.json"]) and
            receipt.get("build_output_sha256") == sha(captured["cargo-build-output.txt"]) and
            receipt.get("root_output_sha256") == sha(captured["cargo-root-output.txt"]),
            "AR captured Cargo fingerprint, package, target and build output do not join")
    directives = (["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
                   "cargo:rustc-cfg=bytes_original_shared_gate"] +
                  ["cargo:rerun-if-changed=" + path for path in AR_BUILD_RERUN_PATHS])
    require(captured["cargo-build-output.txt"] == ("\n".join(directives) + "\n").encode(),
            "AR captured Cargo build directives/configuration changed")
    actual_out = pathlib.Path(str(receipt.get("actual_out_dir", "")))
    compiled_path = pathlib.Path(str(receipt.get("compiled_input_path", "")))
    root_output_path = pathlib.Path(str(receipt.get("root_output_path", "")))
    expected_out = output_resolved.parent / "out"
    require(actual_out.resolve() == expected_out.resolve() and str(actual_out.resolve()) == str(expected_out) and
            compiled_path.name == "public_records.rs" and compiled_path.resolve() == expected_out / "public_records.rs" and
            str(compiled_path.resolve()) == str(expected_out / "public_records.rs") and
            compiled_path.parent.resolve() == expected_out and
            root_output_path.resolve() == expected_out.parent / "root-output" and
            str(root_output_path.resolve()) == str(expected_out.parent / "root-output") and
            captured["cargo-root-output.txt"] == str(expected_out).encode(),
            "AR captured Cargo root-output/OUT_DIR/public_records paths do not resolve consistently")
    selected_record = expected_record if expected_record is not None else (ROOT / "generated/public_records.rs").read_bytes()
    selected_source_map = source_map_bytes if source_map_bytes is not None else (ROOT / "generated/source-map.json").read_bytes()
    require(captured["public_records.rs"] == selected_record and
            receipt.get("compiled_input_sha256") == sha(captured["public_records.rs"]) and
            receipt.get("reconstructed_generated_sha256") == sha(selected_record) and
            receipt.get("source_map_sha256") == sha(selected_source_map) and
            source_map.get("generated/public_records.rs", {}).get("sha256") == sha(selected_record),
            "AR actual compiled OUT_DIR record differs from the independently reconstructed selected record")
    input_hashes = expected_input_hashes
    require(input_hashes is not None and receipt.get("production_rerun_input_sha256") == input_hashes,
            "AR compiled-input receipt does not bind all selected production/extractor rerun inputs")
    return {"captured_actual_Cargo_artifact_count": 4, "fingerprint_inputs_exact": True,
        "build_output_exact": True, "root_output_route_exact": True,
        "actual_out_dir_join_exact": True, "compiled_public_records_sha256": sha(selected_record),
        "compiled_input_and_capture_match_live_selected_build": True}


def assert_ar_live_build(aq: Any, require_capture: bool = True,
                         capture: bool = False) -> dict[str, Any]:
    """Reconstruct the selected records, read the actual Cargo OUT_DIR, and optionally snapshot it."""
    production_manifest_path = CRATE_ROOT / "Cargo.toml"
    try:
        production_manifest = tomllib.loads(production_manifest_path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AR selected bytes 1.11.1 Cargo manifest cannot be parsed: {exc}") from exc
    require(production_manifest.get("package", {}).get("name") == "bytes" and
            production_manifest.get("package", {}).get("version") == "1.11.1" and
            production_manifest.get("package", {}).get("build") is False and
            production_manifest.get("lib", {}).get("path") == "src/lib.rs",
            "AR build rerun source root is not the selected bytes 1.11.1 package")
    build_bytes = (ROOT / "build.rs").read_bytes()
    extractor_bytes = (ROOT / "extract_public.py").read_bytes()
    require(sha(build_bytes) == AR_SOURCE_PINS["build.rs"] and
            sha(extractor_bytes) == AR_SOURCE_PINS["extract_public.py"],
            "AR Cargo build/extractor program changed")
    input_paths = {
        "../../../src/bytes.rs": CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": ROOT / "extract_public.py",
    }
    input_hashes: dict[str, str] = {}
    for literal, selected in input_paths.items():
        resolved = (ROOT / literal).resolve()
        require(not selected.is_symlink() and resolved == selected.resolve() and resolved.is_file(),
                f"AR Cargo rerun literal resolves outside the selected source: {literal}")
        input_hashes[literal] = sha(resolved.read_bytes())

    records = (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text()
    vtables = (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text()
    mutable = (CRATE_ROOT / "src/bytes_mut.rs").read_text()
    bytes_source = (CRATE_ROOT / "src/bytes.rs").read_text()
    ai = aq.AP.AO.AN.AI
    extraction = ai.audit_generated_extractions(bytes_source, ROOT / "generated", records, vtables, mutable)
    shared = ai.extract_struct_source(mutable, "struct Shared {", "BytesMut::Shared")
    bytesmut = ai.extract_struct_source(mutable, "pub struct BytesMut {", "BytesMut")
    expected_record = (records + "\n" + vtables + "\nmod mutable_record {\n"
        "use alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        shared + "\n" + bytesmut + "\n}\nuse mutable_record::BytesMut;\n").encode()
    generated_path = ROOT / "generated/public_records.rs"
    source_map_path = ROOT / "generated/source-map.json"
    require(generated_path.is_file() and not generated_path.is_symlink() and
            source_map_path.is_file() and not source_map_path.is_symlink(),
            "AR extractor output/public source map is missing or redirected")
    generated = generated_path.read_bytes()
    source_map_bytes = source_map_path.read_bytes()
    try:
        source_map = json.loads(source_map_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise CheckError(f"AR generated source map is invalid JSON: {exc}") from exc
    require(generated == expected_record and
            source_map.get("generated/public_records.rs", {}).get("sha256") == sha(generated),
            "AR public record output does not reconstruct from actual bytes 1.11.1 sources")

    target_value = os.environ.get("CARGO_TARGET_DIR")
    if target_value:
        require(pathlib.Path(target_value).resolve() == AR_CARGO_TARGET_DIR,
                "AR Cargo target directory differs from the pinned proof environment")
    target = AR_CARGO_TARGET_DIR
    fingerprints = list((target / "debug/.fingerprint").glob(
        "bytes-original-bytes-cursor-closure-*/run-build-script-build-script-build.json"))
    require(len(fingerprints) == 1 and fingerprints[0].is_file() and not fingerprints[0].is_symlink(),
            "AR requires one selected default-feature Cargo build-script fingerprint")
    fingerprint_path = fingerprints[0].resolve()
    fingerprint_bytes = fingerprint_path.read_bytes()
    try:
        fingerprint = json.loads(fingerprint_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise CheckError(f"AR selected Cargo build fingerprint is invalid JSON: {exc}") from exc
    rerun_rows = [row["RerunIfChanged"] for row in fingerprint.get("local", [])
                  if isinstance(row, dict) and isinstance(row.get("RerunIfChanged"), dict)]
    require(len(rerun_rows) == 1 and rerun_rows[0].get("paths") == AR_BUILD_RERUN_PATHS,
            "AR actual Cargo fingerprint does not bind the exact production/extractor source paths")
    output_rel = pathlib.Path(str(rerun_rows[0].get("output", "")))
    require(not output_rel.is_absolute() and ".." not in output_rel.parts and
            output_rel.parts[:2] == ("debug", "build"),
            "AR Cargo build output escaped the selected target")
    output_path = (target / output_rel).resolve()
    require(output_path.is_file() and fingerprint_path.parent.name == output_path.parent.name and
            fingerprint_path.parent.parent.resolve() == (target / "debug/.fingerprint").resolve() and
            output_path.parent.parent.resolve() == (target / "debug/build").resolve(),
            "AR actual Cargo fingerprint does not join to the selected package output")
    output_bytes = output_path.read_bytes()
    directives = (["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
                   "cargo:rustc-cfg=bytes_original_shared_gate"] +
                  ["cargo:rerun-if-changed=" + path for path in AR_BUILD_RERUN_PATHS])
    require(output_bytes == ("\n".join(directives) + "\n").encode(),
            "AR actual Cargo build output directives differ from the selected build script")
    out_dir = output_path.parent / "out"
    root_output = output_path.parent / "root-output"
    compiled_path = out_dir / "public_records.rs"
    require(root_output.is_file() and not root_output.is_symlink() and
            compiled_path.is_file() and not compiled_path.is_symlink() and
            root_output.read_bytes() == str(out_dir).encode() and
            compiled_path.read_bytes() == expected_record == generated,
            "AR actual OUT_DIR/root-output public record differs from the independently reconstructed record")
    artifacts = {"public_records.rs": compiled_path.read_bytes(),
        "cargo-run-build-fingerprint.json": fingerprint_bytes,
        "cargo-build-output.txt": output_bytes,
        "cargo-root-output.txt": root_output.read_bytes()}
    receipt = {"schema":"ar-compiled-public-records-v1", "status":"pass",
        "probe_package":"bytes-original-bytes-cursor-closure", "build_script_sha256":sha(build_bytes),
        "extractor_sha256":sha(extractor_bytes), "cargo_target_dir":str(target),
        "cargo_build_fingerprint":str(fingerprint_path), "cargo_build_fingerprint_sha256":sha(fingerprint_bytes),
        "captured_cargo_build_fingerprint_path":"generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_cargo_build_fingerprint_sha256":sha(fingerprint_bytes),
        "build_output_path":str(output_path), "build_output_sha256":sha(output_bytes),
        "captured_build_output_path":"generated/compiled-inputs/cargo-build-output.txt",
        "captured_build_output_sha256":sha(output_bytes), "root_output_path":str(root_output),
        "root_output_sha256":sha(artifacts["cargo-root-output.txt"]),
        "captured_root_output_path":"generated/compiled-inputs/cargo-root-output.txt",
        "captured_root_output_sha256":sha(artifacts["cargo-root-output.txt"]),
        "actual_out_dir":str(out_dir), "compiled_input_path":str(compiled_path),
        "compiled_input_sha256":sha(expected_record),
        "captured_input_path":"generated/compiled-inputs/public_records.rs",
        "captured_input_sha256":sha(expected_record), "reconstructed_generated_sha256":sha(expected_record),
        "captured_actual_Cargo_artifact_count":4, "source_map_sha256":sha(source_map_bytes),
        "production_rerun_input_sha256":input_hashes}
    receipt_bytes = (json.dumps(receipt, indent=2) + "\n").encode()

    if capture:
        require(AR_COMPILED_CAPTURE.parent.is_dir() and not AR_COMPILED_CAPTURE.parent.is_symlink(),
                "AR compiled-input parent directory is missing or redirected")
        if AR_COMPILED_CAPTURE.exists():
            require(AR_COMPILED_CAPTURE.is_dir() and not AR_COMPILED_CAPTURE.is_symlink(),
                    "AR compiled-input capture directory is redirected")
            allowed = set(AR_COMPILED_ARTIFACT_FIELDS) | {"public-records-build-receipt.json"}
            require({path.name for path in AR_COMPILED_CAPTURE.iterdir()} <= allowed,
                    "AR compiled-input capture directory contains an unexpected file")
        else:
            AR_COMPILED_CAPTURE.mkdir()
        for name, data in artifacts.items():
            (AR_COMPILED_CAPTURE / name).write_bytes(data)
        (AR_COMPILED_CAPTURE / "public-records-build-receipt.json").write_bytes(receipt_bytes)

    if require_capture or capture:
        capture_receipt_path = AR_COMPILED_CAPTURE / "public-records-build-receipt.json"
        require(capture_receipt_path.is_file() and not capture_receipt_path.is_symlink(),
                "AR compiled Cargo inputs have not been captured; use --capture-compiled-inputs after the final translation")
        capture_receipt_bytes = capture_receipt_path.read_bytes()
        captured_report = assert_ar_compiled_capture(receipt,
            stored_receipt_bytes=capture_receipt_bytes, expected_record=expected_record,
            source_map_bytes=source_map_bytes, expected_input_hashes=input_hashes)
    else:
        captured_report = assert_ar_compiled_capture(receipt, artifacts_override=artifacts,
            stored_receipt_bytes=receipt_bytes, expected_record=expected_record,
            source_map_bytes=source_map_bytes, expected_input_hashes=input_hashes)
    return {"captured_artifact_count":4, "production_rerun_input_sha256":input_hashes,
        "source_extraction":extraction, "actual_OUT_DIR_record_reconstructed":True,
        "source_map_sha256":sha(source_map_bytes), "compiled_public_records_sha256":sha(expected_record),
        "live_Cargo_fingerprint_and_output_checked":True, "snapshot_written":capture,
        "snapshot_matches_live_build":captured_report, "receipt":receipt}


def assert_ar_captured_build(aq: Any) -> dict[str, Any]:
    """Replay the archived four-artifact snapshot without needing a live Cargo target."""
    assert_probe_inputs()
    build_bytes = (ROOT / "build.rs").read_bytes()
    extractor_bytes = (ROOT / "extract_public.py").read_bytes()
    require(sha(build_bytes) == AR_SOURCE_PINS["build.rs"] and
            sha(extractor_bytes) == AR_SOURCE_PINS["extract_public.py"],
            "AR archived Cargo build/extractor source changed")
    try:
        production_manifest = tomllib.loads((CRATE_ROOT / "Cargo.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AR archived production Cargo manifest cannot be parsed: {exc}") from exc
    require(production_manifest.get("package", {}).get("name") == "bytes" and
            production_manifest.get("package", {}).get("version") == "1.11.1" and
            production_manifest.get("package", {}).get("build") is False and
            production_manifest.get("lib", {}).get("path") == "src/lib.rs",
            "AR archived build rerun root is not the selected bytes 1.11.1 package")
    paths = {
        "../../../src/bytes.rs": CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": ROOT / "extract_public.py",
    }
    input_hashes: dict[str, str] = {}
    for literal, selected in paths.items():
        resolved = (ROOT / literal).resolve()
        require(not selected.is_symlink() and resolved == selected.resolve() and resolved.is_file(),
                f"AR archived Cargo rerun input resolves to an unselected source: {literal}")
        input_hashes[literal] = sha(resolved.read_bytes())
    records = (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text()
    vtables = (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text()
    mutable = (CRATE_ROOT / "src/bytes_mut.rs").read_text()
    bytes_source = (CRATE_ROOT / "src/bytes.rs").read_text()
    ai = aq.AP.AO.AN.AI
    extraction = ai.audit_generated_extractions(bytes_source, ROOT / "generated", records, vtables, mutable)
    shared = ai.extract_struct_source(mutable, "struct Shared {", "BytesMut::Shared")
    bytesmut = ai.extract_struct_source(mutable, "pub struct BytesMut {", "BytesMut")
    expected_record = (records + "\n" + vtables + "\nmod mutable_record {\n"
        "use alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        shared + "\n" + bytesmut + "\n}\nuse mutable_record::BytesMut;\n").encode()
    source_map_bytes = (ROOT / "generated/source-map.json").read_bytes()
    receipt_path = AR_COMPILED_CAPTURE / "public-records-build-receipt.json"
    require(receipt_path.is_file() and not receipt_path.is_symlink(),
            "AR archived Cargo receipt is missing or redirected")
    try:
        receipt = json.loads(receipt_path.read_bytes())
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise CheckError(f"AR archived Cargo receipt is invalid JSON: {exc}") from exc
    capture = assert_ar_compiled_capture(receipt, expected_record=expected_record,
        source_map_bytes=source_map_bytes, expected_input_hashes=input_hashes)
    target_value = os.environ.get("CARGO_TARGET_DIR")
    require(not target_value or pathlib.Path(target_value).resolve() == AR_CARGO_TARGET_DIR,
            "AR archive replay selected an unpinned Cargo target directory")
    return {"captured_artifact_count":4, "archive_capture_replayed_without_Cargo":True,
        "source_extraction":extraction, "source_map_sha256":sha(source_map_bytes),
        "compiled_public_records_sha256":sha(expected_record), "capture":capture}


def audit() -> dict[str, Any]:
    report = ancestry_audit()
    mapping = json.loads(MAPPING.read_text())
    require(bool(AR_SOURCE_PINS), "AR closed-source/API pins are not configured")
    for relative, expected in AR_SOURCE_PINS.items():
        _read_pinned(ROOT / relative, expected, f"AR selected input {relative}")
    require(mapping.get("feature") == "" and mapping.get("status") == "generated_unchecked" and
            mapping.get("stage") == "after-ElaborateDrops" and mapping.get("full_original_admitted") is False,
            "AR selected correspondence was generated under a feature or broad-admission label")
    probe_inputs = assert_probe_inputs()
    report.update({"checker_scope":"closed source/native correspondence for the selected Bytes cursor client; proof status is admitted separately",
        "ar_source_pins":AR_SOURCE_PINS,"probe_build_inputs":probe_inputs,
        "not_a_proof_or_unwind_claim":True})
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", type=pathlib.Path, default=ACTIVE)
    parser.add_argument("--mapping", type=pathlib.Path, default=MAPPING)
    parser.add_argument("--ancestry-only", action="store_true")
    parser.add_argument("--capture-compiled-inputs", action="store_true",
                        help="snapshot the four independently checked live Cargo OUT_DIR/build artifacts")
    parser.add_argument("--audit-compiled-capture-only", action="store_true",
                        help="replay the captured four-artifact Cargo receipt without consulting a live target directory")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        require(args.shadow.resolve() == ACTIVE.resolve(), "shadow must select AR generated/active.rs")
        require(args.mapping.resolve() == MAPPING.resolve(), "mapping must select AR generated/mapping.json")
        if args.capture_compiled_inputs:
            require(not args.ancestry_only and not args.audit_compiled_capture_only,
                    "compiled-input capture cannot be combined with another audit mode")
            assert_probe_inputs()
            aq, _ = preflight_published_aq()
            support = assert_ar_support_closure(aq)
            mapping = json.loads(MAPPING.read_text())
            require(mapping.get("feature") == "" and mapping.get("status") == "generated_unchecked" and
                    sha(ACTIVE.read_bytes()) == AR_SOURCE_PINS["generated/active.rs"] and
                    (ROOT / "generated/positive.rs").read_bytes() == ACTIVE.read_bytes(),
                    "compiled-input capture requires the restored selected positive AR source")
            result = {"status":"pass", "checker_scope":"explicit AR compiled-input snapshot after the positive translation",
                "support_source_closure":support,
                "compiled_production_input":assert_ar_live_build(aq, require_capture=False, capture=True),
                "proof_result_not_claimed":True}
        elif args.audit_compiled_capture_only:
            require(not args.ancestry_only, "compiled-capture replay cannot be combined with --ancestry-only")
            aq, lineage = preflight_published_aq()
            support = assert_ar_support_closure(aq)
            result = {"status":"pass", "checker_scope":"AR archived Cargo build-input replay; no live-target or proof claim",
                "aq_ancestry":lineage, "support_source_closure":support,
                "compiled_production_input":assert_ar_captured_build(aq), "proof_result_not_claimed":True}
        else:
            result = ancestry_audit() if args.ancestry_only else audit()
    except Exception as exc:
        result = {"status":"reject", "checker_scope":"AR original Bytes cursor source/native correspondence",
            "reason":f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
