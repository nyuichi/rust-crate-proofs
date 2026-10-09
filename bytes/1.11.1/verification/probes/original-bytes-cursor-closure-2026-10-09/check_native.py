#!/usr/bin/env python3
"""Read-only native source/MIR correspondence check for the AR cursor witness.

This checker performs no Cargo, rustc, Creusot, Why3, or solver invocation.
It binds the captured production bodies and post-ElaborateDrops MIR to the
selected default-native cursor client. It is not a proof of MIR adequacy or
Rust pointer-provenance semantics.
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
AQ_PROBE = PROBES / "original-shared-slice-views-2026-10-09"
AQ_CHECKER_PATH = AQ_PROBE / "check_native.py"
AQ_CHECKER_SHA256 = "0c5099600ba7e9bc4ce18f52d1926f9df3c079b5da91809517f492d533a43eb3"
MIR_DIR = ROOT / "native-mir"
CAPTURE_PATH = MIR_DIR / "capture.json"
EXPECTED_CAPTURE_SHA256 = "bd5d5254c0329d29b906fa02c07528e91be383334929fb455cb00d5194e814ad"
EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\ncommit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\nhost: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\nLLVM version: 22.1.7\n")
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"
EXPECTED_NATIVE_SOURCE_SHA256 = "b0b2519d79479374eb31ab954180f11bd1030931d5057627c95e3e45a5b5c371"
EXPECTED_TEST_SOURCE_SHA256 = "c32d038f5df5cfc156847045c36e34df0eeb59b6ed08b870438464e0db16e830"
EXPECTED_CAPTURE_SCRIPT_SHA256 = "8e39ea565d783eafebe11126eed3c7d2321898a1e6f55aa505dcf7fc9fc7bdf1"
EXPECTED_NATIVE_MANIFEST_SHA256 = "f1a971b05091d841f798063349776a3c00f57d21ebf8bf4d0f1172c10983f20e"
EXPECTED_NATIVE_LOCK_SHA256 = "9a188a683d11b3245acc04a667561053d53a273e8d5e4a7b458399efe1c9a0ab"
EXPECTED_NATIVE_LOG_SHA256 = "8bc041bac76ea6f2c381053d70355406df61ee8601ea1e582f0d9dc66f1886c3"
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
    "package": {"name": "bytes-cursor-closure-native", "version": "0.0.0", "edition": "2021"},
    "workspace": {},
    "lib": {"name": "bytes_cursor_closure_native", "path": "../native.rs"},
    "dependencies": {"bytes": {"path": "../../../../"}},
}
EXPECTED_NATIVE_SOURCE = r"""use bytes::{Buf, Bytes};

pub fn cursor_scope(input: Box<[u8]>, a: usize, b: usize, steps: &[usize]) -> Vec<u8> {
    let mut value = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        owner.slice(a..b)
    };
    let mut i = 0;
    while i < steps.len() {
        let by = core::cmp::min(steps[i], value.remaining());
        value.advance(by);
        i += 1;
    }
    let observed = value.chunk().to_vec();
    let rest = value.remaining();
    value.advance(rest);
    assert_eq!(value.remaining(), 0);
    assert!(value.chunk().is_empty());
    observed
}
"""
EXPECTED_NATIVE_TEST = r"""use bytes_cursor_closure_native::cursor_scope;

#[test]
fn advances_preserve_suffix_and_owned_empty_drop() {
    let runs: &[&[usize]] = &[&[], &[0], &[1], &[usize::MAX], &[1,0,1,2,0,usize::MAX,7], &[usize::MAX,usize::MAX,0], &[2,3,5]];
    for len in [1usize, 2, 7, 31] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        for (a,b) in [(0,len),(len/2,len),(0,0),(len,len),(len/2,len/2)] {
            for steps in runs {
                let mut consumed = 0;
                for by in *steps { consumed += core::cmp::min(*by,b-a-consumed); }
                assert_eq!(cursor_scope(expected.clone().into_boxed_slice(),a,b,steps), expected[a+consumed..b]);
            }
        }
    }
}"""


class AuditError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_aq_checker():
    require(AQ_CHECKER_PATH.is_file() and sha(AQ_CHECKER_PATH.read_bytes()) == AQ_CHECKER_SHA256,
            "published AQ native checker changed before import")
    spec = importlib.util.spec_from_file_location("ar_pinned_aq_native", AQ_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load pinned AQ native checker")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


AQ = load_aq_checker()
AI = AQ.AI


def load_bundle() -> dict[str, Any]:
    """Load exact captured AR values plus the already-published AQ baseline."""
    aq = AQ.load_bundle()
    cap_raw = CAPTURE_PATH.read_bytes()
    capture = json.loads(cap_raw)
    selected = capture.get("selected", [])
    mir_sources = {}
    for row in selected:
        path = ROOT / row["path"]
        mir_sources[row["label"]] = path.read_text()
    return {
        "aq_base": aq,
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
        "production_source": aq["production_source"],
        "production_source_inputs": aq["production_source_inputs"],
        "reviewed_production_manifest": aq["reviewed_production_manifest"],
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
    # The reviewed whole production source is hash pinned by AQ. Still require a
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
    # Recheck the published AQ gate in its original path layout. This call is
    # pure parsing/hash validation; it invokes no build or proof tool.
    try:
        result = AQ.audit_bundle(data["aq_base"])
    except Exception as exc:
        raise AuditError(f"published AQ native baseline rejected: {type(exc).__name__}: {exc}") from exc
    require(result.get("status") == "pass", "published AQ native correspondence baseline no longer passes")
    return {
        "status": "pass",
        "checker_sha256": AQ_CHECKER_SHA256,
        "aq_native_mir_count": result["native_audit"]["selected_mir_count"],
        "aq_inherited_production_mir_count": result["native_audit"]["production_mir_count"],
        "aq_source_map_cursor_extension_unchanged": True,
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
    require_tokens(data["native_source"], EXPECTED_NATIVE_SOURCE, "native cursor client")
    require(sha(data["native_source"].encode()) == EXPECTED_NATIVE_SOURCE_SHA256,
            "native cursor client hash changed")
    require_tokens(data["native_test_source"], EXPECTED_NATIVE_TEST, "native cursor smoke test")
    require(sha(data["native_test_source"].encode()) == EXPECTED_TEST_SOURCE_SHA256,
            "native cursor smoke-test hash changed")
    require(sha(data["native_field_profile_source"].encode()) == EXPECTED_FIELD_PROFILE_SOURCE_SHA256 and
            sha(data["native_field_profile_log"].encode()) == EXPECTED_FIELD_PROFILE_LOG_SHA256 and
            "native Bytes fields have no independent drop glue" in data["native_field_profile_log"],
            "captured default-native no-independent-field-drop profile changed")
    return {
        "client_source_closed": True,
        "client_source_sha256": EXPECTED_NATIVE_SOURCE_SHA256,
        "test_source_sha256": EXPECTED_TEST_SOURCE_SHA256,
        "native_smoke": {"case_count": 140, "lengths": [1, 2, 7, 31],
            "outer_range_cases_per_length": 5, "step_sequences": 7, "execution_only": True},
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
    require("client" in labels and {"inc_start", "remaining", "chunk", "advance", "len"} <= labels,
            "captured MIR omitted a cursor or client body")
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
    inherited = {name: body for name, body in data["aq_base"]["mir_sources"].items() if name != "client"}
    require(len(inherited) == 24 and all(data["mir_sources"].get(name) == body
                                        for name, body in inherited.items()),
            "AR production callback MIR is not byte-exact to its pinned AQ ancestor")
    return {"selected_mir_count": 30, "production_mir_count": 29,
            "client_mir_count": 1, "stage": c["stage"], "rustc": EXPECTED_RUSTC.strip(),
            "cargo": EXPECTED_CARGO.strip(), "selected_paths": resolved,
            "all_selected_mir_hashes_match_capture": True,
            "inherited_aq_production_mirs_byte_exact": 24,
            "capture_script_sha256": EXPECTED_CAPTURE_SCRIPT_SHA256,
            "native_manifest_exact": True, "native_lock_pinned": True,
            "native_test_log_passed": True}


def audit_client_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]["client"]
    require("fn cursor_scope(_1: Box<[u8]>, _2: usize, _3: usize, _4: &[usize]) -> Vec<u8> {" in mir,
            "client MIR function header changed")
    bs = blocks_of(mir, "cursor_scope")
    require(set(bs) == {(i, False) for i in range(29)} | {(i, True) for i in range(29, 36)},
            "cursor client complete normal/cleanup CFG block set changed")
    for key, snippets in {
        (0, False): ["_8 = move _1;",
                     "_7 = <bytes::Bytes as From<Box<[u8]>>>::from(move _8) -> [return: bb1, unwind: bb33];"],
        (1, False): ["_9 = &'_ _7;",
                     "_6 = <bytes::Bytes as Clone>::clone(move _9) -> [return: bb2, unwind: bb32];"],
        (2, False): ["drop(_7) -> [return: bb3, unwind: bb34];"],
        (3, False): ["_10 = &'_ _6;",
                     "_5 = bytes::Bytes::slice::<std::ops::Range<usize>>(move _10, move _11) -> [return: bb4, unwind: bb31];"],
        (4, False): ["drop(_6) -> [return: bb5, unwind: bb34];"],
    }.items():
        sig = [AI.rust_tokens(x) for x in significant(bs[key])]
        for snippet in snippets:
            require(AI.rust_tokens(snippet) in sig,
                    f"cursor construction/initial-owner MIR bb{key[0]} lost `{snippet}`")
    # Loop: the only bound is steps.len(); each iteration indexes one step,
    # clamps it to remaining, advances, increments i, and takes the backedge.
    checks = {
        (5, False): ["_14 = const 0_usize;", "goto -> bb6;"],
        (7, False): ["_17 = Lt(move _18, move _19);",
                    "switchInt(move _17) -> [0: bb14, otherwise: bb8];"],
        (8, False): ["assert(move _25, \"index out of bounds: the length is {} but the index is {}\", move _24, copy _23) -> [success: bb9, unwind: bb30];"],
        (9, False): ["_22 = copy (*_4)[_23];",
                    "_26 = <bytes::Bytes as Buf>::remaining(move _27) -> [return: bb10, unwind: bb30];"],
        (10, False): ["_21 = std::cmp::min::<usize>(move _22, move _26) -> [return: bb11, unwind: bb30];"],
        (11, False): ["_29 = &'_ mut _5;", "_30 = copy _21;",
                     "_28 = <bytes::Bytes as Buf>::advance(move _29, move _30) -> [return: bb12, unwind: bb30];"],
        (12, False): ["_31 = AddWithOverflow(copy _14, const 1_usize);",
                     "assert(!move (_31.1: bool), \"attempt to compute `{} + {}`, which would overflow\", copy _14, const 1_usize) -> [success: bb13, unwind: bb30];"],
        (13, False): ["_14 = move (_31.0: usize);", "goto -> bb6;"],
    }
    for key, wanted in checks.items():
        got = [AI.rust_tokens(x) for x in significant(bs[key])]
        want = [AI.rust_tokens(x) for x in wanted]
        require(all(w in got for w in want), f"cursor loop bb{key[0]} lost an expected operation/edge")
    # The final suffix observation and drain must happen before return, with the
    # Bytes handle's ordinary destructor after its Vec has been moved to _0.
    for key, snippets in {
        (14, False): ["_37 = <bytes::Bytes as Buf>::chunk(move _38) -> [return: bb15, unwind: bb30];"],
        (15, False): ["_35 = std::slice::<impl [u8]>::to_vec(move _36) -> [return: bb16, unwind: bb30];"],
        (16, False): ["_39 = <bytes::Bytes as Buf>::remaining(move _40) -> [return: bb17, unwind: bb29];"],
        (17, False): ["_41 = <bytes::Bytes as Buf>::advance(move _42, move _43) -> [return: bb18, unwind: bb29];"],
        (18, False): ["_47 = <bytes::Bytes as Buf>::remaining(move _48) -> [return: bb19, unwind: bb29];"],
        (20, False): ["_68 = <bytes::Bytes as Buf>::chunk(move _69) -> [return: bb22, unwind: bb29];"],
        (22, False): ["_66 = core::slice::<impl [u8]>::is_empty(move _67) -> [return: bb23, unwind: bb29];"],
        (24, False): ["_0 = move _35;", "goto -> bb26;"],
    }.items():
        sig = [AI.rust_tokens(x) for x in significant(bs[key])]
        for snippet in snippets:
            require(AI.rust_tokens(snippet) in sig,
                    f"cursor final read/drain/check bb{key[0]} lost `{snippet}`")

    # Parse each ordinary Bytes Drop from MIR and bind its normal target and
    # unwind target. No role is inferred from len: value uses the native vtable.
    expected = [
        (2, "_7", "original", "bb3", "bb34", "scope"),
        (4, "_6", "owner", "bb5", "bb34", "detached"),
        (26, "_5", "value", "bb27", "bb34", "detached"),
    ]
    actual_drop_rows = []
    for block, place, owner, succ, unwind, scope in expected:
        drops = [line.strip() for line in bs[(block, False)].splitlines()
                 if re.search(rf"\bdrop\({re.escape(place)}(?:: bytes::Bytes)?\)", line)]
        require(len(drops) == 1, f"expected one native Bytes Drop for {owner} in bb{block}")
        dm = re.fullmatch(rf"drop\({re.escape(place)}(?:: bytes::Bytes)?\) -> \[return: bb(\d+), unwind: bb(\d+)\];", drops[0])
        require(dm is not None and f"bb{dm.group(1)}" == succ and f"bb{dm.group(2)}" == unwind,
                f"native normal Drop edge changed for {owner}: {drops[0]}")
        actual_drop_rows.append({"block": f"bb{block}", "place": place, "owner": owner,
            "successor": succ, "unwind": unwind, "repeated": False, "scope": scope})
    all_normal_drops = [(key[0], m.group(1)) for key, body in bs.items() if not key[1]
                        for m in re.finditer(r"\bdrop\((_\d+)(?:: bytes::Bytes)?\)", body)]
    require(all_normal_drops == [(2, "_7"), (4, "_6"), (26, "_5")],
            "client normal Bytes Drop set contains an unreviewed or reordered Drop")
    mapping = json.loads(data["mapping_text"])
    require(mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == EXPECTED_NATIVE_SOURCE_SHA256,
            "mapping does not bind the actual selected native cursor source")
    capture_row = next(row for row in data["capture"].get("selected", []) if row.get("label") == "client")
    require(mapping.get("native_client_mir") == capture_row.get("path") and
            mapping.get("native_mir_ready") is True,
            "mapping does not bind the selected captured native client MIR")
    debug_places = dict(re.findall(r"debug\s+(\w+)\s*=>\s*(_\d+)\s*;", mir))
    require(mapping.get("debug_places") == debug_places,
            "mapping debug place table differs from parsed client MIR")
    mapped_blocks = mapping.get("mir_blocks", [])
    actual_blocks = [{"block": f"bb{n}", "cleanup": cleanup,
                      "body_sha256": sha(("\n" + body).encode())}
                     for (n, cleanup), body in sorted(bs.items())]
    require(mapped_blocks == actual_blocks,
            "mapping block-body inventory/hash differs from independently parsed client MIR")
    mapped_mir = mapping.get("mir", [])
    selected_mir_rows = sorted(({"path": row["path"], "sha256": row["sha256"]}
                                for row in data["capture"]["selected"]),
                               key=lambda row: row["path"])
    require(mapped_mir == selected_mir_rows,
            "mapping selected MIR path/hash list differs from native capture")
    require(mapping.get("normal_edges") == actual_drop_rows,
            "native parsed Drop edges do not exactly match generated mapping normal_edges")
    cleanup_drops = []
    for block, place, succ in [(29, "_35", "bb30"), (30, "_5", "bb34"),
                               (31, "_6", "bb34"), (32, "_7", "bb34")]:
        drops = [x.strip() for x in bs[(block, True)].splitlines() if re.search(r"\bdrop\(", x)]
        require(len(drops) == 1 and re.search(rf"drop\({re.escape(place)}\).*return: {succ}", drops[0]),
                f"cleanup Drop place/order changed in bb{block}")
        cleanup_drops.append({"block": f"bb{block}", "place": place, "successor": succ})
    require("goto -> bb35;" in significant(bs[(34, True)]) and "resume;" in significant(bs[(35, True)]),
            "native cleanup tail no longer resumes through its captured cleanup ladder")
    return {
        "normal_cfg_exact": True,
        "normal_edges": actual_drop_rows,
        "cleanup_edges": cleanup_drops,
        "normal_drop_order": ["Original", "Owner", "value after saved Vec return"],
        "loop": {"condition": "i < steps.len()", "body_block": "bb8-bb12",
            "step": "min(steps[i], value.remaining())", "advance": "value.advance(by)",
            "increment": "i += 1", "backedge": "bb13 -> bb6", "exit": "bb7 -> bb14",
            "arbitrary_finite_steps_no_fixed_quota": True},
        "suffix_and_drain": {"observed_before_final_advance": True,
            "final_advance_argument": "rest = value.remaining()", "assert_remaining_zero": True,
            "assert_chunk_empty": True, "result_move_precedes_drop": True},
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
    reviewed = AQ.audit_reviewed_inputs(data["aq_base"])
    callbacks = AQ.AP.audit_production_sources(data["aq_base"])
    field_profile = AQ.AP.audit_terminal_field_profile(data["aq_base"])
    cursor_source = audit_cursor_source(data)
    source_profile = audit_source_and_profile(data)
    capture = audit_capture(data)
    client = audit_client_mir(data)
    cursor_mir = audit_cursor_mir(data)
    return {
        "status": "pass",
        "checker_scope": "AR Bytes cursor source/MIR correspondence and inherited AQ callback/field profile; no proof or MIR-adequacy claim",
        "ancestor_aq": ancestor,
        "reviewed_production_inputs": reviewed,
        "production_callbacks": callbacks,
        "terminal_field_profile": field_profile,
        "cursor_source": cursor_source,
        "source_profile": source_profile,
        "capture": capture,
        "client": client,
        "cursor_mir": cursor_mir,
        "native_mir": {"selected_mir_count": 30, "selected_production_mir_count": 29,
            "client_mir_count": 1, "stage": capture["stage"],
            "selected_hashes_match_pinned_capture": True,
            "inherited_aq_production_mirs_byte_exact": capture["inherited_aq_production_mirs_byte_exact"]},
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
        result = {"status": "reject", "checker_scope": "AR cursor source/MIR correspondence",
                  "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
