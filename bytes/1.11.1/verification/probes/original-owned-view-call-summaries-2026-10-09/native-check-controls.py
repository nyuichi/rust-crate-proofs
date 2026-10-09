#!/usr/bin/env python3
"""In-memory AT native source/MIR checker mutation controls."""
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
    spec = importlib.util.spec_from_file_location("at_native_owned_view_checker", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot import AT native correspondence checker")
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


def edit_text(field: str, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d[field] = replace_once(d[field], old, new, label)
    return mutate


def edit_client(old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d["mir_sources"]["client"] = replace_once(d["mir_sources"]["client"], old, new, label)
    return mutate


def edit_client_block(block: int, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        text = d["mir_sources"]["client"]
        marker = f"    bb{block}"
        start = text.index(marker)
        end = text.index("\n    }", start) + len("\n    }")
        d["mir_sources"]["client"] = text[:start] + replace_once(text[start:end], old, new, label) + text[end:]
    return mutate


def edit_mir(label: str, old: str, new: str, why: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d["mir_sources"][label] = replace_once(d["mir_sources"][label], old, new, why)
    return mutate


def edit_map(path: tuple[str, ...], value: Any) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        m = json.loads(d["mapping_text"])
        target: Any = m
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        d["mapping_text"] = json.dumps(m, indent=2) + "\n"
    return mutate


def mutate_map_list(path: tuple[str, ...], callback: Callable[[Any], None]) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        m = json.loads(d["mapping_text"])
        target: Any = m
        for key in path:
            target = target[key]
        callback(target)
        d["mapping_text"] = json.dumps(m, indent=2) + "\n"
    return mutate


def verify_bundle(d: dict[str, Any]) -> None:
    C.audit_bundle(d)


def verify_client(d: dict[str, Any]) -> None:
    C.audit_client_mir(d)


def verify_capture(d: dict[str, Any]) -> None:
    C.audit_capture(d)


def verify_source(d: dict[str, Any]) -> None:
    C.audit_source_and_profile(d)


def verify_cursor_source(d: dict[str, Any]) -> None:
    C.audit_cursor_source(d)


def verify_cursor_mir(d: dict[str, Any]) -> None:
    C.audit_cursor_mir(d)


def mutate_client_drop_early(d: dict[str, Any]) -> None:
    mir = d["mir_sources"]["client"]
    block9 = "    bb9: {\n        StorageDead(_25);\n        StorageLive(_26);\n        _26 = move _24;\n        drop(_6) -> [return: bb10, unwind: bb11];\n    }"
    early = "    bb9: {\n        StorageDead(_25);\n        StorageLive(_26);\n        drop(_6) -> [return: bb10, unwind: bb11];\n        _26 = move _24;\n    }"
    d["mir_sources"]["client"] = replace_once(mir, block9, early, "assignment Drop moved before stashing next")


def mutate_client_drop_after_install(d: dict[str, Any]) -> None:
    mir = d["mir_sources"]["client"]
    old = "        drop(_6) -> [return: bb10, unwind: bb11];"
    new = "        _6 = move _26;"
    d["mir_sources"]["client"] = replace_once(mir, old, new, "assignment Drop omitted before installation")


def mutate_saved_return_before_result(d: dict[str, Any]) -> None:
    mir = d["mir_sources"]["client"]
    old = "    bb16: {\n        _31 = &'_ (*_32);\n        StorageDead(_33);\n        _0 = slice::<impl [u8]>::to_vec(move _31) -> [return: bb17, unwind: bb21];\n    }\n\n    bb17: {\n        StorageDead(_31);\n        StorageDead(_18);\n        drop(_6) -> [return: bb18, unwind: bb25];"
    new = "    bb16: {\n        drop(_6) -> [return: bb17, unwind: bb21];\n        _31 = &'_ (*_32);\n        StorageDead(_33);\n        _0 = slice::<impl [u8]>::to_vec(move _31) -> [return: bb17, unwind: bb21];\n    }\n\n    bb17: {\n        StorageDead(_31);\n        StorageDead(_18);\n        StorageDead(_6);"
    d["mir_sources"]["client"] = replace_once(mir, old, new, "final Drop moved before saved Vec evaluation")


CONTROL_CASES: list[tuple[str, str, Callable[[dict[str, Any]], None], Callable[[dict[str, Any]], None]]] = [
    # Source and execution witness surface.
    ("client_wrong_box_constructor", "native-source", edit_text("native_source", "Bytes::from(input)", "Bytes::new()", "Box constructor"), verify_source),
    ("client_wrong_slice_owner", "native-source", edit_text("native_source", "owner.slice(a..b)", "original.slice(a..b)", "slice owner"), verify_source),
    ("client_wrong_advance_owner", "native-source", edit_text("native_source", "value.advance(advance_by);", "owner.advance(advance_by);", "advance receiver"), verify_source),
    ("client_fixed_rounds_quota", "native-source", edit_text("native_source", "while i < rounds", "while i < core::cmp::min(rounds, 1)", "rounds quota"), verify_source),
    ("client_clone_wrong_receiver", "native-source", edit_text("native_source", "let next = value.clone();", "let next = owner.clone();", "clone receiver"), verify_source),
    ("client_replacement_omitted", "native-source", edit_text("native_source", "value = next;", "let _next = next;", "replacement assignment"), verify_source),
    ("client_loop_index_reset", "native-source", edit_text("native_source", "i += 1;", "i = 0;", "loop index progress"), verify_source),
    ("client_chunk_wrong_receiver", "native-source", edit_text("native_source", "value.chunk().to_vec()", "owner.chunk().to_vec()", "return receiver"), verify_source),
    ("client_to_vec_removed", "native-source", edit_text("native_source", "value.chunk().to_vec()", "value.len()", "Vec observation"), verify_source),
    ("test_round_domain_reduced", "native-test", edit_text("native_test_source", "[0, 1, 2, 7, 31]", "[0, 1, 2]", "runtime rounds"), verify_source),
    ("test_case_count_claim_changed", "native-test", edit_text("native_test_source", "assert_eq!(cases, 145);", "assert_eq!(cases, 144);", "test count"), verify_source),

    # Selected client MIR: construction, repeated assignment Drop, and result.
    ("mir_from_box_constructor_changed", "client-mir", edit_client("<bytes::Bytes as From<Box<[u8]>>>::from(move _9)", "bytes::Bytes::new()", "From<Box> route"), verify_client),
    ("mir_original_clone_changed", "client-mir", edit_client("<bytes::Bytes as Clone>::clone(move _10)", "bytes::Bytes::new()", "original clone"), verify_client),
    ("mir_owner_slice_receiver_changed", "client-mir", edit_client("_11 = &'_ _7;", "_11 = &'_ _8;", "slice owner MIR"), verify_client),
    ("mir_range_specialization_changed", "client-mir", edit_client("slice::<std::ops::Range<usize>>", "slice::<impl std::ops::RangeBounds<usize>>", "Range specialization"), verify_client),
    ("mir_initial_advance_wrong_receiver", "client-mir", edit_client("_16 = &'_ mut _6;", "_16 = &'_ mut _7;", "initial advance receiver"), verify_client),
    ("mir_rounds_bound_replaced", "client-mir", edit_client("_23 = copy _5;", "_23 = const 1_usize;", "rounds operand"), verify_client),
    ("mir_loop_condition_changed", "client-mir", edit_client("_21 = Lt(move _22, move _23);", "_21 = Le(move _22, move _23);", "loop predicate"), verify_client),
    ("mir_iteration_clone_wrong_receiver", "client-mir", edit_client("_25 = &'_ _6;", "_25 = &'_ _7;", "iteration clone receiver"), verify_client),
    ("mir_assignment_drop_omitted", "client-mir", edit_client("drop(_6) -> [return: bb10, unwind: bb11];", "StorageDead(_6);", "old value Drop"), verify_client),
    ("mir_assignment_drop_early_order", "client-mir", mutate_client_drop_early, verify_client),
    ("mir_assignment_drop_after_install", "client-mir", mutate_client_drop_after_install, verify_client),
    ("mir_move_next_wrong_local", "client-mir", edit_client_block(10, "_6 = move _26;", "_6 = move _24;", "move-next source"), verify_client),
    ("mir_loop_counter_reset", "client-mir", edit_client("_18 = move (_27.0: usize);", "_18 = const 0_usize;", "loop counter increment"), verify_client),
    ("mir_loop_backedge_broken", "client-mir", edit_client_block(14, "goto -> bb7;", "goto -> bb6;", "loop backedge"), verify_client),
    ("mir_final_chunk_wrong_receiver", "client-mir", edit_client("_33 = &'_ _6;", "_33 = &'_ _7;", "final chunk receiver"), verify_client),
    ("mir_result_materialization_removed", "client-mir", edit_client("slice::<impl [u8]>::to_vec(move _31)", "core::slice::<impl [u8]>::len(move _31)", "to_vec result"), verify_client),
    ("mir_final_drop_omitted", "client-mir", edit_client("drop(_6) -> [return: bb18, unwind: bb25];", "StorageDead(_6);", "final value Drop"), verify_client),
    ("mir_final_drop_before_saved_result", "client-mir", mutate_saved_return_before_result, verify_client),

    # The displayed MIR mapping is checked against parsed facts, not trusted.
    ("mapping_normal_drop_owner_changed", "mapping", mutate_map_list(("normal_edges",), lambda xs: xs[0].__setitem__("owner", "wrong")), verify_client),
    ("mapping_repeated_assignment_drop_unbound", "mapping", mutate_map_list(("normal_edges",), lambda xs: xs[2].__setitem__("repeated", False)), verify_client),
    ("mapping_drop_target_changed", "mapping", mutate_map_list(("normal_edges",), lambda xs: xs[2].__setitem__("successor", "bb99")), verify_client),
    ("mapping_debug_place_spoofed", "mapping", mutate_map_list(("debug_places",), lambda m: m.__setitem__("value", "_99")), verify_client),
    ("mapping_block_hash_spoofed", "mapping", mutate_map_list(("mir_blocks",), lambda xs: xs[0].__setitem__("body_sha256", "0" * 64)), verify_client),
    ("mapping_assignment_drop_order_changed", "mapping", mutate_map_list(("native_assignment",), lambda m: m.__setitem__("drop_block", "bb10")), verify_client),
    ("mapping_saved_return_place_changed", "mapping", mutate_map_list(("native_saved_return",), lambda m: m.__setitem__("result_place", "_8")), verify_client),
    ("mapping_source_hash_spoofed", "mapping", edit_map(("native_source_sha256",), "0" * 64), verify_client),
    ("capture_client_digest_spoofed", "capture", lambda d: d["capture"]["selected"][0].__setitem__("sha256", "0" * 64), verify_capture),

    # Inherited actual-source/MIR gates: atomic/refcount/vtable/free and pointer.
    ("inherited_shared_clone_ordering_changed", "production-mir", edit_mir("shared_clone", "Ordering::Relaxed", "Ordering::SeqCst", "Shared pointer load ordering"), verify_capture),
    ("inherited_promotable_acquire_removed", "production-mir", edit_mir("promotable_even_clone", "Ordering::Acquire", "Ordering::Relaxed", "promotion acquire"), verify_capture),
    ("inherited_refcount_increment_bypassed", "production-mir", edit_mir("ref_count_increment", "try_increment(move _5)", "try_increment_unchecked(move _5)", "guarded refcount increment"), verify_capture),
    ("inherited_release_acquire_weakened", "production-mir", edit_mir("release_shared", "Ordering::Acquire", "Ordering::Relaxed", "final release ordering"), verify_capture),
    ("inherited_payload_free_removed", "production-mir", edit_mir("free_shared", "std::alloc::dealloc(move _5, move _6)", "core::mem::forget(move _5)", "payload deallocation"), verify_capture),
    ("inherited_control_free_removed", "production-mir", edit_mir("free_shared", "std::alloc::dealloc(move _10, move _12)", "core::mem::forget(move _10)", "control deallocation"), verify_capture),
    ("inherited_inc_start_ptr_add_changed", "production-mir", edit_mir("inc_start", "core::ptr::const_ptr::<impl *const u8>::add(move _15, move _16)", "core::ptr::const_ptr::<impl *const u8>::wrapping_add(move _15, move _16)", "native ptr.add"), verify_capture),
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
        "schema": "at-native-owned-view-clone-controls-v1",
        "checker_sha256": sha(CHECKER_PATH.read_bytes()),
        "control_count": len(rows),
        "rejected_as_expected": sum(row["status"] == "rejected_as_expected" for row in rows),
        "accepted": [row["id"] for row in rows if row["status"] == "accepted"],
        "checker_errors": [row["id"] for row in rows if row["status"] == "checker_error"],
        "controls": rows,
        "mutations_in_memory_only": True,
        "source_mutated_on_disk": False,
        "cargo_or_rust_build_invoked": False,
        "proof_tool_or_solver_invoked": False,
    }


def main() -> int:
    result = run_controls()
    fixture = {"schema": result["schema"],
        "cases": [{"id": row["id"], "component": row["component"]} for row in result["controls"]]}
    FIXTURE_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_PATH.write_text(json.dumps(fixture, indent=2) + "\n")
    result["fixture_sha256"] = sha(FIXTURE_PATH.read_bytes())
    OUTPUT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if not result["accepted"] and not result["checker_errors"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
