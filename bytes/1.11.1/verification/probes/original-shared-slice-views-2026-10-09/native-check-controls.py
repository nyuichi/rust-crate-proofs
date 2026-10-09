#!/usr/bin/env python3
"""Replay AQ native correspondence mutations; does not invoke Cargo or Why3."""
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
OUTPUT_PATH = ROOT / "generated/native-check-controls.json"
FIXTURE_PATH = ROOT / "fixtures/native-check-controls.json"


def load_checker():
    spec = importlib.util.spec_from_file_location("aq_native_view_checker", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot import AQ native checker")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


C = load_checker()


def edit(field: str, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(data: dict[str, Any]) -> None:
        text = data[field]
        count = text.count(old)
        if count != 1:
            raise RuntimeError(f"{label}: expected one mutation anchor, found {count}: {old!r}")
        data[field] = text.replace(old, new, 1)
    return mutate


def mir(label: str, old: str, new: str, why: str) -> Callable[[dict[str, Any]], None]:
    return _mir_edit(label, old, new, why)


def edit_nth(field: str, old: str, new: str, occurrence: int, label: str):
    def mutate(data: dict[str, Any]) -> None:
        text = data[field]
        indexes = []
        start = 0
        while (index := text.find(old, start)) >= 0:
            indexes.append(index)
            start = index + len(old)
        if occurrence >= len(indexes):
            raise RuntimeError(f"{label}: occurrence {occurrence} absent; found {len(indexes)}")
        index = indexes[occurrence]
        data[field] = text[:index] + new + text[index + len(old):]
    return mutate


def edit_production_item(signature: str, old: str, new: str, label: str):
    def mutate(data: dict[str, Any]) -> None:
        source = data["production_source"]
        body = C.code_slice(source, signature, label)
        if body.count(old) != 1:
            raise RuntimeError(f"{label}: expected one body anchor, found {body.count(old)}")
        data["production_source"] = source.replace(body, body.replace(old, new, 1), 1)
    return mutate


def drop_selected_before_read(data: dict[str, Any]) -> None:
    old = "};\n    let observed = AsRef::<[u8]>::as_ref(&selected).to_vec();"
    new = "};\n    drop(selected);\n    let observed = AsRef::<[u8]>::as_ref(&selected).to_vec();"
    text = data["native_source"]
    if text.count(old) != 1:
        raise RuntimeError("selected drop-before-read anchor changed")
    data["native_source"] = text.replace(old, new, 1)


def _mir_edit(label: str, old: str, new: str, why: str) -> Callable[[dict[str, Any]], None]:
    def mutate(data: dict[str, Any]) -> None:
        text = data["mir_sources"][label]
        count = text.count(old)
        if count != 1:
            raise RuntimeError(f"{why}: expected one MIR anchor, found {count}: {old!r}")
        data["mir_sources"][label] = text.replace(old, new, 1)
    return mutate


def verify_source(data: dict[str, Any]) -> Any:
    return C.audit_source(data)


def verify_views(data: dict[str, Any]) -> Any:
    return C.audit_view_bindings(data)


def verify_client(data: dict[str, Any]) -> Any:
    return C.audit_client_mir(data["mir_sources"]["client"], data["mapping_text"])


def verify_slice(data: dict[str, Any]) -> Any:
    return C.audit_slice_mir(data["mir_sources"]["slice"])


def verify_empty(data: dict[str, Any]) -> Any:
    return C.audit_empty_mir(data)


def verify_manifest(data: dict[str, Any]) -> Any:
    return C.audit_reviewed_inputs(data)


def verify_capture(data: dict[str, Any]) -> Any:
    return C.audit_capture(data)


def replace_mapping_edge(data: dict[str, Any]) -> None:
    mapping = json.loads(data["mapping_text"])
    mapping["normal_edges"][0]["successor"] = "bb4"
    data["mapping_text"] = json.dumps(mapping, indent=2) + "\n"


def change_stage(data: dict[str, Any]) -> None:
    data["capture"]["stage"] = "2-2-004.Optimized"


CONTROL_CASES: list[tuple[str, str, Callable[[dict[str, Any]], Any], Callable[[dict[str, Any]], None]]] = [
    ("client_outer_slice_wrong_receiver", "native-client", verify_source,
     edit("native_source", "owner.slice(a..b)", "original.slice(a..b)", "outer receiver")),
    ("client_inner_slice_wrong_receiver", "native-client", verify_source,
     edit("native_source", "first.slice(c..d)", "owner.slice(c..d)", "inner receiver")),
    ("client_selected_drop_before_read", "native-client", verify_source, drop_selected_before_read),
    ("client_selected_read_removed", "native-client", verify_source,
     edit("native_source", "AsRef::<[u8]>::as_ref(&selected).to_vec()", "selected.to_vec()", "read route")),
    ("client_range_arguments_swapped", "native-client", verify_source,
     edit("native_source", "first.slice(c..d)", "first.slice(d..c)", "inner bounds")),
    ("native_test_empty_end_coverage_removed", "native-client-test", verify_source,
     edit("native_test_source", "(len,len),", "(len/2,len),", "empty endpoint")),
    ("slice_source_checked_add_changed", "production-source", verify_views,
     edit_nth("production_source", "n.checked_add(1).expect(\"out of range\")", "n.saturating_add(1)", 0, "checked addition")),
    ("slice_source_assertion_removed", "production-source", verify_views,
     edit("production_source", "assert!(\n            begin <= end,", "debug_assert!(\n            begin <= end,", "begin bound assertion")),
    ("slice_source_empty_constructor_changed", "production-source", verify_views,
     edit("production_source", "Bytes::new_empty_with_ptr(self.ptr.wrapping_add(begin))", "Bytes::new_empty_with_ptr(self.ptr.add(begin))", "empty address")),
    ("slice_source_nonempty_length_changed", "production-source", verify_views,
     edit("production_source", "ret.len = end - begin;", "ret.len = end - begin + 1;", "nonempty length")),
    ("slice_source_nonempty_pointer_changed", "production-source", verify_views,
     edit("production_source", "ret.ptr.add(begin)", "ret.ptr.wrapping_add(begin)", "live pointer add")),
    ("static_constructor_vtable_changed", "production-source", verify_views,
     edit_production_item("fn new_empty_with_ptr(ptr: *const u8) -> Self", "vtable: &STATIC_VTABLE,", "vtable: &PROMOTABLE_EVEN_VTABLE,", "empty static route")),
    ("static_clone_body_changed", "production-source", verify_views,
     edit_production_item("unsafe fn static_clone(", "Bytes::from_static(slice)", "Bytes::copy_from_slice(slice)", "static clone")),
    ("static_drop_body_changed", "production-source", verify_views,
     edit("production_source", "// nothing to drop for &'static [u8]", "drop(ptr)", "static drop")),
    ("without_provenance_changed", "production-source", verify_views,
     edit("production_source", "core::ptr::null::<u8>().wrapping_add(ptr)", "ptr as *const u8", "without provenance")),
    ("as_ref_route_changed", "production-source", verify_views,
     edit("production_source", "fn as_ref(&self) -> &[u8] {\n        self.as_slice()\n    }", "fn as_ref(&self) -> &[u8] {\n        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }\n    }", "AsRef route")),
    ("generated_view_body_changed", "generated-binding", verify_views,
     edit("native_view_bindings", "ret.ptr = unsafe { ret.ptr.add(begin) };", "ret.ptr = ret.ptr;", "generated body")),
    ("generated_view_source_map_changed", "source-map", verify_views,
     edit("source_map_text", "\"view_bodies\"", "\"other_bodies\"", "source map")),
    ("view_extractor_changed", "extractor", verify_views,
     edit("extractor", "def view_body", "def changed_view_body", "extractor")),
    ("client_original_drop_target_changed", "client-mir", verify_client,
     mir("client", "drop(_8) -> [return: bb3, unwind: bb18];", "drop(_7) -> [return: bb3, unwind: bb18];", "Original target")),
    ("client_first_drop_successor_changed", "client-mir", verify_client,
     mir("client", "drop(_11) -> [return: bb6, unwind: bb15];", "drop(_11) -> [return: bb7, unwind: bb15];", "First successor")),
    ("client_owner_drop_unwind_changed", "client-mir", verify_client,
     mir("client", "drop(_7) -> [return: bb7, unwind: bb18];", "drop(_7) -> [return: bb7, unwind: bb19];", "Owner unwind")),
    ("client_selected_drop_before_result_move", "client-mir", verify_client,
     mir("client", "drop(_6) -> [return: bb11, unwind: bb18];", "drop(_20) -> [return: bb11, unwind: bb18];", "Selected drop")),
    ("client_second_slice_wrong_specialization", "client-mir", verify_client,
     mir("client", "bytes::Bytes::slice::<std::ops::Range<usize>>(move _16, move _17)", "bytes::Bytes::slice::<std::ops::RangeBounds<usize>>(move _16, move _17)", "Range specialization")),
    ("client_asref_after_selected_drop", "client-mir", verify_client,
     mir("client", "_22 = <bytes::Bytes as AsRef<[u8]>>::as_ref(move _23)", "_22 = <bytes::Bytes as AsRef<[u8]>>::as_ref(move _6)", "AsRef receiver")),
    ("client_saved_return_missing", "client-mir", verify_client,
     mir("client", "_0 = move _20;", "_0 = copy _20;", "return value move")),
    ("client_cleanup_drop_target_changed", "client-mir", verify_client,
     mir("client", "drop(_6) -> [return: bb18, unwind terminate(cleanup)];", "drop(_7) -> [return: bb18, unwind terminate(cleanup)];", "cleanup edge")),
    ("mapping_edges_changed", "mapping-binding", verify_client, replace_mapping_edge),
    ("slice_start_bound_edges_swapped", "slice-mir", verify_slice,
     mir("slice", "switchInt(move _8) -> [0: bb6, 1: bb5, 2: bb4, otherwise: bb3];", "switchInt(move _8) -> [0: bb5, 1: bb6, 2: bb4, otherwise: bb3];", "start Bound edges")),
    ("slice_end_bound_edges_swapped", "slice-mir", verify_slice,
     mir("slice", "switchInt(move _18) -> [0: bb13, 1: bb12, 2: bb11, otherwise: bb3];", "switchInt(move _18) -> [0: bb12, 1: bb13, 2: bb11, otherwise: bb3];", "end Bound edges")),
    ("slice_bound_assert_bypassed", "slice-mir", verify_slice,
     mir("slice", "switchInt(move _26) -> [0: bb18, otherwise: bb17];", "switchInt(move _26) -> [0: bb17, otherwise: bb18];", "begin assertion edge")),
    ("slice_end_assert_bypassed", "slice-mir", verify_slice,
     mir("slice", "switchInt(move _45) -> [0: bb23, otherwise: bb22];", "switchInt(move _45) -> [0: bb22, otherwise: bb23];", "end assertion edge")),
    ("slice_empty_nonempty_edges_swapped", "slice-mir", verify_slice,
     mir("slice", "switchInt(move _64) -> [0: bb28, otherwise: bb27];", "switchInt(move _64) -> [0: bb27, otherwise: bb28];", "empty branch")),
    ("slice_empty_address_uses_live_add", "slice-mir", verify_slice,
     mir("slice", "wrapping_add(move _69, move _70)", "add(move _69, move _70)", "empty wrapping add")),
    ("slice_empty_branch_clones", "slice-mir", verify_slice,
     mir("slice", "_0 = bytes::Bytes::new_empty_with_ptr(move _68)", "_0 = <bytes::Bytes as Clone>::clone(move _72)", "empty clone")),
    ("slice_nonempty_branch_constructs_empty", "slice-mir", verify_slice,
     mir("slice", "_71 = <bytes::Bytes as Clone>::clone(move _72)", "_71 = bytes::Bytes::new_empty_with_ptr(move _68)", "nonempty constructor")),
    ("slice_subtraction_result_changed", "slice-mir", verify_slice,
     mir("slice", "_75 = SubWithOverflow(copy _73, copy _74);", "_75 = AddWithOverflow(copy _73, copy _74);", "length operation")),
    ("slice_nonempty_pointer_operation_changed", "slice-mir", verify_slice,
     mir("slice", "core::ptr::const_ptr::<impl *const u8>::add(move _77, move _78)", "core::ptr::const_ptr::<impl *const u8>::wrapping_add(move _77, move _78)", "nonempty ptr add")),
    ("empty_constructor_null_path_removed", "empty-mir", verify_empty,
     mir("new_empty_with_ptr", "_7 = panic(const \"assertion failed: !ptr.is_null()\")", "_7 = const ();", "nonnull guard")),
    ("empty_constructor_provenance_detach_removed", "empty-mir", verify_empty,
     mir("new_empty_with_ptr", "bytes::without_provenance(move _9)", "copy _10", "provenance detach")),
    ("empty_constructor_atomic_not_null", "empty-mir", verify_empty,
     mir("new_empty_with_ptr", "_13 = null_mut::<()>()", "_13 = copy _1 as *mut ()", "empty data")),
    ("empty_constructor_length_changed", "empty-mir", verify_empty,
     mir("new_empty_with_ptr", "len: const 0_usize", "len: const 1_usize", "empty length")),
    ("static_clone_mir_route_changed", "empty-mir", verify_empty,
     mir("static_clone", "Bytes::from_static(move _7)", "Bytes::copy_from_slice(move _7)", "static clone MIR")),
    ("static_drop_mir_effect_added", "empty-mir", verify_empty,
     mir("static_drop", "_0 = const ();", "_0 = core::mem::drop(move _2);", "static drop MIR")),
    ("reviewed_input_manifest_base_changed", "production-inputs", verify_manifest,
     edit("reviewed_production_manifest", "361c7cd261507ac0a705b3b836f73240070891c6", "0000000000000000000000000000000000000000", "base commit")),
    ("reviewed_source_input_changed", "production-inputs", verify_manifest,
     lambda d: d["production_source_inputs"].__setitem__("src/bytes.rs", d["production_source_inputs"]["src/bytes.rs"]+"\n// mutation\n")),
    ("capture_stage_changed", "native-capture", verify_capture, change_stage),
]


def main() -> int:
    base = C.load_bundle()
    rows = []
    for case_id, component, audit, mutate in CONTROL_CASES:
        data = copy.deepcopy(base)
        try:
            mutate(data)
            audit(data)
        except Exception as exc:
            rows.append({"id": case_id, "component": component, "status": "rejected",
                         "reason": f"{type(exc).__name__}: {exc}"})
        else:
            rows.append({"id": case_id, "component": component, "status": "accepted"})
    result = {"schema": "aq-native-source-mir-controls-v1",
              "checker_sha256": hashlib.sha256(CHECKER_PATH.read_bytes()).hexdigest(),
              "control_count": len(rows),
              "rejected": sum(row["status"] == "rejected" for row in rows),
              "accepted": [row["id"] for row in rows if row["status"] != "rejected"],
              "controls": rows}
    FIXTURE_PATH.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_PATH.write_text(json.dumps({"schema": result["schema"],
      "cases": [{"id": r["id"], "component": r["component"]} for r in rows]}, indent=2)+"\n")
    OUTPUT_PATH.write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps(result, indent=2))
    return 0 if result["accepted"] == [] else 1


if __name__ == "__main__":
    raise SystemExit(main())
