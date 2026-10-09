#!/usr/bin/env python3
"""Read-only AV native source/MIR audit.

This gate reuses the published AT production-source/MIR audit, requires the 29
captured production bodies to remain byte-identical, and independently parses
the AV promote-then-clone caller MIR. It invokes no compiler, Cargo, proof tool,
or solver. It is not a proof of MIR adequacy or Rust memory-model semantics.
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

EXPECTED_CAPTURE_SHA256 = "47da1dd032bea0f264aec88583bf19e7dee0f58616be48002a8e3034599e8e3b"
EXPECTED_NATIVE_SOURCE_SHA256 = "00c664ecb8f62d05809da2aa50123b6d733267c5def471bc4511030f2c11f936"
EXPECTED_TEST_SOURCE_SHA256 = "0db84cbf506a780f3e6a40f6f3606229614c384d5923518d43efe29643a1c699"
EXPECTED_CAPTURE_SCRIPT_SHA256 = "bc67fdb2c7620421a64c04617e6501b99eef52cee9ae044e74eb822ff3482e08"
EXPECTED_NATIVE_MANIFEST_SHA256 = "b9135ddfb22cd609ac90951dfb51e55643c6c41bfbeb8f52b6b483b9e511231c"
EXPECTED_NATIVE_LOCK_SHA256 = "4a8b76e563428432f05b3f7d86e856b6a1385c36880d2b0e74cbb14bf757723d"
EXPECTED_NATIVE_LOG_SHA256 = "932ec7190e2d3fb354591ece0c036ad4db133ea2327e58b5f280f12db9684362"
EXPECTED_FIELD_SOURCE_SHA256 = "b8dae751e6c7dc007de636ed7cc8d9b6f98cc746252d7373e785778655b58183"
EXPECTED_FIELD_LOG_SHA256 = "3849089883705a9502911036f6510d56a263520626d4fc5ebc023aaa0530c436"
EXPECTED_CLIENT_MIR_SHA256 = "c77c9dd170b99d9944825755be6c28b0c62920838794f23cad7333d84a6a7a89"
EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\ncommit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\nhost: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\nLLVM version: 22.1.7\n")
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"

EXPECTED_NATIVE_SOURCE = """use bytes::{Buf, Bytes};

pub fn promotable_suffix_scope(input: Box<[u8]>, amount: usize) -> Vec<u8> {
    let child = {
        let mut root = Bytes::from(input);
        root.advance(amount);
        root.clone()
    };
    child.chunk().to_vec()
}
"""
EXPECTED_TEST_SOURCE = """use bytes_promotable_suffix_promotion_native::promotable_suffix_scope;

#[test]
fn root_advance_before_first_clone_preserves_exact_suffix() {
    let mut cases = 0usize;
    for len in [1usize, 2, 3, 7, 16, 257] {
        let input: Vec<u8> = (0..len).map(|i| (i.wrapping_mul(37) % 256) as u8).collect();
        for amount in 0..=len {
            let expected = input[amount..].to_vec();
            let actual = promotable_suffix_scope(input.clone().into_boxed_slice(), amount);
            assert_eq!(actual, expected, "len={len}, amount={amount}");
            cases += 1;
        }
    }
    assert_eq!(cases, 292);
    println!("AV native suffix cases: {cases}");
}
"""
EXPECTED_LABELS = {
    "from_box", "clone_impl", "cleanup", "bytes_drop", "as_ref", "as_slice",
    "promotable_even_clone", "promotable_odd_clone", "shallow_clone_vec",
    "shallow_clone_arc", "shared_clone", "slice", "new_empty_with_ptr",
    "static_clone", "static_drop", "without_provenance", "inc_start", "remaining",
    "chunk", "advance", "len", "ref_count_increment", "promotable_even_drop",
    "promotable_odd_drop", "shared_drop", "release_shared", "free_shared", "ptr_map",
    "atomic_with_mut",
}
EXPECTED_NATIVE_MANIFEST = {
    "package": {"name": "bytes-promotable-suffix-promotion-native", "version": "0.0.0", "edition": "2021"},
    "workspace": {},
    "lib": {"name": "bytes_promotable_suffix_promotion_native", "path": "../native.rs"},
    "dependencies": {"bytes": {"path": "../../../../"}},
}


class AuditError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_at_checker():
    require(AT_CHECKER_PATH.is_file() and sha(AT_CHECKER_PATH.read_bytes()) == AT_CHECKER_SHA256,
            "published AT native checker changed before import")
    spec = importlib.util.spec_from_file_location("av_pinned_at_native", AT_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load published AT native checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


AT = load_at_checker()


def load_bundle() -> dict[str, Any]:
    capture_raw = CAPTURE_PATH.read_bytes()
    capture = json.loads(capture_raw)
    mir_sources = {}
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
        "field_source": (ROOT / "native-field-profile.rs").read_text(),
        "field_log": (ROOT / "native-field-profile.log").read_text(),
        "mapping": json.loads((ROOT / "generated/mapping.json").read_text()),
    }


def significant(body: str) -> list[str]:
    return [line.strip() for line in body.splitlines()
            if line.strip() and not line.strip().startswith(("StorageLive(", "StorageDead(", "debug ", "PlaceMention("))]


def block_rows(blocks: dict[tuple[int, bool], str]) -> list[dict[str, Any]]:
    return [{"block": f"bb{number}", "cleanup": cleanup,
             "body_sha256": sha(("\n" + body).encode())}
            for (number, cleanup), body in blocks.items()]


def audit_ancestor(data: dict[str, Any]) -> dict[str, Any]:
    report = AT.audit_bundle(data["at_base"])
    require(report.get("status") == "pass", "published AT production source/MIR gate did not pass")
    return {"status": "pass", "checker_sha256": AT_CHECKER_SHA256,
            "selected_mir_count": report["native_audit"]["selected_mir_count"],
            "production_mir_count": report["native_audit"]["production_mir_count"],
            "reviewed_production_inputs": report["reviewed_production_inputs"],
            "production_callbacks": report["production_callbacks"],
            "cursor_source": report["cursor_source"],
            "terminal_field_profile": report["terminal_field_profile"]}


def audit_source(data: dict[str, Any]) -> dict[str, Any]:
    require(data["native_source"] == EXPECTED_NATIVE_SOURCE and
            sha(data["native_source"].encode()) == EXPECTED_NATIVE_SOURCE_SHA256,
            "AV native source differs from the frozen promote-then-clone witness")
    require(data["native_test_source"] == EXPECTED_TEST_SOURCE and
            sha(data["native_test_source"].encode()) == EXPECTED_TEST_SOURCE_SHA256,
            "AV native test source changed")
    log = data["native_test_log"]
    require("AV native suffix cases: 292" in log and "test result: ok. 1 passed" in log and
            "test result: FAILED" not in log and sha(log.encode()) == EXPECTED_NATIVE_LOG_SHA256,
            "captured 292-case native harness log is not the pinned successful run")
    require(sha(data["field_source"].encode()) == EXPECTED_FIELD_SOURCE_SHA256 and
            sha(data["field_log"].encode()) == EXPECTED_FIELD_LOG_SHA256 and
            "native Bytes fields have no independent drop glue" in data["field_log"],
            "native Bytes field no-drop profile differs from the inherited capture")
    return {
        "source_sha256": EXPECTED_NATIVE_SOURCE_SHA256,
        "function": "promotable_suffix_scope",
        "source_shape": ["Bytes::from(input)", "root.advance(amount)", "root.clone()", "root lexical Drop", "child.chunk().to_vec()"],
        "test_source_sha256": EXPECTED_TEST_SOURCE_SHA256,
        "native_smoke": {"case_count": 292, "lengths": [1, 2, 3, 7, 16, 257],
            "amounts": "0..=len", "includes_full_advance_to_empty": True, "execution_only": True},
        "native_field_profile": {"no_independent_drop_glue": True,
            "source_sha256": EXPECTED_FIELD_SOURCE_SHA256, "log_sha256": EXPECTED_FIELD_LOG_SHA256},
    }


def audit_capture(data: dict[str, Any]) -> dict[str, Any]:
    capture = data["capture"]
    require(sha(data["capture_raw"]) == EXPECTED_CAPTURE_SHA256,
            "AV native capture receipt hash changed")
    require(capture.get("stage") == "2-2-004.ElaborateDrops.after.mir" and
            capture.get("rustc_version") == EXPECTED_RUSTC.rstrip("\n") and
            data["rustc_text"] == EXPECTED_RUSTC and
            capture.get("cargo_version") == EXPECTED_CARGO.rstrip("\n") and
            data["cargo_text"] == EXPECTED_CARGO,
            "AV MIR stage or pinned Rust/Cargo version changed")
    require(sha(data["capture_script"].encode()) == EXPECTED_CAPTURE_SCRIPT_SHA256 and
            "cargo test --locked --offline" in data["capture_script"] and
            "cargo rustc --locked --offline" in data["capture_script"] and
            "-Zdump-mir=all" in data["capture_script"] and "-Zmir-opt-level=0" in data["capture_script"] and
            "creusot" not in data["capture_script"].lower() and "why3" not in data["capture_script"].lower(),
            "native capture script changed or invokes proof tooling")
    require(sha(data["native_manifest"].encode()) == EXPECTED_NATIVE_MANIFEST_SHA256 and
            tomllib.loads(data["native_manifest"]) == EXPECTED_NATIVE_MANIFEST,
            "AV native harness package/dependency path changed")
    require(sha(data["native_lock"].encode()) == EXPECTED_NATIVE_LOCK_SHA256,
            "AV native lockfile changed")

    rows = capture.get("selected", [])
    labels = {row.get("label") for row in rows}
    require(len(rows) == 30 and len(labels) == 30 and labels == EXPECTED_LABELS | {"client"},
            "capture must contain exactly 29 inherited production MIR bodies and one AV caller")
    require(set(data["mir_sources"]) == labels, "loaded MIR labels differ from selected capture inventory")
    at_mirs = data["at_base"]["mir_sources"]
    inherited = {label: body for label, body in at_mirs.items() if label != "client"}
    current_prod = {label: body for label, body in data["mir_sources"].items() if label != "client"}
    require(len(inherited) == 29 and current_prod == inherited,
            "AV production MIR bodies are not byte-exact to the pinned AT ancestor")

    resolved = {}
    for row in rows:
        rel = row.get("path", "")
        label = row["label"]
        require(rel.startswith("native-mir/") and pathlib.PurePosixPath(rel).name == rel.split("/", 1)[1],
                f"MIR path escapes native-mir: {label}")
        path = (ROOT / rel).resolve()
        require(path == (MIR_DIR / pathlib.PurePosixPath(rel).name).resolve() and path.is_file(),
                f"MIR path redirects or is missing: {label}")
        require(sha(path.read_bytes()) == row.get("sha256"), f"MIR hash differs from capture: {label}")
        resolved[label] = str(path)
    files = list(MIR_DIR.glob("*.mir"))
    require(len(files) == 30 and {x.name for x in files} ==
            {pathlib.PurePosixPath(x["path"]).name for x in rows},
            "native-mir contains missing or extra selected MIR files")

    at_capture = data["at_base"]["capture"]
    for field in ("production_manifest_sha256", "production_source_sha256"):
        require(capture.get(field) == at_capture.get(field), f"AV production input pin differs at {field}")
    expected_file_pins = {
        "native_source_sha256": EXPECTED_NATIVE_SOURCE_SHA256,
        "native_test_source_sha256": EXPECTED_TEST_SOURCE_SHA256,
        "native_manifest_sha256": EXPECTED_NATIVE_MANIFEST_SHA256,
        "native_lock_sha256": EXPECTED_NATIVE_LOCK_SHA256,
        "capture_script_sha256": EXPECTED_CAPTURE_SCRIPT_SHA256,
        "native_test_log_sha256": EXPECTED_NATIVE_LOG_SHA256,
        "native_field_profile_sha256": EXPECTED_FIELD_SOURCE_SHA256,
        "native_field_profile_log_sha256": EXPECTED_FIELD_LOG_SHA256,
    }
    require(all(capture.get(k) == v for k, v in expected_file_pins.items()),
            "AV capture receipt does not bind exact source, harness, script, test, and field-profile bytes")
    return {"selected_mir_count": 30, "production_mir_count": 29, "client_mir_count": 1,
            "stage": capture["stage"], "rustc": EXPECTED_RUSTC.strip(), "cargo": EXPECTED_CARGO.strip(),
            "selected_paths": resolved, "all_selected_hashes_match": True,
            "production_mir_byte_exact_to_AT": 29, "capture_script_sha256": EXPECTED_CAPTURE_SCRIPT_SHA256,
            "native_manifest_exact": True, "native_lock_pinned": True, "location_sensitive_capture": True}


def audit_client_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]["client"]
    require(sha(mir.encode()) == EXPECTED_CLIENT_MIR_SHA256,
            "AV client MIR differs from the frozen post-ElaborateDrops body")
    require("fn promotable_suffix_scope(_1: Box<[u8]>, _2: usize) -> Vec<u8> {" in mir,
            "AV native caller MIR signature changed")
    blocks = AT.blocks_of(mir, "promotable_suffix_scope")
    expected = {
        (0, False): ["_5 = move _1;", "_4 = <bytes::Bytes as From<Box<[u8]>>>::from(move _5) -> [return: bb1, unwind: bb11];"],
        (1, False): ["_7 = &'_ mut _4;", "_8 = copy _2;", "_6 = <bytes::Bytes as Buf>::advance(move _7, move _8) -> [return: bb2, unwind: bb10];"],
        (2, False): ["_9 = &'_ _4;", "_3 = <bytes::Bytes as Clone>::clone(move _9) -> [return: bb3, unwind: bb10];"],
        (3, False): ["drop(_4) -> [return: bb4, unwind: bb12];"],
        (4, False): ["_12 = &'_ _3;", "_11 = <bytes::Bytes as Buf>::chunk(move _12) -> [return: bb5, unwind: bb9];"],
        (5, False): ["_10 = &'_ (*_11);", "_0 = slice::<impl [u8]>::to_vec(move _10) -> [return: bb6, unwind: bb9];"],
        (6, False): ["drop(_3) -> [return: bb7, unwind: bb12];"],
        (7, False): ["goto -> bb8;"],
        (8, False): ["return;"],
        (9, True): ["drop(_3) -> [return: bb12, unwind terminate(cleanup)];"],
        (10, True): ["drop(_4) -> [return: bb12, unwind terminate(cleanup)];"],
        (11, True): ["goto -> bb12;"],
        (12, True): ["goto -> bb13;"],
        (13, True): ["resume;"],
    }
    require(set(blocks) == set(expected), "AV caller MIR normal/cleanup basic-block set changed")
    for key, lines in expected.items():
        require(significant(blocks[key]) == lines,
                f"AV caller bb{key[0]} contains a missing, extra, or reordered operation")

    places = dict(re.findall(r"debug\s+(\w+)\s*=>\s*(_\d+)\s*;", mir))
    require(places == {"input": "_1", "amount": "_2", "child": "_3", "root": "_4"},
            "AV client MIR local-role bindings changed")
    all_normal_drops = [(key[0], m.group(1)) for key, body in blocks.items() if not key[1]
                        for m in re.finditer(r"\bdrop\((_\d+)(?:: bytes::Bytes)?\)", body)]
    require(all_normal_drops == [(3, "_4"), (6, "_3")],
            "AV normal path no longer drops root after clone then child after Vec evaluation")
    require("_0 = slice::<impl [u8]>::to_vec(move _10) -> [return: bb6, unwind: bb9];" in significant(blocks[(5, False)]) and
            blocks[(6, False)].find("drop(_3)") >= 0,
            "returned Vec is not evaluated before the child Bytes normal Drop")

    normal_edges = [
        {"block": "bb3", "place": "_4", "owner": "root", "successor": "bb4", "unwind": "bb12",
         "scope": "suffix", "role": "promoted_root_scope_exit"},
        {"block": "bb6", "place": "_3", "owner": "child", "successor": "bb7", "unwind": "bb12",
         "scope": "detached", "role": "surviving_child_final"},
    ]
    rows = block_rows(blocks)
    mapping = data["mapping"]
    capture_row = next(row for row in data["capture"]["selected"] if row["label"] == "client")
    require(mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == EXPECTED_NATIVE_SOURCE_SHA256 and
            mapping.get("native_client_mir") == capture_row["path"] and
            mapping.get("native_mir_ready") is True,
            "generated mapping does not bind the captured AV native caller source/MIR")
    require(mapping.get("debug_places") == places and mapping.get("normal_edges") == normal_edges and
            mapping.get("mir_blocks") == rows,
            "generated AV caller mapping does not match independently parsed MIR facts")
    expected_mir = sorted(({"path": row["path"], "sha256": row["sha256"]}
                           for row in data["capture"]["selected"]), key=lambda x: x["path"])
    require(mapping.get("mir") == expected_mir, "generated mapping selected MIR inventory differs from the native capture")

    return {
        "function": "promotable_suffix_scope", "mir_sha256": EXPECTED_CLIENT_MIR_SHA256,
        "normal_cfg_exact": True, "debug_places": places, "normal_edges": normal_edges, "mir_blocks": rows,
        "advance_before_first_clone": {"advance_block": "bb1", "receiver": "_4", "clone_block": "bb2",
            "clone_receiver": "_4", "same_original_owner": True},
        "normal_drop_order": ["root", "child"],
        "root_drop_before_child_read": True,
        "child_drop_after_vec_evaluation": True,
        "normal_owner_drops": all_normal_drops,
        "cleanup_drops": [{"block": "bb9", "place": "_3"}, {"block": "bb10", "place": "_4"}],
        "mapping_rederived_from_MIR": True,
    }


def audit_bundle(data: dict[str, Any] | None = None) -> dict[str, Any]:
    if data is None:
        data = load_bundle()
    ancestor = audit_ancestor(data)
    source = audit_source(data)
    capture = audit_capture(data)
    client = audit_client_mir(data)
    return {
        "status": "pass",
        "checker_scope": "AV default-native promote-then-first-Clone client plus exact published production callbacks; no proof or MIR-adequacy claim",
        "ancestor_at": ancestor,
        "source_profile": source,
        "capture": capture,
        "native_audit": {"client": client,
            "production_callbacks": ancestor["production_callbacks"],
            "promotable_clone_dispatch": {
                "advance_preserves_data_and_stored_vtable": ancestor["cursor_source"]["default_native_inc_start"]["data_and_vtable_unchanged"],
                "promotable_even_callback_captured": "promotable_even_clone" in data["mir_sources"],
                "promotable_odd_callback_captured": "promotable_odd_clone" in data["mir_sources"],
                "callback_selection_uses_stored_vtable": ancestor["production_callbacks"]["promotable_parity_vtables_bind_clone_and_drop_callbacks"],
                "guarded_atomic_refcount_increment_source_checked": ancestor["production_callbacks"]["native_atomic_mut_and_refcount_increment_sources_exact"],
                "address_parity_observed": False,
            },
            "reviewed_production_inputs": ancestor["reviewed_production_inputs"],
            "cursor_source": ancestor["cursor_source"],
            "terminal_field_profile": ancestor["terminal_field_profile"],
            "selected_mir_count": 30, "production_mir_count": 29},
        "limits": [
            "Runtime witness is execution corroboration only.",
            "Normal completion is mapped; unwind behavior is parsed but excluded from the promotion claim.",
            "Native callback semantics and Rust memory/provenance semantics remain bounded by the inherited source/compiler TCB.",
            "No observation of allocation address parity is claimed; Clone callback selection is inherited from the stored immutable vtable source/MIR audit.",
        ],
    }


def main() -> int:
    try:
        report = audit_bundle()
    except AuditError as exc:
        print(json.dumps({"status": "coverage_failure", "error": str(exc)}, indent=2))
        return 2
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
