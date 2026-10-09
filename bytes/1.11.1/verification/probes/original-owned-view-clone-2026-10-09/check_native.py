#!/usr/bin/env python3
"""Read-only native source/MIR correspondence check for the AT owned-view Clone witness.

This checker performs no Cargo, rustc, Creusot, Why3, or solver invocation.
It binds the captured production bodies and post-ElaborateDrops MIR to the
selected default-native owned-view Clone client. It is not a proof of MIR
adequacy or Rust pointer-provenance semantics.
"""
from __future__ import annotations

import argparse
import copy
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
AS_PROBE = PROBES / "original-nonnull-view-boundaries-2026-10-09"
AS_CHECKER_PATH = AS_PROBE / "check_native.py"
AS_CHECKER_SHA256 = "a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f"
MIR_DIR = ROOT / "native-mir"
CAPTURE_PATH = MIR_DIR / "capture.json"
EXPECTED_CAPTURE_SHA256 = "9b3c0966777ccd89779ec15ff57a5313aa42f306ef900cd10d246b01d507a407"
EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\ncommit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\nhost: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\nLLVM version: 22.1.7\n")
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"
EXPECTED_NATIVE_SOURCE_SHA256 = "9cd33dde8f2995c88417f2baa9faa6deb58991d71f93640ff110c0d091c214e7"
EXPECTED_TEST_SOURCE_SHA256 = "a3d231408fa95b3e4a8255c43cd6fbdf2c0a7c5b86c511451766b57ac869fb53"
EXPECTED_CAPTURE_SCRIPT_SHA256 = "a6543b09a21dd46e8f2e92cdd47aa53c0359d8e3910d16b0e7ed68ff9598f095"
EXPECTED_NATIVE_MANIFEST_SHA256 = "e63a721ceda31711b86732580276fa46c4471670307f7d85f46aa9155dff5cc5"
EXPECTED_NATIVE_LOCK_SHA256 = "a99ded97e092f2c1bd39dad4710cb618d24561e7c25546a0bf9a1e8f5f984076"
EXPECTED_NATIVE_LOG_SHA256 = "44ac2a4cd9d59c456238d04704ac26f0f3877da74ebcc0d06f412ab39a5ac176"
EXPECTED_FIELD_PROFILE_SOURCE_SHA256 = "b8dae751e6c7dc007de636ed7cc8d9b6f98cc746252d7373e785778655b58183"
EXPECTED_FIELD_PROFILE_LOG_SHA256 = "3849089883705a9502911036f6510d56a263520626d4fc5ebc023aaa0530c436"
EXPECTED_EXTRACTOR_SHA256 = "748810dd6d4961c1e729864a83daa3aff771c72e7ef678e9bb01f2cfef44e99b"
EXPECTED_REVIEWED_INPUTS_SHA256 = "e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306"
EXPECTED_CURSOR_BINDINGS_SHA256 = "6df307589aee9da1bd13d6fdd2b1c725d3fbdc6002bb0eb05af12aedcb6b4fa8"
EXPECTED_SOURCE_MAP_SHA256 = "963c24916e6a3b541c27fc5186b0bb24b6f37b1a441198b815df75ebfdce099c"

EXPECTED_CURSOR_BODY_HASHES = {
    "inc_start": "59117581e5d70be68feae2252aee80cc7cf18cbf3da1a6acb01e6155f98f744b",
    "remaining": "65f4b1a4e34688e0c1faae50e741e759bcfdad5485bcbee98b089c54075fd315",
    "chunk": "1b97deed01a0f490a65483d48891121d397399c293b3e7b953fe1ee426aa6af8",
    "advance": "1306de3fe45d81505ea044c468ff0537ac4d2ae919655c2901766f1c4188aa19",
    "len": "87a65c27502d58e14017cc5d4fc84e528bbdf8b7d8530a710794f100cba1d304",
}
EXPECTED_CURSOR_SIGNATURES = {
    "inc_start": "unsafe fn inc_start(&mut self, by: usize)",
    "remaining": "fn remaining(&self) -> usize",
    "chunk": "fn chunk(&self) -> &[u8]",
    "advance": "fn advance(&mut self, cnt: usize)",
    "len": "pub const fn len(&self) -> usize",
}
EXPECTED_NATIVE_MANIFEST = {
    "package": {"name": "bytes-owned-view-clone-native", "version": "0.0.0", "edition": "2021"},
    "workspace": {},
    "lib": {"name": "bytes_owned_view_clone_native", "path": "../native.rs"},
    "dependencies": {"bytes": {"path": "../../../../"}},
}
EXPECTED_NATIVE_SOURCE = r"""use bytes::{Buf, Bytes};

pub fn owned_view_clone_scope(
    input: Box<[u8]>,
    a: usize,
    b: usize,
    advance_by: usize,
    rounds: usize,
) -> Vec<u8> {
    let mut value = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        owner.slice(a..b)
    };
    value.advance(advance_by);
    let mut i = 0;
    while i < rounds {
        let next = value.clone();
        value = next;
        i += 1;
    }
    value.chunk().to_vec()
}
"""
EXPECTED_NATIVE_TEST = r"""use bytes_owned_view_clone_native::owned_view_clone_scope;
use std::collections::BTreeSet;

#[test]
fn clone_witness() {
    let mut cases = 0;
    for len in [1usize, 2, 7, 31] {
        let input: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let ranges: BTreeSet<_> = [(0, len), (len / 2, len), (len - 1, len), (0, 1)].into_iter().collect();
        for (a, b) in ranges {
            let advances: BTreeSet<_> = [0, (b - a) / 2, b - a].into_iter().collect();
            for advance_by in advances {
                for rounds in [0, 1, 2, 7, 31] {
                    assert_eq!(owned_view_clone_scope(input.clone().into_boxed_slice(), a, b, advance_by, rounds), input[a + advance_by..b]);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 145);
    println!("owned-view Clone native cases: {cases}");
}"""


class AuditError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_as_checker():
    require(AS_CHECKER_PATH.is_file() and sha(AS_CHECKER_PATH.read_bytes()) == AS_CHECKER_SHA256,
            "published AS native checker changed before import")
    spec = importlib.util.spec_from_file_location("at_pinned_as_native", AS_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load pinned AS native checker")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


AS = load_as_checker()
AI = AS.AI


def load_bundle() -> dict[str, Any]:
    """Load this exact native capture plus the already-published AS baseline."""
    as_base = AS.load_bundle()
    cap_raw = CAPTURE_PATH.read_bytes()
    capture = json.loads(cap_raw)
    selected = capture.get("selected", [])
    mir_sources = {}
    for row in selected:
        path = ROOT / row["path"]
        mir_sources[row["label"]] = path.read_text()
    return {
        "as_base": as_base,
        "capture": capture,
        "capture_raw": cap_raw,
        "native_source": (ROOT / "native.rs").read_text(),
        "native_test_source": (ROOT / "native-test/tests/clone_witness.rs").read_text(),
        "native_test_log": (ROOT / "native-test/native-run.log").read_text(),
        "native_manifest": (ROOT / "native-test/Cargo.toml").read_text(),
        "native_lock": (ROOT / "native-test/Cargo.lock").read_text(),
        "capture_script": (ROOT / "capture-native.sh").read_text(),
        "rustc_text": (MIR_DIR / "rustc-version.txt").read_text(),
        "cargo_text": (MIR_DIR / "cargo-version.txt").read_text(),
        "mir_sources": mir_sources,
        "mapping_text": (ROOT / "generated/mapping.json").read_text(),
        "production_source": as_base["production_source"],
        "production_source_inputs": as_base["production_source_inputs"],
        "reviewed_production_manifest": as_base["reviewed_production_manifest"],
        "native_field_profile_source": (ROOT / "native-field-profile.rs").read_text(),
        "native_field_profile_log": (ROOT / "native-field-profile.log").read_text(),
        "extractor": (ROOT / "extract_public.py").read_text(),
        "native_cursor_bindings": (ROOT / "generated/native_cursor_bindings.rs").read_text(),
        "source_map_text": (ROOT / "generated/source-map.json").read_text(),
    }


def significant(body: str) -> list[str]:
    return [line.strip() for line in body.splitlines()
            if line.strip() and not line.strip().startswith(("StorageLive(", "StorageDead(", "debug ", "PlaceMention("))]


def blocks_of(mir: str, label: str) -> dict[tuple[int, bool], str]:
    lines = mir.splitlines()
    out: dict[tuple[int, bool], str] = {}
    i = 0
    while i < len(lines):
        m = re.fullmatch(r"    bb(\d+)( \(cleanup\))?: \{", lines[i])
        if not m:
            i += 1
            continue
        key = (int(m.group(1)), m.group(2) is not None)
        require(key not in out, f"{label} duplicates bb{key[0]}")
        i += 1
        body = []
        while i < len(lines) and lines[i] != "    }":
            body.append(lines[i])
            i += 1
        require(i < len(lines), f"{label} has unterminated bb{key[0]}")
        out[key] = "\n".join(body)
        i += 1
    require(bool(out), f"{label} contains no basic blocks")
    return out


def require_tokens(actual: str, expected: str, label: str) -> None:
    require(AI.rust_tokens(actual) == AI.rust_tokens(expected), f"{label} token structure changed")


def extract_body(source: str, signature: str, label: str) -> str:
    # The reviewed whole production source is hash pinned by AS. Still require a
    # unique executable signature and use a comment/string masked search so a
    # comment cannot redirect extraction.
    masked = AI.mask_noncode(source)
    sigmask = AI.mask_noncode(signature)
    starts = [m.start() for m in re.finditer(re.escape(sigmask), masked)]
    require(len(starts) == 1, f"expected one executable cursor method {label}")
    start = starts[0]
    opening = masked.find("{", start + len(sigmask))
    require(opening >= 0, f"cursor method {label} has no body")
    depth = 0
    for i in range(opening, len(masked)):
        if masked[i] == "{":
            depth += 1
        elif masked[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start:i + 1]
    raise AuditError(f"cursor method {label} has an unterminated body")


def audit_ancestor(data: dict[str, Any]) -> dict[str, Any]:
    # Recheck the published AS gate in its original path layout. This call is
    # pure parsing/hash validation; it invokes no build or proof tool.
    try:
        result = AS.audit_bundle(data["as_base"])
    except Exception as exc:
        raise AuditError(f"published AS native baseline rejected: {type(exc).__name__}: {exc}") from exc
    require(result.get("status") == "pass", "published AS native correspondence baseline no longer passes")
    return {
        "status": "pass",
        "checker_sha256": AS_CHECKER_SHA256,
        "as_native_mir_count": result["native_audit"]["selected_mir_count"],
        "as_production_mir_count": result["native_audit"]["production_mir_count"],
        "as_source_map_cursor_extension_unchanged": True,
        "reviewed_production_inputs": result["reviewed_production_inputs"],
        "production_callbacks": result["production_callbacks"],
        "terminal_field_profile": result["terminal_field_profile"],
    }


def audit_cursor_source(data: dict[str, Any]) -> dict[str, Any]:
    source = data["production_source"]
    expected = {name: extract_body(source, sig, name) for name, sig in EXPECTED_CURSOR_SIGNATURES.items()}
    actual_hashes = {name: sha(body.encode()) for name, body in expected.items()}
    require(actual_hashes == EXPECTED_CURSOR_BODY_HASHES, "reviewed production cursor body hash changed")
    require(sha(data["extractor"].encode()) == EXPECTED_EXTRACTOR_SHA256,
            "native public source extractor changed")
    binding = "\n\n".join(expected[name] for name in EXPECTED_CURSOR_SIGNATURES) + "\n"
    require(data["native_cursor_bindings"] == binding,
            "generated/native_cursor_bindings.rs does not equal the selected production method bodies")
    require(sha(data["native_cursor_bindings"].encode()) == EXPECTED_CURSOR_BINDINGS_SHA256,
            "generated cursor source binding hash changed")
    source_map = json.loads(data["source_map_text"])
    require(source_map.get("cursor_bodies") == actual_hashes,
            "source-map cursor_bodies do not match parsed production bodies")
    require(sha(data["source_map_text"].encode()) == EXPECTED_SOURCE_MAP_SHA256,
            "generated source map identity changed")

    inc = expected["inc_start"]
    inc_tokens = AI.rust_tokens(inc)
    require("self.len -= by" in inc and "self.ptr = self.ptr.add(by)" in inc,
            "inc_start no longer subtracts len and uses native ptr.add(by)")
    require(inc.index("self.len -= by") < inc.index("self.ptr = self.ptr.add(by)"),
            "inc_start pointer adjustment moved before the length subtraction")
    require("debug_assert!(self.len >= by" in inc and
            "#[cfg(not(all(creusot, bytes_original_freeze_gate)))]" in inc and
            "#[cfg(all(creusot, bytes_original_freeze_gate))]" in inc,
            "inc_start default-native versus Creusot cfg route changed")
    require("proof_assert!(by == 0usize)" in inc,
            "Creusot-only inc_start route no longer exposes its zero-only proof branch")
    require("self.data" not in inc and "self.vtable" not in inc,
            "inc_start changes the data field or vtable")
    require("self.len()" in expected["remaining"] and "self.as_slice()" in expected["chunk"],
            "Buf::remaining or Buf::chunk no longer forwards to the selected Bytes methods")
    require("cnt <= self.len()" in expected["advance"] and "self.inc_start(cnt)" in expected["advance"],
            "Buf::advance lost its bounds assertion or inc_start call")
    require(re.fullmatch(r"pub const fn len\(&self\) -> usize \{\s*self\.len\s*\}", expected["len"], re.S) is not None,
            "Bytes::len no longer returns its len field directly")
    return {
        "source_body_names": list(EXPECTED_CURSOR_SIGNATURES),
        "source_body_hashes": actual_hashes,
        "generated_native_cursor_bindings": {"path": "generated/native_cursor_bindings.rs",
            "sha256": sha(data["native_cursor_bindings"].encode()), "source_exact": True},
        "source_map": {"path": "generated/source-map.json",
            "sha256": sha(data["source_map_text"].encode()), "cursor_bodies_exact": True},
        "extractor_sha256": EXPECTED_EXTRACTOR_SHA256,
        "default_native_inc_start": {"len_subtraction_precedes_ptr_add": True,
            "pointer_operation": "self.ptr.add(by)", "debug_bounds_assertion": True,
            "data_and_vtable_unchanged": True, "selected_by_native_mir": True},
        "buf_routes": {"advance": "assert cnt <= len; inc_start(cnt)",
            "remaining": "Bytes::len", "chunk": "Bytes::as_slice", "len": "Bytes.len field"},
    }


def audit_source_and_profile(data: dict[str, Any]) -> dict[str, Any]:
    require_tokens(data["native_source"], EXPECTED_NATIVE_SOURCE, "native owned-view clone client")
    require(sha(data["native_source"].encode()) == EXPECTED_NATIVE_SOURCE_SHA256,
            "native owned-view clone client hash changed")
    require_tokens(data["native_test_source"], EXPECTED_NATIVE_TEST, "native clone smoke test")
    require(sha(data["native_test_source"].encode()) == EXPECTED_TEST_SOURCE_SHA256,
            "native clone smoke-test hash changed")
    require(sha(data["native_field_profile_source"].encode()) == EXPECTED_FIELD_PROFILE_SOURCE_SHA256 and
            sha(data["native_field_profile_log"].encode()) == EXPECTED_FIELD_PROFILE_LOG_SHA256 and
            "native Bytes fields have no independent drop glue" in data["native_field_profile_log"],
            "captured default-native no-independent-field-drop profile changed")
    require("owned-view Clone native cases: 145" in data["native_test_log"],
            "captured native harness did not report all 145 owned-view Clone cases")
    return {
        "client_source_closed": True,
        "client_source_sha256": EXPECTED_NATIVE_SOURCE_SHA256,
        "test_source_sha256": EXPECTED_TEST_SOURCE_SHA256,
        "native_smoke": {"case_count": 145, "lengths": [1, 2, 7, 31],
            "ranges_deduplicated": True, "advance_values_deduplicated": True,
            "rounds": [0, 1, 2, 7, 31], "execution_only": True},
        "no_independent_field_drop_glue": {"source_sha256": EXPECTED_FIELD_PROFILE_SOURCE_SHA256,
            "log_sha256": EXPECTED_FIELD_PROFILE_LOG_SHA256, "captured_success": True},
    }


def audit_capture(data: dict[str, Any]) -> dict[str, Any]:
    c = data["capture"]
    require(sha(data["capture_raw"]) == EXPECTED_CAPTURE_SHA256,
            "pinned 30-body native capture receipt changed")
    require(c.get("stage") == "2-2-004.ElaborateDrops.after.mir", "native MIR stage changed")
    require(c.get("rustc_version") == EXPECTED_RUSTC.rstrip("\n") and
            data["rustc_text"] == EXPECTED_RUSTC, "captured rustc identity changed")
    require(c.get("cargo_version") == EXPECTED_CARGO.rstrip("\n") and
            data["cargo_text"] == EXPECTED_CARGO, "captured cargo identity changed")
    require(sha(data["capture_script"].encode()) == EXPECTED_CAPTURE_SCRIPT_SHA256,
            "native capture script changed")
    require(sha(data["native_manifest"].encode()) == EXPECTED_NATIVE_MANIFEST_SHA256 and
            tomllib.loads(data["native_manifest"]) == EXPECTED_NATIVE_MANIFEST,
            "native harness manifest/package route changed")
    require(sha(data["native_lock"].encode()) == EXPECTED_NATIVE_LOCK_SHA256,
            "native harness lockfile changed")
    require(sha(data["native_test_log"].encode()) == EXPECTED_NATIVE_LOG_SHA256 and
            "test result: ok. 1 passed" in data["native_test_log"] and
            "test result: FAILED" not in data["native_test_log"],
            "captured native harness run no longer records the one passing test")
    require("cargo test --locked --offline" in data["capture_script"] and
            "cargo rustc --locked --offline" in data["capture_script"] and
            "-Zdump-mir=all" in data["capture_script"] and "-Zmir-opt-level=0" in data["capture_script"],
            "native capture did not pin offline test and unoptimized MIR commands")
    require("creusot" not in data["capture_script"].lower() and "why3" not in data["capture_script"].lower(),
            "native capture script unexpectedly invokes proof tooling")

    rows = c.get("selected", [])
    labels = {row.get("label") for row in rows}
    require(len(rows) == 30 and len(labels) == 30, "capture must select 30 unique MIR bodies")
    require("client" in labels and {"from_box", "clone_impl", "slice", "advance", "chunk",
            "shared_clone", "shared_drop", "release_shared", "free_shared"} <= labels,
            "captured MIR omitted a selected production callback or client body")
    require(sum(label != "client" for label in labels) == 29,
            "capture must include 29 production bodies plus one client")
    by = {row["label"]: row for row in rows}
    require(set(data["mir_sources"]) == labels, "loaded MIR source label set differs from the pinned capture")
    resolved = {}
    for row in rows:
        label = row["label"]
        rel = row.get("path", "")
        require(rel.startswith("native-mir/") and pathlib.PurePosixPath(rel).name == rel.split("/", 1)[1],
                f"MIR capture path is not a direct native-mir member: {label}")
        actual_path = (ROOT / rel).resolve()
        require(actual_path == (MIR_DIR / pathlib.PurePosixPath(rel).name).resolve(),
                f"MIR capture path redirects: {label}")
        require(actual_path.is_file() and sha(data["mir_sources"][label].encode()) == row.get("sha256"),
                f"MIR bytes do not match the pinned capture row: {label}")
        resolved[label] = str(actual_path)
    mir_files = list(MIR_DIR.glob("*.mir"))
    require(len(mir_files) == 30 and {p.name for p in mir_files} ==
            {pathlib.PurePosixPath(row["path"]).name for row in rows},
            "native-mir directory has missing or extra selected bodies")
    inherited = {name: body for name, body in data["as_base"]["mir_sources"].items() if name != "client"}
    selected_production = {name: body for name, body in data["mir_sources"].items() if name != "client"}
    require(len(inherited) == 29 and selected_production == inherited,
            "AT production callback MIR is not byte-exact to its pinned AS ancestor")
    as_capture = data["as_base"]["capture"]
    for key in ("production_manifest_sha256", "production_source_sha256"):
        require(c.get(key) == as_capture.get(key),
                f"AT captured production input differs from AS at {key}")
    return {"selected_mir_count": 30, "production_mir_count": 29,
            "client_mir_count": 1, "stage": c["stage"], "rustc": EXPECTED_RUSTC.strip(),
            "cargo": EXPECTED_CARGO.strip(), "selected_paths": resolved,
            "all_selected_mir_hashes_match_capture": True,
            "inherited_as_production_mirs_byte_exact": 29,
            "capture_script_sha256": EXPECTED_CAPTURE_SCRIPT_SHA256,
            "native_manifest_exact": True, "native_lock_pinned": True,
            "native_test_log_passed": True}


def audit_client_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]["client"]
    require("fn owned_view_clone_scope(_1: Box<[u8]>, _2: usize, _3: usize, _4: usize, _5: usize) -> Vec<u8> {" in mir,
            "owned-view client MIR function header changed")
    bs = blocks_of(mir, "owned_view_clone_scope")
    expected_blocks = ({(i, False) for i in range(11)} |
                       {(i, False) for i in range(12, 20)} |
                       {(11, True)} | {(i, True) for i in range(20, 27)})
    require(set(bs) == expected_blocks,
            "owned-view client complete normal/cleanup CFG block set changed")

    # Construction retains the native From<Box>, Clone, Range specialization,
    # and owner slice; the two temporary owners are dropped before mutation.
    checks = {
        (0, False): ["_9 = move _1;",
            "_8 = <bytes::Bytes as From<Box<[u8]>>>::from(move _9) -> [return: bb1, unwind: bb24];"],
        (1, False): ["_10 = &'_ _8;",
            "_7 = <bytes::Bytes as Clone>::clone(move _10) -> [return: bb2, unwind: bb23];"],
        (2, False): ["drop(_8) -> [return: bb3, unwind: bb25];"],
        (3, False): ["_11 = &'_ _7;",
            "_6 = bytes::Bytes::slice::<std::ops::Range<usize>>(move _11, move _12) -> [return: bb4, unwind: bb22];"],
        (4, False): ["drop(_7) -> [return: bb5, unwind: bb25];"],
        (5, False): ["_16 = &'_ mut _6;", "_17 = copy _4;",
            "_15 = <bytes::Bytes as Buf>::advance(move _16, move _17) -> [return: bb6, unwind: bb21];"],
        (6, False): ["_18 = const 0_usize;", "goto -> bb7;"],
        (7, False): ["_22 = copy _18;", "_23 = copy _5;",
            "_21 = Lt(move _22, move _23);",
            "switchInt(move _21) -> [0: bb15, otherwise: bb8];"],
        (8, False): ["_25 = &'_ _6;",
            "_24 = <bytes::Bytes as Clone>::clone(move _25) -> [return: bb9, unwind: bb21];"],
        (9, False): ["_26 = move _24;", "drop(_6) -> [return: bb10, unwind: bb11];"],
        (10, False): ["_6 = move _26;", "goto -> bb12;"],
        (11, True): ["_6 = move _26;", "goto -> bb20;"],
        (12, False): ["_27 = AddWithOverflow(copy _18, const 1_usize);",
            "assert(!move (_27.1: bool), \"attempt to compute `{} + {}`, which would overflow\", copy _18, const 1_usize) -> [success: bb13, unwind: bb20];"],
        (13, False): ["_18 = move (_27.0: usize);", "goto -> bb14;"],
        (14, False): ["goto -> bb7;"],
        (15, False): ["_33 = &'_ _6;",
            "_32 = <bytes::Bytes as Buf>::chunk(move _33) -> [return: bb16, unwind: bb21];"],
        (16, False): ["_0 = slice::<impl [u8]>::to_vec(move _31) -> [return: bb17, unwind: bb21];"],
        (17, False): ["drop(_6) -> [return: bb18, unwind: bb25];"],
        (19, False): ["return;"],
    }
    for key, wanted in checks.items():
        got = [AI.rust_tokens(x) for x in significant(bs[key])]
        require(all(AI.rust_tokens(item) in got for item in wanted),
                f"owned-view MIR bb{key[0]} lost an expected operation/order/edge")

    require("_31 = &'_ (*_32);" in significant(bs[(16, False)]),
            "final chunk bytes are no longer borrowed into the Vec result")
    require("StorageDead(_18);" in bs[(17, False)],
            "loop counter lifetime no longer ends on the loop exit")
    require("StorageDead(_6);" in bs[(18, False)] and "goto -> bb19;" in bs[(18, False)],
            "final value storage does not end after its normal Drop")

    expected_drops = [
        (2, "_8", "original", "bb3", "bb25", False, "scope", "scope_exit"),
        (4, "_7", "owner", "bb5", "bb25", False, "detached", "scope_exit"),
        (9, "_6", "value", "bb10", "bb11", True, "detached", "replacement"),
        (17, "_6", "value", "bb18", "bb25", False, "detached", "final_return"),
    ]
    actual_drop_rows = []
    for block, place, owner, succ, unwind, repeated, scope, role in expected_drops:
        drops = [line.strip() for line in bs[(block, False)].splitlines()
                 if re.search(rf"\bdrop\({re.escape(place)}(?:: bytes::Bytes)?\)", line)]
        require(len(drops) == 1, f"expected one native Bytes Drop for {owner} in bb{block}")
        dm = re.fullmatch(rf"drop\({re.escape(place)}(?:: bytes::Bytes)?\) -> \[return: bb(\d+), unwind: (bb\d+|terminate\(cleanup\))\];", drops[0])
        require(dm is not None and f"bb{dm.group(1)}" == succ and dm.group(2) == unwind,
                f"native normal Drop edge changed for {owner}: {drops[0]}")
        actual_drop_rows.append({"block": f"bb{block}", "place": place, "owner": owner,
            "successor": succ, "unwind": unwind, "repeated": repeated, "scope": scope, "role": role})
    all_normal_drops = [(key[0], match.group(1)) for key, body in bs.items() if not key[1]
                        for match in re.finditer(r"\bdrop\((\_\d+)(?:: bytes::Bytes)?\)", body)]
    require(all_normal_drops == [(2, "_8"), (4, "_7"), (9, "_6"), (17, "_6")],
            "client normal Bytes Drop set contains an unreviewed, omitted, or reordered Drop")

    mapping = json.loads(data["mapping_text"])
    require(mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == EXPECTED_NATIVE_SOURCE_SHA256,
            "mapping does not bind the actual selected native owned-view source")
    capture_row = next(row for row in data["capture"].get("selected", []) if row.get("label") == "client")
    require(mapping.get("native_client_mir") == capture_row.get("path") and
            mapping.get("native_mir_ready") is True,
            "mapping does not bind the selected captured native owned-view MIR")
    debug_places = dict(re.findall(r"debug\s+(\w+)\s*=>\s*(_\d+)\s*;", mir))
    require(debug_places == {"input": "_1", "a": "_2", "b": "_3", "advance_by": "_4",
            "rounds": "_5", "value": "_6", "owner": "_7", "original": "_8",
            "i": "_18", "next": "_24"},
            "native client local-role table changed")
    require(mapping.get("debug_places") == debug_places,
            "mapping debug place table differs from parsed client MIR")
    actual_blocks = [{"block": f"bb{n}", "cleanup": cleanup,
                      "body_sha256": sha(("\n" + body).encode())}
                     for (n, cleanup), body in bs.items()]
    require(mapping.get("mir_blocks") == actual_blocks,
            "mapping block-body inventory/hash differs from independently parsed client MIR")
    selected_mir_rows = sorted(({"path": row["path"], "sha256": row["sha256"]}
                                for row in data["capture"]["selected"]), key=lambda row: row["path"])
    require(mapping.get("mir") == selected_mir_rows,
            "mapping selected MIR path/hash list differs from native capture")
    require(mapping.get("normal_edges") == actual_drop_rows,
            "native parsed Drop edges do not exactly match generated mapping normal_edges")

    native_assignment = {
        "next_place": "_24", "temporary_place": "_26", "value_place": "_6",
        "stash_block": "bb9", "stash_statement": "_26 = move _24;",
        "drop_block": "bb9", "install_block": "bb10",
        "install_statement": "_6 = move _26;",
        "normal_successor": "bb10", "unwind_successor": "bb11",
    }
    bb9_tokens = [AI.rust_tokens(x) for x in significant(bs[(9, False)])]
    bb10_tokens = [AI.rust_tokens(x) for x in significant(bs[(10, False)])]
    stash_index = bb9_tokens.index(AI.rust_tokens(native_assignment["stash_statement"])) if \
        AI.rust_tokens(native_assignment["stash_statement"]) in bb9_tokens else -1
    drop_index = bb9_tokens.index(AI.rust_tokens("drop(_6) -> [return: bb10, unwind: bb11];")) if \
        AI.rust_tokens("drop(_6) -> [return: bb10, unwind: bb11];") in bb9_tokens else -1
    install_index = bb10_tokens.index(AI.rust_tokens(native_assignment["install_statement"])) if \
        AI.rust_tokens(native_assignment["install_statement"]) in bb10_tokens else -1
    require(stash_index >= 0 and drop_index > stash_index and install_index >= 0,
            "assignment MIR no longer stashes next, drops old value, then installs next")
    native_saved_return = {"block": "bb16", "result_place": "_0",
        "successor": "bb17", "unwind": "bb21"}
    require(mapping.get("native_assignment") == native_assignment and
            mapping.get("native_saved_return") == native_saved_return,
            "native assignment/return mapping differs from independently parsed MIR")

    cleanup_drops = []
    for block, place, succ in [(20, "_6", "bb25"), (21, "_6", "bb25"),
                               (22, "_7", "bb25"), (23, "_8", "bb25")]:
        matches = [line.strip() for line in bs[(block, True)].splitlines() if re.search(r"\bdrop\(", line)]
        if matches:
            require(len(matches) == 1 and re.search(rf"drop\({re.escape(place)}\).*return: {succ}", matches[0]),
                    f"cleanup Drop place/order changed in bb{block}")
            cleanup_drops.append({"block": f"bb{block}", "place": place, "successor": succ})

    return {
        "normal_cfg_exact": True,
        "normal_edges": actual_drop_rows,
        "cleanup_edges": cleanup_drops,
        "debug_places": debug_places,
        "mir_blocks": actual_blocks,
        "native_assignment": native_assignment,
        "native_saved_return": native_saved_return,
        "normal_drop_order": ["Original before slicing", "Owner before advancing", 
            "old value after each successful clone and before installing next", 
            "final value after Vec result is evaluated"],
        "loop": {"condition": "i < rounds", "clone_receiver": "value",
            "replacement_order": ["clone value into next", "Drop old value", "move next into value", "increment i"],
            "repeated_drop_block": "bb9", "move_next_block": "bb10",
            "increment_block": "bb12-bb13", "backedge": "bb14 -> bb7",
            "exit": "bb7 -> bb15", "no_rounds_quota": True},
        "return_evaluation": {"chunk_receiver": "value", "to_vec_block": "bb16",
            "Vec_result_written_to_return_place_before_final_Drop": True,
            "final_value_drop_block": "bb17"},
        "normal_completion_only": True,
        "unwind_claim": False,
    }


def audit_cursor_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]
    inc = blocks_of(mir["inc_start"], "Bytes::inc_start")
    require(set(inc) == {(i, False) for i in range(9)}, "inc_start CFG block set changed")
    require("_6 = Ge(move _7, move _8);" in significant(inc[(1, False)]) and
            "switchInt(move _6) -> [0: bb3, otherwise: bb2];" in significant(inc[(1, False)]),
            "inc_start debug bound guard CFG changed")
    require("_13 = SubWithOverflow(copy ((*_1).1: usize), copy _12);" in significant(inc[(6, False)]) and
            any("assert(!move (_13.1: bool)" in x and "success: bb7" in x for x in significant(inc[(6, False)])),
            "inc_start length subtraction/overflow edge changed")
    inc7 = significant(inc[(7, False)])
    order = [next((i for i, x in enumerate(inc7) if needle in x), -1) for needle in
             ["((*_1).1: usize) = move (_13.0: usize);", "core::ptr::const_ptr::<impl *const u8>::add(move _15, move _16)"]]
    require(order[0] >= 0 and order[1] > order[0],
            "inc_start MIR no longer updates len before native ptr.add")
    require("((*_1).0: *const u8) = move _14;" in significant(inc[(8, False)]),
            "inc_start MIR does not store the adjusted pointer")
    require(all("((*_1).2:" not in line and "((*_1).3:" not in line
                for _, body in inc.items() for line in significant(body)),
            "inc_start MIR unexpectedly mutates the data field or vtable")

    advance = blocks_of(mir["advance"], "Buf::advance")
    require(set(advance) == {(i, False) for i in range(9)}, "Buf::advance CFG block set changed")
    require("_4 = Le(move _5, move _6);" in significant(advance[(1, False)]) and
            "switchInt(move _4) -> [0: bb3, otherwise: bb2];" in significant(advance[(1, False)]),
            "Buf::advance bounds-check dispatch changed")
    require(any("bytes::Bytes::inc_start(move _26, move _27) -> [return: bb8, unwind continue];" in x
                for x in significant(advance[(2, False)])),
            "Buf::advance success no longer calls Bytes::inc_start")
    require("panic_fmt" in " ".join(significant(advance[(3, False)])) or
            "panic_fmt" in " ".join(significant(advance[(7, False)])),
            "Buf::advance failed bounds path no longer panics")

    remaining = blocks_of(mir["remaining"], "Buf::remaining")
    require(set(remaining) == {(0, False), (1, False)} and
            any("bytes::Bytes::len(move _2)" in x for x in significant(remaining[(0, False)])),
            "Buf::remaining no longer forwards to Bytes::len")
    chunk = blocks_of(mir["chunk"], "Buf::chunk")
    require(set(chunk) == {(0, False), (1, False)} and
            any("bytes::Bytes::as_slice(move _3)" in x for x in significant(chunk[(0, False)])),
            "Buf::chunk no longer forwards to Bytes::as_slice")
    length = blocks_of(mir["len"], "Bytes::len")
    require(set(length) == {(0, False)} and "_0 = copy ((*_1).1: usize);" in significant(length[(0, False)]),
            "Bytes::len no longer reads only the len field")

    # Every Bytes destructor still loads its stored vtable's drop callback and
    # passes actual data, ptr, and current len once, including len == 0.
    drop = blocks_of(mir["bytes_drop"], "Bytes::drop")
    require(set(drop) == {(0, False), (1, False)}, "Bytes::drop callback CFG changed")
    sig = " ".join(significant(drop[(0, False)]))
    require("((*_1).3: &bytes::Vtable)" in sig and "((*_7).4:" in sig and
            "((*_1).2: core::sync::atomic::Atomic<*mut ()>)" in sig and
            "((*_1).0: *const u8)" in sig and "((*_1).1: usize)" in sig and
            "_0 = move _2(move _3, move _5, move _6)" in sig,
            "Bytes::drop no longer makes the single stored-vtable callback with actual fields")
    drop_calls = [line for _, body in drop.items() for line in significant(body)
                  if re.search(r"_0 = move _\d+\(move _\d+, move _\d+, move _\d+\)", line)]
    require(len(drop_calls) == 1 and all("switchInt" not in line for body in drop.values()
                                         for line in significant(body)),
            "Bytes::drop has an extra callback or branches to classify its effect by len")
    return {
        "inc_start": {"normal_cfg_exact": True, "debug_bounds_guard": True,
            "length_subtraction_precedes_native_ptr_add": True,
            "updates_only_pointer_and_len": True, "native_cfg_selected_ptr_add": True},
        "buf_advance": {"normal_cfg_exact": True, "bounds_assertion_precedes_inc_start": True,
            "success_calls_inc_start": True, "failure_panics": True},
        "buf_remaining_calls_len": True,
        "buf_chunk_calls_as_slice": True,
        "bytes_len_reads_len_field": True,
        "bytes_drop": {"single_stored_vtable_drop_call": True,
            "passes_actual_data_ptr_len": True, "no_len_based_callback_selection": True},
    }


def audit_bundle(data: dict[str, Any] | None = None) -> dict[str, Any]:
    if data is None:
        data = load_bundle()
    ancestor = audit_ancestor(data)
    cursor_source = audit_cursor_source(data)
    source_profile = audit_source_and_profile(data)
    capture = audit_capture(data)
    client = audit_client_mir(data)
    cursor_mir = audit_cursor_mir(data)
    return {
        "status": "pass",
        "checker_scope": "AT Bytes owned-view Clone source/MIR correspondence and inherited AS production source/MIR profile; no proof or MIR-adequacy claim",
        "ancestor_as": ancestor,
        "reviewed_production_inputs": ancestor["reviewed_production_inputs"],
        "production_callbacks": ancestor["production_callbacks"],
        "terminal_field_profile": ancestor["terminal_field_profile"],
        "cursor_source": cursor_source,
        "source_profile": source_profile,
        "capture": capture,
        "client": client,
        "cursor_mir": cursor_mir,
        "native_mir": {"selected_mir_count": 30, "selected_production_mir_count": 29,
            "client_mir_count": 1, "stage": capture["stage"],
            "selected_hashes_match_pinned_capture": True,
            "inherited_as_production_mirs_byte_exact": capture["inherited_as_production_mirs_byte_exact"]},
        "native_audit": {
            "cursor_source": cursor_source,
            "source_body_names": cursor_source["source_body_names"],
            "source_body_hashes": cursor_source["source_body_hashes"],
            "client": client,
            "normal_edges": client["normal_edges"],
            "cursor_mir": cursor_mir,
            "normal_drop_order": client["normal_drop_order"],
            "selected_mir_count": 30,
            "production_mir_count": 29,
        },
        "limits": [
            "Native MIR/source correspondence is not a proof of MIR adequacy or Rust ownership semantics.",
            "Only the selected normal completion of this cursor client is mapped; unwind remains excluded.",
            "Generic pointer provenance and compiler interpretation remain external TCB.",
            "The captured runtime smoke test corroborates execution only.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        result = audit_bundle()
    except Exception as exc:
        result = {"status": "reject", "checker_scope": "AT owned-view source/MIR correspondence",
                  "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
