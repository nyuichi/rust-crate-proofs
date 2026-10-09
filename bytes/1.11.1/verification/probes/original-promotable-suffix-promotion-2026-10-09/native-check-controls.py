#!/usr/bin/env python3
"""In-memory AV native source/MIR checker mutation controls; no Rust tools."""
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
    spec = importlib.util.spec_from_file_location("av_native_checker", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot import AV native checker")
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


def edit(field: str, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d[field] = replace_once(d[field], old, new, label)
    return mutate


def edit_client(old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d["mir_sources"]["client"] = replace_once(d["mir_sources"]["client"], old, new, label)
    return mutate


def edit_production(label: str, old: str, new: str, description: str) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        d["mir_sources"][label] = replace_once(d["mir_sources"][label], old, new, description)
    return mutate


def edit_mapping(path: tuple[str, ...], value: Any) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        mapping = copy.deepcopy(d["mapping"])
        target: Any = mapping
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        d["mapping"] = mapping
    return mutate


def mutate_mapping_row(path: tuple[str, ...], callback: Callable[[Any], None]) -> Callable[[dict[str, Any]], None]:
    def mutate(d: dict[str, Any]) -> None:
        mapping = copy.deepcopy(d["mapping"])
        target: Any = mapping
        for key in path:
            target = target[key]
        callback(target)
        d["mapping"] = mapping
    return mutate


def edit_capture_client_hash(d: dict[str, Any]) -> None:
    row = next(row for row in d["capture"]["selected"] if row["label"] == "client")
    row["sha256"] = "0" * 64


def verify_source(d: dict[str, Any]) -> None:
    C.audit_source(d)


def verify_client(d: dict[str, Any]) -> None:
    C.audit_client_mir(d)


def verify_capture(d: dict[str, Any]) -> None:
    C.audit_capture(d)


CONTROL_CASES: list[tuple[str, str, Callable[[dict[str, Any]], None], Callable[[dict[str, Any]], None]]] = [
    # Native witness source and runtime-domain mutations.
    ("source_advance_omitted", "native-source", edit("native_source", "root.advance(amount);", "// advance omitted", "root advance"), verify_source),
    ("source_clone_before_advance", "native-source", edit("native_source", "root.advance(amount);\n        root.clone()", "let cloned = root.clone();\n        root.advance(amount);\n        cloned", "promotion ordering"), verify_source),
    ("source_clone_wrong_owner", "native-source", edit("native_source", "root.clone()", "Bytes::from(input).clone()", "first Clone receiver"), verify_source),
    ("source_read_wrong_owner", "native-source", edit("native_source", "child.chunk().to_vec()", "root.chunk().to_vec()", "suffix read receiver"), verify_source),
    ("test_full_advance_omitted", "native-test", edit("native_test_source", "for amount in 0..=len", "for amount in 0..len", "amount domain"), verify_source),
    ("test_case_count_claim_changed", "native-test", edit("native_test_source", "assert_eq!(cases, 292);", "assert_eq!(cases, 291);", "test case count"), verify_source),

    # Native post-ElaborateDrops caller: operation order, object identity, Drops.
    ("mir_from_box_route_changed", "client-mir", edit_client("<bytes::Bytes as From<Box<[u8]>>>::from(move _5)", "bytes::Bytes::new()", "Box conversion"), verify_client),
    ("mir_advance_receiver_changed", "client-mir", edit_client("_7 = &'_ mut _4;", "_7 = &'_ mut _3;", "advance receiver"), verify_client),
    ("mir_advance_operand_changed", "client-mir", edit_client("_8 = copy _2;", "_8 = const 0_usize;", "advance amount"), verify_client),
    ("mir_first_clone_wrong_receiver", "client-mir", edit_client("_9 = &'_ _4;", "_9 = &'_ _3;", "first Clone receiver"), verify_client),
    ("mir_first_clone_target_changed", "client-mir", edit_client("_3 = <bytes::Bytes as Clone>::clone(move _9) -> [return: bb3, unwind: bb10];", "_3 = <bytes::Bytes as Clone>::clone(move _9) -> [return: bb1, unwind: bb10];", "Clone normal target"), verify_client),
    ("mir_root_drop_omitted", "client-mir", edit_client("drop(_4) -> [return: bb4, unwind: bb12];", "StorageDead(_4);", "root normal Drop"), verify_client),
    ("mir_child_read_wrong_receiver", "client-mir", edit_client("_12 = &'_ _3;", "_12 = &'_ _4;", "child read receiver"), verify_client),
    ("mir_vec_result_removed", "client-mir", edit_client("_0 = slice::<impl [u8]>::to_vec(move _10)", "_0 = const Vec::<u8>::new()", "saved Vec evaluation"), verify_client),
    ("mir_child_drop_omitted", "client-mir", edit_client("drop(_3) -> [return: bb7, unwind: bb12];", "StorageDead(_3);", "child final Drop"), verify_client),
    ("mir_vec_result_successor_changed", "client-mir", edit_client("_0 = slice::<impl [u8]>::to_vec(move _10) -> [return: bb6, unwind: bb9];", "_0 = slice::<impl [u8]>::to_vec(move _10) -> [return: bb7, unwind: bb9];", "Vec evaluation successor"), verify_client),

    # Mapping and receipt cannot override independently parsed MIR.
    ("mapping_root_owner_spoofed", "mapping", mutate_mapping_row(("normal_edges",), lambda xs: xs[0].__setitem__("owner", "child")), verify_client),
    ("mapping_child_drop_target_spoofed", "mapping", mutate_mapping_row(("normal_edges",), lambda xs: xs[1].__setitem__("successor", "bb99")), verify_client),
    ("mapping_debug_place_spoofed", "mapping", mutate_mapping_row(("debug_places",), lambda m: m.__setitem__("root", "_99")), verify_client),
    ("mapping_block_hash_spoofed", "mapping", mutate_mapping_row(("mir_blocks",), lambda xs: xs[0].__setitem__("body_sha256", "0" * 64)), verify_client),
    ("mapping_native_source_hash_spoofed", "mapping", edit_mapping(("native_source_sha256",), "0" * 64), verify_client),
    ("capture_client_hash_spoofed", "capture", edit_capture_client_hash, verify_capture),

    # Inherited native source/MIR gate: both promotion branches, acquire,
    # guarded increment, release, and both frees remain checked through AT.
    ("inherited_stored_vtable_clone_callback_changed", "production-mir",
        edit_production("promotable_even_clone", "Ordering::Acquire", "Ordering::Relaxed", "promotable Clone acquire"), verify_capture),
    ("inherited_payload_free_removed", "production-mir",
        edit_production("free_shared", "std::alloc::dealloc(move _5, move _6)", "core::mem::forget(move _5)", "payload free"), verify_capture),
    ("inherited_control_free_removed", "production-mir",
        edit_production("free_shared", "std::alloc::dealloc(move _10, move _12)", "core::mem::forget(move _10)", "control free"), verify_capture),
]


def run_controls() -> dict[str, Any]:
    base = C.load_bundle()
    # Verify the unmutated gate before testing mutations. This is a read-only
    # source/MIR audit, not a compiler, prover, or solver invocation.
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
        "schema": "av-native-promotable-suffix-controls-v1",
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
