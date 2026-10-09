#!/usr/bin/env python3
"""AQ source/native correspondence gate for nested public Bytes slice views.

The probe checks one built-in ``Range<usize>`` nested-slice client. It does not
admit arbitrary ``RangeBounds``, invalid-range unwind, concurrency, or the full
Bytes API. Inherited AP proof/source inputs are pinned before their checkers are
loaded. The AP proof prefix receives one explicit proof-enum transformation;
all other inherited AP source remains fixed.
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
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
PROBES = ROOT.parent
CRATE_ROOT = ROOT.parents[2]
AP_ROOT = PROBES / "original-shared-finite-owners-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
MAPPING = ROOT / "generated/mapping.json"
AQ_PREFIX = ROOT / "src/promotion.rs"
AQ_EXTENSION = ROOT / "src/slice_extension.rs"
AQ_POINTER_SUPPORT = ROOT / "src/view_pointer.rs"
AQ_CLIENT = ROOT / "generated/elaborated-client.rs"

# AP is the published architecture and source ancestry. Verify every reused
# executable checker and its receipt before importing Python from that probe.
AP_COMMIT = "1191e1c3752182f63793e30c3644971b7b8fa0ea"
AP_CHECKER_SHA256 = "8057cd505d1aafe1e6740c1ffe14edfe3c03fb4b5b810b24e657c0fadb84b79d"
AP_NATIVE_CHECKER_SHA256 = "5c6b0d399114bf3f0d7fd7149573f506faba14043b60c5acb5018f75a16524cc"
AP_CONTROLS_CHECKER_SHA256 = "a63d2cbd1d4a4eea8285bbfe7ec885a498b1134cd04ecce666667a7016f47a80"
AP_CONTROLS_MANIFEST_SHA256 = "931807955178b6fbc083b30f5e3c35cd31981d5f3551d1b9c54df9b1d733b56d"
AP_CONTROLS_RECEIPT_SHA256 = "44794a19cfaed7dfcbbd05ac6ef3cb1c1fc577b03baad28e577d4575a8a289fd"
AP_CORRESPONDENCE_SHA256 = "1179f345818a8a111676c9264b22f127baefe674022e0e5eb90be9c57cf1e188"
AP_CANONICAL_AUDIT_SHA256 = "990a066ac0d4da2695068ae233107fef266861dcbab77b72c713205e91ea86ae"
AP_CANONICAL_ARCHIVE_SHA256 = "a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1"
AP_ACTIVE_SHA256 = "cf05c10ecfe38af6ec999a4577a5de5bd5a8bca1d4422c8b94737b2349ef428b"
AP_PREFIX_SHA256 = "e607bc1c5587b3d5e0a8260a3891807db3ade7ab331c264a3e49d1de443ca984"
AQ_NATIVE_CHECKER_SHA256 = "0c5099600ba7e9bc4ce18f52d1926f9df3c079b5da91809517f492d533a43eb3"
AQ_NATIVE_CONTROLS_SHA256 = "bc88dcc475c491e9c777b8c81eaff4b76fdb2b7e0aba25e794b25e6881934d85"
AQ_NATIVE_CONTROLS_MANIFEST_SHA256 = "2971473668c46810ddd72c52d978d77524ca1b9d2294aee702e511803cd0d3bc"
AQ_NATIVE_CONTROLS_RECEIPT_SHA256 = "e3d10feecfd34a16b138e71f5efa21dc9f731d54a1755364f133d173c67924c8"
AQ_EXTENSION_SHA256 = "a5abe021099a257dafeb14043835e1e20c36600e8d9aa4268c470e163175eb5f"
AQ_POINTER_SUPPORT_SHA256 = "11560870cfe83e8d8ace30ebe27c6efa0c618d1368a749f4ace773b89961c6dd"
AQ_CLIENT_SHA256 = "dab1fc55af21f1f0769eee2ec04f325b2e89c4ebad6b2bc5327aa06b45ba292f"
AQ_ACTIVE_SHA256 = "167f08c84980ff5ab80c7db50909a99879294c98ed4bc11ff7453d05c267e42b"
AQ_LOCK_SHA256 = "ecd445613ef6aed9d4c157434d348ae652e8038d7ae2e0f51b22491e307c2637"
AQ_GENERATOR_SHA256 = "313503b87a0a81961f3ff83db448047be5ab74296fbd1e3776a1219eaf7664c3"
AQ_EXTRACTOR_SHA256 = "6ba81b23f0b8edfc37d442f6617925d29d7a87d74a0aaf6e5bb999b5320b2414"
AQ_CARGO_TARGET_DIR = pathlib.Path("/workspace/bytes-proof-tools/targets/bytes").resolve()

class CheckError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def import_pinned(name: str, path: pathlib.Path, expected_sha: str) -> Any:
    require(path.is_file() and sha(path.read_bytes()) == expected_sha,
            f"pinned executable checker changed before import: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot import pinned checker: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def preflight_published_ap() -> tuple[Any, dict[str, Any]]:
    global AP
    paths = {
        "checker": (AP_ROOT / "check_correspondence.py", AP_CHECKER_SHA256),
        "native_checker": (AP_ROOT / "check_native.py", AP_NATIVE_CHECKER_SHA256),
        "controls_checker": (AP_ROOT / "check_checker_controls.py", AP_CONTROLS_CHECKER_SHA256),
        "controls_manifest": (AP_ROOT / "fixtures/checker-controls.json", AP_CONTROLS_MANIFEST_SHA256),
        "controls_receipt": (AP_ROOT / "generated/checker-controls-receipt.json", AP_CONTROLS_RECEIPT_SHA256),
        "correspondence": (AP_ROOT / "generated/correspondence.json", AP_CORRESPONDENCE_SHA256),
        "canonical_audit": (AP_ROOT / "evidence/AP_CANONICAL_AUDIT.json", AP_CANONICAL_AUDIT_SHA256),
        "canonical_archive": (AP_ROOT / "evidence/ap-positive-canonical-v1.tar.gz", AP_CANONICAL_ARCHIVE_SHA256),
    }
    for label, (path, expected) in paths.items():
        require(path.is_file() and sha(path.read_bytes()) == expected,
                f"published AP {label} input changed: {path}")
    controls = json.loads(paths["controls_receipt"][0].read_text())
    require(controls.get("status") == "pass" and controls.get("controls") == 35 and
            controls.get("rejected_as_expected") == 35 and controls.get("solver_invoked") is False and
            controls.get("cargo_invoked") is False,
            "published AP 35-control structural receipt is incomplete")
    audit = json.loads(paths["canonical_audit"][0].read_text())
    require(audit.get("status") == "audited_pass" and
            audit.get("archive_sha256") == AP_CANONICAL_ARCHIVE_SHA256,
            "published AP canonical evidence is not independently audited")
    correspondence = json.loads(paths["correspondence"][0].read_text())
    require(correspondence.get("status") == "pass" and
            correspondence.get("active_composition", {}).get("active_composition_exact") is True,
            "published AP correspondence did not pass its exact source gate")
    ap = import_pinned("aq_pinned_ap_correspondence", paths["checker"][0], AP_CHECKER_SHA256)
    AP = ap
    inherited = ap.assert_inherited_ap_sources()
    require((AP_ROOT / "generated/positive.rs").is_file() and
            sha((AP_ROOT / "generated/positive.rs").read_bytes()) == AP_ACTIVE_SHA256 and
            sha((AP_ROOT / "src/promotion.rs").read_bytes()) == AP_PREFIX_SHA256,
            "published AP positive active/prefix source identity changed")
    return ap, {"published_commit": AP_COMMIT, "checker_sha256": AP_CHECKER_SHA256,
        "native_checker_sha256": AP_NATIVE_CHECKER_SHA256,
        "canonical_archive_sha256": AP_CANONICAL_ARCHIVE_SHA256,
        "controls": 35, "all_controls_rejected": True,
        "ancestor_source_and_trust_closure": inherited}


def rust_item_span(source: str, kind: str, name: str) -> tuple[int, int, str]:
    """Return a braced Rust item span using the ancestor's comment/string mask."""
    require("AP" in globals(), "ancestor Rust token/masking tools are not loaded")
    masked = AP.AO.AN.AI.mask_noncode(source)
    matches = list(re.finditer(r"\b" + re.escape(kind) + r"\s+" + re.escape(name) + r"\b", masked))
    require(len(matches) == 1, f"expected one {kind} {name} declaration")
    opening = masked.find("{", matches[0].end())
    require(opening >= 0, f"{kind} {name} has no braced body")
    depth = 0
    for pos in range(opening, len(masked)):
        if masked[pos] == "{":
            depth += 1
        elif masked[pos] == "}":
            depth -= 1
            if depth == 0:
                return matches[0].start(), pos + 1, source[matches[0].start():pos + 1]
    raise CheckError(f"unclosed {kind} {name} declaration")


def enum_variant_rows(enum_source: str) -> list[tuple[str, list[str]]]:
    """Parse variant names and token rows, keeping nested tuple/generic commas."""
    tokens = AP.AO.AN.AI.rust_tokens(enum_source)
    try:
        index = tokens.index("{") + 1
    except ValueError as exc:
        raise CheckError("selected proof enum has no tokenized body") from exc
    rows: list[tuple[str, list[str]]] = []
    while index < len(tokens) and tokens[index] != "}":
        if tokens[index] == ",":
            index += 1
            continue
        name = tokens[index]
        require(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name) is not None,
                f"malformed enum variant head: {name}")
        start = index
        index += 1
        paren = square = brace = angle = 0
        while index < len(tokens):
            token = tokens[index]
            if token == "," and paren == square == brace == angle == 0:
                break
            if token == "(" : paren += 1
            elif token == ")": paren -= 1
            elif token == "[": square += 1
            elif token == "]": square -= 1
            elif token == "{": brace += 1
            elif token == "}":
                if paren == square == brace == angle == 0:
                    break
                brace -= 1
            elif token == "<": angle += 1
            elif token == ">" and angle:
                angle -= 1
            require(min(paren, square, brace, angle) >= 0, "unbalanced enum variant delimiters")
            index += 1
        rows.append((name, tokens[start:index]))
        if index < len(tokens) and tokens[index] == ",":
            index += 1
    require(tokens[index] == "}", "unterminated enum token stream")
    return rows


def assert_enum_prefix_transformation(parent_prefix: str, selected_prefix: str,
                                      selected_variant_names: tuple[str, ...]) -> dict[str, Any]:
    """Require exactly one reviewed proof enum declaration to change from AP.

    This deliberately does not call the transformed source byte-exact AP. Every
    byte outside the single `OriginalSharedProof` declaration must remain equal;
    Root and Child payload variants retain AP token identity, and the new View /
    Empty variants are separately checked against the AQ extension/source pins.
    """
    parent_start, parent_end, parent_enum = rust_item_span(parent_prefix, "enum", "OriginalSharedProof")
    selected_start, selected_end, selected_enum = rust_item_span(selected_prefix, "enum", "OriginalSharedProof")
    parent_rows = enum_variant_rows(parent_enum)
    selected_rows = enum_variant_rows(selected_enum)
    require([name for name, _ in parent_rows] == ["Root", "Child"],
            "AP parent proof enum surface changed")
    require(tuple(name for name, _ in selected_rows) == selected_variant_names,
            "AQ proof enum variants differ from the explicitly selected View/Empty sum")
    require(selected_variant_names == ("Root", "Child", "View", "Empty"),
            "AQ must add exactly View and Empty proof cases to the inherited Root/Child sum")
    expected_added = {
        "View": AP.AO.AN.AI.rust_tokens("View(ChildProof, raw_vec::BoundPtr)"),
        "Empty": AP.AO.AN.AI.rust_tokens("Empty(EmptyViewProof)"),
    }
    for name, expected in expected_added.items():
        selected_tokens = next(tokens for variant, tokens in selected_rows if variant == name)
        require(selected_tokens == expected,
                f"AQ {name} proof case has an unreviewed payload or resource-bearing shape")
    for name in ("Root", "Child"):
        parent_tokens = next(tokens for variant, tokens in parent_rows if variant == name)
        selected_tokens = next(tokens for variant, tokens in selected_rows if variant == name)
        require(selected_tokens == parent_tokens,
                f"AQ transformed enum changed inherited {name} resource payload")
    reconstructed = parent_prefix[:parent_start] + selected_enum + parent_prefix[parent_end:]
    require(selected_prefix == reconstructed,
            "AQ promotion prefix has changes outside the one explicitly transformed proof enum")
    return {"parent_prefix_sha256": sha(parent_prefix.encode()),
        "selected_prefix_sha256": sha(selected_prefix.encode()),
        "parent_enum_sha256": sha(parent_enum.encode()),
        "selected_enum_sha256": sha(selected_enum.encode()),
        "selected_variant_names": list(selected_variant_names),
        "inherited_root_child_payload_tokens_preserved": True,
        "transformation": "replace only OriginalSharedProof declaration; do not claim byte-exact prefix"}


def assert_lib_route(selected_lib: str, parent_lib: str) -> dict[str, Any]:
    expected = parent_lib.rstrip("\n") + "\n\n#[cfg(creusot)] mod view_pointer;\n"
    require(selected_lib == expected,
            "AQ lib.rs changes AP module routing beyond the single selected view_pointer module")
    return {"route_exact": True, "added_module": "#[cfg(creusot)] mod view_pointer;"}


def assert_probe_manifest(probe_manifest: bytes | None = None,
                          native_manifest_bytes: bytes | None = None) -> dict[str, Any]:
    probe_data = probe_manifest if probe_manifest is not None else (ROOT / "Cargo.toml").read_bytes()
    native_data = native_manifest_bytes if native_manifest_bytes is not None else (ROOT / "native-test/Cargo.toml").read_bytes()
    try:
        probe = tomllib.loads(probe_data.decode())
        native = tomllib.loads(native_data.decode())
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
        raise CheckError(f"AQ Cargo manifest parse failed: {exc}") from exc
    expected_probe = {"package": {"name": "bytes-original-shared-slice-views", "version": "0.1.0",
            "edition": "2021", "publish": False},
        "dependencies": {"creusot-std": "=0.13.0"}, "workspace": {},
        "features": {"negative_missing_acquire": [], "negative_missing_payload_free": [],
                     "negative_missing_control_free": []}}
    expected_native = {"package": {"name": "bytes-shared-slice-views-native", "version": "0.0.0",
            "edition": "2021"}, "workspace": {},
        "lib": {"name": "bytes_shared_slice_views_native", "path": "../native.rs"},
        "dependencies": {"bytes": {"path": "../../../../"}}}
    require(probe == expected_probe,
            "AQ Cargo package/dependency/default feature surface changed")
    require(native == expected_native,
            "AQ native harness manifest/package dependency route changed")
    require(os.environ.get("BYTES_DROP_FEATURE", "") == "",
            "AQ correspondence gate requires the feature-free selected client")
    assert_probe_lock((ROOT / "Cargo.lock").read_bytes())
    return {"probe_manifest_exact": True, "native_manifest_exact": True,
        "probe_lock_sha256": AQ_LOCK_SHA256, "default_feature_free": True}


def assert_probe_lock(lock: bytes) -> None:
    require(sha(lock) == AQ_LOCK_SHA256,
            "AQ Cargo.lock or its pinned creusot-std dependency resolution changed")


def assert_generator_source(source: bytes) -> None:
    require(sha(source) == AQ_GENERATOR_SHA256,
            "AQ source elaborator/generator or its feature-control surface changed")


def assert_aq_extractor_source_surface(source_bytes: bytes) -> None:
    # AQ's frozen extractor adds the nested-view body extraction; the inherited
    # AP extractor pin is intentionally not reused here.
    require(sha(source_bytes) == AQ_EXTRACTOR_SHA256,
            "AQ production public-record extractor source changed")


def assert_compiled_capture(receipt: dict[str, Any],
                            artifacts_override: dict[str, bytes] | None = None,
                            stored_receipt_bytes: bytes | None = None) -> dict[str, Any]:
    """Validate the archived four real Cargo artifacts and their receipt bindings."""
    capture_dir = ROOT / "generated/compiled-inputs"
    receipt_path = capture_dir / "public-records-build-receipt.json"
    receipt_bytes = stored_receipt_bytes if stored_receipt_bytes is not None else (
        receipt_path.read_bytes() if receipt_path.is_file() else b"")
    require(json.loads(receipt_bytes) == receipt,
            "AQ compiled-input receipt bytes differ from the receipt being checked")
    files = {
        "public_records.rs": "captured_input_sha256",
        "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
        "cargo-build-output.txt": "captured_build_output_sha256",
        "cargo-root-output.txt": "captured_root_output_sha256",
    }
    if artifacts_override is not None:
        require(set(artifacts_override) == set(files),
                "AQ compiled-input replay override has an incomplete artifact set")
    require(receipt.get("status") == "pass" and
            receipt.get("probe_package") == "bytes-original-shared-slice-views" and
            receipt.get("build_script_sha256") == "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925" and
            receipt.get("extractor_sha256") == AQ_EXTRACTOR_SHA256,
            "AQ compiled-input receipt identifies an unreviewed Cargo build/extractor")
    require(receipt.get("captured_input_path") == "generated/compiled-inputs/public_records.rs" and
            receipt.get("captured_cargo_build_fingerprint_path") == "generated/compiled-inputs/cargo-run-build-fingerprint.json" and
            receipt.get("captured_build_output_path") == "generated/compiled-inputs/cargo-build-output.txt" and
            receipt.get("captured_root_output_path") == "generated/compiled-inputs/cargo-root-output.txt",
            "AQ compiled-input receipt redirects a captured artifact")
    captured: dict[str, bytes] = {}
    for name, field in files.items():
        path = capture_dir / name
        if artifacts_override is not None:
            data = artifacts_override[name]
        else:
            require(path.is_file(), f"AQ captured Cargo build artifact is missing: {name}")
            data = path.read_bytes()
        require(receipt.get(field) == sha(data), f"AQ captured Cargo build artifact hash changed: {name}")
        captured[name] = data
    require(receipt.get("captured_actual_Cargo_artifact_count") == 4,
            "AQ build receipt claims a different compiled artifact set")
    fp = json.loads(captured["cargo-run-build-fingerprint.json"])
    rerun = [row["RerunIfChanged"] for row in fp.get("local", [])
             if isinstance(row, dict) and "RerunIfChanged" in row]
    expected_paths = ["../../../src/bytes.rs", "../../../src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs", "../../../src/bytes_mut.rs", "extract_public.py"]
    require(len(rerun) == 1 and rerun[0].get("paths") == expected_paths,
            "AQ captured Cargo fingerprint does not record the exact production build inputs")
    output_rel = pathlib.Path(rerun[0].get("output", ""))
    target = pathlib.Path(receipt.get("cargo_target_dir", ""))
    output_path = pathlib.Path(receipt.get("build_output_path", ""))
    fingerprint_path = pathlib.Path(receipt.get("cargo_build_fingerprint", ""))
    require(not output_rel.is_absolute() and output_rel.parts[:2] == ("debug", "build") and
            target.resolve() == AQ_CARGO_TARGET_DIR and
            (target / output_rel).resolve() == output_path.resolve() and
            fingerprint_path.name == "run-build-script-build-script-build.json" and
            output_path.name == "output" and
            fingerprint_path.parent.name == output_path.parent.name and
            fingerprint_path.parent.parent.name == ".fingerprint" and
            output_path.parent.parent.name == "build" and
            fingerprint_path.parent.parent.parent.resolve() == (target / "debug").resolve() and
            output_path.parent.parent.parent.resolve() == (target / "debug").resolve(),
            "AQ captured fingerprint, target, build output and Cargo package identity do not join")
    directives = (["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
                   "cargo:rustc-cfg=bytes_original_shared_gate"] +
                  ["cargo:rerun-if-changed=" + path for path in expected_paths])
    require(captured["cargo-build-output.txt"] == ("\n".join(directives) + "\n").encode(),
            "AQ captured Cargo build directives changed")
    actual_out = pathlib.Path(receipt.get("actual_out_dir", ""))
    compiled_path = pathlib.Path(receipt.get("compiled_input_path", ""))
    require(actual_out.name == "out" and actual_out.resolve() == (output_path.parent / "out").resolve() and
            compiled_path.name == "public_records.rs" and actual_out == compiled_path.parent and
            pathlib.Path(receipt.get("root_output_path", "")).resolve() == (actual_out.parent / "root-output").resolve() and
            captured["cargo-root-output.txt"] == str(actual_out).encode(),
            "AQ captured Cargo root-output/OUT_DIR records do not resolve consistently")
    require(receipt.get("compiled_input_sha256") == sha(captured["public_records.rs"]) and
            receipt.get("reconstructed_generated_sha256") == sha(captured["public_records.rs"]) and
            receipt.get("source_map_sha256") == sha((ROOT / "generated/source-map.json").read_bytes()),
            "AQ captured compiled record no longer matches reconstructed bytes/source map")
    rerun_sources = {
        "../../../src/bytes.rs": CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": ROOT / "extract_public.py",
    }
    expected_source_hashes = {literal:sha(path.read_bytes()) for literal,path in rerun_sources.items()}
    require(receipt.get("production_rerun_input_sha256") == expected_source_hashes,
            "AQ captured Cargo receipt does not bind the current selected production/extractor inputs")
    return {"captured_artifact_count":4, "fingerprint_inputs_exact":True,
        "build_output_exact":True, "root_output_route_exact":True,
        "compiled_public_records_hash":sha(captured["public_records.rs"])}


def assert_compiled_records(bundle: dict[str, Any]) -> dict[str, Any]:
    """Reconstruct the actual Cargo OUT_DIR public records and snapshot all four artifacts."""
    production = AP.AO.AN.CRATE_ROOT
    production_manifest = tomllib.loads((production / "Cargo.toml").read_text())
    require(production_manifest.get("package", {}).get("name") == "bytes" and
            production_manifest.get("package", {}).get("version") == "1.11.1" and
            production_manifest.get("package", {}).get("build") is False and
            production_manifest.get("lib", {}).get("path") == "src/lib.rs",
            "resolved production package/target is not bytes 1.11.1")
    build = (ROOT / "build.rs").read_bytes()
    extractor = (ROOT / "extract_public.py").read_bytes()
    require(sha(build) == "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
            "AQ build script/extractor sources changed")
    assert_aq_extractor_source_surface(extractor)
    rerun = AP.AO.AN.AL.assert_build_script_surface(build)
    expected_paths = ["../../../src/bytes.rs", "../../../src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs", "../../../src/bytes_mut.rs", "extract_public.py"]
    require(rerun == expected_paths, "AQ Cargo build rerun/input list changed")
    inputs = {
        "../../../src/bytes.rs": production / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": production / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": production / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": production / "src/bytes_mut.rs",
        "extract_public.py": ROOT / "extract_public.py",
    }
    input_hashes: dict[str, str] = {}
    for literal, expected in inputs.items():
        resolved = (ROOT / literal).resolve()
        require(resolved == expected.resolve() and resolved.is_file(),
                f"AQ build literal resolves to an unselected source: {literal}")
        input_hashes[literal] = sha(resolved.read_bytes())

    records = (production / "src/bytes/bytes_record.rs").read_text()
    vtables = (production / "src/bytes/vtable_record.rs").read_text()
    mutable = (production / "src/bytes_mut.rs").read_text()
    ai = AP.AO.AN.AI
    ai.audit_generated_extractions(bundle["production_source"], ROOT / "generated",
                                   records, vtables, mutable)
    shared = ai.extract_struct_source(mutable, "struct Shared {", "BytesMut::Shared")
    bytesmut = ai.extract_struct_source(mutable, "pub struct BytesMut {", "BytesMut")
    expected_record = (records + "\n" + vtables + "\nmod mutable_record {\n"
        "use alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        shared + "\n" + bytesmut + "\n}\nuse mutable_record::BytesMut;\n").encode()
    generated_path = ROOT / "generated/public_records.rs"
    source_map_path = ROOT / "generated/source-map.json"
    generated = generated_path.read_bytes()
    source_map = json.loads(source_map_path.read_text())
    require(generated == expected_record and
            source_map.get("generated/public_records.rs", {}).get("sha256") == sha(generated),
            "AQ translated public records do not reconstruct from selected production source")

    target_value = os.environ.get("CARGO_TARGET_DIR")
    if target_value:
        require(pathlib.Path(target_value).resolve() == AQ_CARGO_TARGET_DIR,
                "AQ Cargo target directory differs from the pinned proof environment")
    target = AQ_CARGO_TARGET_DIR
    fps = list((target / "debug/.fingerprint").glob(
        "bytes-original-shared-slice-views-*/run-build-script-build-script-build.json"))
    require(len(fps) == 1, "AQ requires exactly one selected Cargo build fingerprint")
    fingerprint_path = fps[0]
    fingerprint_bytes = fingerprint_path.read_bytes()
    fingerprint = json.loads(fingerprint_bytes)
    rerun_rows = [item["RerunIfChanged"] for item in fingerprint.get("local", [])
                  if isinstance(item, dict) and "RerunIfChanged" in item]
    require(len(rerun_rows) == 1 and rerun_rows[0].get("paths") == expected_paths,
            "AQ actual Cargo fingerprint does not bind the reviewed build/extractor inputs")
    output_rel = pathlib.Path(rerun_rows[0].get("output", ""))
    require(not output_rel.is_absolute() and output_rel.parts[:2] == ("debug", "build"),
            "AQ Cargo build output escaped selected target directory")
    output = (target / output_rel).resolve()
    out_dir = output.parent
    require(fingerprint_path.parent.name == out_dir.name and output.is_file(),
            "AQ fingerprint/build output identity mismatch")
    directives = (["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
                   "cargo:rustc-cfg=bytes_original_shared_gate"] +
                  ["cargo:rerun-if-changed=" + path for path in expected_paths])
    output_bytes = output.read_bytes()
    require(output_bytes == ("\n".join(directives) + "\n").encode(),
            "AQ actual Cargo build directives/configuration changed")
    root_output = out_dir / "root-output"
    compiled = out_dir / "out/public_records.rs"
    root_bytes = root_output.read_bytes()
    compiled_bytes = compiled.read_bytes()
    require(root_bytes == str(out_dir / "out").encode() and compiled_bytes == expected_record == generated,
            "AQ actual OUT_DIR public records differ from the independently reconstructed record")

    capture_dir = ROOT / "generated/compiled-inputs"
    capture_dir.mkdir(parents=True, exist_ok=True)
    artifacts = {"public_records.rs": compiled_bytes,
        "cargo-run-build-fingerprint.json": fingerprint_bytes,
        "cargo-build-output.txt": output_bytes,
        "cargo-root-output.txt": root_bytes}
    for name, data in artifacts.items():
        (capture_dir / name).write_bytes(data)
    receipt = {"status":"pass", "probe_package":"bytes-original-shared-slice-views",
        "build_script_sha256":sha(build), "extractor_sha256":sha(extractor),
        "cargo_target_dir":str(target), "cargo_build_fingerprint":str(fingerprint_path),
        "cargo_build_fingerprint_sha256":sha(fingerprint_bytes),
        "captured_cargo_build_fingerprint_path":"generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_cargo_build_fingerprint_sha256":sha(fingerprint_bytes),
        "build_output_path":str(output), "build_output_sha256":sha(output_bytes),
        "captured_build_output_path":"generated/compiled-inputs/cargo-build-output.txt",
        "captured_build_output_sha256":sha(output_bytes), "root_output_path":str(root_output),
        "root_output_sha256":sha(root_bytes),
        "captured_root_output_path":"generated/compiled-inputs/cargo-root-output.txt",
        "captured_root_output_sha256":sha(root_bytes), "actual_out_dir":str(compiled.parent),
        "compiled_input_path":str(compiled), "compiled_input_sha256":sha(compiled_bytes),
        "captured_input_path":"generated/compiled-inputs/public_records.rs",
        "captured_input_sha256":sha(compiled_bytes), "reconstructed_generated_sha256":sha(expected_record),
        "captured_actual_Cargo_artifact_count":4,
        "source_map_sha256":sha(source_map_path.read_bytes()), "production_rerun_input_sha256":input_hashes}
    receipt_path = capture_dir / "public-records-build-receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    require(json.loads(receipt_path.read_text()) == receipt,
            "AQ compiled-input receipt did not preserve its captured values")
    assert_compiled_capture(receipt)
    return {"actual_OUT_DIR_record_reconstructed":True,
        "captured_actual_Cargo_artifact_count":4, "receipt":receipt}


def assert_ap_copied_modules() -> dict[str, Any]:
    """Pin inherited AP modules and admit only the two selected AQ modules."""
    ap_names = {path.name for path in (AP_ROOT / "src").glob("*.rs")}
    aq_names = {path.name for path in (ROOT / "src").glob("*.rs")}
    added = aq_names - ap_names
    require(added == {"slice_extension.rs", "view_pointer.rs"},
            "AQ source module inventory differs from AP plus the selected slice/pointer modules")
    for name in sorted(ap_names - {"promotion.rs", "lib.rs"}):
        aq = ROOT / "src" / name
        require(aq.is_file() and aq.read_bytes() == (AP_ROOT / "src" / name).read_bytes(),
                f"AQ inherited proof/support module changed: {name}")
    parent_lib = (AP_ROOT / "src/lib.rs").read_text()
    selected_lib = (ROOT / "src/lib.rs").read_text()
    route = assert_lib_route(selected_lib, parent_lib)
    require(AQ_POINTER_SUPPORT.is_file(), "AQ generic view-pointer support module is missing")
    return {"AP_module_inventory": sorted(ap_names), "AQ_added_modules": sorted(added),
        "all_untransformed_AP_modules_byte_exact": True,
        "lib_route": "AP lib.rs plus exactly #[cfg(creusot)] mod view_pointer;",
        "lib_route_exact": route["route_exact"]}


def extract_rust_fn(source: str, name: str) -> str:
    """Extract one Rust function token span without being fooled by comments/strings."""
    masked = AP.AO.AN.AI.mask_noncode(source)
    matches = list(re.finditer(r"\b(?:pub\s+)?(?:unsafe\s+)?fn\s+" + re.escape(name) + r"\s*\(", masked))
    require(len(matches) == 1, f"expected exactly one production fn {name}")
    start = matches[0].start()
    opening_paren = masked.find("(", matches[0].start())
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
    require(close is not None, f"unterminated signature for production fn {name}")
    # In this selected production source the return types have no brace-bearing
    # where-clause. The first code brace after the closed parameter list opens
    # the body; comments and strings have already been blanked.
    opening = masked.find("{", close + 1)
    require(opening >= 0, f"production fn {name} has no body")
    depth = 0
    for pos in range(opening, len(masked)):
        if masked[pos] == "{":
            depth += 1
        elif masked[pos] == "}":
            depth -= 1
            if depth == 0:
                return source[start:pos + 1]
    raise CheckError(f"unterminated production fn {name}")


def extract_rust_fn_body(source: str, name: str) -> str:
    full = extract_rust_fn(source, name)
    masked = AP.AO.AN.AI.mask_noncode(full)
    opening = masked.find("{")
    require(opening >= 0, f"function {name} has no body")
    return full[opening:]


def rust_outer_attributes(source: str, name: str) -> list[list[str]]:
    """Read contiguous outer attrs for one fn, including `pub unsafe fn` forms."""
    masked = AP.AO.AN.AI.mask_noncode(source)
    matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\b", masked))
    require(len(matches) == 1, f"expected one function for attribute audit: {name}")
    pos = matches[0].start()
    line_start = source.rfind("\n", 0, pos) + 1
    current_line_prefix = source[line_start:pos]
    cleaned_line_prefix = re.sub(r"(?:#\[[^\]]*\]\s*)+", "", current_line_prefix)
    require(re.fullmatch(r"\s*(?:(?:pub(?:\([^)]*\))?)\s+)?(?:unsafe\s+)?", cleaned_line_prefix) is not None,
            f"unreviewed tokens precede function declaration {name}")
    declaration_prefix = re.sub(r"\s*(?:(?:pub(?:\([^)]*\))?)\s+)?(?:unsafe\s+)?$", "", current_line_prefix)
    prefix = source[:line_start] + declaration_prefix
    attrs: list[list[str]] = []
    while True:
        end = len(prefix.rstrip())
        if end == 0 or prefix[end - 1] != "]":
            break
        depth = 0
        opening = None
        for index in range(end - 1, -1, -1):
            if prefix[index] == "]":
                depth += 1
            elif prefix[index] == "[":
                depth -= 1
                if depth == 0:
                    opening = index
                    break
        require(opening is not None and opening > 0 and prefix[opening - 1] == "#",
                f"malformed outer attribute on function {name}")
        attrs.insert(0, AP.AO.AN.AI.rust_tokens(prefix[opening - 1:end]))
        prefix = prefix[:opening - 1]
    return attrs


VIEW_BODY_SHA256 = {
    "slice": "1420af7c361f60d3ce3e9b2ea76ecf551c56d73d6584c594829a1c93f021761c",
    "new_empty_with_ptr": "6ba74d7ccf1f9a1ac9c6619531001d21a72ae6449ee330a90bb6fbd2d98c0bea",
    "static_clone": "4a951a5be01616f20ef0671a8b2dcd03cebe8257235aa9a5ad1acd50e461a4da",
    "static_drop": "b2f98b87242bc5b5b76f73fc406b28af0d63d1282fe99f9232bb6c83fae91f37",
    "without_provenance": "030c522b824ef36fb9f6b37053784ec1e727ad6905af50e2cfbee01c21708e00",
}


def assert_native_view_source_inputs(source: str, source_map: dict[str, Any],
                                    native_bindings: str) -> dict[str, Any]:
    """Pure checker over selected production text plus reconstructed extractions."""
    view_names = tuple(VIEW_BODY_SHA256)
    bodies = {name: extract_rust_fn(source, name) for name in view_names}
    body_hashes = {name: sha(body.encode()) for name, body in bodies.items()}
    require(body_hashes == VIEW_BODY_SHA256,
            "selected actual Bytes slice/empty/static/pointer helper body changed")
    require(source_map.get("view_bodies") == VIEW_BODY_SHA256,
            "AQ extractor source-map view-body identities differ from the checker input pins")
    reconstructed = "\n\n".join(bodies[name] for name in view_names) + "\n"
    require(native_bindings == reconstructed,
            "AQ extracted native view bindings do not reproduce the actual selected production bodies")
    tokens = {name: AP.AO.AN.AI.rust_tokens(body) for name, body in bodies.items()}
    slice_tokens = tokens["slice"]
    for run in ("start_bound", "end_bound", "checked_add", "expect", "wrapping_add", "new_empty_with_ptr", "clone", "ptr", "add"):
        require(run in slice_tokens, f"actual Bytes::slice lost selected operation `{run}`")
    require("Bound" in slice_tokens and all(x in slice_tokens for x in ("Included", "Excluded", "Unbounded")),
            "actual Bytes::slice no longer handles the built-in Range Bound cases")
    require(slice_tokens.index("assert") < slice_tokens.index("if") < slice_tokens.index("wrapping_add") <
            slice_tokens.index("clone") < slice_tokens.index("add"),
            "actual Bytes::slice changed range-check / empty / nonempty operation order")
    empty = tokens["new_empty_with_ptr"]
    require("without_provenance" in empty and "STATIC_VTABLE" in empty and "len" in empty and "0" in empty,
            "actual empty constructor no longer detaches provenance and selects the Static representation")
    require("AtomicPtr" in empty and "null_mut" in empty,
            "actual empty constructor no longer installs a null atomic data field")
    require("from_raw_parts" in tokens["static_clone"] and "from_static" in tokens["static_clone"],
            "actual Static clone callback no longer reconstructs the actual slice and static Bytes")
    require(tokens["static_drop"].count("dealloc") == 0 and tokens["static_drop"].count("free") == 0,
            "actual Static drop callback gained a deallocation effect")
    require("null" in tokens["without_provenance"] and "wrapping_add" in tokens["without_provenance"],
            "actual without_provenance helper does not preserve only the raw address")
    as_slice_tokens = AP.AO.AN.AI.rust_tokens(extract_rust_fn(source, "as_slice"))
    require("from_raw_parts" in as_slice_tokens and "self" in as_slice_tokens and "ptr" in as_slice_tokens and
            "len" in as_slice_tokens,
            "selected native AsRef read path no longer constructs a slice from the actual Bytes ptr/len")
    trait_file = (ROOT / "generated/public_traits.rs").read_text()
    trait_tokens = AP.AO.AN.AI.rust_tokens(trait_file)
    require(" self . as_slice (" in " " + " ".join(trait_tokens) and
            "impl AsRef < [ u8 ] > for Bytes" in " ".join(trait_tokens),
            "selected native AsRef implementation does not dispatch through Bytes::as_slice")
    _, _, static_vtable = rust_item_span(source, "const", "STATIC_VTABLE")
    static_tokens = AP.AO.AN.AI.rust_tokens(static_vtable)
    require(all(x in static_tokens for x in ("clone", "static_clone", "drop", "static_drop")),
            "actual STATIC_VTABLE does not select the reviewed static callbacks")
    return {"actual_production_file": "src/bytes.rs",
        "selected_view_body_sha256": body_hashes,
        "generated_native_view_bindings_reconstructed": True,
        "built_in_Range_bound_handling": True,
        "empty_branch_uses_provenance_free_Static_constructor": True,
        "nonempty_branch_uses_clone_then_shortened_len_and_ptr_add": True,
        "actual_AsRef_dispatches_through_as_slice": True,
        "actual_as_slice_uses_native_ptr_and_len": True,
        "STATIC_VTABLE_binds_static_clone_drop": True,
        "empty_Static_drop_has_no_deallocation": True}


def assert_native_view_source() -> dict[str, Any]:
    """Bind generated extraction bodies back to the exact production input."""
    production = AP.AO.AN.CRATE_ROOT
    bytes_source_path = production / "src/bytes.rs"
    source_map_path = ROOT / "generated/source-map.json"
    native_bindings_path = ROOT / "generated/native_view_bindings.rs"
    require(bytes_source_path.is_file() and source_map_path.is_file() and native_bindings_path.is_file(),
            "AQ production/generated native view-source inputs are missing")
    report = assert_native_view_source_inputs(bytes_source_path.read_text(),
        json.loads(source_map_path.read_text()), native_bindings_path.read_text())
    report["actual_source_path"] = str(bytes_source_path)
    return report


def assert_active_composition(prefix: str, extension: str, client: str,
                              active: str, positive: str) -> dict[str, Any]:
    expected = prefix + "\n" + extension + client
    require(active == expected and positive == expected,
            "AQ active/positive source is not exactly transformed AP positive + view extension + selected client")
    return {"transformed_prefix_sha256": sha(prefix.encode()),
        "slice_extension_sha256": sha(extension.encode()),
        "client_sha256": sha(client.encode()), "active_sha256": sha(active.encode()),
        "active_composition_exact": True,
        "inherited_prefix_relation": "one OriginalSharedProof enum replacement; not byte-exact"}


def assert_mapping_sources(mapping: dict[str, Any], selected_prefix: str,
                           extension: str, client: str, active: str,
                           pointer_support: str) -> dict[str, Any]:
    """Bind source-map claims to live AP/AQ input files and the one enum edit."""
    parent_prefix = (AP_ROOT / "generated/positive.rs").read_text()
    transform = mapping.get("source_transform", {})
    ancestor_literal = "../original-shared-finite-owners-2026-10-09/generated/positive.rs"
    require(transform.get("ancestor_source") == ancestor_literal and
            (ROOT / ancestor_literal).resolve() == (AP_ROOT / "generated/positive.rs").resolve() and
            transform.get("ancestor_sha256") == AP_ACTIVE_SHA256 and
            transform.get("transformed_sha256") == sha(selected_prefix.encode()),
            "AQ mapping does not bind the selected single enum transformation to published AP positive")
    parent_enum = rust_item_span(parent_prefix, "enum", "OriginalSharedProof")[2]
    selected_enum = rust_item_span(selected_prefix, "enum", "OriginalSharedProof")[2]
    require(isinstance(transform.get("old"), str) and isinstance(transform.get("new"), str) and
            AP.AO.AN.AI.rust_tokens(transform["old"]) == AP.AO.AN.AI.rust_tokens(parent_enum) and
            AP.AO.AN.AI.rust_tokens(transform["new"]) == AP.AO.AN.AI.rust_tokens(selected_enum),
            "AQ mapping's old/new enum transformation literals do not match actual AP/AQ source")
    expected_hashes = {"base_source_sha256": sha(selected_prefix.encode()),
        "selected_prefix_sha256": sha(selected_prefix.encode()),
        "extension_sha256": sha(extension.encode()),
        "client_sha256": sha(client.encode()), "active_sha256": sha(active.encode())}
    require(mapping.get("base_source") == "src/promotion.rs" and
            mapping.get("extension_source") == "src/slice_extension.rs" and
            mapping.get("active") == "generated/active.rs" and
            mapping.get("feature") == "" and mapping.get("status") == "generated_unchecked" and
            mapping.get("stage") == "after-ElaborateDrops" and
            mapping.get("terminal_helpers_sha256") == sha(extension.encode()) and
            all(mapping.get(key) == value for key, value in expected_hashes.items()),
            "AQ mapping source paths/hashes/default feature/stage do not match the selected source files")
    require(mapping.get("pointer_support") == {
        "path": "src/view_pointer.rs", "sha256": sha(pointer_support.encode())},
        "AQ mapping does not bind its generic view pointer support module")
    require(mapping.get("native_alpha_renaming") == {"slice":{"begin":"view_begin"}},
            "AQ proof-local native slice identifier mapping is not exactly begin -> view_begin")
    require(mapping.get("helpers") == ["bytes_root_detaching_terminal_drop", "bytes_view_terminal_drop"] and
            mapping.get("callbacks") == ["shared_view_clone_checked", "child_drop_checked", "static_view_drop_checked"] and
            mapping.get("view") == "full original allocation authority; separate absolute view offset; unbound provenance-free Empty" and
            mapping.get("return_evaluation") == {"shadow":"let saved_return=observed","before_selected_drop":True} and
            mapping.get("excluded") == ["invalid-range panic and unwind", "arbitrary RangeBounds implementations",
                "arbitrary concurrent closure", "whole crate"] and
            mapping.get("tcb") == ["native compiler/MIR normal terminal-place elaboration",
                "address nonobservation and no independent Bytes field-drop glue",
                "generic live ptr.add and bounded wrapping_add metadata",
                "native provenance-free empty pointer metadata",
                "inherited generic pointer/field/physical/erased-callback boundaries"],
            "AQ mapping helper/callback/scope/limitation statements changed")
    require(mapping.get("debug_places") == EXPECTED_DEBUG_PLACES,
            "AQ generator debug-place inventory differs from the selected native client")
    return {"source_paths_and_hashes_exact": True, "enum_transform_exact": True,
        "native_alpha_renaming": {"begin":"view_begin"}}


EXPECTED_CLIENT_EDGES = [
    {"block":"bb2","place":"_8","owner":"original","successor":"bb3","unwind":"bb18","repeated":False,"scope":"scope"},
    {"block":"bb5","place":"_11","owner":"first","successor":"bb6","unwind":"bb15","repeated":False,"scope":"detached"},
    {"block":"bb6","place":"_7","owner":"owner","successor":"bb7","unwind":"bb18","repeated":False,"scope":"detached"},
    {"block":"bb10","place":"_6","owner":"selected","successor":"bb11","unwind":"bb18","repeated":False,"scope":"detached"},
]
EXPECTED_DEBUG_PLACES = {"input":"_1","a":"_2","b":"_3","c":"_4","d":"_5",
    "selected":"_6","observed":"_20","owner":"_7","first":"_11","original":"_8"}


def assert_native_mapping(mapping: dict[str, Any], native: dict[str, Any]) -> dict[str, Any]:
    """Join generator mapping to independently parsed MIR client/range facts."""
    native_view = native.get("native_audit", {})
    edges = native.get("client", {}).get("normal_edges")
    require(mapping.get("native_mir_ready") is True and
            native_view.get("slice_mir", {}).get("normal_cfg_exact") is True,
            "AQ selected nested-slice native MIR is not ready or failed its exact control-flow gate")
    require(mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == sha((ROOT / "native.rs").read_bytes()) and
            mapping.get("native_client_mir") == "native-mir/bytes_shared_slice_views_native.nested_slice_scope.2-2-004.ElaborateDrops.after.mir" and
            pathlib.Path(native.get("paths_and_capture", {}).get("resolved_capture_inputs", {}).get("native_source", "")).resolve() == (ROOT / "native.rs").resolve() and
            pathlib.Path(native.get("mir_paths", {}).get("selected", {}).get("client", "")).resolve() ==
                (ROOT / mapping.get("native_client_mir", "")).resolve(),
            "AQ generator's selected native client/source paths or source SHA do not match native capture")
    require(mapping.get("normal_edges") == edges,
            "AQ native CFG checker and selected source mapping disagree on actual client call/drop edges")
    require(edges == EXPECTED_CLIENT_EDGES and native.get("client", {}).get("normal_cfg_exact") is True and
            native.get("client", {}).get("range_specialization", {}).get("native_type") == "std::ops::Range<usize>",
            "AQ selected client MIR drop order or built-in Range specialization changed")
    roles = native.get("client", {}).get("local_roles", {})
    require(roles == {"Original":"_8","Owner":"_7","First":"_11","Selected":"_6","Observed":"_20"} and
            mapping.get("debug_places") == EXPECTED_DEBUG_PLACES and
            native.get("client", {}).get("selected_read_and_saved_return") == {
                "as_ref":"bb7","to_vec":"bb8","result_saved":"bb9","selected_drop":"bb10"} and
            native.get("client", {}).get("normal_completion_only") is True,
            "AQ native local roles, selected read/save/drop order or normal-return-only boundary changed")
    slice_mir = native_view.get("slice_mir", {})
    require(slice_mir.get("source_rangebounds_match_order_checked") is True and
            slice_mir.get("excluded_start_and_included_end_checked_add_calls") == 2 and
            slice_mir.get("bounds_assertions") == [
                "begin <= end (bb16/bb17, failure bb18)",
                "end <= len (bb17/bb22, failure bb23)"],
            "AQ native slice MIR did not preserve the checked bound and assertion order")
    require(slice_mir.get("empty_branch") == {"condition":"end == begin (bb22)","edge":"bb27",
            "pointer_op":"wrapping_add(begin)","constructor":"new_empty_with_ptr","clone":False} and
            slice_mir.get("nonempty_branch") == {"edge":"bb28","clone":"self.clone()",
            "length":"end - begin","pointer_op":"ret.ptr.add(begin)","empty_constructor":False},
            "AQ native MIR empty/nonempty slice branches or pointer operations changed")
    require(native_view.get("empty_view", {}).get("empty_has_no_ticket_or_allocation_authority_claim") is True and
            native_view.get("empty_view", {}).get("empty_native_fields") == {
                "len":0,"atomic_data":"null_mut","vtable":"STATIC_VTABLE"},
            "AQ native Empty representation facts or no-authority boundary changed")
    require(native.get("native_mir", {}).get("selected_mir_count_including_client") == 25 and
            native.get("native_mir", {}).get("selected_production_mir_count") == 24 and
            native.get("native_mir", {}).get("selected_headers_unique") is True,
            "AQ source/native audit did not pin the complete unique selected MIR inventory")
    return {"normal_client_edges_exact": True, "nested_range_cfg_exact": True,
        "empty_nonempty_pointer_paths_exact": True, "selected_mir_count": 25}


def assert_view_extension(extension: str, pointer_support: str,
                          client: str) -> dict[str, Any]:
    """Pin the checked view sum and its generic pointer TCB boundary."""
    require(sha(extension.encode()) == AQ_EXTENSION_SHA256,
            "AQ proof-side view/empty extension differs from the reviewed source")
    require(sha(pointer_support.encode()) == AQ_POINTER_SUPPORT_SHA256,
            "AQ generic pointer adapter module differs from the reviewed TCB source")
    require(sha(client.encode()) == AQ_CLIENT_SHA256,
            "AQ selected nested-slice proof client differs from the reviewed source")
    ai = AP.AO.AN.AI
    extension_tokens = ai.rust_tokens(extension)
    masked = ai.mask_noncode(extension)
    functions = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", masked)
    expected_functions = ["static_view_table", "static_view_table_reification", "bounded_view",
        "view_owned", "view_bound", "view_valid", "view_content", "view_id", "view_fraction",
        "view_public", "view_accepts", "new_empty_view", "shallow_clone_view_checked",
        "shared_view_clone_checked", "shared_view_clone_registration", "clone_shared_view",
        "slice_view", "read_view", "reclaimed", "valid", "was_static",
        "static_view_drop_checked", "static_view_drop_registration", "bytes_view_terminal_drop"]
    require(functions == expected_functions,
            "AQ proof extension added, removed or reordered a helper/callback declaration")
    trusted_names = []
    for name in functions:
        attrs = rust_outer_attributes(extension, name)
        if any("trusted" in attr for attr in attrs):
            trusted_names.append(name)
    require(trusted_names == ["static_view_table_reification", "shared_view_clone_registration",
                              "static_view_drop_registration"],
            "AQ proof extension changed its explicitly reviewed callback/reification trust boundary")
    for name in ("new_empty_view", "shallow_clone_view_checked", "shared_view_clone_checked",
                 "clone_shared_view", "slice_view", "read_view", "static_view_drop_checked",
                 "bytes_view_terminal_drop"):
        attrs = rust_outer_attributes(extension, name)
        require(all("trusted" not in attr for attr in attrs),
                f"body-proved AQ helper gained trust: {name}")
    empty_struct = ai.rust_tokens(rust_item_span(extension, "struct", "EmptyViewProof")[2])
    require(empty_struct == ai.rust_tokens("struct EmptyViewProof { bound:raw_vec::BoundPtr, binding:Ghost<pointer_event::ReadOnlyPointer>, }"),
            "EmptyViewProof contains a live ticket, allocation authority or extra resource field")
    require("p . bound @ == None" in " ".join(extension_tokens),
            "AQ Empty view does not retain unbound BoundPtr metadata")
    for text, label in (("OriginalSharedProof::Empty", "Empty sum case"),
                         ("OriginalSharedProof::View", "View sum case"),
                         ("without_provenance", "provenance-free Empty constructor"),
                         ("wrapping_bounded", "empty address-only offset"),
                         ("add_live", "nonempty live pointer offset"),
                         ("pointer.add", "actual nonempty pointer addition"),
                         ("pointer.wrapping_add", "actual empty pointer addition"),
                         ("borrow_empty", "zero-length physical read"),
                         ("physical_projection::borrow", "live nonempty physical read"),
                         ("static_view_drop_registration", "Static Drop dispatch"),
                         ("shared_view_clone_registration", "Shared clone dispatch"),
                         ("child_drop_registration", "Shared child Drop dispatch"),
                         ("result.view_content", "subsequence content projection")):
        require(text in extension or text in pointer_support,
                f"AQ extension lost selected semantic operation: {label}")
    require("ticket:" not in " ".join(empty_struct) and "physical:" not in " ".join(empty_struct) and
            "Completion" not in " ".join(empty_struct) and "FreeReceipt" not in " ".join(empty_struct),
            "Empty proof carries hidden owner or physical/free authority")
    require("p . binding . inner_logic ( ) . value ( ) == crate :: view_pointer :: null_word ( )" in
            " ".join(extension_tokens) and "view_pointer :: null_pointer ( )" in " ".join(extension_tokens) and
            "new_pointer ( null" in " ".join(extension_tokens) and "bind_read_only" in extension,
            "Empty readonly field binding is not tied to the exact reified native null pointer")
    require("OriginalSharedProof::Empty(_) => Seq::empty()" in extension or
            "OriginalSharedProof::Empty(_)=>Seq::empty()" in extension,
            "Empty view content is not exactly empty")

    pointer_names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", ai.mask_noncode(pointer_support))
    require(pointer_names == ["shifted", "add_live", "wrapping_bounded", "without_provenance",
                              "null_word", "null_pointer"],
            "AQ generic pointer module added or changed callable surface")
    pointer_trusted = [name for name in pointer_names
        if any("trusted" in attr for attr in rust_outer_attributes(pointer_support, name))]
    require(pointer_trusted == ["add_live", "wrapping_bounded", "without_provenance", "null_pointer"],
            "AQ generic pointer TCB trust surface changed")
    require("pointer . add ( count )" in " ".join(ai.rust_tokens(extract_rust_fn_body(pointer_support, "add_live"))) and
            "pointer . wrapping_add ( count )" in " ".join(ai.rust_tokens(extract_rust_fn_body(pointer_support, "wrapping_bounded"))) and
            "null :: < u8 > ( ) . wrapping_add ( pointer as usize )" in
                " ".join(ai.rust_tokens(extract_rust_fn_body(pointer_support, "without_provenance"))),
            "AQ generic pointer adapter no longer maps add/wrapping/no-provenance to exact native operations")
    null_body = " ".join(ai.rust_tokens(extract_rust_fn_body(pointer_support, "null_pointer")))
    null_attrs = rust_outer_attributes(pointer_support, "null_pointer")
    require(null_body == "{ core :: ptr :: null_mut ( ) }" and
            null_attrs == [ai.rust_tokens("#[trusted]"),
                ai.rust_tokens("#[ensures(result==null_word() && result.is_null_logic())]")],
            "AQ native null-word reifier is not the exact explicit std-null TCB boundary")
    require(rust_outer_attributes(pointer_support, "null_word") ==
            [ai.rust_tokens("#[logic(opaque)]")],
            "AQ null-word logic symbol lost its single opaque definition")
    client_tokens = ai.rust_tokens(client)
    require(client_tokens.count("slice_view") == 2 and client_tokens.count("read_view") == 1 and
            "bytes_view_terminal_drop" in client_tokens and "saved_return" in client_tokens,
            "AQ generated proof client no longer contains the exact nested view/drop trace")
    proof_slice_tokens = ai.rust_tokens(extract_rust_fn(extension, "slice_view"))
    native_slice_tokens = ai.rust_tokens(extract_rust_fn(
        (AP.AO.AN.CRATE_ROOT / "src/bytes.rs").read_text(), "slice"))
    require("begin" not in proof_slice_tokens and "view_begin" in proof_slice_tokens and
            native_slice_tokens.count("begin") > 0,
            "AQ proof slice helper must use only the explicit native `begin` -> `view_begin` alpha rename")
    require(proof_slice_tokens.count("checked_add") == 2 and
            proof_slice_tokens.count("assert") == 2 and
            proof_slice_tokens.index("start_bound") < proof_slice_tokens.index("checked_add") <
                proof_slice_tokens.index("end_bound") < proof_slice_tokens.index("assert") <
                proof_slice_tokens.index("wrapping_bounded") < proof_slice_tokens.index("new_empty_view") <
                proof_slice_tokens.index("clone_shared_view") < proof_slice_tokens.index("add_live"),
            "AQ proof slice helper changed the selected Range checks or empty/nonempty operation order")
    return {"extension_sha256": AQ_EXTENSION_SHA256,
        "pointer_support_sha256": AQ_POINTER_SUPPORT_SHA256,
        "client_sha256": AQ_CLIENT_SHA256,
        "extension_functions": functions, "explicit_trusted_extension_items": trusted_names,
        "empty_view_has_no_ticket_or_physical_authority": True,
        "generic_pointer_tcb_calls": pointer_trusted,
        "client_nested_slice_and_drop_trace_checked": True}


def audit() -> dict[str, Any]:
    ap, lineage = preflight_published_ap()
    manifest = assert_probe_manifest()
    assert_generator_source((ROOT / "elaborate.py").read_bytes())
    copied = assert_ap_copied_modules()
    parent_prefix = (AP_ROOT / "generated/positive.rs").read_text()
    selected_prefix = AQ_PREFIX.read_text()
    require(MAPPING.is_file() and ACTIVE.is_file() and AQ_EXTENSION.is_file() and AQ_CLIENT.is_file(),
            "AQ generated active/mapping or frozen extension/client source is missing")
    mapping = json.loads(MAPPING.read_text())
    variants = tuple(name for name, _ in enum_variant_rows(
        rust_item_span(selected_prefix, "enum", "OriginalSharedProof")[2]))
    prefix_relation = assert_enum_prefix_transformation(parent_prefix, selected_prefix, variants)

    extension = AQ_EXTENSION.read_text()
    client = AQ_CLIENT.read_text()
    active = ACTIVE.read_text()
    positive_path = ROOT / "generated/positive.rs"
    require(positive_path.is_file(), "AQ immutable positive active source is missing")
    positive = positive_path.read_text()
    source_mapping = assert_mapping_sources(mapping, selected_prefix, extension, client, active,
                                            AQ_POINTER_SUPPORT.read_text())
    composition = assert_active_composition(selected_prefix, extension, client, active, positive)
    require(sha(active.encode()) == AQ_ACTIVE_SHA256,
            "AQ selected positive active source differs from the reviewed source pin")
    view_extension = assert_view_extension(extension, AQ_POINTER_SUPPORT.read_text(), client)
    require((ROOT / "generated/slice-extension.rs").read_text() == extension and
            (ROOT / "generated/terminal-helper.rs").read_text() == extension and
            (ROOT / "generated/elaborated-client.rs").read_text() == client,
            "AQ generated extension/terminal/client files differ from their selected sources")
    native_view_source = assert_native_view_source()

    native_checker_path = ROOT / "check_native.py"
    require(len(AQ_NATIVE_CHECKER_SHA256) == 64,
            "AQ native checker source pin is not finalized")
    native_controls = ROOT / "native-check-controls.py"
    native_controls_manifest = ROOT / "fixtures/native-check-controls.json"
    native_controls_receipt = ROOT / "generated/native-check-controls.json"
    require(sha(native_controls.read_bytes()) == AQ_NATIVE_CONTROLS_SHA256 and
            sha(native_controls_manifest.read_bytes()) == AQ_NATIVE_CONTROLS_MANIFEST_SHA256 and
            sha(native_controls_receipt.read_bytes()) == AQ_NATIVE_CONTROLS_RECEIPT_SHA256,
            "AQ native correspondence control source/fixture/receipt changed")
    native_control_data = json.loads(native_controls_receipt.read_text())
    require(native_control_data.get("schema") == "aq-native-source-mir-controls-v1" and
            native_control_data.get("control_count") == 47 and native_control_data.get("rejected") == 47 and
            native_control_data.get("accepted") == [] and
            native_control_data.get("checker_sha256") == AQ_NATIVE_CHECKER_SHA256,
            "AQ native correspondence controls did not reject all 47 selected-source/MIR mutations")
    native_module = import_pinned("aq_pinned_native_correspondence", native_checker_path,
                                  AQ_NATIVE_CHECKER_SHA256)
    bundle = native_module.load_bundle()
    compiled = assert_compiled_records(bundle)
    native = native_module.audit_bundle(bundle)
    require(native.get("status") == "pass", "AQ independent native source/MIR correspondence checker rejected")
    native_view = native.get("native_audit", {})
    require(native_view.get("source_body_hashes") == VIEW_BODY_SHA256 and
            native_view.get("range_specialization", {}).get("selected_builtin_type") == "Range<usize>",
            "AQ native checker did not certify the selected actual Range source bodies")
    require(native_view.get("view_bindings", {}).get("generated_reconstructed") is True and
            native_view.get("view_bindings", {}).get("source_map_reconstructed") is True,
            "AQ native checker did not reconstruct extracted native view-helper inputs")
    require(native_view_source["selected_view_body_sha256"] == native_view["source_body_hashes"],
            "AQ source gate and native checker disagree on exact production view-body sources")
    native_mapping = assert_native_mapping(mapping, native)
    return {"status": "pass", "checker_scope": "AQ selected actual built-in Range nested-slice source/native correspondence",
        "ancestor_pins": lineage, "inherited_AP_modules": copied,
        "proof_enum_transformation": prefix_relation, "active_composition": composition,
        "pointer_support": {"source_sha256": sha(AQ_POINTER_SUPPORT.read_bytes()),
                            "mapping_exact": True},
        "view_extension": view_extension,
        "native_view_source": native_view_source,
        "probe_manifest": manifest, "compiled_inputs": compiled,
        "generator_sha256": AQ_GENERATOR_SHA256,
        "source_mapping": source_mapping, "native_mapping": native_mapping,
        "native_audit": native, "mapping_sha256": sha(MAPPING.read_bytes()),
        "limits": ["No full original Bytes API admission.",
            "Only built-in Range<usize> with the stated valid-range preconditions; arbitrary RangeBounds and invalid-range unwind are excluded.",
            "Concurrent operations and unwind completion are excluded.",
            "Generic Rust pointer-provenance, pointer arithmetic and compiler MIR interpretations remain the explicitly listed TCB."]}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", type=pathlib.Path, default=ACTIVE)
    parser.add_argument("--mapping", type=pathlib.Path, default=MAPPING)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        require(args.shadow.resolve() == ACTIVE.resolve(), "shadow override must select AQ generated/active.rs")
        require(args.mapping.resolve() == MAPPING.resolve(), "mapping override must select AQ generated/mapping.json")
        result = audit()
    except Exception as exc:
        result = {"status": "reject", "checker_scope": "AQ nested slice-view source/native correspondence",
                  "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
