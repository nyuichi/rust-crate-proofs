#!/usr/bin/env python3
"""Read-only native source/MIR audit for the AU named owning-return call.

This checker invokes no Cargo, rustc, Creusot, Why3, or solver. It replays the
published AT native gate as its immutable prefix, compares every inherited
production MIR body, and independently parses the newly captured callee and
caller MIR. It does not establish MIR adequacy or Rust memory-model semantics.
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
PROBES = ROOT.parent
AT_PROBE = PROBES / "original-owned-view-clone-2026-10-09"
AT_CHECKER_PATH = AT_PROBE / "check_native.py"
AT_CHECKER_SHA256 = "57a47d7304777417b2514a0b8281983513c4a1d07612aa63a7a10ae80709bf92"
MIR_DIR = ROOT / "native-mir"
CAPTURE_PATH = MIR_DIR / "capture.json"
EXPECTED_CAPTURE_SHA256 = "8587b8887f698ebb943d7e85d221a659698e4851eb1e536fe4b876bad1357d12"
EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\ncommit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\nhost: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\nLLVM version: 22.1.7\n")
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"
EXPECTED_NATIVE_SOURCE = """use bytes::{Buf, Bytes};

pub fn clone_suffix(source: &Bytes, amount: usize) -> Bytes {
    let mut result = source.clone();
    result.advance(amount);
    result
}

pub fn owned_view_call_scope(input: Box<[u8]>, a: usize, b: usize, steps: &[usize]) -> Vec<u8> {
    let mut value = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        owner.slice(a..b)
    };
    let mut i = 0;
    while i < steps.len() {
        let amount = core::cmp::min(steps[i], value.remaining());
        let next = clone_suffix(&value, amount);
        value = next;
        i += 1;
    }
    value.chunk().to_vec()
}
"""
EXPECTED_TEST_SOURCE = """use bytes_owned_view_call_summaries_native::owned_view_call_scope;
use std::collections::BTreeSet;

#[test]
fn clone_witness() {
    let mut cases = 0;
    for len in [1usize, 2, 7, 31] {
        let input: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let ranges: BTreeSet<_> = [(0, len), (len / 2, len), (len - 1, len), (0, 1)].into_iter().collect();
        for (a, b) in ranges {
            let span = b - a;
            let sequences: BTreeSet<_> = [vec![], vec![0], vec![span], vec![span + 1], vec![usize::MAX], vec![1; span + 1], vec![0, 1, usize::MAX, 3]].into_iter().collect();
            for steps in sequences {
                let mut consumed = 0;
                for &step in &steps {
                    consumed += step.min(span - consumed);
                }
                assert_eq!(owned_view_call_scope(input.clone().into_boxed_slice(), a, b, &steps), input[a + consumed..b]);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 84);
    println!("modular owned-view native cases: {cases}");
}
"""
EXPECTED_NATIVE_SOURCE_SHA256 = "a9e889311f0a01fee178f4f9aa9ea1957cff59fb19221497b9b412fa8d8e66d9"
EXPECTED_TEST_SOURCE_SHA256 = "8a9762640bd08c8cf64003478b202925fa1ba040a1f3da0751be8caa58f17759"
EXPECTED_CAPTURE_SCRIPT_SHA256 = "2ca9612bcb9acd836d39ab1099e4f7aa880029edb8b3b66f07933751f42680c1"
EXPECTED_NATIVE_MANIFEST_SHA256 = "086a54f7126a814580f95fd126970d5ffcba21f334cae41802dd81273388d8d6"
EXPECTED_NATIVE_LOCK_SHA256 = "cb822a9917b8c14cb7536ba1215706b88c5d34b01953f1d9faf9e9acadad6bb1"
EXPECTED_NATIVE_LOG_SHA256 = "ee8cc210eb233fa94bd14a64da47258fab520f09115a5eccdff433c1e063f386"
EXPECTED_FIELD_PROFILE_SOURCE_SHA256 = "b8dae751e6c7dc007de636ed7cc8d9b6f98cc746252d7373e785778655b58183"
EXPECTED_FIELD_PROFILE_LOG_SHA256 = "3849089883705a9502911036f6510d56a263520626d4fc5ebc023aaa0530c436"
EXPECTED_NATIVE_MANIFEST = {
    "package": {"name": "bytes-owned-view-call-summaries-native", "version": "0.0.0", "edition": "2021"},
    "workspace": {},
    "lib": {"name": "bytes_owned_view_call_summaries_native", "path": "../native.rs"},
    "dependencies": {"bytes": {"path": "../../../../"}},
}
EXPECTED_LABELS = {
    "from_box", "clone_impl", "cleanup", "bytes_drop", "as_ref", "as_slice",
    "promotable_even_clone", "promotable_odd_clone", "shallow_clone_vec",
    "shallow_clone_arc", "shared_clone", "slice", "new_empty_with_ptr",
    "static_clone", "static_drop", "without_provenance", "inc_start", "remaining",
    "chunk", "advance", "len", "ref_count_increment", "promotable_even_drop",
    "promotable_odd_drop", "shared_drop", "release_shared", "free_shared",
    "ptr_map", "atomic_with_mut",
}
EXPECTED_CALLEE_MIR_SHA256 = "e4410520eb9af04ca79667a1bcf2b14604676b1f9a19d54683692c8f44b53519"
EXPECTED_CLIENT_MIR_SHA256 = "9d7287c7097400cc92e5dffe4ccf1292da1c5a4287b21065df9012070961113f"


class AuditError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def import_pinned_at():
    require(AT_CHECKER_PATH.is_file() and sha(AT_CHECKER_PATH.read_bytes()) == AT_CHECKER_SHA256,
            "published AT native checker changed before import")
    spec = importlib.util.spec_from_file_location("au_pinned_at_native", AT_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load pinned AT native checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


AT = import_pinned_at()
AI = AT.AI


def load_bundle() -> dict[str, Any]:
    """Read captured AU inputs and the pinned AT ancestor; no tools are run."""
    capture_raw = CAPTURE_PATH.read_bytes()
    capture = json.loads(capture_raw)
    mir_sources: dict[str, str] = {}
    for row in capture.get("selected", []):
        mir_sources[row["label"]] = (ROOT / row["path"]).read_text()
    return {
        "at_base": AT.load_bundle(),
        "capture": capture,
        "capture_raw": capture_raw,
        "mir_sources": mir_sources,
        "native_source": (ROOT / "native.rs").read_text(),
        "native_test_source": (ROOT / "native-test/tests/clone_witness.rs").read_text(),
        "native_test_log": (ROOT / "native-test/native-run.log").read_text(),
        "native_manifest": (ROOT / "native-test/Cargo.toml").read_text(),
        "native_lock": (ROOT / "native-test/Cargo.lock").read_text(),
        "capture_script": (ROOT / "capture-native.sh").read_text(),
        "rustc_text": (MIR_DIR / "rustc-version.txt").read_text(),
        "cargo_text": (MIR_DIR / "cargo-version.txt").read_text(),
        "native_field_profile_source": (ROOT / "native-field-profile.rs").read_text(),
        "native_field_profile_log": (ROOT / "native-field-profile.log").read_text(),
        "mapping": json.loads((ROOT / "generated/mapping.json").read_text()),
    }


def significant(body: str) -> list[str]:
    return [line.strip() for line in body.splitlines()
            if line.strip() and not line.strip().startswith(("StorageLive(", "StorageDead(", "debug ", "PlaceMention("))]


def block_rows(blocks: dict[tuple[int, bool], str]) -> list[dict[str, Any]]:
    return [{"block": f"bb{number}", "cleanup": cleanup,
             "body_sha256": sha(("\n" + body).encode())}
            for (number, cleanup), body in blocks.items()]


def debug_places(mir: str) -> dict[str, str]:
    return dict(re.findall(r"debug\s+(\w+)\s*=>\s*(_\d+)\s*;", mir))


def audit_ancestor(data: dict[str, Any]) -> dict[str, Any]:
    report = AT.audit_bundle(data["at_base"])
    require(report.get("status") == "pass", "published AT native baseline no longer passes its pinned source/MIR audit")
    return {"status": "pass", "checker_sha256": AT_CHECKER_SHA256,
            "selected_mir_count": report["native_audit"]["selected_mir_count"],
            "production_mir_count": report["native_audit"]["production_mir_count"]}


def audit_source_and_execution(data: dict[str, Any]) -> dict[str, Any]:
    require(data["native_source"] == EXPECTED_NATIVE_SOURCE and
            sha(data["native_source"].encode()) == EXPECTED_NATIVE_SOURCE_SHA256,
            "native callee/caller source differs from the frozen AU witness")
    require(data["native_test_source"] == EXPECTED_TEST_SOURCE and
            sha(data["native_test_source"].encode()) == EXPECTED_TEST_SOURCE_SHA256,
            "native 84-case witness source differs from the frozen capture")
    require("modular owned-view native cases: 84" in data["native_test_log"] and
            "test result: ok. 1 passed" in data["native_test_log"] and
            "test result: FAILED" not in data["native_test_log"] and
            sha(data["native_test_log"].encode()) == EXPECTED_NATIVE_LOG_SHA256,
            "captured native witness does not report its successful 84 execution cases")
    require(sha(data["native_field_profile_source"].encode()) == EXPECTED_FIELD_PROFILE_SOURCE_SHA256 and
            sha(data["native_field_profile_log"].encode()) == EXPECTED_FIELD_PROFILE_LOG_SHA256 and
            "native Bytes fields have no independent drop glue" in data["native_field_profile_log"],
            "inherited native field-drop profile changed")
    return {"source_sha256": EXPECTED_NATIVE_SOURCE_SHA256,
            "callee_source_signature": "fn clone_suffix(source: &Bytes, amount: usize) -> Bytes",
            "caller_source_signature": "fn owned_view_call_scope(input: Box<[u8]>, a: usize, b: usize, steps: &[usize]) -> Vec<u8>",
            "witness": {"case_count": 84, "lengths": [1, 2, 7, 31],
                "range_counts_by_length": [1, 3, 4, 4],
                "step_sequences_per_range": 7,
                "includes_zero_overshoot_and_usize_max": True,
                "execution_only": True},
            "native_field_profile": {"no_independent_drop_glue": True,
                "source_sha256": EXPECTED_FIELD_PROFILE_SOURCE_SHA256,
                "log_sha256": EXPECTED_FIELD_PROFILE_LOG_SHA256}}


def audit_capture(data: dict[str, Any]) -> dict[str, Any]:
    capture = data["capture"]
    require(sha(data["capture_raw"]) == EXPECTED_CAPTURE_SHA256,
            "pinned AU capture receipt changed")
    require(capture.get("stage") == "2-2-004.ElaborateDrops.after.mir",
            "captured MIR stage changed")
    require(capture.get("rustc_version") == EXPECTED_RUSTC.rstrip("\n") and
            data["rustc_text"] == EXPECTED_RUSTC and
            capture.get("cargo_version") == EXPECTED_CARGO.rstrip("\n") and
            data["cargo_text"] == EXPECTED_CARGO,
            "pinned rustc/cargo version changed")
    require(sha(data["capture_script"].encode()) == "2ca9612bcb9acd836d39ab1099e4f7aa880029edb8b3b66f07933751f42680c1",
            "native capture script differs from the actual AU capture")
    require("fn owned_view_call_scope(" in data["capture_script"] and
            "fn clone_suffix(" in data["capture_script"] and
            "cargo test --locked --offline" in data["capture_script"] and
            "cargo rustc --locked --offline" in data["capture_script"] and
            "-Zdump-mir=all" in data["capture_script"] and
            "-Zmir-opt-level=0" in data["capture_script"] and
            "creusot" not in data["capture_script"].lower() and
            "why3" not in data["capture_script"].lower(),
            "native capture does not compile/test and select both ordinary Rust bodies offline")
    require(sha(data["native_manifest"].encode()) == "086a54f7126a814580f95fd126970d5ffcba21f334cae41802dd81273388d8d6" and
            tomllib.loads(data["native_manifest"]) == EXPECTED_NATIVE_MANIFEST,
            "native harness manifest/package path changed")
    require(sha(data["native_lock"].encode()) == "cb822a9917b8c14cb7536ba1215706b88c5d34b01953f1d9faf9e9acadad6bb1",
            "native harness lockfile changed")

    rows = capture.get("selected", [])
    labels = {row.get("label") for row in rows}
    require(len(rows) == 31 and len(labels) == 31 and
            labels == EXPECTED_LABELS | {"client", "call_summary_body"},
            "capture must contain exactly 29 production bodies and the caller/callee bodies")
    require(set(data["mir_sources"]) == labels,
            "loaded MIR files do not equal selected capture labels")
    at_mirs = data["at_base"]["mir_sources"]
    inherited = {k: v for k, v in at_mirs.items() if k != "client"}
    actual_prod = {k: v for k, v in data["mir_sources"].items()
                   if k not in {"client", "call_summary_body"}}
    require(len(inherited) == 29 and actual_prod == inherited,
            "AU production MIR differs from the 29 byte-exact audited AT production MIR bodies")
    resolved = {}
    for row in rows:
        rel = row.get("path", "")
        label = row["label"]
        require(rel.startswith("native-mir/") and
                pathlib.PurePosixPath(rel).name == rel.split("/", 1)[1],
                f"MIR path is not a direct native-mir member: {label}")
        path = (ROOT / rel).resolve()
        require(path == (MIR_DIR / pathlib.PurePosixPath(rel).name).resolve() and path.is_file(),
                f"MIR path redirects or is missing: {label}")
        require(sha(path.read_bytes()) == row.get("sha256"),
                f"MIR file bytes differ from selected capture row: {label}")
        resolved[label] = str(path)
    mir_files = list(MIR_DIR.glob("*.mir"))
    require(len(mir_files) == 31 and
            {p.name for p in mir_files} == {pathlib.PurePosixPath(r["path"]).name for r in rows},
            "native-mir directory has a missing or unselected MIR member")
    at_capture = data["at_base"]["capture"]
    for k in ("production_manifest_sha256", "production_source_sha256"):
        require(capture.get(k) == at_capture.get(k), f"AU production input differs from AT at {k}")
    require(capture.get("native_source_sha256") == EXPECTED_NATIVE_SOURCE_SHA256 and
            capture.get("native_test_source_sha256") == EXPECTED_TEST_SOURCE_SHA256 and
            capture.get("native_manifest_sha256") == sha(data["native_manifest"].encode()) and
            capture.get("native_lock_sha256") == sha(data["native_lock"].encode()) and
            capture.get("capture_script_sha256") == sha(data["capture_script"].encode()) and
            capture.get("native_test_log_sha256") == sha(data["native_test_log"].encode()) and
            capture.get("native_field_profile_sha256") == sha(data["native_field_profile_source"].encode()) and
            capture.get("native_field_profile_log_sha256") == sha(data["native_field_profile_log"].encode()),
            "capture receipt does not bind the observed native/test/manifest inputs")
    return {"selected_mir_count": 31, "production_mir_count": 29,
            "native_client_mir_count": 2, "stage": capture["stage"],
            "rustc": EXPECTED_RUSTC.strip(), "cargo": EXPECTED_CARGO.strip(),
            "selected_paths": resolved, "all_selected_hashes_match": True,
            "production_mir_byte_exact_to_AT": 29,
            "capture_script_sha256": sha(data["capture_script"].encode()),
            "native_manifest_exact": True, "native_lock_pinned": True,
            "location_sensitive_capture": True}


def audit_call_summary_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]["call_summary_body"]
    require(sha(mir.encode()) == EXPECTED_CALLEE_MIR_SHA256,
            "native clone_suffix MIR hash changed")
    require("fn clone_suffix(_1: &bytes::Bytes, _2: usize) -> bytes::Bytes {" in mir,
            "native call-summary MIR signature changed")
    blocks = AT.blocks_of(mir, "clone_suffix")
    expected = {
        (0, False): ["_4 = &'_ (*_1);",
            "_3 = <bytes::Bytes as Clone>::clone(move _4) -> [return: bb1, unwind continue];"],
        (1, False): ["_6 = &'_ mut _3;", "_7 = copy _2;",
            "_5 = <bytes::Bytes as Buf>::advance(move _6, move _7) -> [return: bb2, unwind: bb4];"],
        (2, False): ["_0 = move _3;", "goto -> bb3;"],
        (3, False): ["return;"],
        (4, True): ["drop(_3) -> [return: bb5, unwind terminate(cleanup)];"],
        (5, True): ["resume;"],
    }
    require(set(blocks) == set(expected), "clone_suffix full normal/cleanup CFG block set changed")
    for key, expected_lines in expected.items():
        require(significant(blocks[key]) == expected_lines,
                f"clone_suffix block bb{key[0]} contains an extra, missing, or reordered operation")
    places = debug_places(mir)
    require(places == {"source": "_1", "amount": "_2", "result": "_3"},
            "clone_suffix MIR debug-place binding changed")
    normal_drops = [(n, m.group(1)) for (n, cleanup), body in blocks.items() if not cleanup
                    for m in re.finditer(r"\bdrop\((_\d+)(?:: bytes::Bytes)?\)", body)]
    require(normal_drops == [], "clone_suffix normal path drops/consumes its result before return")
    cleanup_drops = [(n, m.group(1)) for (n, cleanup), body in blocks.items() if cleanup
                     for m in re.finditer(r"\bdrop\((_\d+)(?:: bytes::Bytes)?\)", body)]
    require(cleanup_drops == [(4, "_3")], "clone_suffix cleanup no longer drops the cloned owner on unwind")
    rows = block_rows(blocks)
    call = {"clone": {"block": "bb0", "receiver": "_1", "result": "_3",
                      "target": "<bytes::Bytes as Clone>::clone", "normal": "bb1"},
            "advance": {"block": "bb1", "receiver": "_3", "amount": "_2",
                         "target": "<bytes::Bytes as Buf>::advance", "normal": "bb2", "unwind": "bb4"},
            "return_move": {"block": "bb2", "source": "_3", "result": "_0", "normal": "bb3"},
            "cleanup_drop": {"block": "bb4", "place": "_3", "successor": "bb5"}}
    return {"function": "clone_suffix", "mir_sha256": EXPECTED_CALLEE_MIR_SHA256,
            "normal_cfg_exact": True, "debug_places": places, "mir_blocks": rows,
            "operations": call, "normal_owner_returns": [{"block": "bb2", "source_place": "_3", "result_place": "_0"}],
            "normal_owner_drops": [], "cleanup_owner_drop_only": True}


def audit_client_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]["client"]
    require(sha(mir.encode()) == EXPECTED_CLIENT_MIR_SHA256,
            "native owned_view_call_scope MIR hash changed")
    require("fn owned_view_call_scope(_1: Box<[u8]>, _2: usize, _3: usize, _4: &[usize]) -> Vec<u8> {" in mir,
            "native caller MIR signature changed")
    blocks = AT.blocks_of(mir, "owned_view_call_scope")
    expected = {
        (0, False): ["_8 = move _1;", "_7 = <bytes::Bytes as From<Box<[u8]>>>::from(move _8) -> [return: bb1, unwind: bb27];"],
        (1, False): ["_9 = &'_ _7;", "_6 = <bytes::Bytes as Clone>::clone(move _9) -> [return: bb2, unwind: bb26];"],
        (2, False): ["drop(_7) -> [return: bb3, unwind: bb28];"],
        (3, False): ["_10 = &'_ _6;", "_12 = copy _2;", "_13 = copy _3;",
            "_11 = std::ops::Range::<usize> { start: move _12, end: move _13 };",
            "_5 = bytes::Bytes::slice::<std::ops::Range<usize>>(move _10, move _11) -> [return: bb4, unwind: bb25];"],
        (4, False): ["drop(_6) -> [return: bb5, unwind: bb28];"],
        (5, False): ["_14 = const 0_usize;", "goto -> bb6;"],
        (6, False): ["_18 = copy _14;", "_20 = &'_ (*_4);",
            "_19 = core::slice::<impl [usize]>::len(move _20) -> [return: bb7, unwind: bb24];"],
        (7, False): ["_17 = Lt(move _18, move _19);", "switchInt(move _17) -> [0: bb18, otherwise: bb8];"],
        (8, False): ["_23 = copy _14;", "_24 = PtrMetadata(copy _4);", "_25 = Lt(copy _23, copy _24);",
            'assert(move _25, "index out of bounds: the length is {} but the index is {}", move _24, copy _23) -> [success: bb9, unwind: bb24];'],
        (9, False): ["_22 = copy (*_4)[_23];", "_27 = &'_ _5;",
            "_26 = <bytes::Bytes as Buf>::remaining(move _27) -> [return: bb10, unwind: bb24];"],
        (10, False): ["_21 = std::cmp::min::<usize>(move _22, move _26) -> [return: bb11, unwind: bb24];"],
        (11, False): ["_30 = &'_ _5;", "_29 = &'_ (*_30);", "_31 = copy _21;",
            "_28 = clone_suffix(move _29, move _31) -> [return: bb12, unwind: bb24];"],
        (12, False): ["_32 = move _28;", "drop(_5) -> [return: bb13, unwind: bb14];"],
        (13, False): ["_5 = move _32;", "goto -> bb15;"],
        (14, True): ["_5 = move _32;", "goto -> bb23;"],
        (15, False): ["_33 = AddWithOverflow(copy _14, const 1_usize);",
            'assert(!move (_33.1: bool), "attempt to compute `{} + {}`, which would overflow", copy _14, const 1_usize) -> [success: bb16, unwind: bb23];'],
        (16, False): ["_14 = move (_33.0: usize);", "_16 = const ();", "goto -> bb17;"],
        (17, False): ["goto -> bb6;"],
        (18, False): ["_15 = const ();", "_39 = &'_ _5;",
            "_38 = <bytes::Bytes as Buf>::chunk(move _39) -> [return: bb19, unwind: bb24];"],
        (19, False): ["_37 = &'_ (*_38);", "_0 = std::slice::<impl [u8]>::to_vec(move _37) -> [return: bb20, unwind: bb24];"],
        (20, False): ["drop(_5) -> [return: bb21, unwind: bb28];"],
        (21, False): ["goto -> bb22;"],
        (22, False): ["return;"],
        (23, True): ["goto -> bb24;"],
        (24, True): ["drop(_5) -> [return: bb28, unwind terminate(cleanup)];"],
        (25, True): ["drop(_6) -> [return: bb28, unwind terminate(cleanup)];"],
        (26, True): ["drop(_7) -> [return: bb28, unwind terminate(cleanup)];"],
        (27, True): ["goto -> bb28;"],
        (28, True): ["goto -> bb29;"],
        (29, True): ["resume;"],
    }
    require(set(blocks) == set(expected), "caller complete normal/cleanup CFG block set changed")
    for key, wanted in expected.items():
        require(significant(blocks[key]) == wanted,
                f"caller block bb{key[0]} has an extra, missing, or reordered operation")
    places = debug_places(mir)
    expected_places = {"input": "_1", "a": "_2", "b": "_3", "steps": "_4", "value": "_5",
                       "owner": "_6", "original": "_7", "i": "_14", "amount": "_21", "next": "_28"}
    require(places == expected_places, "caller MIR debug places changed")
    drops = [(number, re.search(r"\bdrop\((_\d+)(?:: bytes::Bytes)?\)", body).group(1))
             for (number, cleanup), body in blocks.items() if not cleanup
             for _ in [0] if re.search(r"\bdrop\((_\d+)(?:: bytes::Bytes)?\)", body)]
    require(drops == [(2, "_7"), (4, "_6"), (12, "_5"), (20, "_5")],
            "caller normal Bytes Drop places/order changed")
    call_lines = [line for body in blocks.values() for line in significant(body)
                  if re.search(r"=\s+clone_suffix\(", line)]
    require(call_lines == ["_28 = clone_suffix(move _29, move _31) -> [return: bb12, unwind: bb24];"],
            "caller does not make exactly one direct named clone_suffix call")
    mapping = data["mapping"]
    computed_blocks = block_rows(blocks)
    expected_edges = [
        {"block": "bb2", "place": "_7", "owner": "original", "successor": "bb3", "unwind": "bb28", "repeated": False, "scope": "scope", "role": "scope_exit"},
        {"block": "bb4", "place": "_6", "owner": "owner", "successor": "bb5", "unwind": "bb28", "repeated": False, "scope": "detached", "role": "scope_exit"},
        {"block": "bb12", "place": "_5", "owner": "value", "successor": "bb13", "unwind": "bb14", "repeated": True, "scope": "detached", "role": "replacement"},
        {"block": "bb20", "place": "_5", "owner": "value", "successor": "bb21", "unwind": "bb28", "repeated": False, "scope": "detached", "role": "final_return"},
    ]
    assignment = {"next_place": "_28", "temporary_place": "_32", "value_place": "_5",
        "stash_block": "bb12", "stash_statement": "_32 = move _28;", "drop_block": "bb12",
        "install_block": "bb13", "install_statement": "_5 = move _32;",
        "normal_successor": "bb13", "unwind_successor": "bb14"}
    saved_return = {"block": "bb19", "result_place": "_0", "successor": "bb20", "unwind": "bb24"}
    require(mapping.get("native_source") == "native.rs" and mapping.get("native_source_sha256") == EXPECTED_NATIVE_SOURCE_SHA256 and
            mapping.get("native_mir_ready") is True and
            mapping.get("native_client_mir") == next(r["path"] for r in data["capture"]["selected"] if r["label"] == "client") and
            mapping.get("normal_edges") == expected_edges and mapping.get("debug_places") == places and
            mapping.get("mir_blocks") == computed_blocks and mapping.get("native_assignment") == assignment and
            mapping.get("native_saved_return") == saved_return,
            "generated native caller mapping does not equal independently parsed caller MIR")
    expected_inventory = sorted(({"path": row["path"], "sha256": row["sha256"]}
                                 for row in data["capture"]["selected"]), key=lambda r: r["path"])
    require(mapping.get("mir") == expected_inventory,
            "generated native MIR inventory differs from the independently loaded capture")
    return {"function": "owned_view_call_scope", "mir_sha256": EXPECTED_CLIENT_MIR_SHA256,
            "normal_cfg_exact": True, "debug_places": places,
            "mir_blocks": computed_blocks, "normal_edges": expected_edges,
            "call_summary_call": {"block": "bb11", "target": "clone_suffix", "receiver": "_29",
                "amount": "_31", "result": "_28", "normal_target": "bb12", "unwind_target": "bb24"},
            "assignment": assignment, "saved_return": saved_return,
            "normal_drop_order": ["original bb2", "owner bb4", "previous value bb12 before install bb13", "returned Vec evaluated bb19 before final value bb20"],
            "runtime_loop_unbounded_by_constant_quota": True,
            "normal_completion_only": True, "unwind_claim": False}


def audit_named_call_mapping(data: dict[str, Any], callee: dict[str, Any], client: dict[str, Any]) -> dict[str, Any]:
    summaries = data["mapping"].get("named_call_summaries", [])
    require(len(summaries) == 1, "mapping must carry exactly one named owning-return summary")
    summary = summaries[0]
    callee_path = next(r["path"] for r in data["capture"]["selected"] if r["label"] == "call_summary_body")
    client_path = next(r["path"] for r in data["capture"]["selected"] if r["label"] == "client")
    callee_blocks = callee["mir_blocks"]
    expected_callsites = [{"block": "bb11", "result_place": "_28", "argument_places": ["_29", "_31"],
                           "successor": "bb12", "unwind": "bb24",
                           "body_sha256": next(r["body_sha256"] for r in client["mir_blocks"] if r["block"] == "bb11" and not r["cleanup"])}]
    expected = {
        "schema": "named_direct_call_v1", "native_callee": "clone_suffix", "shadow_callee": "clone_suffix_checked",
        "native_signature": "fn clone_suffix(source: &Bytes, amount: usize) -> Bytes",
        "native_argument_modes": ["shared_borrow", "copy"], "native_result_mode": "move_owner",
        "caller_function": "owned_view_call_scope", "caller_mir": client_path,
        "caller_mir_sha256": EXPECTED_CLIENT_MIR_SHA256,
        "proof_coma_path": "verif/bytes_original_owned_view_call_summaries_rlib/promotion/clone_suffix_checked.coma",
        "proof_hash_binding": "fresh_post_translation_checker_receipt",
        "proof_target": "promotion::clone_suffix_checked", "trusted_summary": False,
        "caller_inlines_callee": False,
        "callee_mir": callee_path, "callee_mir_sha256": EXPECTED_CALLEE_MIR_SHA256,
        "callee_debug_places": callee["debug_places"], "callee_blocks": callee_blocks,
        "normal_owner_returns": [{"block": "bb2", "source_place": "_3", "result_place": "_0"}],
        "normal_owner_drops": [], "callsites": expected_callsites,
        "native_operations": ["Clone::clone(source)", "Buf::advance(result,amount)", "move result to return"],
        "shadow_operations": ["clone_owned_api(source,ghost_reborrow(scope))", "advance_api(result,amount)", "move result to return"],
        "excluded": ["unwind", "indirect or unreviewed callees", "untracked escape"],
    }
    for key, value in expected.items():
        require(summary.get(key) == value, f"named-call mapping field {key!r} differs from native source/MIR facts")
    require(summary.get("erased_arguments") == [{"index": 2, "name": "scope", "type": "Ghost<&mut DetachedScope>",
        "caller_expression": "detached.borrow_mut()", "mode": "exclusive_ghost_reborrow"}],
        "named summary erased ghost reborrow mapping changed")
    require(isinstance(summary.get("contract_sha256"), str) and re.fullmatch(r"[0-9a-f]{64}", summary["contract_sha256"]) and
            isinstance(summary.get("source_item_sha256"), str) and re.fullmatch(r"[0-9a-f]{64}", summary["source_item_sha256"]),
            "proof summary mapping omits the contract/source-item hash join")
    return {"schema": summary["schema"], "native_callee": summary["native_callee"],
            "shadow_callee": summary["shadow_callee"], "native_mir_join_rederived": True,
            "proof_contract_and_source_hashes_present_for_main_checker_join": True,
            "trusted_summary": summary["trusted_summary"], "caller_inlines_callee": summary["caller_inlines_callee"]}


def audit_bundle(data: dict[str, Any] | None = None) -> dict[str, Any]:
    if data is None:
        data = load_bundle()
    ancestor = audit_ancestor(data)
    source = audit_source_and_execution(data)
    capture = audit_capture(data)
    callee = audit_call_summary_mir(data)
    client = audit_client_mir(data)
    mapping = audit_named_call_mapping(data, callee, client)
    return {
        "status": "pass",
        "checker_scope": "AU closed named-call owned-return source/MIR correspondence over pinned AT native gate; no proof or MIR-adequacy claim",
        "ancestor_at": ancestor,
        "source_and_execution": source,
        "capture": capture,
        "native_audit": {
            "selected_mir_count": 31,
            "production_mir_count": 29,
            "client_mir_count": 2,
            "client": client,
            "call_summary_body": callee,
            "named_call_summary": mapping,
            "selected_hashes_match_capture": True,
            "production_mir_byte_exact_to_AT": True,
        },
        "limits": [
            "The 84 native executions corroborate behavior only; they are not a proof.",
            "Only this closed normal-completion caller and the named native helper body are joined; unwind is excluded.",
            "The exact mapping does not prove Rust compiler MIR adequacy or generic summary semantics beyond the proved contract.",
            "Inherited Bytes pointer provenance, atomics, compiler, scoped history and erased-callback interpretations remain external TCB.",
        ],
    }


def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        result = audit_bundle()
    except Exception as exc:
        result = {"status": "reject", "checker_scope": "AU named-call native source/MIR correspondence",
                  "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result.get("status") == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
