#!/usr/bin/env python3
"""AS exact-source correspondence for three nonnull view-boundary repairs.

The checker binds the complete published AR source and its canonical proof
archive, then admits exactly three source transformations: one global
non-null conjunct on ``view_valid``, and non-null preconditions on the two
generic empty-pointer adapters.  Native correspondence and the actual Cargo
OUT_DIR capture are checked independently.  This is not a whole-crate or
unwind admission.
"""
from __future__ import annotations

import argparse
import copy
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
AR_ROOT = PROBES / "original-bytes-cursor-closure-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
POSITIVE = ROOT / "generated/positive.rs"
MAPPING = ROOT / "generated/mapping.json"
CAPTURE_DIR = ROOT / "generated/compiled-inputs"
TARGET_DIR = pathlib.Path("/workspace/bytes-proof-tools/targets/bytes").resolve()
PACKAGE = "bytes-original-nonnull-view-boundaries"
RERUN_PATHS = ["../../../src/bytes.rs", "../../../src/bytes/bytes_record.rs",
    "../../../src/bytes/vtable_record.rs", "../../../src/bytes_mut.rs", "extract_public.py"]
ARTIFACT_FIELDS = {
    "public_records.rs": "captured_input_sha256",
    "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
    "cargo-build-output.txt": "captured_build_output_sha256",
    "cargo-root-output.txt": "captured_root_output_sha256",
}

# Published AR canonical-v2 anchors.  Verify bytes before importing any
# ancestor Python checker; the archive and its receipt are checked separately.
AR_COMMIT = "ccca44a694d6553226149f089996bf16943c9f25"
AR_CHECKER_SHA256 = "bce69486ae82aa5aa0f01dde80a67b18cb1fce25287e87881ebe618cac78d526"
AR_AUDIT_JSON_SHA256 = "e9a9fee181e77635e7602f04ffd214c4673248d195a8b306d1ae0c8b70e1c2c3"
AR_AUDIT_MD_SHA256 = "df604e4d7fcc529f8e67b71692f9da68fec5cf03005081228e349fc26ee1e670"
AR_ARCHIVE_SHA256 = "b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7"
AR_RECEIPT_SHA256 = "dd1958e09df6cb6a3c3538d23a27abf51cec13e6fd243ff314aaa798d4a4a90d"
AR_ARCHIVE_MEMBERS = 1312
AR_TARGETS = 150
AR_PROVER_LEAVES = 1328
AR_SOURCE_COMMIT = "8449794d8115286c26fda1ff115c7c222e62e660"
AR_NATIVE_CHECKER_SHA256 = "a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f"
AR_NATIVE_CONTROLS_SHA256 = "6253f4e631fea8f0ab8cec3c369311dee3a824736f82cf8817a0f2a6de5528d3"
AR_NATIVE_FIXTURE_SHA256 = "e79368d7c01de089c510ec0c3bc1afb2dec846335fd9bb87fe9d050b52ccd906"
AR_NATIVE_RECEIPT_SHA256 = "ed703c3bc26743f73d8aef9f5b9fc42335a25ade73d0b8a2fb74c2bf1023ed37"
# AS has its own native checker and controls, even while the initial pinned
# implementation is byte-identical to AR's unchanged native gate.
AS_NATIVE_CHECKER_SHA256 = "a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f"
AS_NATIVE_CONTROLS_SHA256 = "6253f4e631fea8f0ab8cec3c369311dee3a824736f82cf8817a0f2a6de5528d3"
AS_NATIVE_FIXTURE_SHA256 = "e79368d7c01de089c510ec0c3bc1afb2dec846335fd9bb87fe9d050b52ccd906"
AS_NATIVE_RECEIPT_SHA256 = "ed703c3bc26743f73d8aef9f5b9fc42335a25ade73d0b8a2fb74c2bf1023ed37"
AR_COMPILED_ARTIFACT_SHA256 = {
    "public_records.rs": "41eeb72bd7a4e3b59f042311508a436033c1a01e227035beb955bec749a8169b",
    "cargo-run-build-fingerprint.json": "4e2778ada2cfb52797a0801457ff2a9a8c50798f8141b377afab79590f69289f",
    "cargo-build-output.txt": "0cab1e3220b72008729103a994de3b490da5f9fd8e7148efd9f8dd5f9f508344",
    "cargo-root-output.txt": "2db6dc53f9aeb9dfd3f0fbae320fe27fec5f3effaec62db3a0d382f4fc4734d3",
}

AR_POSITIVE_SHA256 = "b30f0128f9b6a2ec935359930af4c8f835d24ab1c031f3618f5829f5f504c113"
AR_PHYSICAL_SHA256 = "b4890cf7e9df7e008770264d7f978fdbd32522684d5c93f9e50bc2c9dcd7730f"
AR_VIEW_POINTER_SHA256 = "11560870cfe83e8d8ace30ebe27c6efa0c618d1368a749f4ace773b89961c6dd"
AR_MAPPING_SHA256 = "40f7fc525fffc73ce49664c54e207b53567474ded0b30ba9e6d5e9678b60df75"

AS_SOURCE_PINS = {
    "Cargo.toml": "da8c39007a61878597d9fdb1bd3b4a5eecd2ba1136af03b1a33cab81e91b4414",
    "Cargo.lock": "dd77bb3b5a3b9f712ac20c7b2ae2d664fc95ca3bd6b20f67b95f9174c4236d32",
    "build.rs": "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
    "extract_public.py": "748810dd6d4961c1e729864a83daa3aff771c72e7ef678e9bb01f2cfef44e99b",
    "elaborate.py": "5dca5f473d75a52a0f869903a306daa12425566e2ed2681c43ada93b0c01feca",
    "run-proof.sh": "50f0b025559c8a99edbb871be94537d2525575494bd9af158dcecbb6b6360f67",
}
AS_EXPECTED_SUPPORT_PATHS = {
    "src/promotion.rs", "src/physical_projection.rs", "src/view_pointer.rs",
}

VIEW_OLD = "#[logic(prophetic)] fn view_valid(self)->bool {pearlite! {match self.original_shared.inner_logic() {"
VIEW_NEW = "#[logic(prophetic)] fn view_valid(self)->bool {pearlite! {!self.ptr.is_null_logic() && match self.original_shared.inner_logic() {"
EMPTY_OLD = """#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer == bound.inner_logic().raw_pointer() as *const u8)]
#[ensures(result@.len() == 0)]
pub unsafe fn borrow_empty"""
EMPTY_NEW = EMPTY_OLD.replace("#[trusted]", "#[trusted]\n#[requires(!pointer.is_null_logic())]", 1)
WRAP_OLD = """#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
#[requires(match bound.inner_logic()@ {
    Some((_,cap,offset))=>offset+count@<=cap,None=>count==0usize})]"""
WRAP_NEW = WRAP_OLD.replace("#[trusted]", "#[trusted]\n#[requires(!pointer.is_null_logic())]", 1)
EXPECTED_TRANSFORMS = [
    ("generated/positive.rs", "src/promotion.rs", AR_POSITIVE_SHA256, VIEW_OLD, VIEW_NEW),
    ("src/physical_projection.rs", "src/physical_projection.rs", AR_PHYSICAL_SHA256, EMPTY_OLD, EMPTY_NEW),
    ("src/view_pointer.rs", "src/view_pointer.rs", AR_VIEW_POINTER_SHA256, WRAP_OLD, WRAP_NEW),
]


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


def _check_ar_archive() -> dict[str, Any]:
    evidence = AR_ROOT / "evidence"
    audit_json_path = evidence / "AR_CANONICAL_AUDIT.json"
    audit_md_path = evidence / "AR_CANONICAL_AUDIT.md"
    archive_path = evidence / "ar-positive-canonical-v2.tar.gz"
    receipt_path = evidence / "ar-positive-canonical-v2.json"
    for path, expected, label in (
        (audit_json_path, AR_AUDIT_JSON_SHA256, "AR canonical audit JSON"),
        (audit_md_path, AR_AUDIT_MD_SHA256, "AR canonical audit report"),
        (archive_path, AR_ARCHIVE_SHA256, "AR canonical archive"),
        (receipt_path, AR_RECEIPT_SHA256, "AR canonical archive receipt"),
        (AR_ROOT / "check_correspondence.py", AR_CHECKER_SHA256, "published AR checker"),
    ):
        require(sha(_read_regular(path, f"{label} missing or redirected")) == expected,
                f"{label} hash differs from published AR v2")
    audit = json.loads(audit_json_path.read_text())
    receipt = json.loads(receipt_path.read_text())
    require(audit.get("schema") == "ar-canonical-independent-audit-v1" and
            audit.get("result") == "audited_pass_selected_gate" and
            audit.get("full_original_admitted") is False and
            audit.get("archive", {}).get("sha256") == AR_ARCHIVE_SHA256 and
            audit.get("archive", {}).get("members") == AR_ARCHIVE_MEMBERS and
            audit.get("archive", {}).get("targets") == AR_TARGETS and
            audit.get("archive", {}).get("prover_leaves") == AR_PROVER_LEAVES and
            audit.get("archive", {}).get("null_leaves") == 0 and
            audit.get("archive", {}).get("structural_leaves") == 0 and
            audit.get("archive", {}).get("target_policy", {}).get("excluded") == {} and
            audit.get("archive", {}).get("target_policy", {}).get("features") == [] and
            audit.get("archive", {}).get("target_policy", {}).get("correspondence_exit_status") == 0,
            "AR v2 canonical audit is not the exact selected no-exclusion proof receipt")
    stats = receipt.get("statistics", {})
    policy = receipt.get("target_policy", {})
    rows = receipt.get("members")
    require(receipt.get("archive") == archive_path.name and receipt.get("archive_sha256") == AR_ARCHIVE_SHA256 and
            receipt.get("status") == "proved" and stats == {"files":AR_TARGETS,"prover":AR_PROVER_LEAVES,"null":0,"structural":0} and
            len(receipt.get("targets", [])) == AR_TARGETS and policy.get("excluded") == {} and
            policy.get("features") == [] and policy.get("diagnostic") is False and
            policy.get("correspondence_exit_status") == 0 and isinstance(rows, list) and
            len(rows) == AR_ARCHIVE_MEMBERS,
            "AR v2 receipt does not contain the full 150-target positive proof inventory")
    row_hashes = {row.get("path"): row.get("sha256") for row in rows if isinstance(row, dict)}
    require(len(row_hashes) == AR_ARCHIVE_MEMBERS and None not in row_hashes and
            all(isinstance(name, str) and isinstance(value, str) for name, value in row_hashes.items()),
            "AR v2 archive member table is duplicated or malformed")
    current_probe = 0
    current_production = 0
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        require(len(members) == AR_ARCHIVE_MEMBERS and len(set(names)) == AR_ARCHIVE_MEMBERS and
                set(names) == set(row_hashes) and all(member.isfile() for member in members),
                "AR v2 archive member names/types differ from its complete receipt")
        for member in members:
            require(not pathlib.PurePosixPath(member.name).is_absolute() and
                    ".." not in pathlib.PurePosixPath(member.name).parts,
                    f"unsafe path in AR v2 archive: {member.name}")
            handle = archive.extractfile(member)
            require(handle is not None, f"cannot read AR v2 member {member.name}")
            content = handle.read()
            require(sha(content) == row_hashes[member.name], f"AR v2 member hash mismatch: {member.name}")
            # AR README/TCB were updated as publication documentation after the
            # immutable archive; current checker/source/Cargo inputs remain
            # compared at their exact original paths.
            if member.name.startswith("probe/"):
                relative = pathlib.PurePosixPath(member.name[len("probe/"):])
                if relative.parts and relative.parts[0] == "evidence":
                    continue
                if relative.as_posix() in {"README.md", "TCB.md"}:
                    continue
                selected = AR_ROOT.joinpath(*relative.parts)
                require(selected.is_file() and not selected.is_symlink() and selected.read_bytes() == content,
                        f"AR source/checker/member differs from canonical v2 archive: {relative}")
                current_probe += 1
            elif member.name.startswith("inputs/repository/bytes/1.11.1/"):
                rel = pathlib.PurePosixPath(member.name[len("inputs/repository/bytes/1.11.1/"):])
                selected = CRATE_ROOT.joinpath(*rel.parts)
                require(selected.is_file() and not selected.is_symlink() and selected.read_bytes() == content,
                        f"AR captured repository source differs from current source: {rel}")
                current_production += 1
    return {"archive_sha256":AR_ARCHIVE_SHA256,"member_count":AR_ARCHIVE_MEMBERS,
        "proof_targets":AR_TARGETS,"prover_leaves":AR_PROVER_LEAVES,"null":0,"structural":0,
        "unique_regular_members":True,"all_member_hashes_verified":True,
        "current_AR_probe_files_compared":current_probe,
        "current_production_files_compared":current_production,
        "AR_documentation_current_files_excluded_from_archive_equality":["README.md","TCB.md"]}


def _import_ar_checker() -> Any:
    path = AR_ROOT / "check_correspondence.py"
    require(sha(_read_regular(path, "published AR checker is absent")) == AR_CHECKER_SHA256,
            "published AR checker changed before import")
    spec = importlib.util.spec_from_file_location("as_pinned_ar_correspondence", path)
    require(spec is not None and spec.loader is not None, "cannot load pinned AR checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _replay_ar_without_live_target(ar: Any) -> dict[str, Any]:
    """Recheck published AR source/native lineage and its captured Cargo inputs."""
    for relative, expected in ar.AR_SOURCE_PINS.items():
        actual = _read_regular(AR_ROOT / relative, f"AR selected source missing: {relative}")
        require(sha(actual) == expected, f"AR selected source changed: {relative}")
    aq, lineage = ar.preflight_published_aq()
    native = ar.audit_native_capture()
    mapping = json.loads((AR_ROOT / "generated/mapping.json").read_text())
    prefix = (AR_ROOT / "src/promotion.rs").read_text()
    extension = (AR_ROOT / "src/cursor_extension.rs").read_text()
    pointer = (AR_ROOT / "src/cursor_pointer.rs").read_text()
    client = (AR_ROOT / "generated/elaborated-client.rs").read_text()
    active = (AR_ROOT / "generated/active.rs").read_text()
    positive = (AR_ROOT / "generated/positive.rs").read_text()
    aq_positive = (ar.AQ_ROOT / "generated/positive.rs").read_text()
    composition = ar.assert_vacant_enum_transformation(aq, aq_positive, prefix, mapping)
    route = ar.assert_lib_route(aq, (AR_ROOT / "src/lib.rs").read_text())
    support = ar.assert_ar_support_closure(aq, (AR_ROOT / "src/lib.rs").read_text())
    composed = ar.assert_ar_composition(aq, mapping, prefix, extension, client, active, positive, pointer)
    surface = ar.assert_ar_shadow_surface(aq, extension, pointer, client, mapping)
    native_join = ar.assert_native_mapping(mapping, native)
    compiled = ar.assert_ar_captured_build(aq)
    ar.assert_probe_inputs()
    native_controls = assert_ar_native_controls()
    return {"published_commit":AR_COMMIT,"canonical_source_commit":AR_SOURCE_COMMIT,
        "canonical_v2":True,"AQ_lineage":lineage,"AR_enum_prefix":composition,
        "AR_lib_route":route,"AR_support_closure":support,"AR_composition":composed,
        "AR_API_surface":surface,"AR_native_mapping":native_join,
        "AR_compiled_inputs":compiled,"AR_native_controls":native_controls,"AR_selected_native_count":30,
        "AR_selected_production_native_count":29}


def assert_ar_native_controls() -> dict[str, Any]:
    checker = AR_ROOT / "native-check-controls.py"
    fixture = AR_ROOT / "fixtures/native-check-controls.json"
    receipt_path = AR_ROOT / "generated/native-check-controls.json"
    for path, digest, label in (
        (checker, AR_NATIVE_CONTROLS_SHA256, "AR native control checker"),
        (fixture, AR_NATIVE_FIXTURE_SHA256, "AR native control fixture"),
        (receipt_path, AR_NATIVE_RECEIPT_SHA256, "AR native control receipt"),
    ):
        require(sha(_read_regular(path, f"{label} missing")) == digest,
                f"{label} differs from the published 76-control replay")
    data = json.loads(receipt_path.read_text())
    rows = data.get("controls")
    require(data.get("schema") == "ar-native-cursor-source-mir-controls-v1" and
            data.get("checker_sha256") == AR_NATIVE_CHECKER_SHA256 and
            data.get("control_count") == 76 and data.get("rejected_as_expected") == 76 and
            data.get("accepted") == [] and data.get("checker_errors") == [] and
            isinstance(rows, list) and len(rows) == 76 and
            all(row.get("status") == "rejected_as_expected" for row in rows),
            "AR native control receipt is not a complete 76/76 zero-error replay")
    return {"control_count":76,"rejected_as_expected":76,"accepted":0,
        "checker_errors":0,"checker_fixture_receipt_pinned":True}


def _apply_one(before: bytes, old: str, new: str, label: str) -> bytes:
    try:
        source = before.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise CheckError(f"{label} is not UTF-8 source") from exc
    require(source.count(old) == 1, f"{label} transformation anchor is missing or duplicated")
    return source.replace(old, new, 1).encode()


def assert_as_transformations(ar: Any,
                              src_tree_override: dict[str, bytes] | None = None,
                              mapping_override: dict[str, Any] | None = None,
                              active_override: bytes | None = None,
                              positive_override: bytes | None = None) -> dict[str, Any]:
    """Derive each accepted source file from canonical AR bytes and exact literals."""
    ar_mapping_bytes = _read_regular(AR_ROOT / "generated/mapping.json", "AR mapping missing")
    require(sha(ar_mapping_bytes) == AR_MAPPING_SHA256, "published AR native/source mapping changed")
    ar_mapping = json.loads(ar_mapping_bytes)
    src_tree = src_tree_override if src_tree_override is not None else _regular_tree(ROOT / "src", "AS")
    mapping = mapping_override if mapping_override is not None else json.loads(_read_regular(MAPPING, "AS mapping missing"))
    active = active_override if active_override is not None else _read_regular(ACTIVE, "AS active source missing")
    positive = positive_override if positive_override is not None else _read_regular(POSITIVE, "AS positive source missing")

    ancestor_src = _regular_tree(AR_ROOT / "src", "published AR")
    require(set(src_tree) == set(ancestor_src), "AS source inventory differs from complete published AR source tree")
    expected_outputs: dict[str, bytes] = {}
    rows: list[dict[str, Any]] = []
    for ancestor_literal, output, expected_ancestor_hash, old, new in EXPECTED_TRANSFORMS:
        ancestor_path = (AR_ROOT / ancestor_literal).resolve()
        expected_path = (AR_ROOT / ancestor_literal).resolve()
        require(ancestor_path == expected_path and ancestor_path.is_file() and not ancestor_path.is_symlink(),
                f"AS transform ancestor route is invalid: {ancestor_literal}")
        before = ancestor_path.read_bytes()
        require(sha(before) == expected_ancestor_hash, f"published AR transform ancestor changed: {ancestor_literal}")
        after = _apply_one(before, old, new, output)
        expected_outputs[output] = after
        rows.append({"ancestor_source":"../" + AR_ROOT.name + "/" + ancestor_literal,
            "ancestor_sha256":expected_ancestor_hash,"output":output,"old":old,"new":new,
            "replacements":1,"transformed_sha256":sha(after),"native_bodies_unchanged":True})
        require(src_tree.get(output.removeprefix("src/")) == after,
                f"AS source differs from the exact reviewed transformation: {output}")
    for relative, before in ancestor_src.items():
        output = "src/" + relative
        if output in expected_outputs:
            continue
        require(src_tree[relative] == before,
                f"AS changed an inherited AR module outside the three selected contracts: {relative}")

    expected_inventory = {"src/" + name:sha(data) for name, data in sorted(src_tree.items())}
    require(mapping.get("ancestor_commit") == AR_COMMIT and
            mapping.get("ancestor_mapping") == "../" + AR_ROOT.name + "/generated/mapping.json" and
            mapping.get("ancestor_mapping_sha256") == AR_MAPPING_SHA256 and
            mapping.get("source_transformations") == rows and
            mapping.get("base_source") == "src/promotion.rs" and
            mapping.get("base_source_sha256") == sha(expected_outputs["src/promotion.rs"]) and
            mapping.get("active") == "generated/active.rs" and
            mapping.get("active_sha256") == sha(active) and
            mapping.get("diagnostic_added_targets") == 0 and
            mapping.get("support_inventory") == expected_inventory,
            "AS mapping does not record exactly the three independently reconstructed source changes")
    require(active == expected_outputs["src/promotion.rs"] and positive == active and
            src_tree["promotion.rs"] == active,
            "AS active/positive source is not exactly the transformed complete AR positive input")

    # No other mapping claim may drift from AR.  The fields below are the only
    # deliberate AS-specific additions/changes; the actual client/MIR facts,
    # state/fold descriptions and normal-edge metadata remain inherited.
    ignored = {"source_transform", "selected_prefix_sha256", "extension_source", "extension_sha256",
        "terminal_helpers_sha256", "client_sha256", "cursor_pointer_support"}
    changed = {"ancestor_commit", "ancestor_mapping", "ancestor_mapping_sha256", "source_transformations",
        "base_source_sha256", "active_sha256", "diagnostic_added_targets", "support_inventory"}
    inherited_expected = {k:v for k,v in ar_mapping.items() if k not in ignored | changed}
    inherited_actual = {k:v for k,v in mapping.items() if k not in changed}
    require(inherited_actual == inherited_expected,
            "AS changed inherited AR client, helper, native mapping, frame or scope metadata")
    return {"ancestor":"published AR canonical-v2 positive source",
        "exact_transform_count":3,"transformed_files":sorted(expected_outputs),
        "all_other_AR_source_files_byte_exact":True,"complete_AR_positive_prefix":True,
        "native_bodies_unchanged_by_transform_metadata":True,
        "active_sha256":sha(active),"mapping_fields_inherited_except_explicit_transform_metadata":True}


def assert_probe_manifest(environment: dict[str, str] | None = None, *,
                          manifest_text: str | None = None, lock_bytes: bytes | None = None,
                          build_bytes: bytes | None = None, extractor_bytes: bytes | None = None,
                          generator_bytes: bytes | None = None, launcher_bytes: bytes | None = None) -> dict[str, Any]:
    expected_features = {"negative_missing_acquire": [], "negative_missing_payload_free": [],
        "negative_missing_control_free": []}
    expected = {"package":{"name":PACKAGE,"version":"0.1.0","edition":"2021","publish":False},
        "dependencies":{"creusot-std":"=0.13.0"},"workspace":{},"features":expected_features}
    try:
        manifest_source = (ROOT / "Cargo.toml").read_text() if manifest_text is None else manifest_text
        manifest = tomllib.loads(manifest_source)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AS Cargo manifest could not be parsed: {exc}") from exc
    source_bytes = {
        "Cargo.toml": manifest_source.encode() if manifest_text is not None else (ROOT / "Cargo.toml").read_bytes(),
        "Cargo.lock": lock_bytes if lock_bytes is not None else (ROOT / "Cargo.lock").read_bytes(),
        "build.rs": build_bytes if build_bytes is not None else (ROOT / "build.rs").read_bytes(),
        "extract_public.py": extractor_bytes if extractor_bytes is not None else (ROOT / "extract_public.py").read_bytes(),
        "elaborate.py": generator_bytes if generator_bytes is not None else (ROOT / "elaborate.py").read_bytes(),
        "run-proof.sh": launcher_bytes if launcher_bytes is not None else (ROOT / "run-proof.sh").read_bytes(),
    }
    require(manifest == expected and all(sha(source_bytes[name]) == AS_SOURCE_PINS[name] for name in AS_SOURCE_PINS),
            "AS Cargo package, feature/build route, generator or lockfile changed")
    env = dict(os.environ) if environment is None else environment
    require(not env.get("BYTES_DROP_FEATURE") and not env.get("BYTES_SCOPE_SOURCE_CONTROL") and
            env.get("BYTES_SCOPE_DIAGNOSTIC", "0") != "1" and
            env.get("BYTES_DROP_CHECKER_SKIP", "0") != "1" and
            env.get("BYTES_TRANSLATE_ONLY", "0") != "1",
            "AS checker cannot admit a feature, source-control, diagnostic or translation-only run")
    return {"manifest_exact":True,"lock_and_build_route_pinned":True,
        "features":[],"diagnostic_or_feature_environment":False}


def _import_native() -> Any:
    path = ROOT / "check_native.py"
    require(sha(_read_regular(path, "AS native checker missing")) == AS_NATIVE_CHECKER_SHA256,
            "AS native checker differs from its separately pinned selected MIR gate")
    spec = importlib.util.spec_from_file_location("as_pinned_native_checker", path)
    require(spec is not None and spec.loader is not None, "cannot load pinned AS native checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def assert_as_native_controls() -> dict[str, Any]:
    paths = {
        "checker":ROOT / "native-check-controls.py",
        "fixture":ROOT / "fixtures/native-check-controls.json",
        "receipt":ROOT / "generated/native-check-controls.json",
    }
    expected = {"checker":AS_NATIVE_CONTROLS_SHA256,"fixture":AS_NATIVE_FIXTURE_SHA256,
        "receipt":AS_NATIVE_RECEIPT_SHA256}
    for label, path in paths.items():
        require(sha(_read_regular(path, f"AS native {label} missing or redirected")) == expected[label],
            f"AS native {label} is not the pinned 76-control selected native gate")
    receipt = json.loads(paths["receipt"].read_text())
    rows = receipt.get("controls")
    require(receipt.get("control_count") == 76 and receipt.get("rejected_as_expected") == 76 and
        receipt.get("accepted") == [] and receipt.get("checker_errors") == [] and
        isinstance(rows, list) and len(rows) == 76 and
        all(row.get("status") == "rejected_as_expected" for row in rows),
        "AS native-control receipt is not a complete 76/76 zero-error replay")
    return {"control_count":76,"rejected_as_expected":76,"accepted":0,"checker_errors":0,
        "checker_fixture_receipt_pinned":True}


def audit_native(mapping: dict[str, Any]) -> dict[str, Any]:
    native_path = ROOT / "native.rs"
    require(sha(_read_regular(native_path, "AS native witness missing")) ==
            "b0b2519d79479374eb31ab954180f11bd1030931d5057627c95e3e45a5b5c371",
            "AS native witness body differs from the published AR witness")
    module = _import_native()
    bundle = module.load_bundle()
    report = module.audit_bundle(bundle)
    require(report.get("status") == "pass" and
            sha((ROOT / "native-mir/capture.json").read_bytes()) == "bd5d5254c0329d29b906fa02c07528e91be383334929fb455cb00d5194e814ad" and
            mapping.get("native_mir_ready") is True and mapping.get("native_source_sha256") == sha(native_path.read_bytes()),
            "AS inherited native source/MIR correspondence failed")
    ar_mapping = json.loads((AR_ROOT / "generated/mapping.json").read_text())
    for key in ("native_alpha_renaming","helpers","callbacks","cursor_methods","ownership_frame","fold",
        "return_evaluation","excluded","native_source","native_source_sha256","native_client_mir",
        "native_mir_ready","debug_places","normal_edges","mir_blocks","mir"):
        require(mapping.get(key) == ar_mapping.get(key), f"AS native/client mapping changed inherited AR fact: {key}")
    native_controls = assert_as_native_controls()
    return {"status":"pass","selected_mir_count":30,"production_mir_count":29,
        "native_client_and_all_selected_bodies_byte_pinned_to_AR":True,
        "native_report":report,"AS_native_controls":native_controls}


def _expected_public_records(ar: Any) -> tuple[bytes, dict[str, str]]:
    paths = {
        "../../../src/bytes.rs": CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": ROOT / "extract_public.py",
    }
    hashes = {}
    for literal, path in paths.items():
        resolved = (ROOT / literal).resolve()
        require(not path.is_symlink() and path.is_file() and resolved == path.resolve(),
                f"AS build rerun input escapes its selected source route: {literal}")
        hashes[literal] = sha(resolved.read_bytes())
    records = paths["../../../src/bytes/bytes_record.rs"].read_text()
    vtables = paths["../../../src/bytes/vtable_record.rs"].read_text()
    mutable = paths["../../../src/bytes_mut.rs"].read_text()
    bytes_source = paths["../../../src/bytes.rs"].read_text()
    aq, _lineage = ar.preflight_published_aq()
    ai = aq.AP.AO.AN.AI
    ai.audit_generated_extractions(bytes_source, ROOT / "generated", records, vtables, mutable)
    shared = ai.extract_struct_source(mutable, "struct Shared {", "BytesMut::Shared")
    bytesmut = ai.extract_struct_source(mutable, "pub struct BytesMut {", "BytesMut")
    expected = (records + "\n" + vtables + "\nmod mutable_record {\n"
        "use alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        shared + "\n" + bytesmut + "\n}\nuse mutable_record::BytesMut;\n").encode()
    return expected, hashes


def assert_compiled_capture(receipt: dict[str, Any], artifacts: dict[str, bytes],
                            stored_receipt: bytes, expected_record: bytes,
                            source_map_bytes: bytes, input_hashes: dict[str, str]) -> dict[str, Any]:
    require(set(artifacts) == set(ARTIFACT_FIELDS), "AS compiled capture must contain exactly four Cargo artifacts")
    require(json.loads(stored_receipt) == receipt and receipt.get("schema") == "as-compiled-public-records-v1" and
            receipt.get("status") == "pass" and receipt.get("probe_package") == PACKAGE and
            receipt.get("build_script_sha256") == AS_SOURCE_PINS["build.rs"] and
            receipt.get("extractor_sha256") == AS_SOURCE_PINS["extract_public.py"],
            "AS compiled Cargo receipt has wrong package/build/extractor identity")
    for name, field in ARTIFACT_FIELDS.items():
        require(receipt.get(field) == sha(artifacts[name]), f"AS captured Cargo artifact changed: {name}")
    fingerprint = json.loads(artifacts["cargo-run-build-fingerprint.json"])
    source_map = json.loads(source_map_bytes)
    rerun = [x["RerunIfChanged"] for x in fingerprint.get("local", []) if isinstance(x, dict) and
        isinstance(x.get("RerunIfChanged"), dict)]
    require(len(rerun) == 1 and rerun[0].get("paths") == RERUN_PATHS,
            "AS captured fingerprint does not bind exact production/extractor inputs")
    target = pathlib.Path(receipt.get("cargo_target_dir", ""))
    output_rel = pathlib.Path(str(rerun[0].get("output", "")))
    output = pathlib.Path(receipt.get("build_output_path", ""))
    fingerprint_path = pathlib.Path(receipt.get("cargo_build_fingerprint", ""))
    require(target.resolve() == TARGET_DIR and receipt.get("cargo_target_dir") == str(TARGET_DIR) and
            not output_rel.is_absolute() and ".." not in output_rel.parts and
            output_rel.parts[:2] == ("debug","build") and (target / output_rel).resolve() == output.resolve() and
            receipt.get("build_output_path") == str(output.resolve()) and
            receipt.get("cargo_build_fingerprint") == str(fingerprint_path.resolve()) and
            output.name == "output" and fingerprint_path.name == "run-build-script-build-script-build.json" and
            output.parent.name == fingerprint_path.parent.name and
            output.parent.name.startswith(PACKAGE + "-") and
            fingerprint_path.parent.parent.resolve() == (TARGET_DIR / "debug/.fingerprint").resolve() and
            output.parent.parent.resolve() == (TARGET_DIR / "debug/build").resolve(),
            "AS captured Cargo fingerprint, package, target and output paths do not join")
    directives = ["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate"] + ["cargo:rerun-if-changed=" + x for x in RERUN_PATHS]
    require(artifacts["cargo-build-output.txt"] == ("\n".join(directives) + "\n").encode(),
            "AS captured build output differs from exact build-script directives")
    actual_out = pathlib.Path(receipt.get("actual_out_dir", ""))
    compiled_path = pathlib.Path(receipt.get("compiled_input_path", ""))
    root_output = pathlib.Path(receipt.get("root_output_path", ""))
    expected_out = output.resolve().parent / "out"
    require(actual_out.resolve() == expected_out and compiled_path.name == "public_records.rs" and
            compiled_path.resolve() == expected_out / "public_records.rs" and
            compiled_path.parent.resolve() == expected_out and
            root_output.resolve() == expected_out.parent / "root-output" and
            artifacts["cargo-root-output.txt"] == str(expected_out).encode(),
            "AS captured Cargo root-output/OUT_DIR/public_records paths do not resolve consistently")
    require(artifacts["public_records.rs"] == expected_record and
            receipt.get("compiled_input_sha256") == sha(expected_record) and
            receipt.get("reconstructed_generated_sha256") == sha(expected_record) and
            receipt.get("source_map_sha256") == sha(source_map_bytes) and
            source_map.get("generated/public_records.rs", {}).get("sha256") == sha(expected_record) and
            receipt.get("production_rerun_input_sha256") == input_hashes and
            receipt.get("captured_actual_Cargo_artifact_count") == 4,
            "AS compiled OUT_DIR record does not reconstruct from pinned production sources")
    return {"status":"pass","captured_artifact_count":4,"fingerprint_inputs_exact":True,
        "build_output_exact":True,"OUT_DIR_root_output_join_exact":True,
        "public_records_reconstructed":True}


def _read_captured_artifacts() -> tuple[dict[str, bytes], bytes]:
    require(CAPTURE_DIR.is_dir() and not CAPTURE_DIR.is_symlink(), "AS compiled-input capture directory is missing")
    expected_names = set(ARTIFACT_FIELDS) | {"public-records-build-receipt.json"}
    actual_names = {p.name for p in CAPTURE_DIR.iterdir()}
    require(actual_names == expected_names and all((CAPTURE_DIR / n).is_file() and not (CAPTURE_DIR / n).is_symlink()
        for n in expected_names), "AS compiled capture has missing, extra or redirected artifacts")
    return ({name:(CAPTURE_DIR / name).read_bytes() for name in ARTIFACT_FIELDS},
        (CAPTURE_DIR / "public-records-build-receipt.json").read_bytes())


def assert_compiled_capture_only(ar: Any) -> dict[str, Any]:
    expected, input_hashes = _expected_public_records(ar)
    artifacts, receipt_bytes = _read_captured_artifacts()
    receipt = json.loads(receipt_bytes)
    report = assert_compiled_capture(receipt, artifacts, receipt_bytes, expected,
        _read_regular(ROOT / "generated/source-map.json", "AS source map missing"), input_hashes)
    return {**report,"archive_capture_replayed_without_live_Cargo_target":True,
        "actual_path_joined_to_captured_Cargo_root_output":True}


def assert_live_build(ar: Any, capture: bool = False, require_capture: bool = True) -> dict[str, Any]:
    expected, input_hashes = _expected_public_records(ar)
    source_map_bytes = _read_regular(ROOT / "generated/source-map.json", "AS source map missing")
    require((ROOT / "generated/public_records.rs").read_bytes() == expected and
            json.loads(source_map_bytes).get("generated/public_records.rs", {}).get("sha256") == sha(expected),
            "AS extractor output differs from selected production sources")
    env_target = os.environ.get("CARGO_TARGET_DIR")
    require(not env_target or pathlib.Path(env_target).resolve() == TARGET_DIR,
            "AS run selected a different Cargo target directory")
    fingerprints = list((TARGET_DIR / "debug/.fingerprint").glob(PACKAGE + "-*/run-build-script-build-script-build.json"))
    require(len(fingerprints) == 1 and fingerprints[0].is_file() and not fingerprints[0].is_symlink(),
            "AS requires one selected feature-free Cargo build fingerprint")
    fp_path = fingerprints[0].resolve()
    fp_bytes = fp_path.read_bytes()
    try:
        fingerprint = json.loads(fp_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise CheckError(f"AS Cargo fingerprint is invalid: {exc}") from exc
    rerun = [x["RerunIfChanged"] for x in fingerprint.get("local", []) if isinstance(x, dict) and
        isinstance(x.get("RerunIfChanged"), dict)]
    require(len(rerun) == 1 and rerun[0].get("paths") == RERUN_PATHS,
            "AS actual Cargo fingerprint has unexpected source inputs")
    output_rel = pathlib.Path(str(rerun[0].get("output", "")))
    require(not output_rel.is_absolute() and ".." not in output_rel.parts and
            output_rel.parts[:2] == ("debug","build"), "AS Cargo output path escapes target")
    output = (TARGET_DIR / output_rel).resolve()
    require(output.is_file() and output.parent.name == fp_path.parent.name and
            fp_path.parent.parent.resolve() == (TARGET_DIR / "debug/.fingerprint").resolve() and
            output.parent.parent.resolve() == (TARGET_DIR / "debug/build").resolve(),
            "AS fingerprint does not resolve to its own Cargo build output")
    output_bytes = output.read_bytes()
    directives = ["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate"] + ["cargo:rerun-if-changed=" + x for x in RERUN_PATHS]
    require(output_bytes == ("\n".join(directives) + "\n").encode(),
            "AS live Cargo build directives differ from selected build script")
    out_dir = output.parent / "out"
    root_output = output.parent / "root-output"
    compiled = out_dir / "public_records.rs"
    require(root_output.is_file() and not root_output.is_symlink() and compiled.is_file() and
            not compiled.is_symlink() and root_output.read_bytes() == str(out_dir).encode() and
            compiled.read_bytes() == expected,
            "AS actual Cargo OUT_DIR/public_records differs from reconstructed record")
    artifacts = {"public_records.rs":compiled.read_bytes(),
        "cargo-run-build-fingerprint.json":fp_bytes,
        "cargo-build-output.txt":output_bytes,
        "cargo-root-output.txt":root_output.read_bytes()}
    receipt = {"schema":"as-compiled-public-records-v1","status":"pass","probe_package":PACKAGE,
        "build_script_sha256":sha((ROOT / "build.rs").read_bytes()),
        "extractor_sha256":sha((ROOT / "extract_public.py").read_bytes()),"cargo_target_dir":str(TARGET_DIR),
        "cargo_build_fingerprint":str(fp_path),"cargo_build_fingerprint_sha256":sha(fp_bytes),
        "build_output_path":str(output),"build_output_sha256":sha(output_bytes),
        "root_output_path":str(root_output),"root_output_sha256":sha(artifacts["cargo-root-output.txt"]),
        "actual_out_dir":str(out_dir),"compiled_input_path":str(compiled),
        "compiled_input_sha256":sha(expected),"reconstructed_generated_sha256":sha(expected),
        "source_map_sha256":sha(source_map_bytes),"production_rerun_input_sha256":input_hashes,
        "captured_actual_Cargo_artifact_count":4,
        "captured_input_path":"generated/compiled-inputs/public_records.rs",
        "captured_cargo_build_fingerprint_path":"generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_build_output_path":"generated/compiled-inputs/cargo-build-output.txt",
        "captured_root_output_path":"generated/compiled-inputs/cargo-root-output.txt",
        "captured_input_sha256":sha(expected),"captured_cargo_build_fingerprint_sha256":sha(fp_bytes),
        "captured_build_output_sha256":sha(output_bytes),"captured_root_output_sha256":sha(artifacts["cargo-root-output.txt"])}
    receipt_bytes = (json.dumps(receipt,indent=2)+"\n").encode()
    if capture:
        CAPTURE_DIR.mkdir(parents=True,exist_ok=True)
        for name,data in artifacts.items(): (CAPTURE_DIR / name).write_bytes(data)
        (CAPTURE_DIR / "public-records-build-receipt.json").write_bytes(receipt_bytes)
    if require_capture or capture:
        stored_artifacts, stored_receipt = _read_captured_artifacts()
        captured = assert_compiled_capture(json.loads(stored_receipt), stored_artifacts, stored_receipt,
            expected, source_map_bytes, input_hashes)
    else:
        captured = assert_compiled_capture(receipt, artifacts, receipt_bytes, expected, source_map_bytes, input_hashes)
    return {"status":"pass","captured_artifact_count":4,"live_fingerprint_and_output_checked":True,
        "snapshot_written":capture,"snapshot_matches_live_build":captured,
        "compiled_public_records_sha256":sha(expected),"receipt":receipt}


def audit() -> dict[str, Any]:
    archive_report = _check_ar_archive()
    ar = _import_ar_checker()
    ar_report = _replay_ar_without_live_target(ar)
    transformations = assert_as_transformations(ar)
    mapping = json.loads(MAPPING.read_text())
    native = audit_native(mapping)
    cargo_inputs = assert_probe_manifest()
    compiled = assert_live_build(ar, capture=False, require_capture=True)
    return {"status":"pass","checker_scope":"exact AS nonnull source delta over the published AR selected source gate",
        "full_original_admitted":False,"published_AR_canonical_ancestry":archive_report,
        "AR_source_gate_replay":ar_report,"AS_exact_transformations":transformations,
        "AS_native_correspondence":native,"AS_probe_build_inputs":cargo_inputs,
        "AS_compiled_production_input":compiled,
        "claims":{"native_byte_operations_changed":False,"borrow_empty_nonnull_precondition":True,
            "wrapping_bounded_nonnull_precondition":True,"complete_view_valid_global_nonnull":True,
            "proof_result_or_unwind_claimed":False}}


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
        require(args.shadow.resolve() == ACTIVE.resolve(), "shadow must select AS generated/active.rs")
        require(args.mapping.resolve() == MAPPING.resolve(), "mapping must select AS generated/mapping.json")
        archive = _check_ar_archive()
        ar = _import_ar_checker()
        ar_report = _replay_ar_without_live_target(ar)
        if args.capture_compiled_inputs:
            require(not args.ancestry_only and not args.audit_compiled_capture_only,
                    "capture cannot be combined with another mode")
            require(AS_SOURCE_PINS["build.rs"] == sha((ROOT / "build.rs").read_bytes()),
                    "AS build script changed before capture")
            cargo = assert_probe_manifest()
            transformations = assert_as_transformations(ar)
            report = {"status":"pass","checker_scope":"explicit AS four-artifact snapshot after positive translation",
                "published_AR_canonical_ancestry":archive,"AR_source_gate_replay":ar_report,
                "AS_exact_transformations":transformations,"AS_probe_build_inputs":cargo,
                "AS_compiled_production_input":assert_live_build(ar,capture=True,require_capture=False),
                "proof_result_not_claimed":True}
        elif args.audit_compiled_capture_only:
            require(not args.ancestry_only,"capture replay cannot be combined with ancestry-only mode")
            transformations = assert_as_transformations(ar)
            report = {"status":"pass","checker_scope":"AS archived Cargo inputs; no live Cargo target or proof claim",
                "published_AR_canonical_ancestry":archive,"AR_source_gate_replay":ar_report,
                "AS_exact_transformations":transformations,
                "AS_compiled_production_input":assert_compiled_capture_only(ar),"proof_result_not_claimed":True}
        elif args.ancestry_only:
            transformations = assert_as_transformations(ar)
            report = {"status":"pass","checker_scope":"AS transformation/AR ancestry only; no proof or Cargo claim",
                "published_AR_canonical_ancestry":archive,"AR_source_gate_replay":ar_report,
                "AS_exact_transformations":transformations}
        else:
            report = audit()
    except Exception as exc:
        report = {"status":"reject","checker_scope":"AS selected source/native/Cargo correspondence",
            "reason":f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(report,indent=2,ensure_ascii=False)+"\n"
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True)
        args.output.write_text(rendered)
    print(rendered,end="")
    return 0 if report.get("status")=="pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
