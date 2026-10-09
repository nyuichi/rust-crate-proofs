#!/usr/bin/env python3
"""Read-only native source/MIR gate for the AZ arbitrary-step Root Clone loop.

This checker reads pinned source, MIR, capture receipts, and the published AY operational
baseline without replaying old audits. It runs no compiler, Cargo, proof tool, or solver. Its result is a
correspondence witness for the captured default-native path, not a MIR
adequacy or Rust memory-model proof.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import pathlib
import re
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
PROBES = ROOT.parent
AV_PROBE = PROBES / "original-promotable-suffix-promotion-2026-10-09"
AV_CHECKER = AV_PROBE / "check_native.py"
AV_CHECKER_SHA256 = "f1cfaf1ac60107643e936d22a098c06671ac6f2b0178b9cb6354444f5f4d31df"
MIR_DIR = ROOT / "native-mir"
CAPTURE_PATH = MIR_DIR / "capture.json"

EXPECTED_CAPTURE_SHA256 = 'bf91d4d278b8dacb2ae42e7cf13293d5a6903c8db4ab13280162412acfe4d1cd'
EXPECTED_NATIVE_SOURCE_SHA256 = 'b3efb9e686f4fe656496933a2085768a0406ae942e9d06e900b968e3fc865118'
EXPECTED_TEST_SOURCE_SHA256 = '5f49e3357f917ec8a9357543f442386e112b932745bb5c733bbd6c006fcf9012'
EXPECTED_CAPTURE_SCRIPT_SHA256 = '637cbcd7db2df4b3f8a2d9aa04801d9e69dfff7c8c65cb832adddc58d82b04a3'
EXPECTED_NATIVE_MANIFEST_SHA256 = 'cb2c0d416e9c966f86d94baa5f92eba0f366741e32c2f64ef9976f33da0e5ef7'
EXPECTED_NATIVE_LOCK_SHA256 = '76aea639291e21f497cf627d36080c571e41cf7dd31e25e52ce52063693d40da'
EXPECTED_NATIVE_LOG_SHA256 = '2111cde1afb13952edf5c66850510ac61600a07944762084a1ae9ff04a5eb381'
EXPECTED_FIELD_SOURCE_SHA256 = "b8dae751e6c7dc007de636ed7cc8d9b6f98cc746252d7373e785778655b58183"
EXPECTED_FIELD_LOG_SHA256 = "3849089883705a9502911036f6510d56a263520626d4fc5ebc023aaa0530c436"
EXPECTED_CLIENT_MIR_SHA256 = '9a5dc6f05100ca0d9a22ccbdb9b023dd3cc52855d84f893b1789b50d74811f44'
EXPECTED_PRODUCTION_SOURCE_SHA256 = "95789896965446187ecd5cc7bf52f447435fcdc40f4e19c0c49c5a59b22495fd"
EXPECTED_PRODUCTION_MANIFEST_SHA256 = "4c2a59a19d9d8fc05c8be9610d0687d99107498a5981756439e2b39eb876c24a"
EXPECTED_REVIEWED_INPUTS_SHA256 = "e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306"

EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\ncommit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\nhost: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\nLLVM version: 22.1.7\n")
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"
EXPECTED_NATIVE_SOURCE = 'use bytes::{Buf, Bytes};\n\npub fn root_clone_steps(input: Box<[u8]>, steps: &[usize]) -> Vec<u8> {\n    let mut value = Bytes::from(input);\n    let mut index = 0usize;\n    while index < steps.len() {\n        let amount = core::cmp::min(steps[index], value.len());\n        value.advance(amount);\n        {\n            let _peer = value.clone();\n        }\n        index += 1;\n    }\n    value.chunk().to_vec()\n}\n'
EXPECTED_TEST_SOURCE = 'use bytes_root_phase_clone_native::root_clone_steps;\n\n#[test]\nfn arbitrary_step_samples_return_exact_capped_suffix() {\n    let mut cases = 0usize;\n    for len in [1usize, 2, 3, 7, 16, 257] {\n        let input: Vec<u8> = (0..len).map(|i| (i.wrapping_mul(37) % 256) as u8).collect();\n        let patterns = [\n            vec![], vec![0], vec![1], vec![len], vec![len + 1], vec![usize::MAX],\n            vec![0, 0, 0], vec![1, 1, 1], vec![len, 0, usize::MAX],\n            vec![usize::MAX, usize::MAX], vec![0, len, 0], vec![1, usize::MAX, 1, 0],\n            vec![0; 64], vec![1; len + 2], vec![len / 2, len / 2, 1],\n        ];\n        for steps in patterns {\n            let mut offset = 0usize;\n            for step in &steps { offset += (*step).min(len - offset); }\n            let actual = root_clone_steps(input.clone().into_boxed_slice(), &steps);\n            assert_eq!(actual, input[offset..], "len={len}, steps={steps:?}");\n            cases += 1;\n        }\n    }\n    assert_eq!(cases, 90);\n    println!("AZ native step-pattern cases: {cases}");\n}\n'

EXPECTED_NATIVE_MANIFEST = {
    "package": {"name": "bytes-root-phase-clone-native", "version": "0.0.0", "edition": "2021"},
    "workspace": {},
    "lib": {"name": "bytes_root_phase_clone_native", "path": "../native.rs"},
    "dependencies": {"bytes": {"path": "../../../../"}},
}

EXPECTED_LABELS = {
    "from_box", "clone_impl", "cleanup", "bytes_drop", "as_ref", "as_slice",
    "promotable_even_clone", "promotable_odd_clone", "shallow_clone_vec",
    "shallow_clone_arc", "shared_clone", "slice", "new_empty_with_ptr",
    "static_clone", "static_drop", "without_provenance", "inc_start", "remaining",
    "chunk", "advance", "len", "ref_count_increment", "free_boxed_slice",
    "promotable_even_drop", "promotable_odd_drop", "shared_drop", "release_shared",
    "free_shared", "ptr_map", "atomic_with_mut",
}


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_av_checker():
    require(AV_CHECKER.is_file() and sha(AV_CHECKER.read_bytes()) == AV_CHECKER_SHA256,
            "published AV native checker changed before AZ import")
    spec = importlib.util.spec_from_file_location("ax_pinned_av_native", AV_CHECKER)
    require(spec is not None and spec.loader is not None, "cannot load pinned AV native checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


AV = load_av_checker()


def load_bundle() -> dict[str, Any]:
    raw = CAPTURE_PATH.read_bytes()
    capture = json.loads(raw)
    mir_sources: dict[str, str] = {}
    for row in capture.get("selected", []):
        mir_sources[row["label"]] = (ROOT / row["path"]).read_text()
    mapping_path = ROOT / "generated/mapping.json"
    return {
        "av_base": AV.load_bundle(),
        "capture": capture,
        "capture_raw": raw,
        "mir_sources": mir_sources,
        "native_source": (ROOT / "native.rs").read_text(),
        "native_test_source": (ROOT / "native-test/tests/root_clone_witness.rs").read_text(),
        "native_test_log": (ROOT / "native-test/native-run.log").read_text(),
        "native_manifest": (ROOT / "native-test/Cargo.toml").read_text(),
        "native_lock": (ROOT / "native-test/Cargo.lock").read_text(),
        "capture_script": (ROOT / "capture-native.sh").read_text(),
        "rustc_text": (MIR_DIR / "rustc-version.txt").read_text(),
        "cargo_text": (MIR_DIR / "cargo-version.txt").read_text(),
        "field_source": (ROOT / "native-field-profile.rs").read_text(),
        "field_log": (ROOT / "native-field-profile.log").read_text(),
        "production_source": (CRATE_ROOT / "src/bytes.rs").read_text(),
        "production_manifest": (CRATE_ROOT / "Cargo.toml").read_text(),
        "production_reviewed_inputs": (ROOT / "reviewed-production-inputs.json").read_text(),
        "mapping": json.loads(mapping_path.read_text()) if mapping_path.is_file() else None,
    }


def significant(body: str) -> list[str]:
    return [line.strip() for line in body.splitlines()
            if line.strip() and not line.strip().startswith(("StorageLive(", "StorageDead(", "debug ", "PlaceMention("))]


def block_rows(blocks: dict[tuple[int, bool], str]) -> list[dict[str, Any]]:
    return [{"block": f"bb{number}", "cleanup": cleanup,
             "body_sha256": sha(("\n" + body).encode())}
            for (number, cleanup), body in sorted(blocks.items())]


def extract_item(source: str, signature: str, label: str) -> str:
    # The entire source file is hash-pinned. Still extract the executable item
    # from its unique signature and balanced body instead of trusting a receipt.
    starts = [m.start() for m in re.finditer(re.escape(signature), source)]
    require(len(starts) == 1, f"expected one source item {label}, found {len(starts)}")
    start = starts[0]
    opening = source.find("{", start + len(signature))
    require(opening >= 0, f"source item {label} has no body")
    depth = 0
    in_string = False
    escaped = False
    for index in range(opening, len(source)):
        char = source[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return source[start:index + 1]
    raise AuditError(f"source item {label} has an unterminated body")


def has_in_order(body: str, fragments: list[str], label: str) -> None:
    cursor = 0
    for fragment in fragments:
        index = body.find(fragment, cursor)
        require(index >= 0, f"{label} missing or reorders source operation {fragment!r}")
        cursor = index + len(fragment)


def assert_block_facts(mir: str, function_name: str,
                       expected: dict[tuple[int, bool], list[str]],
                       label: str, *, exact_key_set: set[tuple[int, bool]] | None = None) -> dict[tuple[int, bool], str]:
    blocks = AV.AT.blocks_of(mir, function_name)
    if exact_key_set is not None:
        require(set(blocks) == exact_key_set, f"{label} basic-block/cleanup inventory changed")
    for key, statements in expected.items():
        require(key in blocks and significant(blocks[key]) == statements,
                f"{label} bb{key[0]} contains missing, extra, or reordered operations")
    return blocks


def audit_ancestor(data: dict[str, Any]) -> dict[str, Any]:
    raw = (PROBES / "original-root-phase-view-2026-10-09/generated/mapping.json").read_bytes()
    require(sha(raw) == "7865230973c2672cb3b56538be2adac72579dc7ebd0ed805b519a4aee5c332d8", "published AY operational baseline mapping changed")
    baseline = json.loads(raw)["native_report"]
    require(baseline["status"] == "pass" and baseline["capture"]["capture_sha256"] ==
            "42c8f72252f1dcf5a0f5b89853a2985498171def8ac4f12662d01e2e62bb42d6",
            "published AY operational baseline is not the audited capture")
    return baseline["ancestor_av"]


def audit_capture(data: dict[str, Any], ancestor: dict[str, Any]) -> dict[str, Any]:
    capture = data["capture"]
    require(sha(data["capture_raw"]) == EXPECTED_CAPTURE_SHA256,
            "AZ native capture receipt hash changed")
    require(capture.get("stage") == "2-2-004.ElaborateDrops.after.mir" and
            capture.get("rustc_version") == EXPECTED_RUSTC.rstrip("\n") and
            data["rustc_text"] == EXPECTED_RUSTC and
            capture.get("cargo_version") == EXPECTED_CARGO.rstrip("\n") and
            data["cargo_text"] == EXPECTED_CARGO,
            "AZ MIR stage or pinned Rust/Cargo version changed")
    script = data["capture_script"]
    require(sha(script.encode()) == EXPECTED_CAPTURE_SCRIPT_SHA256 and
            "cargo test --locked --offline" in script and
            "cargo rustc --locked --offline" in script and
            "-Zdump-mir=all" in script and "-Zmir-opt-level=0" in script and
            "why3" not in script.lower() and "creusot verify" not in script.lower(),
            "AZ native capture script changed or invokes proof tooling")
    require(sha(data["native_manifest"].encode()) == EXPECTED_NATIVE_MANIFEST_SHA256 and
            tomllib.loads(data["native_manifest"]) == EXPECTED_NATIVE_MANIFEST and
            sha(data["native_lock"].encode()) == EXPECTED_NATIVE_LOCK_SHA256,
            "AZ native test package or lockfile changed")

    rows = capture.get("selected", [])
    labels = {row.get("label") for row in rows}
    require(len(rows) == 31 and len(labels) == 31 and labels == EXPECTED_LABELS | {"client"},
            "AZ capture must contain 30 selected production bodies and one client")
    require(set(data["mir_sources"]) == labels, "loaded AZ MIR labels differ from capture inventory")
    av_mirs = data["av_base"]["mir_sources"]
    current_production = {label: body for label, body in data["mir_sources"].items() if label != "client"}
    av_production = {label: body for label, body in av_mirs.items() if label != "client"}
    require(len(av_production) == 29 and len(current_production) == 30 and
            all(current_production.get(label) == body for label, body in av_production.items()) and
            current_production.get("free_boxed_slice") is not None,
            "29 inherited AV production MIR bodies are not byte-exact or AZ free helper is missing")

    ax_capture_raw = (PROBES / "original-root-phase-view-2026-10-09/native-mir/capture.json").read_bytes()
    require(sha(ax_capture_raw) == "42c8f72252f1dcf5a0f5b89853a2985498171def8ac4f12662d01e2e62bb42d6",
            "published AY capture authority changed")
    ax_rows = json.loads(ax_capture_raw)["selected"]
    ax_production = {row["label"]: row["sha256"] for row in ax_rows if row["label"] != "client"}
    require(set(current_production) == set(ax_production) and
            all(sha(body.encode()) == ax_production[label] for label, body in current_production.items()),
            "AZ production MIR inventory is not exact published AY thirty bodies")
    selected_paths = {}
    for row in rows:
        rel = pathlib.PurePosixPath(row.get("path", ""))
        require(not rel.is_absolute() and ".." not in rel.parts and rel.parts[0] == "native-mir" and
                len(rel.parts) == 2, f"MIR path escapes native-mir: {row.get('label')}")
        path = (ROOT / pathlib.Path(*rel.parts)).resolve()
        require(path.parent == MIR_DIR.resolve() and path.is_file(),
                f"MIR path redirects or is missing: {row.get('label')}")
        require(sha(path.read_bytes()) == row.get("sha256"), f"MIR hash differs from capture: {row.get('label')}")
        selected_paths[row["label"]] = {"path": str(rel), "sha256": row["sha256"]}
    mir_files = list(MIR_DIR.glob("*.mir"))
    require(len(mir_files) == 31 and
            {path.name for path in mir_files} == {pathlib.PurePosixPath(row["path"]).name for row in rows},
            "native-mir contains missing or extra selected bodies")

    av_capture = data["av_base"]["capture"]
    for field in ("production_manifest_sha256", "production_source_sha256"):
        require(capture.get(field) == av_capture.get(field), f"AZ production pin differs from AV at {field}")
    require(sha(data["production_source"].encode()) == EXPECTED_PRODUCTION_SOURCE_SHA256 and
            capture.get("production_source_sha256") == EXPECTED_PRODUCTION_SOURCE_SHA256 and
            sha(data["production_manifest"].encode()) == EXPECTED_PRODUCTION_MANIFEST_SHA256 and
            capture.get("production_manifest_sha256") == EXPECTED_PRODUCTION_MANIFEST_SHA256,
            "captured production source or manifest differs from current reviewed input")
    require(sha(data["native_source"].encode()) == EXPECTED_NATIVE_SOURCE_SHA256 and
            capture.get("native_source_sha256") == EXPECTED_NATIVE_SOURCE_SHA256 and
            sha(data["native_test_source"].encode()) == EXPECTED_TEST_SOURCE_SHA256 and
            capture.get("native_test_source_sha256") == EXPECTED_TEST_SOURCE_SHA256 and
            sha(data["native_test_log"].encode()) == EXPECTED_NATIVE_LOG_SHA256 and
            capture.get("native_test_log_sha256") == EXPECTED_NATIVE_LOG_SHA256,
            "AZ source, test or native log is not bound to the captured bytes")
    require("AZ native step-pattern cases: 90" in data["native_test_log"] and
            "test result: ok. 1 passed" in data["native_test_log"] and
            "test result: FAILED" not in data["native_test_log"],
            "captured AZ native witness log does not report the pinned successful run")
    require(sha(data["field_source"].encode()) == EXPECTED_FIELD_SOURCE_SHA256 and
            sha(data["field_log"].encode()) == EXPECTED_FIELD_LOG_SHA256 and
            "native Bytes fields have no independent drop glue" in data["field_log"],
            "AZ native Bytes field profile differs from inherited capture")
    reviewed = json.loads(data["production_reviewed_inputs"])
    require(sha(data["production_reviewed_inputs"].encode()) == EXPECTED_REVIEWED_INPUTS_SHA256 and
            ancestor["reviewed_production_inputs"]["manifest_sha256"] == EXPECTED_REVIEWED_INPUTS_SHA256 and
            reviewed.get("base_commit") == "361c7cd261507ac0a705b3b836f73240070891c6" and
            len(reviewed.get("files", {})) == 63 and
            reviewed["files"].get("Cargo.toml") == EXPECTED_PRODUCTION_MANIFEST_SHA256 and
            reviewed["files"].get("src/bytes.rs") == EXPECTED_PRODUCTION_SOURCE_SHA256,
            "AZ reviewed production-input set differs from the pinned AV 63-file set")
    return {"stage": capture["stage"], "selected_mir_count": 31,
            "production_mir_count": 30, "client_mir_count": 1,
            "production_mir_byte_exact_to_av": 29, "production_mir_byte_exact_to_ay": 30,
            "selected_paths": selected_paths, "capture_sha256": EXPECTED_CAPTURE_SHA256,
            "production_source_sha256": EXPECTED_PRODUCTION_SOURCE_SHA256,
            "production_manifest_sha256": capture["production_manifest_sha256"],
            "native_harness_cases": 90, "capture_versions_pinned": True}


def audit_native_source(data: dict[str, Any]) -> dict[str, Any]:
    source = data["production_source"]
    require(sha(source.encode()) == EXPECTED_PRODUCTION_SOURCE_SHA256,
            "production bytes.rs source changed after native capture")
    item_specs = {
        "from_box": "fn from(slice: Box<[u8]>) -> Bytes",
        "advance": "fn advance(&mut self, cnt: usize)",
        "inc_start": "unsafe fn inc_start(&mut self, by: usize)",
        "len": "pub const fn len(&self) -> usize",
        "chunk": "fn chunk(&self) -> &[u8]",
        "as_slice": "fn as_slice(&self) -> &[u8]",
        "even_drop": "unsafe fn promotable_even_drop(data: &mut AtomicPtr<()>, ptr: *const u8, len: usize)",
        "odd_drop": "unsafe fn promotable_odd_drop(data: &mut AtomicPtr<()>, ptr: *const u8, len: usize)",
        "free_boxed_slice": "unsafe fn free_boxed_slice(buf: *mut u8, offset: *const u8, len: usize)",
    }
    source_items = {name: extract_item(source, signature, name) for name, signature in item_specs.items()}
    drop_impl = extract_item(source, "impl Drop for Bytes", "Bytes Drop impl")
    vtables = {
        "even": extract_item(source, "static PROMOTABLE_EVEN_VTABLE: Vtable", "even promotable vtable"),
        "odd": extract_item(source, "static PROMOTABLE_ODD_VTABLE: Vtable", "odd promotable vtable"),
    }

    ctor = source_items["from_box"]
    for phrase in (
        "#[cfg(all(creusot, bytes_original_constructor_gate))]",
        "#[cfg(not(all(creusot, bytes_original_constructor_gate)))]",
        "if slice.is_empty() {",
        "return Bytes::new();",
        "let len = slice.len();",
        "let ptr = Box::into_raw(slice) as *mut u8;",
        "if ptr as usize & 0x1 == 0 {",
        "let data = ptr_map(ptr, |addr| addr | KIND_VEC);",
        "data: AtomicPtr::new(data.cast()),",
        "vtable: &PROMOTABLE_EVEN_VTABLE,",
        "data: AtomicPtr::new(ptr.cast()),",
        "vtable: &PROMOTABLE_ODD_VTABLE,",
    ):
        require(phrase in ctor, f"From<Box> source route is missing {phrase!r}")
    require("PROMOTABLE_EVEN_VTABLE" in vtables["even"] and
            "clone: promotable_even_clone" in vtables["even"] and
            "drop: promotable_even_drop" in vtables["even"] and
            "PROMOTABLE_ODD_VTABLE" in vtables["odd"] and
            "clone: promotable_odd_clone" in vtables["odd"] and
            "drop: promotable_odd_drop" in vtables["odd"],
            "source parity vtables do not bind the matching Clone and Drop callbacks")

    advance = source_items["advance"]
    require("cnt <= self.len()" in advance and "self.inc_start(cnt)" in advance,
            "Buf::advance source no longer checks len then calls inc_start")
    inc = source_items["inc_start"]
    require("debug_assert!(self.len >= by" in inc and "self.len -= by;" in inc and
            "self.ptr = self.ptr.add(by);" in inc and ".data" not in inc and ".vtable" not in inc,
            "inc_start does not perform checked length subtraction and pointer add")
    require("self.len" in source_items["len"], "Bytes::len no longer returns the visible length")
    require("self.as_slice()" in source_items["chunk"] and
            "slice::from_raw_parts(self.ptr, self.len)" in source_items["as_slice"],
            "chunk/as_slice does not read the current Bytes pointer and length")

    even = source_items["even_drop"]
    odd = source_items["odd_drop"]
    require("data.with_mut(|shared|" in even and "data.with_mut(|shared|" in odd and
            "if kind == KIND_ARC" in even and "release_shared(shared.cast())" in even and
            "if kind == KIND_ARC" in odd and "release_shared(shared.cast())" in odd and
            "debug_assert_eq!(kind, KIND_VEC)" in even and
            "debug_assert_eq!(kind, KIND_VEC)" in odd and
            "free_boxed_slice(buf, ptr, len)" in even and
            "free_boxed_slice(shared.cast(), ptr, len)" in odd and
            "ptr_map(shared.cast(), |addr| addr & !KIND_MASK)" in even,
            "raw even/odd Drop source lost actual ARC/raw branches or base decode")
    require("const KIND_ARC: usize = 0b0;" in source and
            "const KIND_VEC: usize = 0b1;" in source and
            "const KIND_MASK: usize = 0b1;" in source,
            "native vtable route tag meanings changed")
    for label, body, raw_call in (("even", even, "free_boxed_slice(buf, ptr, len)"),
                                  ("odd", odd, "free_boxed_slice(shared.cast(), ptr, len)")):
        require(body.count("data.with_mut(") == 1 and body.count("let shared = *shared;") == 1 and
                body.count("release_shared(shared.cast())") == 1 and body.count(raw_call) == 1,
                f"AZ {label} callback must read owned data once and retain one release/free per arm")
        has_in_order(body, ["data.with_mut(|shared|", "let shared = *shared;",
                           "let kind = shared as usize & KIND_MASK;", "if kind == KIND_ARC",
                           "release_shared(shared.cast());", "} else {", "debug_assert_eq!(kind, KIND_VEC)",
                           raw_call], f"AZ {label} native kind branch")
        require("data.load(" not in body and "Ordering::Acquire" not in body,
                f"AZ {label} consuming callback invents an additional atomic load")
    drop_body = drop_impl
    require("(self.vtable.drop)(&mut self.data, self.ptr, self.len)" in drop_body,
            "Bytes::drop no longer calls the stored vtable with data/current ptr/current len")
    free = source_items["free_boxed_slice"]
    has_in_order(free, ["offset.offset_from(buf)", "+ len", "dealloc(buf,",
                        "Layout::from_size_align(cap, 1)"], "free_boxed_slice")

    hashes = {name: sha(body.encode()) for name, body in source_items.items()}
    hashes["bytes_drop_impl"] = sha(drop_impl.encode())
    hashes["even_vtable"] = sha(vtables["even"].encode())
    hashes["odd_vtable"] = sha(vtables["odd"].encode())
    return {
        "source_sha256": EXPECTED_PRODUCTION_SOURCE_SHA256,
        "source_items": {name: {"signature": item_specs.get(name, name), "sha256": digest}
                         for name, digest in hashes.items()},
        "from_box": {
            "empty_branch": "Bytes::new()", "native_witness_premise": "input length > 0",
            "nonempty_path": ["slice.len", "Box::into_raw", "base parity test", "one AtomicPtr::new", "construct Bytes"],
            "data_binding": "the one actual Bytes.data AtomicPtr initialized from base/tag; no parallel atomic",
            "even_selector": "base parity even -> PROMOTABLE_EVEN_VTABLE and tagged base via ptr_map(KIND_VEC)",
            "odd_selector": "base parity odd -> PROMOTABLE_ODD_VTABLE and base pointer in data",
            "vtable_bound_to_original_base": True,
        },
        "advance": {"guard": "cnt <= self.len()", "next": "self.inc_start(cnt)"},
        "inc_start": {"length": "self.len -= by", "pointer": "self.ptr = self.ptr.add(by)",
                      "data_unchanged": True, "vtable_unchanged": True,
                      "kind_tags": {"KIND_ARC": 0, "KIND_VEC": 1, "KIND_MASK": 1}},
        "read": {"Buf::chunk": "self.as_slice()", "as_slice": "from_raw_parts(self.ptr, self.len)"},
        "drop_dispatch": "stored self.vtable.drop(&mut self.data, self.ptr, self.len)",
        "callback_routes": {
            "even": {"arc_alternative_retained": True, "raw_branch": "decode tagged data base, free_boxed_slice(base, current_ptr, current_len)"},
            "odd": {"arc_alternative_retained": True, "raw_branch": "cast stored base, free_boxed_slice(base, current_ptr, current_len)"},
            "selection_key": "immutable original base parity / stored vtable, never advanced pointer parity",
            "shared_route_taken_by_witness": "nonempty steps: first promotion then Shared reclones",
        },
        "free_boxed_slice": {"operations": ["offset_from(base)", "checked distance + current_len",
            "Layout(size, align=1)", "dealloc(original base, layout)"],
            "source_sha256": hashes["free_boxed_slice"]},
    }


CLIENT_BLOCKS = {(0, False): ['_4 = move _1;', '_3 = <bytes::Bytes as From<Box<[u8]>>>::from(move _4) -> [return: bb1, unwind: bb18];'], (1, False): ['_5 = const 0_usize;', 'goto -> bb2;'], (2, False): ['_9 = copy _5;', "_11 = &'_ (*_2);", '_10 = core::slice::<impl [usize]>::len(move _11) -> [return: bb3, unwind: bb17];'], (3, False): ['_8 = Lt(move _9, move _10);', 'switchInt(move _8) -> [0: bb12, otherwise: bb4];'], (4, False): ['_14 = copy _5;', '_15 = PtrMetadata(copy _2);', '_16 = Lt(copy _14, copy _15);', 'assert(move _16, "index out of bounds: the length is {} but the index is {}", move _15, copy _14) -> [success: bb5, unwind: bb17];'], (5, False): ['_13 = copy (*_2)[_14];', "_18 = &'_ _3;", '_17 = bytes::Bytes::len(move _18) -> [return: bb6, unwind: bb17];'], (6, False): ['_12 = std::cmp::min::<usize>(move _13, move _17) -> [return: bb7, unwind: bb17];'], (7, False): ["_20 = &'_ mut _3;", '_21 = copy _12;', '_19 = <bytes::Bytes as Buf>::advance(move _20, move _21) -> [return: bb8, unwind: bb17];'], (8, False): ["_24 = &'_ _3;", '_23 = <bytes::Bytes as Clone>::clone(move _24) -> [return: bb9, unwind: bb17];'], (9, False): ['_22 = const ();', 'drop(_23) -> [return: bb10, unwind: bb17];'], (10, False): ['_25 = AddWithOverflow(copy _5, const 1_usize);', 'assert(!move (_25.1: bool), "attempt to compute `{} + {}`, which would overflow", copy _5, const 1_usize) -> [success: bb11, unwind: bb17];'], (11, False): ['_5 = move (_25.0: usize);', '_7 = const ();', 'goto -> bb2;'], (12, False): ['_6 = const ();', "_31 = &'_ _3;", '_30 = <bytes::Bytes as Buf>::chunk(move _31) -> [return: bb13, unwind: bb17];'], (13, False): ["_29 = &'_ (*_30);", '_0 = std::slice::<impl [u8]>::to_vec(move _29) -> [return: bb14, unwind: bb17];'], (14, False): ['drop(_3) -> [return: bb15, unwind: bb19];'], (15, False): ['goto -> bb16;'], (16, False): ['return;'], (17, True): ['drop(_3) -> [return: bb19, unwind terminate(cleanup)];'], (18, True): ['goto -> bb19;'], (19, True): ['goto -> bb20;'], (20, True): ['resume;']}

def audit_client_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir=data["mir_sources"]["client"]
    require(sha(mir.encode())==EXPECTED_CLIENT_MIR_SHA256,"AZ frozen client MIR changed")
    require(data["native_source"]==EXPECTED_NATIVE_SOURCE and data["native_test_source"]==EXPECTED_TEST_SOURCE,"AZ loop witness source changed")
    require("fn root_clone_steps(_1: Box<[u8]>, _2: &[usize]) -> Vec<u8>" in mir,"AZ runtime slice signature changed")
    blocks=assert_block_facts(mir,"root_clone_steps",CLIENT_BLOCKS,"AZ client",exact_key_set=set(CLIENT_BLOCKS))
    places=dict(re.findall(r"debug (\w+) => (_\d+);",mir))
    require(places=={"input":"_1","steps":"_2","value":"_3","index":"_5","amount":"_12","_peer":"_23"},"AZ owner/index/step places changed")
    drops=[(n,m.group(1)) for (n,cleanup),body in blocks.items() if not cleanup for m in re.finditer(r"drop\((_\d+)\)",body)]
    require(drops==[(9,"_23"),(14,"_3")],"AZ per-iteration peer/final Root Drop order changed")
    operations={
        "from_box":{"block":"bb0","input":"_1","result":"_3","normal":"bb1"},
        "index_init":{"block":"bb1","place":"_5","value":0,"normal":"bb2"},
        "loop_guard":{"blocks":["bb2","bb3"],"index":"_5","slice":"_2","condition":"index < steps.len()","true":"bb4","false":"bb12"},
        "step_index":{"blocks":["bb4","bb5"],"index":"_5","slice":"_2","result":"_13","bounds_guard":True},
        "root_len":{"block":"bb5","receiver":"_3","result":"_17","normal":"bb6"},
        "min":{"block":"bb6","arguments":["_13","_17"],"result":"_12","normal":"bb7"},
        "advance":{"block":"bb7","receiver":"_3","amount":"_12","normal":"bb8"},
        "clone":{"block":"bb8","receiver":"_3","result":"_23","normal":"bb9"},
        "peer_drop":{"block":"bb9","place":"_23","normal":"bb10","unwind":"bb17"},
        "index_increment":{"blocks":["bb10","bb11"],"place":"_5","amount":1,"overflow_checked":True,"normal":"bb2"},
        "chunk":{"block":"bb12","receiver":"_3","result":"_30","normal":"bb13"},
        "to_vec":{"block":"bb13","slice_ref":"_29","result":"_0","normal":"bb14"}}
    edges=[{"block":"bb9","place":"_23","owner":"_peer","normal":"bb10","unwind":"bb17"},
           {"block":"bb14","place":"_3","owner":"value","normal":"bb15","unwind":"bb19"}]
    return {"mir_sha256":EXPECTED_CLIENT_MIR_SHA256,"normal_cfg_exact":True,"debug_places":places,
        "operations":operations,"normal_edges":edges,"mir_blocks":block_rows(blocks),
        "normal_drop_order":["_peer per iteration","value after loop"],"normal_owner_drops":drops,
        "loop_body_path":["bb4","bb5","bb6","bb7","bb8","bb9","bb10","bb11","bb2"],
        "loop_backedge":{"from":"bb11","to":"bb2"},"exit_path":["bb12","bb13","bb14","bb15","bb16"],
        "peer_drop_before_backedge":True,"saved_return_precedes_drop":True,"runtime_steps_not_fixed":True,
        "capped_amount_uses_same_root_remaining":True,"drop_flags":[],"cleanup_excluded":True}


CLONE_BLOCKS = {'even': {(0, False): ["_5 = &'_ (*_1);", '_6 = core::sync::atomic::Ordering::Acquire;', '_4 = Atomic::<*mut ()>::load(move _5, move _6) -> [return: bb1, unwind continue];'], (1, False): ['_9 = copy _4;', '_8 = move _9 as usize (PointerExposeProvenance);', '_7 = BitAnd(move _8, const bytes::KIND_MASK);', '_11 = copy _7;', '_10 = Eq(move _11, const bytes::KIND_ARC);', 'switchInt(move _10) -> [0: bb5, otherwise: bb2];'], (2, False): ['_13 = copy _4;', '_12 = core::ptr::mut_ptr::<impl *mut ()>::cast::<bytes::Shared>(move _13) -> [return: bb3, unwind continue];'], (3, False): ['_14 = copy _2;', '_15 = copy _3;', '_0 = shallow_clone_arc(move _12, move _14, move _15) -> [return: bb4, unwind continue];'], (4, False): ['goto -> bb14;'], (5, False): ['_17 = const true;', 'switchInt(move _17) -> [0: bb9, otherwise: bb6];'], (6, False): ["_20 = &'_ _7;", '_47 = const bytes::promotable_even_clone::promoted[0];', "_21 = &'_ (*_47);", '_19 = (move _20, move _21);', '_23 = copy (_19.0: &usize);', '_24 = copy (_19.1: &usize);', '_26 = copy (*_23);', '_27 = copy (*_24);', '_25 = Eq(move _26, move _27);', 'switchInt(move _25) -> [0: bb8, otherwise: bb7];'], (7, False): ['_18 = const ();', '_16 = const ();', 'goto -> bb10;'], (8, False): ['_29 = core::panicking::AssertKind::Eq;', '_31 = move _29;', "_33 = &'_ (*_23);", "_32 = &'_ (*_33);", "_35 = &'_ (*_24);", "_34 = &'_ (*_35);", "_36 = Option::<Arguments<'_>>::None;", '_30 = core::panicking::assert_failed::<usize, usize>(move _31, move _32, move _34, move _36) -> unwind continue;'], (9, False): ['_16 = const ();', 'goto -> bb10;'], (10, False): ['_39 = copy _4;', '_38 = core::ptr::mut_ptr::<impl *mut ()>::cast::<u8>(move _39) -> [return: bb11, unwind continue];'], (11, False): ['_40 = {closure@src/bytes.rs:1340:42: 1340:48};', '_37 = ptr_map::<{closure@src/bytes.rs:1340:42: 1340:48}>(move _38, move _40) -> [return: bb12, unwind continue];'], (12, False): ["_41 = &'_ (*_1);", '_43 = copy _4;', '_42 = move _43 as *const () (PtrToPtr);', '_44 = copy _37;', '_45 = copy _2;', '_46 = copy _3;', '_0 = shallow_clone_vec(move _41, move _42, move _44, move _45, move _46) -> [return: bb13, unwind continue];'], (13, False): ['goto -> bb14;'], (14, False): ['return;']}, 'odd': {(0, False): ["_5 = &'_ (*_1);", '_6 = core::sync::atomic::Ordering::Acquire;', '_4 = Atomic::<*mut ()>::load(move _5, move _6) -> [return: bb1, unwind continue];'], (1, False): ['_9 = copy _4;', '_8 = move _9 as usize (PointerExposeProvenance);', '_7 = BitAnd(move _8, const bytes::KIND_MASK);', '_11 = copy _7;', '_10 = Eq(move _11, const bytes::KIND_ARC);', 'switchInt(move _10) -> [0: bb4, otherwise: bb2];'], (2, False): ['_14 = copy _4;', '_13 = move _14 as *mut bytes::Shared (PtrToPtr);', '_12 = copy _13;', '_15 = copy _2;', '_16 = copy _3;', '_0 = shallow_clone_arc(move _12, move _15, move _16) -> [return: bb3, unwind continue];'], (3, False): ['goto -> bb12;'], (4, False): ['_18 = const true;', 'switchInt(move _18) -> [0: bb8, otherwise: bb5];'], (5, False): ["_21 = &'_ _7;", '_45 = const bytes::promotable_odd_clone::promoted[0];', "_22 = &'_ (*_45);", '_20 = (move _21, move _22);', '_24 = copy (_20.0: &usize);', '_25 = copy (_20.1: &usize);', '_27 = copy (*_24);', '_28 = copy (*_25);', '_26 = Eq(move _27, move _28);', 'switchInt(move _26) -> [0: bb7, otherwise: bb6];'], (6, False): ['_19 = const ();', '_17 = const ();', 'goto -> bb9;'], (7, False): ['_30 = core::panicking::AssertKind::Eq;', '_32 = move _30;', "_34 = &'_ (*_24);", "_33 = &'_ (*_34);", "_36 = &'_ (*_25);", "_35 = &'_ (*_36);", "_37 = Option::<Arguments<'_>>::None;", '_31 = core::panicking::assert_failed::<usize, usize>(move _32, move _33, move _35, move _37) -> unwind continue;'], (8, False): ['_17 = const ();', 'goto -> bb9;'], (9, False): ["_38 = &'_ (*_1);", '_40 = copy _4;', '_39 = move _40 as *const () (PtrToPtr);', '_42 = copy _4;', '_41 = core::ptr::mut_ptr::<impl *mut ()>::cast::<u8>(move _42) -> [return: bb10, unwind continue];'], (10, False): ['_43 = copy _2;', '_44 = copy _3;', '_0 = shallow_clone_vec(move _38, move _39, move _41, move _43, move _44) -> [return: bb11, unwind continue];'], (11, False): ['goto -> bb12;'], (12, False): ['return;']}}

def audit_clone_routes(data):
    source=data['production_source'];rows={}
    for parity in ('even','odd'):
        signature=f'unsafe fn promotable_{parity}_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes'
        body=extract_item(source,signature,parity+' Clone')
        has_in_order(body,['let shared = data.load(Ordering::Acquire);','let kind = shared as usize & KIND_MASK;',
                          'if kind == KIND_ARC',('shallow_clone_arc(shared.cast(), ptr, len)' if parity=='even' else 'shallow_clone_arc(shared as _, ptr, len)'),'} else {',
                          'debug_assert_eq!(kind, KIND_VEC);','shallow_clone_vec(data, shared,'],parity+' Clone routes')
        require(body.count('data.load(')==1 and body.count('Ordering::Acquire')==1,
                parity+' Clone must perform exactly one actual Acquire')
        require(('ptr_map(shared.cast(), |addr| addr & !KIND_MASK)' in body) if parity=='even' else
                ('shallow_clone_vec(data, shared, shared.cast(), ptr, len)' in body),parity+' original-base decode changed')
        label='promotable_'+parity+'_clone';mir=data['mir_sources'][label]
        blocks=assert_block_facts(mir,label,CLONE_BLOCKS[parity],parity+' Clone',exact_key_set=set(CLONE_BLOCKS[parity]))
        require(sum('Atomic::<*mut ()>::load(' in line for b in blocks.values() for line in significant(b))==1,
                parity+' MIR duplicates Acquire')
        rows[parity]={'source_item_sha256':sha(body.encode()),'mir_sha256':sha(mir.encode()),
            'owned_root_acquire_count':1,'actual_kind_guard':'KIND_ARC','shared_route':'shallow_clone_arc(shared, current ptr, current len)',
            'raw_route':'shallow_clone_vec(data, expected stored word, original base, current ptr, current len)',
            'original_base_decode':'clear stored tag' if parity=='even' else 'direct stored base cast',
            'mir_blocks':block_rows(blocks),'native_kind_arms_checked':True}
    return rows

def audit_production_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]
    # Constructor: empty path is present in native MIR but unreachable under the
    # client's nonempty-input domain. Both base-parity branches create the one
    # real data AtomicPtr and store the immutable original-base vtable.
    from_box_expected = {
        (1, False): ["switchInt(move _3) -> [0: bb3, otherwise: bb2];"],
        (2, False): ["_0 = bytes::Bytes::new() -> [return: bb14, unwind: bb17];"],
        (3, False): ["_2 = const ();", "_7 = &'_ (*_1);", "_6 = core::slice::<impl [u8]>::len(move _7) -> [return: bb4, unwind: bb17];"],
        (4, False): ["_36 = const false;", "_10 = move _1;", "_9 = Box::<[u8]>::into_raw(move _10) -> [return: bb5, unwind: bb16];"],
        (5, False): ["_8 = move _9 as *mut u8 (PtrToPtr);", "_14 = copy _8;", "_13 = move _14 as usize (PointerExposeProvenance);", "_12 = BitAnd(move _13, const 1_usize);", "_11 = Eq(move _12, const 0_usize);", "switchInt(move _11) -> [0: bb10, otherwise: bb6];"],
        (6, False): ["_16 = copy _8;", "_17 = {closure@src/bytes.rs:1139:37: 1139:43};", "_15 = ptr_map::<{closure@src/bytes.rs:1139:37: 1139:43}>(move _16, move _17) -> [return: bb7, unwind: bb17];"],
        (8, False): ["_21 = Atomic::<*mut ()>::new(move _22) -> [return: bb9, unwind: bb17];"],
        (9, False): ["_26 = const {alloc288: &Vtable};", "_25 = &'_ (*_26);", "_24 = &'_ (*_25);", "_0 = bytes::Bytes { ptr: move _18, len: move _20, data: move _21, vtable: move _24 };", "goto -> bb13;"],
        (10, False): ["_28 = copy _8;", "_27 = move _28 as *const u8 (PtrToPtr);", "_29 = copy _6;", "_32 = copy _8;", "_31 = core::ptr::mut_ptr::<impl *mut u8>::cast::<()>(move _32) -> [return: bb11, unwind: bb17];"],
        (11, False): ["_30 = Atomic::<*mut ()>::new(move _31) -> [return: bb12, unwind: bb17];"],
        (12, False): ["_35 = const {alloc295: &Vtable};", "_34 = &'_ (*_35);", "_33 = &'_ (*_34);", "_0 = bytes::Bytes { ptr: move _27, len: move _29, data: move _30, vtable: move _33 };", "goto -> bb13;"],
    }
    from_box = assert_block_facts(mir["from_box"], "bytes::<impl at src/bytes.rs:1120:1: 1120:31>::from",
                                  from_box_expected, "From<Box>")
    from_text = mir["from_box"]
    require("alloc288 (static: PROMOTABLE_EVEN_VTABLE" in from_text and
            "alloc289 (fn: promotable_even_clone)" in from_text and
            "alloc293 (fn: promotable_even_drop)" in from_text and
            "alloc295 (static: PROMOTABLE_ODD_VTABLE" in from_text and
            "alloc296 (fn: promotable_odd_clone)" in from_text and
            "alloc299 (fn: promotable_odd_drop)" in from_text,
            "From<Box> MIR vtable constants do not bind the exact even/odd callbacks")

    advance_expected = {
        (0, False): ["_5 = copy _2;", "_7 = &'_ (*_1);", "_6 = bytes::Bytes::len(move _7) -> [return: bb1, unwind continue];"],
        (1, False): ["_4 = Le(move _5, move _6);", "switchInt(move _4) -> [0: bb3, otherwise: bb2];"],
        (2, False): ["_3 = const ();", "_26 = &'_ mut (*_1);", "_27 = copy _2;", "_25 = bytes::Bytes::inc_start(move _26, move _27) -> [return: bb8, unwind continue];"],
    }
    advance_blocks = assert_block_facts(mir["advance"], "bytes::<impl at src/bytes.rs:752:1: 752:19>::advance",
                                        advance_expected, "Buf::advance")
    inc_expected = {
        (7, False): ["((*_1).1: usize) = move (_13.0: usize);", "_15 = copy ((*_1).0: *const u8);",
                     "_16 = copy _2;", "_14 = core::ptr::const_ptr::<impl *const u8>::add(move _15, move _16) -> [return: bb8, unwind continue];"],
        (8, False): ["((*_1).0: *const u8) = move _14;", "_0 = const ();", "return;"],
    }
    inc_blocks = assert_block_facts(mir["inc_start"], "bytes::<impl at src/bytes.rs:89:1: 89:11>::inc_start",
                                    inc_expected, "inc_start")
    inc_bb6 = significant(inc_blocks[(6, False)])
    require(len(inc_bb6) == 3 and inc_bb6[0] == "_12 = copy _2;" and
            inc_bb6[1] == "_13 = SubWithOverflow(copy ((*_1).1: usize), copy _12);" and
            inc_bb6[2].startswith("assert(!move (_13.1: bool)") and
            "[success: bb7, unwind continue]" in inc_bb6[2],
            "inc_start no longer checks length-subtraction overflow before writes")
    require(not any(re.search(r"^\(\(\*_1\)\.(?:2|3):.*?\)\s*=", statement)
                    for body in inc_blocks.values() for statement in significant(body)),
            "inc_start MIR writes Bytes.data or Bytes.vtable")

    len_blocks = assert_block_facts(mir["len"], "bytes::<impl at src/bytes.rs:89:1: 89:11>::len",
                                    {(0, False): ["_0 = copy ((*_1).1: usize);", "return;"]}, "Bytes::len")
    chunk_blocks = assert_block_facts(mir["chunk"], "bytes::<impl at src/bytes.rs:752:1: 752:19>::chunk",
                                      {(0, False): ["_3 = &'_ (*_1);", "_2 = bytes::Bytes::as_slice(move _3) -> [return: bb1, unwind continue];"],
                                       (1, False): ["_0 = &'_ (*_2);", "return;"]}, "Buf::chunk")
    slice_blocks = assert_block_facts(mir["as_slice"], "bytes::<impl at src/bytes.rs:89:1: 89:11>::as_slice",
                                      {(0, False): ["_3 = copy ((*_1).0: *const u8);", "_4 = copy ((*_1).1: usize);",
                                       "_2 = core::slice::from_raw_parts::<'_, u8>(move _3, move _4) -> [return: bb1, unwind continue];"],
                                       (1, False): ["_0 = &'_ (*_2);", "return;"]}, "Bytes::as_slice")

    drop_blocks = assert_block_facts(mir["bytes_drop"], "bytes::<impl at src/bytes.rs:729:1: 729:20>::drop",
                                     {(0, False): ["_7 = deref_copy ((*_1).3: &bytes::Vtable);",
                                      "_2 = copy ((*_7).4: for<'a> unsafe fn(&'a mut core::sync::atomic::Atomic<*mut ()>, *const u8, usize));",
                                      "_4 = &'_ mut ((*_1).2: core::sync::atomic::Atomic<*mut ()>);", "_3 = &'_ mut (*_4);",
                                      "_5 = copy ((*_1).0: *const u8);", "_6 = copy ((*_1).1: usize);",
                                      "_0 = move _2(move _3, move _5, move _6) -> [return: bb1, unwind continue];"],
                                      (1, False): ["return;"]}, "Bytes::drop")

    free_blocks = assert_block_facts(mir["free_boxed_slice"], "free_boxed_slice",
                                     {(0, False): ["_7 = copy _2;", "_9 = copy _1;", "_8 = move _9 as *const u8 (PtrToPtr);",
                                      "_6 = core::ptr::const_ptr::<impl *const u8>::offset_from(move _7, move _8) -> [return: bb1, unwind continue];"],
                                      (2, False): ["_4 = move (_11.0: usize);", "_12 = copy _1;", "_15 = copy _4;",
                                      "_14 = Layout::from_size_align(move _15, const 1_usize) -> [return: bb3, unwind continue];"],
                                      (3, False): ["_13 = Result::<Layout, LayoutError>::unwrap(move _14) -> [return: bb4, unwind continue];"],
                                      (4, False): ["_0 = std::alloc::dealloc(move _12, move _13) -> [return: bb5, unwind continue];"],
                                      (5, False): ["return;"]}, "free_boxed_slice")
    free_bb1 = significant(free_blocks[(1, False)])
    require(len(free_bb1) == 4 and free_bb1[:3] == ["_5 = move _6 as usize (IntToInt);",
            "_10 = copy _3;", "_11 = AddWithOverflow(copy _5, copy _10);"] and
            free_bb1[3].startswith("assert(!move (_11.1: bool)") and
            "[success: bb2, unwind continue]" in free_bb1[3],
            "free_boxed_slice no longer checks distance + suffix length before dealloc")

    even_drop_blocks = assert_block_facts(mir["promotable_even_drop"], "promotable_even_drop",
        {(0, False): ["_5 = &'_ mut (*_1);", "_7 = &'_ _2;", "_8 = &'_ _3;",
         "_6 = {closure@src/bytes.rs:1413:19: 1413:27} { ptr: move _7, len: move _8 };",
         "_4 = <Atomic<*mut ()> as AtomicMut<()>>::with_mut::<{closure@src/bytes.rs:1413:19: 1413:27}, ()>(move _5, move _6) -> [return: bb1, unwind continue];"],
         (1, False): ["_0 = const ();", "return;"]}, "promotable_even_drop")
    odd_drop_blocks = assert_block_facts(mir["promotable_odd_drop"], "promotable_odd_drop",
        {(0, False): ["_5 = &'_ mut (*_1);", "_7 = &'_ _2;", "_8 = &'_ _3;",
         "_6 = {closure@src/bytes.rs:1448:19: 1448:27} { ptr: move _7, len: move _8 };",
         "_4 = <Atomic<*mut ()> as AtomicMut<()>>::with_mut::<{closure@src/bytes.rs:1448:19: 1448:27}, ()>(move _5, move _6) -> [return: bb1, unwind continue];"],
         (1, False): ["_0 = const ();", "return;"]}, "promotable_odd_drop")

    def row(label: str) -> dict[str, Any]:
        return {"label": label, "sha256": sha(mir[label].encode()),
                "capture_path": next(r["path"] for r in data["capture"]["selected"] if r["label"] == label)}

    return {
        "from_box": {"selected_mir": row("from_box"), "empty_branch": "bb1 -> bb2 -> Bytes::new",
            "nonempty_branch": "bb1 -> bb3 -> bb4 -> bb5",
            "even_branch": "bb5 -> bb6 -> bb7 -> bb8 -> bb9; PROMOTABLE_EVEN_VTABLE",
            "odd_branch": "bb5 -> bb10 -> bb11 -> bb12; PROMOTABLE_ODD_VTABLE",
            "both_store_one_atomic_data_field": True, "base_parity_selects_original_vtable": True},
        "advance": {"selected_mir": row("advance"), "guard_blocks": ["bb0", "bb1"],
            "in_bounds_dispatch": "bb1 -> bb2 -> Bytes::inc_start on same receiver and amount",
            "bounds": "cnt <= Bytes::len(self)"},
        "inc_start": {"selected_mir": row("inc_start"), "length_write": "Bytes.len = old_len - by",
            "pointer_write": "Bytes.ptr = old_ptr.add(by)", "data_field_write": False,
            "vtable_field_write": False},
        "read": {"chunk_mir": row("chunk"), "as_slice_mir": row("as_slice"),
            "route": "Buf::chunk -> Bytes::as_slice -> from_raw_parts(current ptr, current len)"},
        "drop": {"selected_mir": row("bytes_drop"), "stored_vtable_call": True,
            "arguments": ["&mut self.data", "self.ptr", "self.len"]},
        "callbacks": {"even": {"selected_mir": row("promotable_even_drop"), "with_mut": True,
                          "source_keeps_arc_branch": True, "shared_branch_calls_release_shared": True,
                          "owned_field_read_exactly_once": True, "native_kind_guard": "KIND_ARC",
                          "raw_branch_calls_free_boxed_slice": True},
            "odd": {"selected_mir": row("promotable_odd_drop"), "with_mut": True,
                         "source_keeps_arc_branch": True, "shared_branch_calls_release_shared": True,
                          "owned_field_read_exactly_once": True, "native_kind_guard": "KIND_ARC",
                          "raw_branch_calls_free_boxed_slice": True},
            "raw_kind_from_constructor": "KIND_VEC", "phase_at_root_drop": "Raw iff steps empty; Shared otherwise", "peer_release_precedes_loop_backedge": True},
        "free_boxed_slice": {"selected_mir": row("free_boxed_slice"),
            "distance": "offset_from(offset, base)", "size": "distance + len",
            "layout_alignment": 1, "deallocated_pointer": "original base argument", "receipt_is_physical_only": True},
    }


def audit_bundle(data: dict[str, Any] | None = None) -> dict[str, Any]:
    if data is None:
        data = load_bundle()
    ancestor = audit_ancestor(data)
    capture = audit_capture(data, ancestor)
    source = audit_native_source(data)
    client = audit_client_mir(data)
    production = audit_production_mir(data)
    production["root_clone_callbacks"] = audit_clone_routes(data)
    return {
        "status": "pass",
        "checker_scope": "AZ runtime capped-step loop with first promotion/Shared Root reclone, lexical per-iteration peer Drop, normal Root Drop; bounded correspondence",
        "ancestor_av": ancestor,
        "operational_baseline_reused": {"increment":"AY", "mapping_sha256":"7865230973c2672cb3b56538be2adac72579dc7ebd0ed805b519a4aee5c332d8", "ancestor_checker_reexecuted":False},
        "capture": capture,
        "source_profile": source,
        "native_audit": {"client": client, "production": production,
            "selected_mir_count": 31, "production_mir_count": 30,
            "inherited_av_production_mir_count": 29, "new_raw_free_mir_count": 1},
        "limits": [
            "The native harness is execution corroboration only and does not demonstrate both allocation-address parities.",
            "The checked witness uses nonempty Box input and a runtime slice of arbitrary capped steps; zero iterations, repeated zero, oversized steps and fully drained owned-zero views are included.",
            "The ordinary captured return Drop is checked; unwind behavior is excluded.",
            "Both native Clone kind arms are source-bound: empty steps retain Raw; first iteration promotes; later iterations reclone Shared. Every peer drops before the backedge. Physical Raw receipt and Shared Completion remain distinct.",
            "The inherited AV source/MIR baseline and generic provenance, pointer arithmetic, field permission, physical borrow/free and erasure boundaries remain explicit TCB.",
        ],
    }


def main() -> int:
    try:
        print(json.dumps(audit_bundle(), indent=2))
        return 0
    except AuditError as exc:
        print(json.dumps({"status": "coverage_failure", "error": str(exc)}, indent=2))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
