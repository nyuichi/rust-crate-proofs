#!/usr/bin/env python3
"""In-memory mutation controls for AR native cursor/source/MIR checking."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import pathlib
import sys
from typing import Any, Callable

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_native.py"
FIXTURE_PATH = ROOT / "fixtures/native-check-controls.json"
OUTPUT_PATH = ROOT / "generated/native-check-controls.json"


def load_checker():
    spec = importlib.util.spec_from_file_location("ar_native_cursor_checker", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot import AR native cursor checker")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


C = load_checker()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one mutation anchor, found {count}: {old!r}")
    return text.replace(old, new, 1)


def replace_first(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise RuntimeError(f"{label}: mutation anchor not found: {old!r}")
    return text.replace(old, new, 1)


def edit(field: str, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    return lambda d: d.__setitem__(field, replace_once(d[field], old, new, label))


def edit_map(field: str, key: str, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d[field][key] = replace_once(d[field][key], old, new, label)
    return mutate


def edit_capture(key: str, old: Any, new: Any) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        if d["capture"].get(key) != old:
            raise RuntimeError(f"capture mutation anchor changed: {key}")
        d["capture"][key] = new
    return mutate


def edit_capture_row(label: str, key: str, old: Any, new: Any) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        rows = [row for row in d["capture"]["selected"] if row["label"] == label]
        if len(rows) != 1 or rows[0].get(key) != old:
            raise RuntimeError(f"capture row mutation anchor changed: {label}.{key}")
        rows[0][key] = new
    return mutate


def edit_client_mir(old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    return edit_map("mir_sources", "client", old, new, label)


def edit_client_block(block: int, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        text = d["mir_sources"]["client"]
        marker = f"    bb{block}: {{"
        start = text.index(marker)
        end = text.index("\n    }", start) + len("\n    }")
        body = text[start:end]
        changed = replace_once(body, old, new, label)
        d["mir_sources"]["client"] = text[:start] + changed + text[end:]
    return mutate


def edit_mir(label: str, old: str, new: str, why: str) -> Callable[[dict[str, Any]], None]:
    return edit_map("mir_sources", label, old, new, why)


def edit_captured_mir_and_refresh_hashes(
    label: str, old: str, new: str, why: str
) -> Callable[[dict[str, Any]], None]:
    """Mutate a selected MIR body and its parsed receipt/mapping hash rows.

    The raw capture receipt remains the immutable pin.  This lets full audit
    reject a refreshed selected-row claim through the inherited AQ byte-exact
    lineage check, instead of accepting a stale hash mismatch as the only gate.
    """
    def mutate(d: dict[str, Any]) -> None:
        changed = replace_once(d["mir_sources"][label], old, new, why)
        d["mir_sources"][label] = changed
        digest = sha(changed.encode())
        rows = [row for row in d["capture"]["selected"] if row["label"] == label]
        if len(rows) != 1:
            raise RuntimeError(f"expected one selected MIR row for {label}, got {len(rows)}")
        rows[0]["sha256"] = digest

        mapping = json.loads(d["mapping_text"])
        path = rows[0]["path"]
        mapped = [row for row in mapping.get("mir", []) if row.get("path") == path]
        if len(mapped) != 1:
            raise RuntimeError(f"expected one mapping MIR row for {path}, got {len(mapped)}")
        mapped[0]["sha256"] = digest
        d["mapping_text"] = json.dumps(mapping, indent=2) + "\n"
    return mutate


def edit_source_item(old: str, new: str, why: str) -> Callable[[dict[str, Any]], None]:
    return edit("production_source", old, new, why)


def change_native_log(_: dict[str, Any]) -> None:
    raise RuntimeError("the successful log is immutable; test mutations target its captured bytes")


def verify_source(d: dict[str, Any]) -> None:
    C.audit_source_and_profile(d)


def verify_cursor_source(d: dict[str, Any]) -> None:
    C.audit_cursor_source(d)


def verify_client(d: dict[str, Any]) -> None:
    C.audit_client_mir(d)
    C.audit_capture(d)


def verify_cursor_mir(d: dict[str, Any]) -> None:
    C.audit_cursor_mir(d)
    C.audit_capture(d)


def verify_capture(d: dict[str, Any]) -> None:
    C.audit_capture(d)


def verify_bundle(d: dict[str, Any]) -> None:
    C.audit_bundle(d)


def verify_ancestor(d: dict[str, Any]) -> None:
    C.audit_ancestor(d)


def mutate_map_edges(d: dict[str, Any]) -> None:
    m = json.loads(d["mapping_text"])
    m["normal_edges"][0]["successor"] = "bb99"
    d["mapping_text"] = json.dumps(m, indent=2) + "\n"


def mutate_cursor_map(d: dict[str, Any]) -> None:
    m = json.loads(d["source_map_text"])
    m["cursor_bodies"]["advance"] = "0" * 64
    d["source_map_text"] = json.dumps(m, indent=2) + "\n"


def mutate_debug_places(d: dict[str, Any]) -> None:
    m = json.loads(d["mapping_text"])
    m["debug_places"]["value"] = "_99"
    d["mapping_text"] = json.dumps(m, indent=2) + "\n"


def mutate_block_hash(d: dict[str, Any]) -> None:
    m = json.loads(d["mapping_text"])
    m["mir_blocks"][0]["body_sha256"] = "0" * 64
    d["mapping_text"] = json.dumps(m, indent=2) + "\n"


def mutate_mapping_mir_hash(d: dict[str, Any]) -> None:
    m = json.loads(d["mapping_text"])
    m["mir"][0]["sha256"] = "0" * 64
    d["mapping_text"] = json.dumps(m, indent=2) + "\n"


def mutate_profile_log(d: dict[str, Any]) -> None:
    d["native_field_profile_log"] = "native Bytes fields have independent drop glue\n"


def mutate_source_manifest(d: dict[str, Any]) -> None:
    d["aq_base"]["reviewed_production_manifest"] = d["aq_base"]["reviewed_production_manifest"].replace(
        "361c7cd261507ac0a705b3b836f73240070891c6", "0" * 40, 1)


def mutate_ancestor_bytes(d: dict[str, Any]) -> None:
    d["aq_base"]["production_source"] = replace_once(
        d["aq_base"]["production_source"], "pub fn slice(", "pub fn sliced(", "AQ inherited slice")


def mutate_client_mapping_source_hash(d: dict[str, Any]) -> None:
    m = json.loads(d["mapping_text"])
    m["native_source_sha256"] = "0" * 64
    d["mapping_text"] = json.dumps(m, indent=2) + "\n"


CONTROL_CASES: list[tuple[str, str, Callable[[dict[str, Any]], None], Callable[[dict[str, Any]], None]]] = [
    # Native client/API witness mutations.
    ("client_wrong_crate_constructor", "native-source", edit("native_source", "Bytes::from(input)", "Bytes::new()", "constructor"), verify_source),
    ("client_wrong_slice_receiver", "native-source", edit("native_source", "owner.slice(a..b)", "original.slice(a..b)", "slice receiver"), verify_source),
    ("client_fixed_iteration_quota", "native-source", edit("native_source", "while i < steps.len()", "while i < core::cmp::min(steps.len(), 1)", "loop quota"), verify_source),
    ("client_step_not_clamped", "native-source", edit("native_source", "core::cmp::min(steps[i], value.remaining())", "steps[i]", "step clamp"), verify_source),
    ("client_wrong_step_source", "native-source", edit("native_source", "core::cmp::min(steps[i], value.remaining())", "core::cmp::min(steps[0], value.remaining())", "step index"), verify_source),
    ("client_wrong_cursor_mutated", "native-source", edit("native_source", "value.advance(by)", "owner.advance(by)", "cursor receiver"), verify_source),
    ("client_index_not_advanced", "native-source", edit("native_source", "i += 1;", "i += 0;", "loop index increment"), verify_source),
    ("client_observation_not_vec", "native-source", edit("native_source", "value.chunk().to_vec()", "value.remaining()", "observed value"), verify_source),
    ("client_drain_omitted", "native-source", edit("native_source", "value.advance(rest);", "let _drained = rest;", "final drain"), verify_source),
    ("client_drain_wrong_count", "native-source", edit("native_source", "value.advance(rest);", "value.advance(0);", "final drain argument"), verify_source),
    ("client_zero_remaining_assert_removed", "native-source", edit("native_source", "assert_eq!(value.remaining(), 0);", "let _remaining = value.remaining();", "final remaining assert"), verify_source),
    ("client_empty_chunk_assert_removed", "native-source", edit("native_source", "assert!(value.chunk().is_empty());", "let _empty = value.chunk();", "final empty chunk assert"), verify_source),
    ("client_returns_drained_chunk", "native-source", edit("native_source", "observed\n}", "value.chunk().to_vec()\n}", "client return"), verify_source),
    ("client_test_empty_ranges_removed", "native-test", edit("native_test_source", "(0,0),(len,len),(len/2,len/2)", "(0,len),(len/2,len)", "empty range cases"), verify_source),
    ("client_test_oversized_steps_removed", "native-test", edit("native_test_source", "&[usize::MAX]", "&[1]", "oversized step cases"), verify_source),
    ("client_test_suffix_formula_changed", "native-test", edit("native_test_source", "expected[a+consumed..b]", "expected[a..b]", "runtime suffix oracle"), verify_source),
    ("client_from_box_mir_mutated", "client-mir", edit_client_mir("<bytes::Bytes as From<Box<[u8]>>>::from(move _8)", "bytes::Bytes::new()", "Box constructor MIR"), verify_client),
    ("client_clone_mir_mutated", "client-mir", edit_client_mir("<bytes::Bytes as Clone>::clone(move _9)", "bytes::Bytes::new()", "original clone MIR"), verify_client),
    ("client_range_specialization_changed", "client-mir", edit_client_mir("slice::<std::ops::Range<usize>>", "slice::<impl std::ops::RangeBounds<usize>>", "actual slice specialization"), verify_client),

    # Actual production API source/extraction mutations.
    ("inc_start_wrapping_add", "cursor-source", edit_source_item("self.ptr = self.ptr.add(by)", "self.ptr = self.ptr.wrapping_add(by)", "inc_start pointer operation"), verify_cursor_source),
    ("inc_start_len_increment", "cursor-source", edit_source_item("self.len -= by", "self.len += by", "inc_start length arithmetic"), verify_cursor_source),
    ("inc_start_remove_debug_assert", "cursor-source", edit_source_item('debug_assert!(self.len >= by, "internal: inc_start out of bounds");', "let _ = self.len;", "inc_start guard"), verify_cursor_source),
    ("inc_start_mutates_vtable", "cursor-source", edit_source_item("self.len -= by;", "self.vtable = self.vtable;\n        self.len -= by;", "inc_start vtable mutation"), verify_cursor_source),
    ("advance_assert_relaxed", "cursor-source", edit_source_item("cnt <= self.len()", "cnt < self.len()", "Buf advance assertion"), verify_cursor_source),
    ("advance_skips_inc_start", "cursor-source", edit_source_item("self.inc_start(cnt);", "let _ = cnt;", "Buf advance body"), verify_cursor_source),
    ("remaining_not_len", "cursor-source", edit_source_item("fn remaining(&self) -> usize {\n        self.len()\n    }", "fn remaining(&self) -> usize {\n        0\n    }", "Buf remaining body"), verify_cursor_source),
    ("chunk_not_as_slice", "cursor-source", edit_source_item("fn chunk(&self) -> &[u8] {\n        self.as_slice()\n    }", "fn chunk(&self) -> &[u8] {\n        &[]\n    }", "Buf chunk body"), verify_cursor_source),
    ("len_not_len_field", "cursor-source", edit_source_item("pub const fn len(&self) -> usize {\n        self.len\n    }", "pub const fn len(&self) -> usize {\n        0\n    }", "Bytes len body"), verify_cursor_source),
    ("cursor_extraction_body_mutated", "generated-cursor-binding", edit("native_cursor_bindings", "self.ptr.add(by)", "self.ptr.wrapping_add(by)", "extracted inc_start"), verify_cursor_source),
    ("cursor_source_map_hash_spoof", "source-map", mutate_cursor_map, verify_cursor_source),
    ("cursor_extractor_changed", "extractor", edit("extractor", "cursors={name:view_body(sig)", "cursors={name:changed_body(sig)", "cursor extractor"), verify_cursor_source),
    ("reviewed_manifest_changed", "production-inputs", mutate_source_manifest, verify_ancestor),
    ("inherited_slice_source_changed", "production-inputs", mutate_ancestor_bytes, verify_ancestor),
    ("field_profile_log_spoofed", "native-field-profile", mutate_profile_log, verify_source),
    ("field_profile_source_changed", "native-field-profile", edit("native_field_profile_source", "needs_drop::<usize>()", "needs_drop::<Vec<u8>>()", "field profile"), verify_source),

    # Client loop CFG and normal Drop/control facts.
    ("client_loop_condition_swapped", "client-mir", edit_client_mir("switchInt(move _17) -> [0: bb14, otherwise: bb8];", "switchInt(move _17) -> [0: bb8, otherwise: bb14];", "loop condition"), verify_client),
    ("client_step_index_wrong_local", "client-mir", edit_client_mir("_22 = copy (*_4)[_23];", "_22 = copy (*_4)[_14];", "step index local"), verify_client),
    ("client_remaining_wrong_receiver", "client-mir", edit_client_mir("_27 = &'_ _5;", "_27 = &'_ _6;", "loop remaining receiver"), verify_client),
    ("client_min_operands_swapped", "client-mir", edit_client_mir("std::cmp::min::<usize>(move _22, move _26)", "std::cmp::min::<usize>(move _26, move _22)", "min operands"), verify_client),
    ("client_advance_wrong_receiver", "client-mir", edit_client_mir("_29 = &'_ mut _5;", "_29 = &'_ mut _6;", "loop advance receiver"), verify_client),
    ("client_index_add_changed", "client-mir", edit_client_mir("AddWithOverflow(copy _14, const 1_usize)", "SubWithOverflow(copy _14, const 1_usize)", "loop index update"), verify_client),
    ("client_loop_backedge_broken", "client-mir", edit_client_block(13, "goto -> bb6;", "goto -> bb14;", "loop backedge"), verify_client),
    ("client_read_wrong_owner", "client-mir", edit_client_mir("_38 = &'_ _5;", "_38 = &'_ _6;", "final chunk receiver"), verify_client),
    ("client_to_vec_removed", "client-mir", edit_client_mir("std::slice::<impl [u8]>::to_vec(move _36)", "core::slice::<impl [u8]>::len(move _36)", "read materialization"), verify_client),
    ("client_rest_advance_wrong_arg", "client-mir", edit_client_mir("_43 = copy _39;", "_43 = const 0_usize;", "final rest argument"), verify_client),
    ("client_remaining_check_negated", "client-mir", edit_client_mir("_53 = Eq(move _54, move _55);", "_53 = Ne(move _54, move _55);", "zero remaining check"), verify_client),
    ("client_empty_check_negated", "client-mir", edit_client_mir("switchInt(move _66) -> [0: bb25, otherwise: bb24];", "switchInt(move _66) -> [0: bb24, otherwise: bb25];", "empty chunk check"), verify_client),
    ("client_saved_vec_copy", "client-mir", edit_client_mir("_0 = move _35;", "_0 = copy _35;", "save return before drop"), verify_client),
    ("client_original_drop_target_changed", "client-mir", edit_client_mir("drop(_7) -> [return: bb3, unwind: bb34];", "drop(_6) -> [return: bb3, unwind: bb34];", "Original Drop target"), verify_client),
    ("client_owner_drop_order_changed", "client-mir", edit_client_mir("drop(_6) -> [return: bb5, unwind: bb34];", "drop(_6) -> [return: bb26, unwind: bb34];", "Owner Drop edge"), verify_client),
    ("client_value_drop_before_saved_return", "client-mir", edit_client_block(24, "_0 = move _35;", "_0 = move _35;\n        drop(_5) -> [return: bb27, unwind: bb34];", "value Drop after return save"), verify_client),
    ("client_cleanup_value_target_changed", "client-mir", edit_client_mir("drop(_5) -> [return: bb34, unwind terminate(cleanup)];", "drop(_6) -> [return: bb34, unwind terminate(cleanup)];", "cleanup value Drop"), verify_client),
    ("client_mapping_normal_edge_redirected", "mapping", mutate_map_edges, verify_client),
    ("client_mapping_source_hash_wrong", "mapping", mutate_client_mapping_source_hash, verify_client),
    ("client_mapping_debug_place_spoofed", "mapping", mutate_debug_places, verify_client),
    ("client_mapping_block_hash_spoofed", "mapping", mutate_block_hash, verify_client),
    ("client_mapping_selected_mir_hash_spoofed", "mapping", mutate_mapping_mir_hash, verify_client),

    # Production MIR CFG mutations. The active bundle still pins all 30 exact
    # captures in addition to these operation/order assertions.
    ("inc_start_sub_changed", "cursor-mir", edit_mir("inc_start", "SubWithOverflow(copy ((*_1).1: usize), copy _12)", "AddWithOverflow(copy ((*_1).1: usize), copy _12)", "length subtraction"), verify_cursor_mir),
    ("inc_start_ptr_add_changed", "cursor-mir", edit_mir("inc_start", "const_ptr::<impl *const u8>::add(move _15, move _16)", "const_ptr::<impl *const u8>::wrapping_add(move _15, move _16)", "native ptr.add"), verify_cursor_mir),
    ("inc_start_ptr_store_wrong_field", "cursor-mir", edit_mir("inc_start", "((*_1).0: *const u8) = move _14;", "((*_1).2: core::sync::atomic::Atomic<*mut ()>) = move _14;", "pointer field write"), verify_cursor_mir),
    ("advance_bounds_branch_swapped", "advance", edit_mir("advance", "switchInt(move _4) -> [0: bb3, otherwise: bb2];", "switchInt(move _4) -> [0: bb2, otherwise: bb3];", "advance bounds branch"), verify_cursor_mir),
    ("advance_inc_start_call_removed", "advance", edit_mir("advance", "bytes::Bytes::inc_start(move _26, move _27)", "bytes::Bytes::len(move _26)", "advance success call"), verify_cursor_mir),
    ("remaining_len_route_changed", "remaining", edit_mir("remaining", "bytes::Bytes::len(move _2)", "bytes::Bytes::as_slice(move _2)", "remaining route"), verify_cursor_mir),
    ("chunk_as_slice_route_changed", "chunk", edit_mir("chunk", "bytes::Bytes::as_slice(move _3)", "bytes::Bytes::len(move _3)", "chunk route"), verify_cursor_mir),
    ("len_field_changed", "len", edit_mir("len", "((*_1).1: usize)", "((*_1).0: *const u8)", "len field"), verify_cursor_mir),
    ("inherited_shared_drop_mir_changed", "inherited-production-mir", edit_mir("release_shared", "free_shared", "drop_shared", "inherited Shared release MIR"), verify_capture),
    ("free_shared_payload_deallocation_removed", "free-shared-payload-mir", edit_captured_mir_and_refresh_hashes(
        "free_shared",
        "_4 = std::alloc::dealloc(move _5, move _6) -> [return: bb3, unwind continue];",
        "_4 = const ();",
        "payload backing allocation deallocation in free_shared"), verify_bundle),
    ("free_shared_control_deallocation_removed", "free-shared-control-mir", edit_captured_mir_and_refresh_hashes(
        "free_shared",
        "_9 = std::alloc::dealloc(move _10, move _12) -> [return: bb6, unwind continue];",
        "_9 = const ();",
        "Shared control allocation deallocation in free_shared"), verify_bundle),

    # Captured lineage, compiler stage, and test/tool evidence.
    ("capture_stage_changed", "capture", edit_capture("stage", "2-2-004.ElaborateDrops.after.mir", "2-2-004.Optimized.mir"), verify_capture),
    ("capture_client_digest_spoofed", "capture", edit_capture_row("client", "sha256", "4203a5b244f6ccf7cd8c2fe9d70ffc20d32d6f6e7068ddf0ec5446fb779ba04d", "0" * 64), verify_capture),
    ("capture_inc_start_path_redirected", "capture", edit_capture_row("inc_start", "path", "native-mir/bytes.bytes-{impl#0}-inc_start.2-2-004.ElaborateDrops.after.mir", "native-mir/other.mir"), verify_capture),
    ("capture_missing_production_method", "capture", edit_capture_row("chunk", "label", "chunk", "unused_chunk"), verify_capture),
    ("native_manifest_path_redirected", "native-manifest", edit("native_manifest", "../../../../", "../../../../../", "native package path"), verify_capture),
    ("native_lock_mutated", "native-lock", edit("native_lock", "name = \"bytes\"", "name = \"other-bytes\"", "native locked package"), verify_capture),
    ("native_test_log_claim_changed", "native-run-log", edit("native_test_log", "test result: ok. 1 passed", "test result: FAILED", "captured smoke result"), verify_capture),
    ("native_capture_procedure_mir_stage_changed", "capture-script", lambda d: d.__setitem__("capture_script", replace_first(d["capture_script"], "-Zmir-opt-level=0", "-Zmir-opt-level=3", "MIR optimization level")), verify_capture),
]


def run_controls() -> dict[str, Any]:
    base = C.load_bundle()
    C.audit_bundle(base)
    rows = []
    for case_id, component, mutate, audit in CONTROL_CASES:
        data = copy.deepcopy(base)
        try:
            mutate(data)
            audit(data)
        except C.AuditError as exc:
            rows.append({"id": case_id, "component": component,
                         "status": "rejected_as_expected", "reason": str(exc)})
        except Exception as exc:
            rows.append({"id": case_id, "component": component,
                         "status": "checker_error", "reason": f"{type(exc).__name__}: {exc}"})
        else:
            rows.append({"id": case_id, "component": component, "status": "accepted"})
    return {
        "schema": "ar-native-cursor-source-mir-controls-v1",
        "checker_sha256": sha(CHECKER_PATH.read_bytes()),
        "control_count": len(rows),
        "rejected_as_expected": sum(r["status"] == "rejected_as_expected" for r in rows),
        "accepted": [r["id"] for r in rows if r["status"] == "accepted"],
        "checker_errors": [r["id"] for r in rows if r["status"] == "checker_error"],
        "controls": rows,
        "mutations_in_memory_only": True,
        "source_mutated_on_disk": False,
        "cargo_or_rust_build_invoked": False,
        "proof_tool_or_solver_invoked": False,
    }


def main() -> int:
    result = run_controls()
    fixture = {"schema": result["schema"],
               "cases": [{"id": row["id"], "component": row["component"]}
                         for row in result["controls"]]}
    FIXTURE_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_PATH.write_text(json.dumps(fixture, indent=2) + "\n")
    result["fixture_sha256"] = hashlib.sha256(FIXTURE_PATH.read_bytes()).hexdigest()
    OUTPUT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if not result["accepted"] and not result["checker_errors"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
