#!/usr/bin/env python3
"""AU closed named-call source/proof/native correspondence gate.

This gate replays the published AT source gate from its canonical archive,
checks that AU adds one separately body-proved named helper and caller, then
joins that summary to actual native MIR, the freshly translated Coma target,
and the completed proof snapshot. It admits neither general Clone/From nor
unwind, escaped owners, concurrency, or the complete original crate.
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
AT_ROOT = PROBES / "original-owned-view-clone-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
POSITIVE = ROOT / "generated/positive.rs"
MAPPING = ROOT / "generated/mapping.json"
CAPTURE_DIR = ROOT / "generated/compiled-inputs"
TARGET_DIR = pathlib.Path("/workspace/bytes-proof-tools/targets/bytes").resolve()
PACKAGE = "bytes-original-owned-view-call-summaries"
AT_PACKAGE = "bytes-original-owned-view-clone"
AU_EXTENSION = "call_summary_extension.rs"
AU_EXTENSION_REL = "src/call_summary_extension.rs"
AU_HELPER = "clone_suffix_checked"
AU_CLIENT = "owned_view_call_scope"
AU_HELPER_COMA = "verif/bytes_original_owned_view_call_summaries_rlib/promotion/clone_suffix_checked.coma"
AU_HELPER_PROOF = "verif/bytes_original_owned_view_call_summaries_rlib/promotion/clone_suffix_checked/proof.json"
AU_RERUN_PATHS = [
    "../../../src/bytes.rs",
    "../../../src/bytes/bytes_record.rs",
    "../../../src/bytes/vtable_record.rs",
    "../../../src/bytes_mut.rs",
    "extract_public.py",
]
AU_ARTIFACT_FIELDS = {
    "public_records.rs": "captured_input_sha256",
    "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
    "cargo-build-output.txt": "captured_build_output_sha256",
    "cargo-root-output.txt": "captured_root_output_sha256",
}
AU_FEATURES = {
    "negative_missing_acquire": [],
    "negative_missing_payload_free": [],
    "negative_missing_control_free": [],
}

# Published parent AT canonical inputs. These are independently replayed before
# importing the AT checker or relying on any of its parsed source.
AT_COMMIT = "e3a9b0f8003c1dd2eb5de55cd36695cc2e7f74e0"
AT_CHECKER_SHA256 = "54ccb9469212fcc5306ca564827f630069b5603c04a50d07fcb323f6ad102150"
AT_AUDIT_JSON_SHA256 = "838a22960752b425a1c9c1c8428961e79b9a20a93ecaeb57111bf2682960361c"
AT_AUDIT_MD_SHA256 = "f4c8b95fd8e4bdb2b9d0b3e9f96636a4430442f948407c55be378d7769bcfd9d"
AT_ARCHIVE_SHA256 = "958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d"
AT_RECEIPT_SHA256 = "78e9694d4e0150ee6a566c62b96c698b7117bfe4302af53bd78115c65b6fab4e"
AT_REPLAY_SHA256 = "cd782e03d1e5aacce72731db95ce8f8a2949d173e7ac53ed8ac5a0dec9688ca0"
AT_ARCHIVE_MEMBERS = 1544
AT_TARGETS = 155
AT_PROVER_LEAVES = 1417
AT_POSITIVE_SHA256 = "b5790f53182c689186442dc54428693a0c5144bb46febdb1a0824e8ccffee945"
AT_NATIVE_CHECKER_SHA256 = "57a47d7304777417b2514a0b8281983513c4a1d07612aa63a7a10ae80709bf92"
AT_NATIVE_CONTROLS_SHA256 = "43c253328eaf8f79e7e1e57b24d8b996334a5575e080301d676be5ae0045e068"
AT_NATIVE_FIXTURE_SHA256 = "f6dbec119aee5491f14ab73def971e4eaee5197b13515fb36229b158a5207099"
AT_NATIVE_RECEIPT_SHA256 = "9591a9416a2a19ec23d48235c8f9a7f4acffa03086777872b351f3166a6d636a"
AT_NATIVE_CONTROL_COUNT = 45
AT_MAIN_CONTROL_COUNT = 52
AT_BUILD_INPUT_HASHES = {
    "build.rs": "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
    "extract_public.py": "748810dd6d4961c1e729864a83daa3aff771c72e7ef678e9bb01f2cfef44e99b",
}

# AU inputs fixed after the positive body run. Mapping is parsed and checked
# independently below; these hashes supplement those semantic joins.
AU_GENERATOR_SHA256 = "346d5a2208dcee4e71f5e2646e8a19bff471c1dfc02264b40cde030fb52d2705"
AU_MAPPING_SHA256 = "07e58146a8881572db20d3d9d18d99246f4e6911c5277cd56a86e102d559ecd7"
AU_EXTENSION_SHA256 = "e596b537bdc5cf230eecf07686bad546434183d8b52fae8d2c880d4886815972"
AU_CLIENT_SHA256 = "dc0f3fad8fe071e5d64232efde3602ee693c490896be8e4f9be8907d5f32c8da"
AU_ACTIVE_SHA256 = "f3627094e2121b4b52d5d2c881adc6cd23d1f80ac8c33cdf0e21a0ed56ecc1e9"
AU_LAUNCHER_SHA256 = "e8c10a06efd398de18238b53baefa350bc3c73c66f5ceaa0c0fc2991d0f9bd36"
AU_MANIFEST_SHA256 = "b4467776d2ca4b48de7bb8ca5a3ba87979d201d410004d04df41d6e7233282e3"
AU_LOCK_SHA256 = "2139a17a90058982de4b3abfbc65dc7a4d846045bb02fe127fb5a4e636fbd1c1"
AU_NATIVE_SOURCE_SHA256 = "a9e889311f0a01fee178f4f9aa9ea1957cff59fb19221497b9b412fa8d8e66d9"
AU_NATIVE_CHECKER_SHA256 = "2d36f4b36b463d371f0f59fe4627e8b244729a7bdc89dd6b11b22107a2ab95c8"
AU_CAPTURE_SHA256 = "8587b8887f698ebb943d7e85d221a659698e4851eb1e536fe4b876bad1357d12"
AU_SELECTED_MIRS = 31
AU_PRODUCTION_MIRS = 29
AU_DIAGNOSTIC_ARCHIVE_SHA256 = "ffc00ffe50a408f971f2a56e1d58f7173d3d9856ebeb65e97c7a725d9e9393bd"
AU_DIAGNOSTIC_RECEIPT_SHA256 = "6a57cba12daee897e79a4214e8627ba7ecc933133545abd8e742737a3c7956ee"
AU_DIAGNOSTIC_MEMBERS = 1543
AU_DIAGNOSTIC_TARGETS = 157
AU_DIAGNOSTIC_PROVER_LEAVES = 1447
AU_SOURCE_CONTROL_EXCLUSIONS = {"README.md", "TCB.md"}


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
    stats = {"files": len(proofs), "prover": 0, "null": 0, "structural": 0}

    def visit(node: Any) -> None:
        if node is None:
            stats["null"] += 1
        elif isinstance(node, dict) and "children" in node:
            children = node["children"]
            if not children:
                stats["structural"] += 1
            for child in children:
                visit(child)
        elif isinstance(node, dict) and "prover" in node:
            stats["prover"] += 1
        else:
            raise CheckError(f"unknown proof tree node: {node!r}")

    for proof in proofs:
        coma = proof.get("proofs", {}).get("Coma")
        require(isinstance(coma, dict), "archived proof is missing its Coma proof tree")
        for node in coma.values():
            visit(node)
    return stats


def _read_archive(path: pathlib.Path, expected_sha: str,
                  expected_members: int, label: str) -> dict[str, bytes]:
    raw = _read_regular(path, f"{label} archive missing or redirected")
    require(sha(raw) == expected_sha, f"{label} archive hash differs from its frozen receipt")
    files: dict[str, bytes] = {}
    with tarfile.open(path, "r:gz") as tar:
        members = tar.getmembers()
        names = [m.name for m in members]
        require(len(members) == expected_members and len(set(names)) == expected_members and
                all(m.isfile() for m in members), f"{label} archive is not a unique regular-file archive")
        for member in members:
            pure = pathlib.PurePosixPath(member.name)
            require(not pure.is_absolute() and ".." not in pure.parts,
                    f"unsafe {label} member path: {member.name}")
            stream = tar.extractfile(member)
            require(stream is not None, f"cannot read {label} archive member {member.name}")
            files[member.name] = stream.read()
    return files


def _verify_target_manifest(receipt: dict[str, Any], files: dict[str, bytes],
                            target_count: int, prover_count: int,
                            label: str, *, admitted: bool) -> dict[str, Any]:
    targets = receipt.get("targets")
    policy = receipt.get("target_policy", {})
    require(receipt.get("status") == ("proved" if admitted else "diagnostic") and
            receipt.get("archive_sha256") == (AT_ARCHIVE_SHA256 if label == "AT" else AU_DIAGNOSTIC_ARCHIVE_SHA256) and
            isinstance(targets, list) and len(targets) == target_count,
            f"{label} proof receipt status/archive/target inventory changed")
    if admitted:
        require(policy.get("excluded") == {} and policy.get("features") == [] and
                policy.get("diagnostic") is False and policy.get("correspondence_exit_status") == 0,
                f"{label} published proof receipt is not a complete no-exclusion admission")
    else:
        require(policy.get("excluded") == {} and policy.get("features") == [] and
                policy.get("diagnostic") is True and policy.get("correspondence_exit_status") == 2,
                f"{label} origin must remain explicitly diagnostic and non-admitted")
    rows: dict[str, str] = {}
    proof_paths: set[str] = set()
    coma_paths: set[str] = set()
    proofs = []
    for row in targets:
        coma_path, proof_path = row.get("coma"), row.get("proof")
        require(isinstance(coma_path, str) and isinstance(proof_path, str) and
                coma_path.startswith("probe/verif/") and proof_path.startswith("probe/verif/") and
                coma_path in files and proof_path in files,
                f"{label} target references missing/unsafe archive members")
        require(row.get("coma_sha256") == sha(files[coma_path]) and
                row.get("proof_sha256") == sha(files[proof_path]),
                f"{label} proof target source/hash mismatch: {coma_path}")
        require(coma_path not in coma_paths and proof_path not in proof_paths,
                f"{label} target table duplicates a proof or Coma module")
        coma_paths.add(coma_path)
        proof_paths.add(proof_path)
        proofs.append(json.loads(files[proof_path]))
    archive_comas = {p for p in files if p.startswith("probe/verif/") and p.endswith(".coma")}
    archive_proofs = {p for p in files if p.startswith("probe/verif/") and p.endswith("/proof.json")}
    require(coma_paths == archive_comas and proof_paths == archive_proofs and
            _proof_stats(proofs) == {"files": target_count, "prover": prover_count,
                "null": 0, "structural": 0},
            f"{label} archive has missing/extra targets or null/structural proof leaves")
    return {"targets": target_count, "prover_leaves": prover_count,
            "null": 0, "structural": 0, "complete_target_pairs": True,
            "excluded": policy.get("excluded"), "features": policy.get("features"),
            "diagnostic": policy.get("diagnostic")}


def assert_at_canonical_archive() -> tuple[dict[str, Any], dict[str, bytes]]:
    evidence = AT_ROOT / "evidence"
    audit_path = evidence / "AT_CANONICAL_AUDIT.json"
    audit_md = evidence / "AT_CANONICAL_AUDIT.md"
    archive_path = evidence / "at-positive-canonical-v2.tar.gz"
    receipt_path = evidence / "at-positive-canonical-v2.json"
    replay_path = evidence / "at-positive-canonical-v2-audit.json"
    for path, digest, label in (
        (audit_path, AT_AUDIT_JSON_SHA256, "AT canonical audit JSON"),
        (audit_md, AT_AUDIT_MD_SHA256, "AT canonical audit Markdown"),
        (archive_path, AT_ARCHIVE_SHA256, "AT canonical archive"),
        (receipt_path, AT_RECEIPT_SHA256, "AT canonical receipt"),
        (replay_path, AT_REPLAY_SHA256, "AT independent replay"),
        (AT_ROOT / "check_correspondence.py", AT_CHECKER_SHA256, "AT correspondence checker"),
    ):
        require(sha(_read_regular(path, f"{label} missing or redirected")) == digest,
                f"{label} hash differs from publication")
    audit = json.loads(audit_path.read_text())
    receipt_bytes = receipt_path.read_bytes()
    receipt = json.loads(receipt_bytes)
    replay = json.loads(replay_path.read_text())
    integrity = audit.get("archive_integrity", {})
    require(audit.get("schema") == "at-canonical-independent-audit-v1" and
            audit.get("result") == "pass" and
            audit.get("archive_sha256") == AT_ARCHIVE_SHA256 and
            integrity.get("unique_safe_regular_members") is True and
            integrity.get("members") == AT_ARCHIVE_MEMBERS and
            integrity.get("every_member_hash_matches_manifest") is True and
            integrity.get("targets") == AT_TARGETS and
            integrity.get("all_coma_and_proof_hashes_match") is True and
            integrity.get("exclusions") == 0 and integrity.get("features") == [] and
            integrity.get("correspondence_exit_status") == 0 and
            integrity.get("diagnostic") is False and
            integrity.get("statistics") == {"files": AT_TARGETS, "prover": AT_PROVER_LEAVES,
                "null": 0, "structural": 0},
            "AT canonical audit does not record the published positive proof result")
    require(replay.get("status") == "proved" and
            replay.get("archive_sha256") == AT_ARCHIVE_SHA256 and
            replay.get("members_verified") == AT_ARCHIVE_MEMBERS and
            replay.get("statistics") == {"files": AT_TARGETS, "prover": AT_PROVER_LEAVES,
                "null": 0, "structural": 0},
            "AT independent replay does not verify its full proof archive")
    members = receipt.get("members")
    require(receipt.get("archive_sha256") == AT_ARCHIVE_SHA256 and
            receipt.get("status") == "proved" and
            receipt.get("statistics") == {"files": AT_TARGETS, "prover": AT_PROVER_LEAVES,
                "null": 0, "structural": 0} and
            isinstance(receipt.get("targets"), list) and len(receipt["targets"]) == AT_TARGETS and
            isinstance(members, list) and len(members) == AT_ARCHIVE_MEMBERS,
            "AT proof receipt does not bind its complete target/member tables")
    member_hashes = {row.get("path"): row.get("sha256") for row in members if isinstance(row, dict)}
    require(len(member_hashes) == AT_ARCHIVE_MEMBERS and None not in member_hashes,
            "AT member hash table is duplicated or malformed")
    files = _read_archive(archive_path, AT_ARCHIVE_SHA256, AT_ARCHIVE_MEMBERS, "AT")
    require(set(files) == set(member_hashes) and
            all(sha(data) == member_hashes[name] for name, data in files.items()),
            "AT archive member hashes differ from the published receipt")
    proof_result = _verify_target_manifest(receipt, files, AT_TARGETS, AT_PROVER_LEAVES,
        "AT", admitted=True)
    # Tie current source/Cargo/checker inputs to the canonical archive; later
    # documentation and additional historical evidence are outside this set.
    for member, data in files.items():
        if not member.startswith("probe/"):
            continue
        relative = pathlib.PurePosixPath(member[len("probe/"):])
        if relative.parts and relative.parts[0] == "evidence":
            continue
        if relative.as_posix() in {"README.md", "TCB.md"}:
            continue
        current = AT_ROOT.joinpath(*relative.parts)
        require(current.is_file() and not current.is_symlink() and current.read_bytes() == data,
                f"published AT source/proof input differs from canonical archive: {relative}")
    return ({"status": "pass", "archive_sha256": AT_ARCHIVE_SHA256,
        "members": AT_ARCHIVE_MEMBERS, **proof_result}, files)


def _import_at_checker() -> Any:
    path = AT_ROOT / "check_correspondence.py"
    require(sha(_read_regular(path, "published AT checker is missing")) == AT_CHECKER_SHA256,
            "published AT checker changed before import")
    spec = importlib.util.spec_from_file_location("au_pinned_at_checker", path)
    require(spec is not None and spec.loader is not None, "cannot load published AT checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def assert_published_at_ancestry(at_archive: dict[str, bytes]) -> dict[str, Any]:
    at_checker = _import_at_checker()
    as_archive_report, as_archive = at_checker.assert_as_canonical_archive()
    ancestry = at_checker.assert_published_as_ancestry(as_archive)
    at_inputs = at_checker._load_inputs()
    at_source = at_checker.assert_at_source_closure(as_archive, at_inputs["source_tree"])
    at_mapping = json.loads(at_inputs["mapping_bytes"])
    at_composition = at_checker.assert_at_composition(at_inputs["source_tree"], at_inputs["generator"],
        at_mapping, at_inputs["active"], at_inputs["positive"], at_inputs["extension_generated"],
        at_inputs["terminal_generated"], at_inputs["client"], ancestry["_aq_ar"], ancestry["_ar_checker"])
    at_manifest = at_checker.assert_probe_manifest(at_inputs["manifest"], at_inputs["lock"],
        at_inputs["build"], at_inputs["extractor"], at_inputs["generator"], at_inputs["launcher"],
        at_inputs["environment"])
    at_native = at_checker.assert_native(at_mapping)
    at_cargo = at_checker.assert_compiled_capture_only(ancestry["_as_checker"], ancestry["_ar_checker"],
        at_inputs["compiled_capture"])
    require(at_cargo.get("status") == "pass" and at_native.get("status") == "pass",
            "published AT captured native/Cargo correspondence did not replay")
    return {"status": "pass", "published_commit": AT_COMMIT,
        "canonical_archive": {"sha256": AT_ARCHIVE_SHA256, "members": AT_ARCHIVE_MEMBERS,
            "targets": AT_TARGETS, "prover_leaves": AT_PROVER_LEAVES},
        "canonical_archive_replayed": True,
        "published_AT_checker_sha256": AT_CHECKER_SHA256,
        "AT_source_gate": at_source, "AT_composition_gate": at_composition,
        "AT_native_gate": {"selected_mir_count": at_native.get("selected_mir_count"),
            "production_mir_count": at_native.get("production_mir_count"),
            "native_controls": at_native.get("native_controls")},
        "AT_manifest_gate": at_manifest, "AT_compiled_input_gate": at_cargo,
        "AS_AR_AQ_AP_ancestry_replayed_by_published_AT_checker": True,
        "live_Cargo_target_read": False,
        "_at_checker": at_checker, "_at_archive": at_archive,
        "_as_archive_report": as_archive_report, "_as_archive": as_archive,
        "_ancestry": ancestry}


def assert_au_source_closure(at_archive: dict[str, bytes], source_tree: dict[str, bytes]) -> dict[str, Any]:
    at_sources = {name[len("probe/src/"):]: data for name, data in at_archive.items()
                  if name.startswith("probe/src/") and name.endswith(".rs")}
    require(at_sources and "probe/generated/positive.rs" in at_archive,
            "published AT archive omits its complete source or positive composition")
    expected = set(at_sources) | {AU_EXTENSION}
    require(set(source_tree) == expected,
            f"AU source inventory is not complete-AT plus one summary extension: missing={sorted(expected-set(source_tree))}, extra={sorted(set(source_tree)-expected)}")
    for name, data in at_sources.items():
        if name == "promotion.rs":
            # The published AT proof input is generated/positive.rs. Its
            # checked-in source/promotion.rs is an older generator base and is
            # intentionally replaced by that proved positive composition.
            continue
        require(source_tree[name] == data,
                f"AU changed inherited AT Rust source: {name}")
    require(source_tree["promotion.rs"] == at_archive["probe/generated/positive.rs"] and
            sha(source_tree["promotion.rs"]) == AT_POSITIVE_SHA256,
            "AU is not the complete published AT positive source prefix")
    require(source_tree["lib.rs"] == at_sources["lib.rs"],
            "AU changed the inherited AT module route")
    return {"complete_AT_Rust_source_byte_exact": True,
        "AT_module_count": len(at_sources), "only_new_Rust_module": f"src/{AU_EXTENSION}",
        "published_AT_prefix_sha256": AT_POSITIVE_SHA256, "lib_route_byte_exact": True}


def _load_client_literal(generator_bytes: bytes) -> str:
    try:
        tree = ast.parse(generator_bytes.decode("utf-8"))
        values = []
        for node in tree.body:
            if isinstance(node, ast.Assign) and any(
                    isinstance(t, ast.Name) and t.id == "CLIENT" for t in node.targets):
                values.append(ast.literal_eval(node.value))
        require(len(values) == 1 and isinstance(values[0], str),
                "AU generator must contain exactly one literal native-compatible client")
        return values[0]
    except (UnicodeDecodeError, SyntaxError, ValueError) as exc:
        raise CheckError(f"AU client literal cannot be independently reconstructed: {exc}") from exc


def _split_rust_list(text: str) -> list[str]:
    """Split a closed Rust parameter/argument list at top-level commas."""
    out: list[str] = []
    start = 0
    round_depth = angle_depth = square_depth = brace_depth = 0
    for i, char in enumerate(text):
        if char == "(": round_depth += 1
        elif char == ")": round_depth -= 1
        elif char == "<": angle_depth += 1
        elif char == ">": angle_depth -= 1
        elif char == "[": square_depth += 1
        elif char == "]": square_depth -= 1
        elif char == "{": brace_depth += 1
        elif char == "}": brace_depth -= 1
        elif char == "," and not (round_depth or angle_depth or square_depth or brace_depth):
            piece = text[start:i].strip()
            if piece:
                out.append(piece)
            start = i + 1
        require(min(round_depth, angle_depth, square_depth, brace_depth) >= 0,
                "Rust signature/list delimiters are unbalanced")
    require(not (round_depth or angle_depth or square_depth or brace_depth),
            "Rust signature/list delimiters are unbalanced")
    piece = text[start:].strip()
    if piece:
        out.append(piece)
    return out


def _parse_rust_signature(signature: str) -> dict[str, Any]:
    match = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", signature)
    require(match is not None, "named-call signature lacks a Rust function declaration")
    open_paren = signature.find("(", match.start())
    depth = 0
    close_paren = None
    for i in range(open_paren, len(signature)):
        if signature[i] == "(": depth += 1
        elif signature[i] == ")":
            depth -= 1
            if depth == 0:
                close_paren = i
                break
    require(close_paren is not None, "named-call signature has an unterminated parameter list")
    tail = signature[close_paren + 1:].strip()
    require(tail.startswith("->"), "named-call summary requires an explicit return type")
    result_type = re.sub(r"\s+", "", tail[2:])
    require(result_type and not result_type.endswith("{"), "named-call result type is malformed")
    parameters = []
    for parameter in _split_rust_list(signature[open_paren + 1:close_paren]):
        pieces = _split_rust_list(parameter.replace(":", ",", 1))
        require(len(pieces) == 2, "named-call parameter must have one name and one type")
        name = re.sub(r"^mut\s+", "", pieces[0].strip())
        rust_type = re.sub(r"\s+", "", pieces[1])
        require(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name) is not None and rust_type,
                "named-call parameter name/type is malformed")
        parameters.append({"name": name, "type": rust_type})
    return {"name": match.group(1), "parameters": parameters, "result": result_type}


def _function_source_spans(source: str, function_name: str, *, sole_item: bool = False) -> tuple[str, str, str]:
    """Return a selected Rust function's exact contract, signature, and item."""
    functions = list(re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", source))
    selected = [m for m in functions if m.group(1) == function_name]
    require(len(selected) == 1 and (not sole_item or len(functions) == 1),
            f"named-call source must contain one selected function {function_name!r}")
    fn = selected[0]
    # The checked extension is a closed module with leading attributes. For a
    # general item without attributes, start at `fn`; otherwise bind from its
    # first immediately preceding attribute line.
    line_start = source.rfind("\n", 0, fn.start()) + 1
    if sole_item:
        attr_start = source.find("#[", 0, fn.start())
        if attr_start < 0:
            attr_start = fn.start()
    else:
        attr_start = fn.start()
    open_paren = source.find("(", fn.start())
    depth = 0
    close_paren = None
    for i in range(open_paren, len(source)):
        if source[i] == "(": depth += 1
        elif source[i] == ")":
            depth -= 1
            if depth == 0:
                close_paren = i
                break
    require(close_paren is not None, "named-call function parameters are unterminated")
    body_delimiter = re.search(r"\s*\{", source[close_paren + 1:])
    require(body_delimiter is not None, "named-call function has no body delimiter")
    body_open = close_paren + 1 + body_delimiter.end() - 1
    signature = source[fn.start():body_open].rstrip()
    depth = 0
    item_end = None
    for index in range(body_open, len(source)):
        if source[index] == "{": depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                item_end = index + 1
                break
    require(item_end is not None, "named-call function body is unterminated")
    contract = source[attr_start:body_open].rstrip()
    item = source[attr_start:item_end]
    return contract, signature, item


def _helper_source_spans(source: str, function_name: str) -> tuple[str, str, str]:
    return _function_source_spans(source, function_name, sole_item=True)


def assert_au_composition(source_tree: dict[str, bytes], generator: bytes,
                          mapping: dict[str, Any], active: bytes, positive: bytes,
                          extension_generated: bytes, terminal_generated: bytes,
                          client: bytes) -> dict[str, Any]:
    prefix = source_tree["promotion.rs"]
    extension = source_tree[AU_EXTENSION]
    try:
        extension_text = extension.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise CheckError("AU summary extension is not UTF-8") from exc
    client_source = _load_client_literal(generator).encode()
    require(sha(generator) == AU_GENERATOR_SHA256 and
            sha(extension) == AU_EXTENSION_SHA256 and sha(client) == AU_CLIENT_SHA256 and
            sha(active) == AU_ACTIVE_SHA256,
            "AU selected generator/extension/client/active source changed")
    expected_active = prefix + b"\n" + extension + client
    require(active == expected_active and positive == expected_active and
            extension_generated == extension and terminal_generated == extension and
            client == client_source,
            "AU active/client composition differs from the inherited prefix and exact source inputs")
    expected_keys = {
        "feature", "status", "stage", "full_original_admitted", "ancestor_commit",
        "ancestor_source", "ancestor_sha256", "base_source", "base_source_sha256",
        "selected_prefix_sha256", "extension_source", "extension_sha256", "client_sha256",
        "active", "active_sha256", "support_inventory", "helpers", "logic_helpers",
        "terminal_helpers", "ownership_frame", "loop_inventory", "assignment_effect",
        "return_evaluation", "excluded", "named_call_summaries", "native_source",
        "native_source_sha256", "native_client_mir", "native_mir_ready", "debug_places",
        "normal_edges", "mir_blocks", "native_assignment", "native_saved_return", "mir",
    }
    require(set(mapping) == expected_keys,
            f"AU mapping key surface changed: extra={sorted(set(mapping)-expected_keys)}, missing={sorted(expected_keys-set(mapping))}")
    require(mapping.get("feature") == "" and mapping.get("status") == "generated_unchecked" and
            mapping.get("stage") == "after-ElaborateDrops" and mapping.get("full_original_admitted") is False and
            mapping.get("ancestor_commit") == AT_COMMIT and
            mapping.get("ancestor_source") == "../original-owned-view-clone-2026-10-09/generated/positive.rs" and
            mapping.get("ancestor_sha256") == AT_POSITIVE_SHA256 and
            mapping.get("base_source") == "src/promotion.rs" and
            mapping.get("base_source_sha256") == sha(prefix) and
            mapping.get("selected_prefix_sha256") == AT_POSITIVE_SHA256 and
            mapping.get("extension_source") == AU_EXTENSION_REL and
            mapping.get("extension_sha256") == sha(extension) and
            mapping.get("client_sha256") == sha(client) and
            mapping.get("active") == "generated/active.rs" and
            mapping.get("active_sha256") == sha(active) and
            mapping.get("support_inventory") == {"src/" + n: sha(b) for n,b in sorted(source_tree.items())},
            "AU mapping does not bind the selected published prefix/extension/client composition")
    require(mapping.get("helpers") == [AU_HELPER] and mapping.get("logic_helpers") == [] and
            mapping.get("terminal_helpers") == ["bytes_root_detaching_terminal_drop", "bytes_view_terminal_drop", "bytes_cursor_terminal_drop"] and
            mapping.get("ownership_frame") == "same_allocation/has_allocation/shares_view_allocation: complete allocation identity; current ticket/fraction dynamic" and
            mapping.get("loop_inventory") == "cursor map exactly singleton current actual ticket id/fraction; no iteration quota" and
            mapping.get("assignment_effect") == {"order": ["evaluate named clone_suffix return", "drop old value", "install next", "increment i"], "native_address_nonobserving": True} and
            mapping.get("return_evaluation") == {"shadow": "let saved_return=observed", "before_value_drop": True} and
            mapping.get("excluded") == ["Root and general Static Clone", "arbitrary escaping/concurrent ownership", "unwind", "whole crate"],
            "AU mapping helper, assignment effect, ownership frame, or scope exclusions changed")
    require([m.group(1) for m in re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", extension_text)] == [AU_HELPER],
            "AU summary extension has an unknown function/callback")
    masked = re.sub(r"//[^\n]*|/\*.*?\*/|\"(?:\\.|[^\"\\])*\"", " ", extension_text, flags=re.S)
    require(not re.search(r"#\s*\[\s*(?:trusted|assume|axiom|extern_spec|cfg_attr)\b|macro_rules!", masked),
            "AU helper extension adds an unreviewed trusted/assumed/spec/macro boundary")
    body = extension_text[extension_text.index(" {", re.search(r"\bfn\s+clone_suffix_checked", extension_text).start()) + 2:]
    body = body[:body.rfind("}")]
    require(re.findall(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(", body) == ["clone_owned_api", "advance_api"] and
            body.count("ghost! {") == 1,
            "AU summary helper has an extra or reordered call effect")
    return {"composition_exact": True, "published_AT_prefix_sha256": sha(prefix),
        "extension_sha256": sha(extension), "client_sha256": sha(client),
        "active_sha256": sha(active), "mapping_support_inventory_exact": True,
        "named_helper_function_surface": [AU_HELPER], "helper_executable_calls": ["clone_owned_api", "advance_api"],
        "closed_client_full_literal_checked": True}


def mapping_json(mapping: dict[str, Any]) -> bytes:
    return (json.dumps(mapping, indent=2) + "\n").encode()


def _source_call_arguments(source: str, function_name: str) -> list[str]:
    matches = list(re.finditer(rf"\b{re.escape(function_name)}\s*\(", source))
    require(len(matches) == 1, f"closed client must contain one direct call to {function_name!r}")
    open_paren = source.find("(", matches[0].start())
    depth = 0
    close_paren = None
    for index in range(open_paren, len(source)):
        if source[index] == "(": depth += 1
        elif source[index] == ")":
            depth -= 1
            if depth == 0:
                close_paren = index
                break
    require(close_paren is not None, "closed client direct call is unterminated")
    return _split_rust_list(source[open_paren + 1:close_paren])


def _source_call_binding(source: str, function_name: str) -> str:
    rows = list(re.finditer(rf"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*{re.escape(function_name)}\s*\(", source))
    require(len(rows) == 1,
            f"closed caller must bind the returned value from one direct {function_name!r} call")
    return rows[0].group(1)


def _canonical_signature(signature: str) -> str:
    return re.sub(r"\s+", "", signature)


def _native_argument_mode(rust_type: str) -> str:
    if rust_type.startswith("&") and not rust_type.startswith("&mut"):
        return "shared_borrow"
    if rust_type in {"bool", "char", "u8", "u16", "u32", "u64", "u128", "usize",
                     "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"}:
        return "copy"
    raise CheckError(f"named direct call has an unsupported native argument type {rust_type!r}")


def _selected_module_for_active(lib_source: bytes) -> str:
    text = lib_source.decode("utf-8")
    rows = re.findall(r'#\s*\[cfg\s*\(\s*creusot\s*\)\s*\]\s*#\s*\[path\s*=\s*"\.\./generated/active\.rs"\s*\]\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;', text)
    require(len(rows) == 1, "AU lib.rs must route one selected Creusot module to generated/active.rs")
    return rows[0]


def validate_named_direct_call_v1(summary: dict[str, Any], *,
                                   native_report: dict[str, Any],
                                   shadow_source: bytes,
                                   shadow_caller_source: bytes,
                                   native_source: bytes,
                                   lib_source: bytes,
                                   manifest: bytes,
                                   source_inventory: dict[str, bytes],
                                   source_map: bytes,
                                   diagnostic_files: dict[str, bytes]) -> dict[str, Any]:
    """Validate a mapping row as an observed, direct, Ghost-erased call summary.

    This validator derives both function identities/signatures from the selected
    native and shadow source, checks their call argument lists and actual MIR
    call/return places, then derives the module/Coma/proof paths from the active
    module route and Cargo package. It does not accept a row as its own evidence.
    """
    require(summary.get("schema") == "named_direct_call_v1",
            "AU summary row does not use named_direct_call_v1")
    allowed = {
        "schema", "native_callee", "shadow_callee", "native_signature", "shadow_signature",
        "native_argument_modes", "native_result_mode", "erased_arguments", "shadow_source",
        "shadow_source_sha256", "contract_sha256", "source_item_sha256", "source_item_hash_format",
        "contract_hash_format", "caller_function", "caller_mir", "caller_mir_sha256",
        "proof_coma_path", "proof_hash_binding", "proof_target", "trusted_summary",
        "caller_inlines_callee", "callee_mir", "callee_mir_sha256", "callee_debug_places",
        "callee_blocks", "normal_owner_returns", "normal_owner_drops", "callsites",
        "native_operations", "shadow_operations", "excluded",
    }
    require(set(summary) == allowed,
            f"named_direct_call_v1 fields changed: extra={sorted(set(summary)-allowed)}, missing={sorted(allowed-set(summary))}")
    native_name = summary.get("native_callee")
    shadow_name = summary.get("shadow_callee")
    require(isinstance(native_name, str) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", native_name) and
            isinstance(shadow_name, str) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", shadow_name),
            "named summary callee names are malformed")
    native_text = native_source.decode("utf-8")
    shadow_text = shadow_source.decode("utf-8")
    require(native_report.get("source_and_execution", {}).get("source_sha256") == sha(native_source),
            "named summary source declarations do not belong to the native source bundle audited with MIR")
    native_contract, native_signature_text, _ = _function_source_spans(native_text, native_name)
    shadow_contract, shadow_signature_text, shadow_item = _helper_source_spans(shadow_text, shadow_name)
    native_sig = _parse_rust_signature(native_signature_text)
    shadow_sig = _parse_rust_signature(shadow_signature_text)
    require(native_sig["name"] == native_name and shadow_sig["name"] == shadow_name and
            len(native_sig["parameters"]) == 2 and
            len(shadow_sig["parameters"]) >= len(native_sig["parameters"]),
            "native and shadow named-call signatures are not compatible")
    source_signatures = native_report.get("source_and_execution", {})
    require(_canonical_signature(source_signatures.get("callee_source_signature", "")) ==
            _canonical_signature(native_signature_text),
            "native callee source signature is not joined to the independently audited source report")
    require(_canonical_signature(summary.get("native_signature", "")) ==
            _canonical_signature(native_signature_text) and
            _canonical_signature(summary.get("shadow_signature", "")) ==
            _canonical_signature(shadow_signature_text),
            "named summary signatures do not match their selected source declarations")
    derived_modes = [_native_argument_mode(p["type"]) for p in native_sig["parameters"]]
    require(summary.get("native_argument_modes") == derived_modes,
            "named summary native argument modes do not follow the native source types")
    require(all(shadow_sig["parameters"][i]["type"] == native_sig["parameters"][i]["type"]
                for i in range(len(native_sig["parameters"]))),
            "named summary shadow parameters do not retain the native argument type/order prefix")
    copy_result_types = {"bool", "char", "u8", "u16", "u32", "u64", "u128", "usize",
                         "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"}
    if native_sig["result"] in copy_result_types:
        derived_result_mode = "copy"
    else:
        require(native_sig["result"] == "Bytes",
                "named_direct_call_v1 has no reviewed ownership classification for this nonprimitive result type")
        derived_result_mode = "move_owner"
    require(summary.get("native_result_mode") == derived_result_mode and
            shadow_sig["result"] == native_sig["result"],
            "named summary result ownership does not match the native/shadow result types")
    erased = summary.get("erased_arguments")
    require(isinstance(erased, list) and len(shadow_sig["parameters"]) ==
            len(native_sig["parameters"]) + len(erased),
            "named summary must account for every extra shadow-only argument")
    for offset, row in enumerate(erased):
        expected_index = len(native_sig["parameters"]) + offset
        require(isinstance(row, dict) and set(row) == {"index", "name", "type", "caller_expression", "mode"} and
                row.get("index") == expected_index,
                "named summary erased arguments must be positional, contiguous, and complete")
        parameter = shadow_sig["parameters"][expected_index]
        require(parameter["name"] == row.get("name") and parameter["type"] ==
                re.sub(r"\s+", "", str(row.get("type", ""))) and
                parameter["type"].startswith("Ghost<&mut") and
                row.get("mode") == "exclusive_ghost_reborrow",
                "named summary erased argument is not a trailing exclusive Ghost reborrow")

    require(summary.get("source_item_hash_format") ==
            "exact_utf8_first_attribute_through_matching_closing_brace_inclusive" and
            summary.get("contract_hash_format") ==
            "exact_utf8_first_attribute_through_signature_excluding_space_before_body" and
            summary.get("shadow_source_sha256") == sha(shadow_source) and
            summary.get("source_item_sha256") == sha(shadow_item.encode()) and
            summary.get("contract_sha256") == sha(shadow_contract.encode()),
            "named summary source/contract hashes do not match exact selected source spans")
    source_path = summary.get("shadow_source")
    require(isinstance(source_path, str) and source_path in source_inventory and
            source_inventory[source_path] == shadow_source and
            summary.get("shadow_source_sha256") == sha(source_inventory[source_path]),
            "named summary source path does not resolve within the selected closed source inventory")
    require(source_map,
            "AU extraction map is empty")

    client = native_report.get("native_audit", {}).get("client", {})
    callee = native_report.get("native_audit", {}).get("call_summary_body", {})
    parsed_summary = native_report.get("native_audit", {}).get("named_call_summary", {})
    caller_name = summary.get("caller_function")
    require(caller_name == client.get("function") and
            native_name == callee.get("function") and
            parsed_summary.get("native_mir_join_rederived") is True and
            summary.get("trusted_summary") is False and summary.get("caller_inlines_callee") is False,
            "named summary caller/native identity or trust boundary is not independently rederived")
    native_caller_contract, native_caller_sig, native_caller_item = _function_source_spans(native_text, caller_name)
    native_calls = _source_call_arguments(native_caller_item, native_name)
    native_binding = _source_call_binding(native_caller_item, native_name)
    shadow_caller_text = shadow_caller_source.decode("utf-8")
    _, shadow_caller_sig, shadow_caller_item = _function_source_spans(shadow_caller_text, caller_name)
    shadow_calls = _source_call_arguments(shadow_caller_item, shadow_name)
    shadow_binding = _source_call_binding(shadow_caller_item, shadow_name)
    require(len(native_calls) == len(native_sig["parameters"]) and
            len(shadow_calls) == len(shadow_sig["parameters"]),
            "named direct call source arguments do not match the parsed declarations")
    require(all(_canonical_signature(native_calls[i]) == _canonical_signature(shadow_calls[i])
                for i in range(len(native_calls))),
            "named shadow call changed the native argument expression or order")
    require(native_binding == shadow_binding and
            client.get("debug_places", {}).get(native_binding) == client.get("call_summary_call", {}).get("result"),
            "native and shadow returned-owner bindings do not map to the independently parsed MIR result place")
    for row in erased:
        i = row["index"]
        require(_canonical_signature(shadow_calls[i]) == _canonical_signature(row["caller_expression"]),
                "named summary erased Ghost expression differs from the selected shadow caller")
    call = client.get("call_summary_call", {})
    calls = summary.get("callsites")
    require(isinstance(calls, list) and len(calls) == 1,
            "named summary must map one direct native callsite")
    site = calls[0]
    require(len(native_sig["parameters"]) == 2 and
            set(call) == {"block", "target", "receiver", "amount", "result", "normal_target", "unwind_target"},
            "named_direct_call_v1 is explicitly limited to the audited two-argument MIR call shape")
    require(site == {"block": call.get("block"), "result_place": call.get("result"),
                     "argument_places": [call.get("receiver"), call.get("amount")],
                     "successor": call.get("normal_target"), "unwind": call.get("unwind_target"),
                     "body_sha256": next((row.get("body_sha256") for row in mapping_blocks(client)
                                           if row.get("block") == call.get("block")), None)} and
            call.get("target") == native_name,
            "named summary callsite does not match the parsed native target, operands, result, and edges")
    caller_mir = client.get("mir_path")
    callee_mir = callee.get("path")
    require(summary.get("caller_mir") == caller_mir and
            summary.get("caller_mir_sha256") == client.get("mir_sha256") and
            summary.get("callee_mir") == callee_mir and
            summary.get("callee_mir_sha256") == callee.get("mir_sha256") and
            summary.get("callee_debug_places") == callee.get("debug_places") and
            summary.get("callee_blocks") == callee.get("mir_blocks") and
            summary.get("normal_owner_returns") == callee.get("normal_owner_returns") and
            summary.get("normal_owner_drops") == callee.get("normal_owner_drops") and
            callee.get("normal_cfg_exact") is True and len(callee.get("normal_owner_returns", [])) == 1 and
            callee.get("normal_owner_drops") == [],
            "named summary does not return one normal owner from its independently parsed callee MIR")
    require(summary.get("proof_hash_binding") == "fresh_post_translation_checker_receipt",
            "named summary proof binding is not the post-translation checker receipt")
    lib_source_text = lib_source.decode("utf-8")
    module_name = _selected_module_for_active(lib_source)
    package = tomllib.loads(manifest.decode("utf-8")).get("package", {}).get("name")
    require(isinstance(package, str) and re.fullmatch(r"[A-Za-z0-9_-]+", package),
            "AU Cargo package cannot derive the named proof module path")
    target_root = package.replace("-", "_") + "_rlib"
    proof_target = f"{module_name}::{shadow_name}"
    coma_path = f"verif/{target_root}/{module_name}/{shadow_name}.coma"
    proof_path = f"verif/{target_root}/{module_name}/{shadow_name}/proof.json"
    require(summary.get("proof_target") == proof_target and summary.get("proof_coma_path") == coma_path,
            "named summary proof target/path does not derive from the selected module and callee")
    coma = _read_regular(ROOT / coma_path, "AU named helper Coma target is missing")
    proof_bytes = _read_regular(ROOT / proof_path, "AU named helper proof target is missing")
    require(coma.count(f"let {shadow_name} ".encode()) == 1 and
            f"{shadow_name} ensures".encode() in coma and f"{shadow_name} requires".encode() in coma,
            "translated AU named Coma target does not contain its selected function/contract")
    proof = json.loads(proof_bytes)
    coma_proofs = proof.get("proofs", {}).get("Coma", {})
    require(f"vc_{shadow_name}" in coma_proofs and
            _proof_stats([proof]).get("null") == 0 and _proof_stats([proof]).get("structural") == 0,
            "AU named helper is not a body-proof target in the completed proof snapshot")
    require(diagnostic_files.get("probe/" + coma_path) == coma and
            diagnostic_files.get("probe/" + proof_path) == proof_bytes,
            "AU named helper Coma/proof bytes differ from the completed proof-reuse origin")
    caller_coma_path = f"verif/{target_root}/{module_name}/{caller_name}.coma"
    caller_proof_path = f"verif/{target_root}/{module_name}/{caller_name}/proof.json"
    caller_coma = _read_regular(ROOT / caller_coma_path, "AU named-summary caller Coma target is missing")
    caller_proof_bytes = _read_regular(ROOT / caller_proof_path, "AU named-summary caller proof target is missing")
    caller_coma_text = caller_coma.decode("utf-8")
    caller_marker = f"let {caller_name} ["
    require(caller_coma_text.count(caller_marker) == 1,
            "AU caller Coma target lacks one selected executable caller definition")
    caller_region = caller_coma_text[caller_coma_text.index(caller_marker):]
    call_lines = list(re.finditer(rf"(?m)^\s*\|\s*s\d+\s*=\s*{re.escape(shadow_name)}\b([^\n]*)", caller_region))
    require(len(call_lines) == 1,
            "AU caller Coma body must use exactly one direct call to the named helper")
    continuation = re.search(r"fun\s*\([^)]*\)\s*->\s*\[\s*&\s*([A-Za-z_][A-Za-z0-9_]*)\s*<-\s*_x\s*\]",
                             call_lines[0].group(1))
    require(continuation is not None and continuation.group(1) == shadow_binding,
            "AU caller Coma does not route the named summary's returned owner into the selected local")
    caller_proof = json.loads(caller_proof_bytes)
    require(f"vc_{caller_name}" in caller_proof.get("proofs", {}).get("Coma", {}) and
            _proof_stats([caller_proof]).get("null") == 0 and
            _proof_stats([caller_proof]).get("structural") == 0,
            "AU named-summary caller body is not itself in the completed proof snapshot")
    require(diagnostic_files.get("probe/" + caller_coma_path) == caller_coma and
            diagnostic_files.get("probe/" + caller_proof_path) == caller_proof_bytes,
            "AU named-summary caller Coma/proof bytes differ from the completed proof-reuse origin")
    parsed_goal = proof_target
    require(summary.get("native_signature") is not None and native_contract,
            "native named function lacks a selected source declaration")
    return {"schema": "named_direct_call_v1", "native_callee": native_name,
        "shadow_callee": shadow_name, "actual_helper_item_sha256": sha(shadow_item.encode()),
        "actual_contract_sha256": sha(shadow_contract.encode()), "translated_coma_path": coma_path,
        "translated_coma_sha256": sha(coma), "body_proof_path": proof_path,
        "body_proof_sha256": sha(proof_bytes), "proof_target": parsed_goal,
        "caller_coma_path": caller_coma_path, "caller_coma_sha256": sha(caller_coma),
        "caller_proof_path": caller_proof_path, "caller_proof_sha256": sha(caller_proof_bytes),
        "direct_call_native_mir_join": True, "erased_cursor_channel_bound": True,
        "body_proof_target_present": True, "diagnostic_origin_bytes_match": True,
        "native_owner_return_bound": True, "caller_modular_use_proved": True,
        "returned_owner_binding_joined": True, "trusted_summary": False,
        "caller_inlines_callee": False}


def mapping_blocks(client: dict[str, Any]) -> list[dict[str, Any]]:
    return client.get("mir_blocks", [])


def assert_au_named_summary(mapping: dict[str, Any], native_report: dict[str, Any],
                            extension: bytes, source_map: bytes,
                            at_archive: dict[str, bytes], diagnostic_files: dict[str, bytes],
                            native_source: bytes, shadow_caller_source: bytes,
                            lib_source: bytes, manifest: bytes,
                            source_inventory: dict[str, bytes]) -> dict[str, Any]:
    native = native_report.get("native_audit", {})
    client = native.get("client", {})
    callee = native.get("call_summary_body", {})
    parsed_summary = native.get("named_call_summary", {})
    summaries = mapping.get("named_call_summaries")
    require(isinstance(summaries, list) and len(summaries) == 1,
            "AU mapping must contain exactly one closed named direct-call summary")
    summary = summaries[0]
    return validate_named_direct_call_v1(summary, native_report=native_report,
        shadow_source=extension, shadow_caller_source=shadow_caller_source,
        native_source=native_source, lib_source=lib_source, manifest=manifest,
        source_inventory=source_inventory, source_map=source_map,
        diagnostic_files=diagnostic_files)


def assert_au_native(mapping: dict[str, Any]) -> dict[str, Any]:
    checker_path = ROOT / "check_native.py"
    checker_bytes = _read_regular(checker_path, "AU native checker is missing")
    require(sha(checker_bytes) == AU_NATIVE_CHECKER_SHA256,
            "AU native checker changed before import")
    spec = importlib.util.spec_from_file_location("au_pinned_native_correspondence", checker_path)
    require(spec is not None and spec.loader is not None, "cannot load AU native checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    bundle = module.load_bundle()
    bundle["mapping"] = mapping
    try:
        report = module.audit_bundle(bundle)
    except module.AuditError as exc:
        raise CheckError(str(exc)) from exc
    require(report.get("status") == "pass", "AU native source/MIR correspondence did not pass")
    native = report.get("native_audit", {})
    client = native.get("client", {})
    selected = bundle.get("capture", {}).get("selected", [])
    expected_mir = sorted(({"path": row.get("path"), "sha256": row.get("sha256")} for row in selected),
                          key=lambda row: row["path"])
    client_rows = [row for row in selected if row.get("label") == "client"]
    require(len(client_rows) == 1, "AU capture has no unique selected client MIR")
    client["mir_path"] = client_rows[0].get("path")
    callee_rows = [row for row in selected if row.get("label") == "call_summary_body"]
    require(len(callee_rows) == 1, "AU capture has no unique named helper MIR")
    native.setdefault("call_summary_body", {})["path"] = callee_rows[0].get("path")
    require(native.get("selected_mir_count") == AU_SELECTED_MIRS and
            native.get("production_mir_count") == AU_PRODUCTION_MIRS and
            mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == AU_NATIVE_SOURCE_SHA256 and
            mapping.get("native_mir_ready") is True and
            mapping.get("native_client_mir") == client.get("mir_path") and
            mapping.get("normal_edges") == client.get("normal_edges") and
            mapping.get("debug_places") == client.get("debug_places") and
            mapping.get("mir_blocks") == client.get("mir_blocks") and
            mapping.get("native_assignment") == client.get("assignment") and
            mapping.get("native_saved_return") == client.get("saved_return") and
            mapping.get("mir") == expected_mir,
            "AU mapping caller/MIR inventory differs from independently captured native MIR")
    capture_bytes = _read_regular(ROOT / "native-mir/capture.json", "AU native MIR capture receipt missing")
    require(sha(capture_bytes) == AU_CAPTURE_SHA256,
            "AU native capture receipt changed")
    return {"status": "pass", "selected_mir_count": native.get("selected_mir_count"),
        "production_mir_count": native.get("production_mir_count"),
        "native_report": report, "mapping_edges_rederived_from_MIR": True,
        "capture_sha256": AU_CAPTURE_SHA256}


def assert_au_manifest(inputs: dict[str, Any], at_archive: dict[str, bytes]) -> dict[str, Any]:
    try:
        manifest = tomllib.loads(inputs["manifest"].decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AU Cargo manifest could not be parsed: {exc}") from exc
    expected = {"package": {"name": PACKAGE, "version": "0.1.0", "edition": "2021", "publish": False},
        "dependencies": {"creusot-std": "=0.13.0"}, "workspace": {}, "features": AU_FEATURES}
    require(manifest == expected and sha(inputs["manifest"]) == AU_MANIFEST_SHA256,
            "AU Cargo package, dependency, feature surface, or workspace route changed")
    at_lock = at_archive.get("probe/Cargo.lock")
    require(at_lock is not None and sha(inputs["lock"]) == AU_LOCK_SHA256 and
            inputs["lock"].count(b'name = "' + PACKAGE.encode() + b'"') == 1 and
            inputs["lock"].replace(b'name = "' + PACKAGE.encode() + b'"',
                b'name = "' + AT_PACKAGE.encode() + b'"', 1) == at_lock,
            "AU Cargo.lock differs from published AT dependency resolution except package identity")
    require(inputs["build"] == at_archive.get("probe/build.rs") and
            inputs["extractor"] == at_archive.get("probe/extract_public.py") and
            sha(inputs["build"]) == AT_BUILD_INPUT_HASHES["build.rs"] and
            sha(inputs["extractor"]) == AT_BUILD_INPUT_HASHES["extract_public.py"],
            "AU changed the inherited production build/extraction route")
    require(sha(inputs["generator"]) == AU_GENERATOR_SHA256 and
            sha(inputs["launcher"]) == AU_LAUNCHER_SHA256,
            "AU generator or proof launcher changed")
    env = inputs["environment"]
    forbidden = [env.get(k, "") for k in ("BYTES_DROP_FEATURE", "BYTES_SCOPE_SOURCE_CONTROL", "BYTES_CARGO_FEATURES")]
    require(not any(forbidden) and env.get("BYTES_SCOPE_DIAGNOSTIC", "0") != "1" and
            env.get("BYTES_DROP_CHECKER_SKIP", "0") != "1" and env.get("BYTES_TRANSLATE_ONLY", "0") != "1",
            "AU cannot admit a feature, source-control, diagnostic, checker-skip, or translation-only run")
    return {"manifest_exact": True, "package": PACKAGE,
        "lock_matches_published_AT_except_package_name": True,
        "feature_surface": sorted(AU_FEATURES), "build_and_extractor_routes_inherited": True,
        "generator_launcher_mapping_pinned": True, "feature_or_diagnostic_environment": False}


def _expected_public_record(as_checker: Any, ar: Any) -> tuple[bytes, dict[str, str]]:
    return as_checker._expected_public_records(ar)


def _load_captured_artifacts() -> tuple[dict[str, bytes], bytes]:
    artifacts = {name: _read_regular(CAPTURE_DIR / name, f"AU captured Cargo artifact missing: {name}")
                 for name in AU_ARTIFACT_FIELDS}
    receipt = _read_regular(CAPTURE_DIR / "public-records-build-receipt.json", "AU Cargo capture receipt missing")
    return artifacts, receipt


def assert_au_compiled_capture(receipt: dict[str, Any], artifacts: dict[str, bytes],
                               stored_receipt: bytes, expected_record: bytes,
                               source_map: bytes, input_hashes: dict[str, str]) -> dict[str, Any]:
    require(set(artifacts) == set(AU_ARTIFACT_FIELDS), "AU Cargo snapshot must contain exactly four artifacts")
    require(json.loads(stored_receipt) == receipt and receipt.get("schema") == "au-compiled-public-records-v1" and
            receipt.get("status") == "pass" and receipt.get("probe_package") == PACKAGE,
            "AU compiled Cargo receipt has the wrong package/schema")
    require(receipt.get("build_script_sha256") == sha(_read_regular(ROOT / "build.rs", "AU build script missing")) and
            receipt.get("extractor_sha256") == sha(_read_regular(ROOT / "extract_public.py", "AU extractor missing")),
            "AU compiled receipt does not bind its build/extractor source")
    for name, field in AU_ARTIFACT_FIELDS.items():
        require(receipt.get(field) == sha(artifacts[name]), f"AU compiled artifact changed: {name}")
    fingerprint = json.loads(artifacts["cargo-run-build-fingerprint.json"])
    source_map_data = json.loads(source_map)
    rerun = [entry["RerunIfChanged"] for entry in fingerprint.get("local", [])
        if isinstance(entry, dict) and isinstance(entry.get("RerunIfChanged"), dict)]
    require(len(rerun) == 1 and rerun[0].get("paths") == AU_RERUN_PATHS,
            "AU Cargo fingerprint does not bind the selected production rerun inputs")
    target = pathlib.Path(receipt.get("cargo_target_dir", ""))
    output_rel = pathlib.Path(str(rerun[0].get("output", "")))
    output = pathlib.Path(receipt.get("build_output_path", ""))
    fingerprint_path = pathlib.Path(receipt.get("cargo_build_fingerprint", ""))
    require(target.resolve() == TARGET_DIR and receipt.get("cargo_target_dir") == str(TARGET_DIR) and
            not output_rel.is_absolute() and ".." not in output_rel.parts and output_rel.parts[:2] == ("debug", "build") and
            (target / output_rel).resolve() == output.resolve() and receipt.get("build_output_path") == str(output.resolve()) and
            receipt.get("cargo_build_fingerprint") == str(fingerprint_path.resolve()) and output.name == "output" and
            fingerprint_path.name == "run-build-script-build-script-build.json" and
            output.parent.name == fingerprint_path.parent.name and output.parent.name.startswith(PACKAGE + "-") and
            fingerprint_path.parent.parent.resolve() == (TARGET_DIR / "debug/.fingerprint").resolve() and
            output.parent.parent.resolve() == (TARGET_DIR / "debug/build").resolve(),
            "AU Cargo package, fingerprint, target and build-output paths do not join")
    directives = ["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate"] + ["cargo:rerun-if-changed=" + p for p in AU_RERUN_PATHS]
    require(artifacts["cargo-build-output.txt"] == ("\n".join(directives) + "\n").encode(),
            "AU Cargo build output differs from exact selected build script directives")
    actual_out = pathlib.Path(receipt.get("actual_out_dir", ""))
    compiled = pathlib.Path(receipt.get("compiled_input_path", ""))
    root_output = pathlib.Path(receipt.get("root_output_path", ""))
    expected_out = output.resolve().parent / "out"
    require(actual_out.resolve() == expected_out and compiled.name == "public_records.rs" and
            compiled.resolve() == expected_out / "public_records.rs" and compiled.parent.resolve() == expected_out and
            root_output.resolve() == expected_out.parent / "root-output" and
            artifacts["cargo-root-output.txt"] == str(expected_out).encode(),
            "AU captured Cargo root-output, OUT_DIR and compiled record paths do not join")
    require(artifacts["public_records.rs"] == expected_record and
            receipt.get("compiled_input_sha256") == sha(expected_record) and
            receipt.get("reconstructed_generated_sha256") == sha(expected_record) and
            receipt.get("source_map_sha256") == sha(source_map) and
            source_map_data.get("generated/public_records.rs", {}).get("sha256") == sha(expected_record) and
            receipt.get("production_rerun_input_sha256") == input_hashes and
            receipt.get("captured_actual_Cargo_artifact_count") == 4,
            "AU actual Cargo OUT_DIR record is not reconstructed from production sources")
    return {"status": "pass", "captured_artifact_count": 4,
        "fingerprint_inputs_exact": True, "build_output_exact": True,
        "OUT_DIR_root_output_join_exact": True, "public_records_reconstructed": True}


def assert_au_live_build(parent_context: dict[str, Any], source_map: bytes,
                         *, capture: bool, require_capture: bool) -> dict[str, Any]:
    as_checker = parent_context["_ancestry"]["_as_checker"]
    ar = parent_context["_ancestry"]["_ar_checker"]
    expected, input_hashes = _expected_public_record(as_checker, ar)
    matches = list(TARGET_DIR.glob(f"debug/.fingerprint/{PACKAGE}-*/run-build-script-build-script-build.json"))
    require(len(matches) == 1, "AU Cargo package must have exactly one build-script fingerprint")
    fp_path = matches[0].resolve()
    fp_bytes = _read_regular(fp_path, "AU selected Cargo build fingerprint missing")
    fingerprint = json.loads(fp_bytes)
    rerun = [entry["RerunIfChanged"] for entry in fingerprint.get("local", [])
        if isinstance(entry, dict) and isinstance(entry.get("RerunIfChanged"), dict)]
    require(len(rerun) == 1 and rerun[0].get("paths") == AU_RERUN_PATHS,
            "AU live Cargo fingerprint has unexpected rerun inputs")
    output_rel = pathlib.Path(str(rerun[0].get("output", "")))
    require(not output_rel.is_absolute() and ".." not in output_rel.parts and output_rel.parts[:2] == ("debug", "build"),
            "AU Cargo output path escapes the selected target")
    output = (TARGET_DIR / output_rel).resolve()
    require(output.is_file() and output.parent.name == fp_path.parent.name and
            fp_path.parent.parent.resolve() == (TARGET_DIR / "debug/.fingerprint").resolve() and
            output.parent.parent.resolve() == (TARGET_DIR / "debug/build").resolve(),
            "AU fingerprint does not resolve to the selected package build output")
    output_bytes = _read_regular(output, "AU Cargo build output missing")
    directives = ["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate"] + ["cargo:rerun-if-changed=" + p for p in AU_RERUN_PATHS]
    require(output_bytes == ("\n".join(directives) + "\n").encode(),
            "AU live Cargo build directives changed")
    out_dir = output.parent / "out"
    root_output = output.parent / "root-output"
    compiled = out_dir / "public_records.rs"
    require(root_output.is_file() and not root_output.is_symlink() and compiled.is_file() and
            not compiled.is_symlink() and root_output.read_bytes() == str(out_dir).encode() and
            compiled.read_bytes() == expected,
            "AU live OUT_DIR/public_records does not equal the source reconstruction")
    artifacts = {"public_records.rs": compiled.read_bytes(),
        "cargo-run-build-fingerprint.json": fp_bytes, "cargo-build-output.txt": output_bytes,
        "cargo-root-output.txt": root_output.read_bytes()}
    receipt = {"schema": "au-compiled-public-records-v1", "status": "pass", "probe_package": PACKAGE,
        "build_script_sha256": sha(_read_regular(ROOT / "build.rs", "AU build script missing")),
        "extractor_sha256": sha(_read_regular(ROOT / "extract_public.py", "AU extractor missing")),
        "cargo_target_dir": str(TARGET_DIR), "cargo_build_fingerprint": str(fp_path),
        "build_output_path": str(output), "root_output_path": str(root_output),
        "actual_out_dir": str(out_dir), "compiled_input_path": str(compiled),
        "compiled_input_sha256": sha(expected), "reconstructed_generated_sha256": sha(expected),
        "source_map_sha256": sha(source_map), "production_rerun_input_sha256": input_hashes,
        "captured_actual_Cargo_artifact_count": 4,
        "captured_input_path": "generated/compiled-inputs/public_records.rs",
        "captured_cargo_build_fingerprint_path": "generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_build_output_path": "generated/compiled-inputs/cargo-build-output.txt",
        "captured_root_output_path": "generated/compiled-inputs/cargo-root-output.txt"}
    for name, field in AU_ARTIFACT_FIELDS.items():
        receipt[field] = sha(artifacts[name])
    receipt_bytes = (json.dumps(receipt, indent=2) + "\n").encode()
    if capture:
        CAPTURE_DIR.mkdir(parents=True, exist_ok=True)
        for name, data in artifacts.items():
            (CAPTURE_DIR / name).write_bytes(data)
        (CAPTURE_DIR / "public-records-build-receipt.json").write_bytes(receipt_bytes)
    if capture or require_capture:
        saved, saved_receipt = _load_captured_artifacts()
        receipt_bytes = _read_regular(CAPTURE_DIR / "public-records-build-receipt.json", "AU saved Cargo receipt missing")
        checked = assert_au_compiled_capture(json.loads(receipt_bytes), saved, receipt_bytes,
            expected, source_map, input_hashes)
    else:
        checked = assert_au_compiled_capture(receipt, artifacts, receipt_bytes,
            expected, source_map, input_hashes)
    return {"status": "pass", "captured_artifact_count": 4,
        "snapshot_written": capture, "live_fingerprint_and_output_checked": True,
        "snapshot_matches_live_build": checked}


def _load_captured_artifacts() -> tuple[dict[str, bytes], bytes]:
    artifacts = {name: _read_regular(CAPTURE_DIR / name, f"AU captured Cargo artifact missing: {name}")
                 for name in AU_ARTIFACT_FIELDS}
    receipt = _read_regular(CAPTURE_DIR / "public-records-build-receipt.json", "AU captured Cargo receipt missing")
    return artifacts, receipt


def _load_inputs(*, need_capture: bool = True) -> dict[str, Any]:
    artifacts = receipt = None
    if need_capture:
        artifacts, receipt = _load_captured_artifacts()
    return {"source_tree": _regular_tree(ROOT / "src", "AU"),
        "manifest": _read_regular(ROOT / "Cargo.toml", "AU Cargo manifest missing"),
        "lock": _read_regular(ROOT / "Cargo.lock", "AU Cargo lockfile missing"),
        "build": _read_regular(ROOT / "build.rs", "AU build script missing"),
        "extractor": _read_regular(ROOT / "extract_public.py", "AU production extractor missing"),
        "generator": _read_regular(ROOT / "elaborate.py", "AU generator missing"),
        "launcher": _read_regular(ROOT / "run-proof.sh", "AU launcher missing"),
        "mapping_bytes": _read_regular(MAPPING, "AU mapping missing"),
        "active": _read_regular(ACTIVE, "AU active source missing"),
        "positive": _read_regular(POSITIVE, "AU positive source missing"),
        "extension_generated": _read_regular(ROOT / "generated/call-summary-extension.rs", "AU generated extension missing"),
        "terminal_generated": _read_regular(ROOT / "generated/terminal-helper.rs", "AU generated helper missing"),
        "client": _read_regular(ROOT / "generated/elaborated-client.rs", "AU generated caller missing"),
        "native_source": _read_regular(ROOT / "native.rs", "AU native witness missing"),
        "source_map": _read_regular(ROOT / "generated/source-map.json", "AU source map missing"),
        "compiled_capture": None if not need_capture else {
            "artifacts": artifacts, "receipt_bytes": receipt,
            "source_map": _read_regular(ROOT / "generated/source-map.json", "AU source map missing")},
        "environment": dict(os.environ)}


def assert_au_proof_reuse(inputs: dict[str, Any]) -> tuple[dict[str, Any], dict[str, bytes]]:
    evidence = ROOT / "evidence"
    receipt_path = evidence / "au-full-diagnostic-v1.json"
    archive_path = evidence / "au-full-diagnostic-v1.tar.gz"
    receipt_bytes = _read_regular(receipt_path, "AU completed diagnostic proof receipt is missing")
    archive_bytes = _read_regular(archive_path, "AU completed diagnostic proof archive is missing")
    require(sha(receipt_bytes) == AU_DIAGNOSTIC_RECEIPT_SHA256 and
            sha(archive_bytes) == AU_DIAGNOSTIC_ARCHIVE_SHA256,
            "AU diagnostic proof-reuse origin differs from its immutable capture")
    receipt = json.loads(receipt_bytes)
    require(receipt.get("archive") == archive_path.name and receipt.get("archive_sha256") == AU_DIAGNOSTIC_ARCHIVE_SHA256 and
            receipt.get("statistics") == {"files": AU_DIAGNOSTIC_TARGETS, "prover": AU_DIAGNOSTIC_PROVER_LEAVES,
                "null": 0, "structural": 0} and
            receipt.get("target_policy", {}).get("excluded") == {} and
            receipt.get("target_policy", {}).get("features") == [] and
            receipt.get("target_policy", {}).get("diagnostic") is True and
            receipt.get("target_policy", {}).get("correspondence_exit_status") == 2,
            "AU proof origin must be the explicitly diagnostic full 157-target run")
    files = _read_archive(archive_path, AU_DIAGNOSTIC_ARCHIVE_SHA256, AU_DIAGNOSTIC_MEMBERS, "AU diagnostic origin")
    proof_result = _verify_target_manifest(receipt, files, AU_DIAGNOSTIC_TARGETS,
        AU_DIAGNOSTIC_PROVER_LEAVES, "AU diagnostic origin", admitted=False)
    for name, data in inputs["source_tree"].items():
        require(files.get("probe/src/" + name) == data,
                f"AU proof reuse source differs from completed diagnostic: src/{name}")
    for relative, data in (("Cargo.toml", inputs["manifest"]), ("Cargo.lock", inputs["lock"]),
            ("build.rs", inputs["build"]), ("extract_public.py", inputs["extractor"]),
            ("generated/active.rs", inputs["active"]), ("generated/positive.rs", inputs["positive"]),
            ("generated/elaborated-client.rs", inputs["client"]), ("native.rs", inputs["native_source"])):
        archived = files.get("probe/" + relative)
        require(archived == data, f"AU proof reuse input differs from diagnostic archive: {relative}")
    target_rows = receipt["targets"]
    current_comas = {p.relative_to(ROOT).as_posix() for p in (ROOT / "verif").rglob("*.coma")}
    current_proofs = {p.relative_to(ROOT).as_posix() for p in (ROOT / "verif").rglob("proof.json")}
    expected_comas = {row["coma"].removeprefix("probe/") for row in target_rows}
    expected_proofs = {row["proof"].removeprefix("probe/") for row in target_rows}
    require(current_comas == expected_comas and current_proofs == expected_proofs,
            "AU current translated/proof target inventory differs from completed diagnostic run")
    for row in target_rows:
        for field in ("coma", "proof"):
            path = row[field]
            current = _read_regular(ROOT / path.removeprefix("probe/"), f"AU proof reuse target missing: {path}")
            require(current == files[path] and sha(current) == row[field + "_sha256"],
                    f"AU translated/proof target changed after the completed diagnostic run: {path}")
    return ({"origin": "au-full-diagnostic-v1", "origin_archive_sha256": AU_DIAGNOSTIC_ARCHIVE_SHA256,
        "origin_receipt_sha256": AU_DIAGNOSTIC_RECEIPT_SHA256, "origin_status": "diagnostic_non_admitted",
        "origin_checker_exit_status": 2, **proof_result,
        "current_source_Cargo_and_all_target_pairs_byte_exact": True,
        "current_rust_and_coma_inputs_identical_to_completed_proof": True,
        "prover_rerun_claimed": False,
        "new_admission_depends_on_current_correspondence_and_receipt": True}, files)


def audit_inputs(inputs: dict[str, Any], *, cargo_mode: str = "captured",
                 parent_context: dict[str, Any] | None = None) -> dict[str, Any]:
    if parent_context is None:
        at_archive_report, at_archive = assert_at_canonical_archive()
        parent_context = assert_published_at_ancestry(at_archive)
        parent_context["canonical_archive_report"] = at_archive_report
    at_archive = parent_context["_at_archive"]
    source_closure = assert_au_source_closure(at_archive, inputs["source_tree"])
    mapping = json.loads(inputs["mapping_bytes"])
    composition = assert_au_composition(inputs["source_tree"], inputs["generator"], mapping,
        inputs["active"], inputs["positive"], inputs["extension_generated"],
        inputs["terminal_generated"], inputs["client"])
    manifest = assert_au_manifest(inputs, at_archive)
    native = assert_au_native(mapping)
    diagnostic, diagnostic_files = assert_au_proof_reuse(inputs)
    named = assert_au_named_summary(mapping, native["native_report"],
        inputs["source_tree"][AU_EXTENSION], inputs["source_map"], at_archive, diagnostic_files,
        inputs["native_source"], inputs["client"], inputs["source_tree"]["lib.rs"], inputs["manifest"],
        {"src/" + k: v for k, v in inputs["source_tree"].items()})
    if cargo_mode == "captured":
        capture = inputs["compiled_capture"]
        expected_record, input_hashes = _expected_public_record(parent_context["_ancestry"]["_as_checker"],
            parent_context["_ancestry"]["_ar_checker"])
        compiled = assert_au_compiled_capture(json.loads(capture["receipt_bytes"]),
            capture["artifacts"], capture["receipt_bytes"], expected_record,
            capture["source_map"], input_hashes)
        compiled["capture_mode"] = "archived four actual Cargo artifacts; no live Cargo target inspected"
    elif cargo_mode == "live":
        compiled = assert_au_live_build(parent_context, inputs["source_map"], capture=False, require_capture=True)
    elif cargo_mode == "capture":
        compiled = assert_au_live_build(parent_context, inputs["source_map"], capture=True, require_capture=False)
    elif cargo_mode == "skip":
        compiled = {"status": "not_checked", "reason": "explicit source-only ancestry mode"}
    else:
        raise CheckError(f"unknown AU Cargo audit mode: {cargo_mode}")
    return {"status": "pass",
        "checker_scope": "one closed named Clone/advance summary called through the selected AU client over published AT",
        "full_original_admitted": False,
        "published_AT_ancestry": {k:v for k,v in parent_context.items() if not k.startswith("_")},
        "AU_source_closure": source_closure,
        "AU_source_and_mapping": composition,
        "AU_manifest_and_features": manifest,
        "AU_native_correspondence": {"selected_mir_count": native["selected_mir_count"],
            "production_mir_count": native["production_mir_count"], "native_checker_sha256": AU_NATIVE_CHECKER_SHA256},
        "AU_named_call_summary": named,
        "AU_completed_proof_reuse": diagnostic,
        "AU_compiled_production_input": compiled,
        "claims": {"named_helper": "body-proved Clone then Buf::advance owning-return summary; direct caller uses the proved contract",
            "erased_cursor": "one exclusive ghost reborrow, erased from the native argument list",
            "native_binding": "one direct clone_suffix call and the selected MIR return/drop edges",
            "proof_reuse": "the completed diagnostic proof is reused only after exact Rust/Cargo/Coma/proof-byte identity and this passing correspondence gate",
            "full_original_admitted": False, "unwind_or_general_calls": False}}


def audit(*, cargo_mode: str = "captured", inputs_override: dict[str, Any] | None = None,
          parent_context: dict[str, Any] | None = None) -> dict[str, Any]:
    inputs = _load_inputs() if inputs_override is None else inputs_override
    return audit_inputs(inputs, cargo_mode=cargo_mode, parent_context=parent_context)


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
        require(args.shadow.resolve() == ACTIVE.resolve(), "shadow must select AU generated/active.rs")
        require(args.mapping.resolve() == MAPPING.resolve(), "mapping must select AU generated/mapping.json")
        if args.ancestry_only:
            at_report, at_archive = assert_at_canonical_archive()
            parent_context = assert_published_at_ancestry(at_archive)
            report = {"status": "pass", "checker_scope": "published AT ancestry only; no AU admission",
                "AT_archive": at_report,
                "AT_replay": {k:v for k,v in parent_context.items() if not k.startswith("_")}}
        elif args.capture_compiled_inputs:
            inputs = _load_inputs(need_capture=False)
            at_report, at_archive = assert_at_canonical_archive()
            parent_context = assert_published_at_ancestry(at_archive)
            report = {"status": "pass", "checker_scope": "AU source and actual four-artifact Cargo capture",
                "AT_archive": at_report,
                "AU_source_closure": assert_au_source_closure(at_archive, inputs["source_tree"]),
                "AU_compiled_production_input": assert_au_live_build(parent_context,
                    inputs["source_map"], capture=True, require_capture=False),
                "proof_result_not_claimed": True}
        elif args.audit_compiled_capture_only:
            inputs = _load_inputs()
            report = audit_inputs(inputs, cargo_mode="captured")
            report["checker_scope"] = "AU complete source/native/compiled capture audit; no live Cargo target"
        else:
            report = audit()
    except Exception as exc:
        report = {"status": "reject", "checker_scope": "AU selected named-call source/native/Cargo correspondence",
            "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if report.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
