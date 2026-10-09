#!/usr/bin/env python3
"""Replay structural mutations against the independent native checker.

These are checker controls only. They do not invoke Creusot/Why3 and are not
semantic proof controls. Each mutation refreshes its evidence receipt hash so
the relevant source/MIR rule, rather than stale-hash detection alone, must
reject it.
"""
from __future__ import annotations

import copy
import importlib.util
import json
import pathlib
import re
import sys
from typing import Any, Callable

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_native.py"
OUTPUT_PATH = ROOT / "generated/native-check-controls.json"


def load_checker():
    spec = importlib.util.spec_from_file_location("al_promotable_native_checker", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load check_native.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


CHECKER = load_checker()


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one mutation anchor, found {count}: {old!r}")
    return text.replace(old, new, 1)


def set_source(data: dict[str, Any], field: str, new_text: str, receipt_field: str) -> None:
    data[field] = new_text
    data["capture"][receipt_field] = CHECKER.sha(new_text.encode())


def set_mir(data: dict[str, Any], label: str, new_text: str) -> None:
    data["mir_sources"][label] = new_text
    row = next(row for row in data["capture"]["selected"] if row["label"] == label)
    row["sha256"] = CHECKER.sha(new_text.encode())


def rewrite_mir_block(mir: str, block: int, transform: Callable[[str], str]) -> str:
    pattern = re.compile(rf"(?m)(^    bb{block}: \{{\n)(.*?)(^    \}})", re.S)
    matches = list(pattern.finditer(mir))
    if len(matches) != 1:
        raise RuntimeError(f"bb{block}: expected one normal block, found {len(matches)}")
    match = matches[0]
    body = transform(match.group(2))
    return mir[:match.start()] + match.group(1) + body + match.group(3) + mir[match.end():]


def mutate_count_one(data: dict[str, Any]) -> None:
    text = data["production_source"]
    text = replace_once(text, "ref_cnt: AtomicUsize::new(2)",
                        "ref_cnt: AtomicUsize::new(1)", "count-one source")
    set_source(data, "production_source", text, "production_source_sha256")


def mutate_weak_cas(data: dict[str, Any]) -> None:
    text = data["production_source"]
    text = replace_once(text, "atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire)",
                        "atom.compare_exchange_weak(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire)",
                        "weak-CAS source")
    set_source(data, "production_source", text, "production_source_sha256")


def mutate_wrong_expected_source(data: dict[str, Any]) -> None:
    text = data["production_source"]
    text = replace_once(text, "atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire)",
                        "atom.compare_exchange(shared as _, shared as _, Ordering::AcqRel, Ordering::Acquire)",
                        "wrong expected source")
    set_source(data, "production_source", text, "production_source_sha256")


def mutate_missing_acquire_source(data: dict[str, Any]) -> None:
    text = data["production_source"]
    text = replace_once(text, "Ordering::AcqRel, Ordering::Acquire)",
                        "Ordering::AcqRel, Ordering::Relaxed)", "missing-Acquire source")
    set_source(data, "production_source", text, "production_source_sha256")


def mutate_extra_writer_source(data: dict[str, Any]) -> None:
    text = data["production_source"]
    text = replace_once(text, "    match atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire) {",
                        "    atom.store(shared as _, Ordering::Release);\n"
                        "    match atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire) {",
                        "extra-writer source")
    set_source(data, "production_source", text, "production_source_sha256")


def mutate_omit_child_cleanup(data: dict[str, Any]) -> None:
    text = data["native_source"]
    text = replace_once(text, "    child.cleanup();\n", "", "omitted child cleanup")
    set_source(data, "native_source", text, "native_source_sha256")


def mutate_swap_cleanup_read(data: dict[str, Any]) -> None:
    text = data["native_source"]
    old = "    child.cleanup();\n    let observed = AsRef::<[u8]>::as_ref(&original).to_vec();"
    new = "    let observed = AsRef::<[u8]>::as_ref(&original).to_vec();\n    child.cleanup();"
    text = replace_once(text, old, new, "swapped cleanup/read client")
    set_source(data, "native_source", text, "native_source_sha256")


def mutate_receipt_alias_route(data: dict[str, Any]) -> None:
    # Resolves to native.rs but violates the reviewed literal receipt route.
    data["capture"]["native_source"] = "native-test/../native.rs"


def mutate_manifest_alias_route(data: dict[str, Any]) -> None:
    text = data["native_manifest"]
    text = replace_once(text, 'path = "../native.rs"',
                        'path = "../native-test/../native.rs"', "native manifest alias route")
    set_source(data, "native_manifest", text, "native_manifest_sha256")


def mutate_dependency_route(data: dict[str, Any]) -> None:
    text = data["native_manifest"]
    text = replace_once(text, 'path = "../../../../"',
                        'path = "../../../"', "bytes dependency route")
    set_source(data, "native_manifest", text, "native_manifest_sha256")


def mutate_count_one_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = replace_once(text, "Atomic::<usize>::new(const 2_usize)",
                        "Atomic::<usize>::new(const 1_usize)", "count-one MIR")
    set_mir(data, "shallow_clone_vec", text)


def mutate_weak_cas_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = replace_once(text, "Atomic::<*mut ()>::compare_exchange(",
                        "Atomic::<*mut ()>::compare_exchange_weak(", "weak-CAS MIR")
    set_mir(data, "shallow_clone_vec", text)


def mutate_wrong_expected_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = replace_once(text, "_34 = copy _2;", "_34 = copy _18;", "wrong expected MIR")
    set_mir(data, "shallow_clone_vec", text)


def mutate_missing_acquire_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = replace_once(text, "_39 = core::sync::atomic::Ordering::Acquire;",
                        "_39 = core::sync::atomic::Ordering::Relaxed;", "missing-Acquire MIR")
    set_mir(data, "shallow_clone_vec", text)


def mutate_extra_writer_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    anchor = "_30 = Atomic::<*mut ()>::compare_exchange(move _31, move _32, move _35, move _38, move _39)"
    text = replace_once(text, anchor,
                        "_30 = Atomic::<*mut ()>::store(move _31, move _32, move _38);\n        " + anchor,
                        "extra-writer MIR")
    set_mir(data, "shallow_clone_vec", text)


def mutate_omit_cleanup_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["client"]
    line = "        _6 = bytes::Bytes::cleanup(move _7) -> [return: bb3, unwind: bb13];\n"
    text = replace_once(text, line, "", "omitted child cleanup MIR")
    set_mir(data, "client", text)


def mutate_swap_cleanup_read_mir(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["client"]
    clean = "        _6 = bytes::Bytes::cleanup(move _7) -> [return: bb3, unwind: bb13];"
    read = "        _10 = <bytes::Bytes as AsRef<[u8]>>::as_ref(move _11) -> [return: bb4, unwind: bb14];"
    text = replace_once(text, clean, "@@CHILD_CLEANUP@@", "swap MIR cleanup placeholder")
    text = replace_once(text, read, clean, "swap MIR read statement")
    text = replace_once(text, "@@CHILD_CLEANUP@@", read, "swap MIR cleanup finish")
    set_mir(data, "client", text)


def mutate_swap_winner_loser_assignments(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    def swap_bb15(body: str) -> str:
        return replace_once(body, "((_30 as Ok).0: *mut ())", "@@CAS_RESULT@@", "bb15 result placeholder")
    def swap_bb14(body: str) -> str:
        return replace_once(body, "((_30 as Err).0: *mut ())", "((_30 as Ok).0: *mut ())", "bb14 result variant")
    text = rewrite_mir_block(text, 15, swap_bb15)
    text = rewrite_mir_block(text, 14, swap_bb14)
    text = rewrite_mir_block(text, 15, lambda body: replace_once(
        body, "@@CAS_RESULT@@", "((_30 as Err).0: *mut ())", "bb15 result variant"))
    set_mir(data, "shallow_clone_vec", text)


def mutate_move_winner_record_to_loser(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    line = "        _0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 };\n"
    text = rewrite_mir_block(text, 22, lambda body: replace_once(body, line, "", "remove winner record"))
    text = rewrite_mir_block(text, 14, lambda body: line + body)
    set_mir(data, "shallow_clone_vec", text)


def mutate_move_loser_cleanup_to_winner(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    statements = (
        "        _63 = move (*_60);\n"
        "        _62 = core::mem::forget::<bytes::Shared>(move _63) -> [return: bb24, unwind: bb29];\n"
    )
    text = rewrite_mir_block(text, 23,
                             lambda body: replace_once(body, statements, "", "remove loser cleanup"))
    text = rewrite_mir_block(text, 15, lambda body: replace_once(
        body, "        _43 = const true;\n", "        _43 = const true;\n" + statements,
        "insert misplaced loser cleanup"))
    set_mir(data, "shallow_clone_vec", text)


def mutate_wrong_cas_block_target(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = rewrite_mir_block(text, 12, lambda body: replace_once(
        body, "switchInt(move _40) -> [0: bb15, 1: bb14, otherwise: bb13];",
        "switchInt(move _40) -> [0: bb16, 1: bb14, otherwise: bb13];",
        "wrong winner bb target"))
    set_mir(data, "shallow_clone_vec", text)


def mutate_wrong_loser_edge(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = rewrite_mir_block(text, 14, lambda body: replace_once(
        body, "[return: bb23, unwind: bb32]", "[return: bb24, unwind: bb32]",
        "wrong loser bb edge"))
    set_mir(data, "shallow_clone_vec", text)


def mutate_return_field_order(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = replace_once(text,
        "_0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 };",
        "_0 = bytes::Bytes { len: move _51, ptr: move _50, data: move _52, vtable: move _56 };",
        "winner return field order")
    set_mir(data, "shallow_clone_vec", text)


def mutate_wrong_selected_header(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["shallow_clone_vec"]
    text = replace_once(text, "fn shallow_clone_vec(", "fn shallow_clone_arc(",
                        "selected MIR definition header")
    set_mir(data, "shallow_clone_vec", text)


CONTROL_CASES: list[tuple[str, str, str, Callable[[dict[str, Any]], None]]] = [
    ("shared_count_one_source", "production-source", "audit_production_sources", mutate_count_one),
    ("weak_compare_exchange_source", "production-source", "audit_production_sources", mutate_weak_cas),
    ("wrong_compare_exchange_expected_source", "production-source", "audit_production_sources", mutate_wrong_expected_source),
    ("missing_acquire_source", "production-source", "audit_production_sources", mutate_missing_acquire_source),
    ("extra_pointer_writer_source", "production-source", "audit_production_sources", mutate_extra_writer_source),
    ("omitted_child_cleanup_source", "client-source", "audit_native_client", mutate_omit_child_cleanup),
    ("swapped_cleanup_read_source", "client-source", "audit_native_client", mutate_swap_cleanup_read),
    ("same_file_receipt_alias_route", "receipt-route", "audit_paths_and_receipt", mutate_receipt_alias_route),
    ("same_file_native_manifest_alias_route", "manifest-route", "audit_paths_and_receipt", mutate_manifest_alias_route),
    ("wrong_bytes_dependency_route", "manifest-route", "audit_paths_and_receipt", mutate_dependency_route),
    ("shared_count_one_mir", "native-mir", "audit_native_mir", mutate_count_one_mir),
    ("weak_compare_exchange_mir", "native-mir", "audit_native_mir", mutate_weak_cas_mir),
    ("wrong_compare_exchange_expected_mir", "native-mir", "audit_native_mir", mutate_wrong_expected_mir),
    ("missing_acquire_mir", "native-mir", "audit_native_mir", mutate_missing_acquire_mir),
    ("extra_pointer_writer_mir", "native-mir", "audit_native_mir", mutate_extra_writer_mir),
    ("omitted_child_cleanup_mir", "client-mir", "audit_native_client", mutate_omit_cleanup_mir),
    ("swapped_cleanup_read_mir", "client-mir", "audit_native_client", mutate_swap_cleanup_read_mir),
    ("swap_cas_winner_loser_assignments", "native-mir-branch", "audit_native_mir", mutate_swap_winner_loser_assignments),
    ("move_winner_shared_vtable_record_to_loser", "native-mir-branch", "audit_native_mir", mutate_move_winner_record_to_loser),
    ("move_loser_cleanup_into_winner", "native-mir-branch", "audit_native_mir", mutate_move_loser_cleanup_to_winner),
    ("wrong_cas_winner_block_target", "native-mir-branch", "audit_native_mir", mutate_wrong_cas_block_target),
    ("wrong_loser_cleanup_block_edge", "native-mir-branch", "audit_native_mir", mutate_wrong_loser_edge),
    ("wrong_winner_return_field_order", "native-mir-branch", "audit_native_mir", mutate_return_field_order),
    ("wrong_selected_mir_function_header", "native-mir-headers", "audit_native_mir", mutate_wrong_selected_header),
]

BRANCH_GLOBAL_MARKERS = {
    "swap_cas_winner_loser_assignments": [
        "_41 = copy ((_30 as Err).0: *mut ());",
        "_59 = copy ((_30 as Ok).0: *mut ());",
        "Box::<bytes::Shared>::from_raw(move _61)",
        "core::mem::forget::<bytes::Shared>(move _63)",
        "_0 = shallow_clone_arc(move _64, move _67, move _68)",
    ],
    "move_winner_shared_vtable_record_to_loser": [
        "_0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 }",
        "alloc302 (static: bytes::SHARED_VTABLE",
        "Box::<bytes::Shared>::from_raw(move _61)",
        "core::mem::forget::<bytes::Shared>(move _63)",
        "_0 = shallow_clone_arc(move _64, move _67, move _68)",
    ],
    "move_loser_cleanup_into_winner": [
        "_41 = copy ((_30 as Ok).0: *mut ());",
        "_59 = copy ((_30 as Err).0: *mut ());",
        "Box::<bytes::Shared>::from_raw(move _61)",
        "core::mem::forget::<bytes::Shared>(move _63)",
        "_0 = shallow_clone_arc(move _64, move _67, move _68)",
        "_0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 }",
    ],
    "wrong_cas_winner_block_target": [
        "_41 = copy ((_30 as Ok).0: *mut ());",
        "_59 = copy ((_30 as Err).0: *mut ());",
        "Box::<bytes::Shared>::from_raw(move _61)",
        "core::mem::forget::<bytes::Shared>(move _63)",
        "_0 = shallow_clone_arc(move _64, move _67, move _68)",
        "_0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 }",
    ],
}


def run() -> dict[str, Any]:
    baseline = CHECKER.audit_bundle(CHECKER.load_bundle())
    results = []
    for name, mutation_class, audit_name, mutate in CONTROL_CASES:
        data = copy.deepcopy(CHECKER.load_bundle())
        mutate(data)
        preserved_markers = []
        marker_errors = []
        for marker in BRANCH_GLOBAL_MARKERS.get(name, []):
            actual_count = CHECKER.token_count(CHECKER.AI.rust_tokens(data["mir_sources"]["shallow_clone_vec"]),
                                               CHECKER.AI.rust_tokens(marker))
            if actual_count == 1:
                preserved_markers.append(marker)
            else:
                marker_errors.append({"marker": marker, "count": actual_count})
        try:
            # Call the relevant semantic/source/MIR checker on the in-memory
            # mutation. The unmodified baseline separately passed the full
            # receipt-to-filesystem audit above.
            getattr(CHECKER, audit_name)(data)
        except (CHECKER.AuditError, OSError, ValueError, KeyError, TypeError) as exc:
            results.append({"name": name, "class": mutation_class,
                            "rejected": True, "reason": str(exc),
                            "global_markers_preserved": not marker_errors,
                            "preserved_markers": preserved_markers,
                            "marker_errors": marker_errors})
        else:
            results.append({"name": name, "class": mutation_class,
                            "rejected": False, "reason": "mutated bundle was accepted",
                            "global_markers_preserved": not marker_errors,
                            "preserved_markers": preserved_markers,
                            "marker_errors": marker_errors})
    failures = [item["name"] for item in results
                if not item["rejected"] or not item["global_markers_preserved"]]
    return {
        "status": "pass" if not failures else "fail",
        "baseline_status": baseline["status"],
        "control_count": len(results),
        "rejected_count": sum(1 for item in results if item["rejected"]),
        "controls": results,
        "solver_invoked": False,
        "control_scope": "structural checker mutations only; no semantic proof control or proof claim",
        "checker_scope": baseline["checker_scope"],
        "selected_mir_count_including_client": baseline["paths_and_capture"]["selected_mir_count"],
        "selected_production_mir_count": baseline["paths_and_capture"]["selected_production_mir_count"],
        "limitations": baseline["limitations"],
    }


def main() -> int:
    try:
        result = run()
    except (CHECKER.AuditError, OSError, ValueError, KeyError, TypeError, RuntimeError) as exc:
        print(f"native checker controls failed to run: {exc}", file=sys.stderr)
        return 2
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
