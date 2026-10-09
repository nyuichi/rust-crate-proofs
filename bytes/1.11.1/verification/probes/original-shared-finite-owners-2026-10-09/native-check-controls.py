#!/usr/bin/env python3
"""Replay AP native source/MIR mutations; this suite does not run Cargo or Why3."""
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
FIXTURE_PATH = ROOT / "fixtures/native-check-controls.json"


def load_checker():
    spec = importlib.util.spec_from_file_location("ap_finite_owners_native_checker", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load AP check_native.py")
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


def src(field: str, receipt: str, old: str, new: str, label: str) -> Callable[[dict[str, Any]], None]:
    def mutate(data: dict[str, Any]) -> None:
        set_source(data, field, replace_once(data[field], old, new, label), receipt)
    return mutate


def mir(*args: str) -> Callable[[dict[str, Any]], None]:
    if len(args) == 3:
        label, old, new, why = "client", args[0], args[1], args[2]
    elif len(args) == 4:
        label, old, new, why = args
    else:
        raise TypeError("mir mutation expects (old,new,why) or (label,old,new,why)")
    def mutate(data: dict[str, Any]) -> None:
        set_mir(data, label, replace_once(data["mir_sources"][label], old, new, why))
    return mutate


def source_block(field: str, receipt: str, old: str, new: str, label: str):
    return src(field, receipt, old, new, label)


def mutate_mir_header(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["client"]
    set_mir(data, "client", replace_once(text, "fn finite_shared_scope(", "fn another_scope(", "client header"))


def mutate_receipt_source_alias(data: dict[str, Any]) -> None:
    data["capture"]["native_source"] = "native-test/../native.rs"


def mutate_receipt_mir_alias(data: dict[str, Any]) -> None:
    data["capture"]["selected"][0]["path"] = data["capture"]["selected"][0]["path"].replace(
        "native-mir/", "native-mir/../native-mir/", 1)


def mutate_manifest_alias(data: dict[str, Any]) -> None:
    text = data["native_manifest"]
    set_source(data, "native_manifest", replace_once(text, 'path = "../native.rs"',
        'path = "../native-test/../native.rs"', "manifest alias"), "native_manifest_sha256")


def mutate_dependency_path(data: dict[str, Any]) -> None:
    text = data["native_manifest"]
    set_source(data, "native_manifest", replace_once(text, 'path = "../../../../"',
        'path = "../../../"', "bytes dependency"), "native_manifest_sha256")


def mutate_capture_command(data: dict[str, Any]) -> None:
    text = data["capture_script"]
    old = '-Zdump-mir=all -Zdump-mir-dir="$scratch_dir/client" -Zmir-opt-level=0'
    new = '-Zdump-mir=all -Zdump-mir-dir="$scratch_dir/client" -Zmir-opt-level=1'
    set_source(data, "capture_script", replace_once(text, old, new, "optimized MIR capture"),
               "capture_script_sha256")


def mutate_receipt_selected_set(data: dict[str, Any]) -> None:
    data["capture"]["selected"].pop()


def mutate_reviewed_source_manifest(data: dict[str, Any]) -> None:
    data["reviewed_production_manifest"] = data["reviewed_production_manifest"].replace(
        '"base_commit": "361c7cd261507ac0a705b3b836f73240070891c6"',
        '"base_commit": "0000000000000000000000000000000000000000"', 1)


def mutate_reviewed_input(data: dict[str, Any]) -> None:
    data["production_source_inputs"]["src/bytes.rs"] += "\n// changed source input\n"


def mutate_client_block_extra_call(data: dict[str, Any]) -> None:
    text = data["mir_sources"]["client"]
    set_mir(data, "client", replace_once(text,
        "        _10 = const ();\n        drop(_26)",
        "        _10 = const ();\n        core::mem::forget::<bytes::Bytes>(move _26) -> [return: bb15, unwind: bb24];\n        drop(_26)",
        "unreviewed peer effect"))


def mutate_client_block_statement(data: dict[str, Any], block: int, old: str, new: str, label: str) -> None:
    text = data["mir_sources"]["client"]
    pattern = re.compile(rf"(?m)(^    bb{block}: \{{\n)(.*?)(^    \}})", re.S)
    matches = list(pattern.finditer(text))
    if len(matches) != 1:
        raise RuntimeError(f"bb{block}: expected one client block, found {len(matches)}")
    match = matches[0]
    body = replace_once(match.group(2), old, new, label)
    rewritten = text[:match.start()] + match.group(1) + body + match.group(3) + text[match.end():]
    set_mir(data, "client", rewritten)


def mutate_drain_backedge(data: dict[str, Any]) -> None:
    mutate_client_block_statement(data, 16, "goto -> bb11;", "goto -> bb17;", "drain backedge")


def mutate_container_drop_before_read(data: dict[str, Any]) -> None:
    mutate_client_block_statement(data, 10, "goto -> bb11;", "goto -> bb20;", "skip drain before read")


CONTROL_CASES: list[tuple[str, str, str, Callable[[dict[str, Any]], None]]] = [
    # Native program changes: the exact source grammar must keep the runtime
    # count, Vec ownership, both loops, and lexical peer lifetime intact.
    ("native_count_condition_changed", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "while made < count", "while made <= count", "count condition")),
    ("native_count_bounded_to_sample", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "while made < count", "while made < count.min(31)", "bounded count")),
    ("native_creation_loop_removed", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "    while made < count {\n        owners.push(survivor.clone());\n        made += 1;\n    }\n", "", "creation loop")),
    ("native_clone_wrong_source", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "owners.push(survivor.clone());", "owners.push(original.clone());", "clone source")),
    ("native_push_omitted", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "        owners.push(survivor.clone());\n", "", "push omitted")),
    ("native_push_duplicate", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "        owners.push(survivor.clone());\n", "        owners.push(survivor.clone());\n        owners.push(survivor.clone());\n", "duplicate push")),
    ("native_drain_loop_omitted", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "    while let Some(peer) = owners.pop() {\n        // peer's lexical normal Drop retires its actual ticket.\n    }\n", "", "drain omitted")),
    ("native_peer_explicitly_dropped", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "        // peer's lexical normal Drop retires its actual ticket.\n", "        drop(peer);\n", "explicit peer drop")),
    ("native_peer_forgotten", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "        // peer's lexical normal Drop retires its actual ticket.\n", "        core::mem::forget(peer);\n", "peer forget")),
    ("native_peer_escaped", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "        // peer's lexical normal Drop retires its actual ticket.\n", "        let _escaped = &peer as *const Bytes;\n", "peer escape")),
    ("native_drain_after_read", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "    while let Some(peer) = owners.pop() {\n        // peer's lexical normal Drop retires its actual ticket.\n    }\n    let observed =", "    let observed =", "late drain")),
    ("native_clear_container", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "    while let Some(peer) = owners.pop() {", "    owners.clear();\n    while let Some(peer) = owners.pop() {", "clear container")),
    ("native_leak_container", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "    let observed = AsRef", "    core::mem::forget(owners);\n    let observed = AsRef", "container leak")),
    ("native_alternate_container", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "let mut owners = Vec::new();", "let mut owners = Vec::with_capacity(count);", "alternate Vec constructor")),
    ("native_pop_discarded_peer", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "while let Some(peer) = owners.pop()", "while owners.pop().is_some()", "discard popped peer")),
    ("native_owner_loop_capped_by_fixed_constant", "client-source", "audit_native_client",
     src("native_source", "native_source_sha256", "owners.push(survivor.clone());", "owners.push(survivor.clone());\n        if made == 7 { break; }", "fixed quota")),

    # Exact MIR graph controls: refreshed MIR receipt hashes prevent a stale
    # receipt from serving as the only rejection reason.
    ("mir_wrong_create_loop_condition", "client-mir", "audit_native_client",
     mir("client", "_11 = Lt(move _12, move _13);", "_11 = Le(move _12, move _13);", "create condition")),
    ("mir_create_true_edge_bypass_clone", "client-mir", "audit_native_client",
     mir("client", "switchInt(move _11) -> [0: bb10, otherwise: bb6];", "switchInt(move _11) -> [0: bb10, otherwise: bb7];", "create true edge")),
    ("mir_create_backedge_skipped", "client-mir", "audit_native_client",
     mir("client", "StorageDead(_11);\n        goto -> bb5;", "StorageDead(_11);\n        goto -> bb10;", "create loop backedge")),
    ("mir_count_increment_removed", "client-mir", "audit_native_client",
     mir("_18 = AddWithOverflow(copy _8, const 1_usize);", "_18 = copy _8;", "count increment")),
    ("mir_clone_from_wrong_local", "client-mir", "audit_native_client",
     mir("_17 = &'_ _3;", "_17 = &'_ _4;", "clone receiver local")),
    ("mir_push_wrong_container", "client-mir", "audit_native_client",
     mir("_15 = &'_ mut _7;", "_15 = &'_ mut _3;", "push receiver local")),
    ("mir_push_wrong_payload", "client-mir", "audit_native_client",
     mir("Vec::<bytes::Bytes>::push(move _15, move _16)", "Vec::<bytes::Bytes>::push(move _15, move _3)", "push payload")),
    ("mir_pop_wrong_container", "client-mir", "audit_native_client",
     mir("_24 = &'_ mut _7;", "_24 = &'_ mut _3;", "pop receiver local")),
    ("mir_pop_some_none_edges_swapped", "client-mir", "audit_native_client",
     mir("switchInt(move _25) -> [1: bb13, otherwise: bb14];", "switchInt(move _25) -> [1: bb14, otherwise: bb13];", "pop discriminant branch")),
    ("mir_some_payload_not_moved", "client-mir", "audit_native_client",
     mir("_26 = move ((_23 as Some).0: bytes::Bytes);", "_26 = copy ((_23 as Some).0: bytes::Bytes);", "Some payload transfer")),
    ("mir_some_payload_dropped_wrong_place", "client-mir", "audit_native_client",
     mir("drop(_26) -> [return: bb15, unwind: bb24];", "drop(_3) -> [return: bb15, unwind: bb24];", "peer Drop target")),
    ("mir_peer_drop_omitted", "client-mir", "audit_native_client",
     mir("drop(_26) -> [return: bb15, unwind: bb24];", "StorageDead(_26);", "peer Drop removed")),
    ("mir_duplicate_peer_drop", "client-mir", "audit_native_client",
     mir("_10 = const ();\n        drop(_26)", "_10 = const ();\n        drop(_26) -> [return: bb15, unwind: bb24];\n        drop(_26)", "duplicate peer Drop")),
    ("mir_peer_drop_wrong_successor", "client-mir", "audit_native_client",
     mir("drop(_26) -> [return: bb15, unwind: bb24];", "drop(_26) -> [return: bb16, unwind: bb24];", "peer Drop successor")),
    ("mir_drain_backedge_skipped", "client-mir", "audit_native_client",
     mutate_drain_backedge),
    ("mir_none_edge_reenters_drain", "client-mir", "audit_native_client",
     mir("goto -> bb38;", "goto -> bb11;", "None path")),
    ("mir_some_branch_bypasses_move_and_drop", "client-mir", "audit_native_client",
     mir("switchInt(move _25) -> [1: bb13, otherwise: bb14];", "switchInt(move _25) -> [1: bb17, otherwise: bb14];", "Some branch")),
    ("mir_pop_result_bypasses_discriminant", "client-mir", "audit_native_client",
     mir("_25 = discriminant(_23);", "_25 = const 0_isize;", "pop result discriminant")),
    ("mir_container_drop_before_read", "client-mir", "audit_native_client",
     mutate_container_drop_before_read),
    ("mir_container_drop_removed", "client-mir", "audit_native_client",
     mir("drop(_7) -> [return: bb21, unwind: bb27];", "StorageDead(_7);", "Vec Drop")),
    ("mir_survivor_drop_removed", "client-mir", "audit_native_client",
     mir("drop(_3) -> [return: bb22, unwind: bb30];", "StorageDead(_3);", "survivor Drop")),
    ("mir_vec_survivor_drop_order_swapped", "client-mir", "audit_native_client",
     mir("drop(_7) -> [return: bb21, unwind: bb27];", "drop(_3) -> [return: bb21, unwind: bb27];", "Vec/survivor order")),
    ("mir_survivor_drop_before_return_eval", "client-mir", "audit_native_client",
     mir("_0 = move _30;", "drop(_3);\n        _0 = move _30;", "saved return ordering")),
    ("mir_return_bypasses_native_drops", "client-mir", "audit_native_client",
     mir("_0 = move _30;\n        goto -> bb20;", "_0 = move _30;\n        goto -> bb23;", "return bypasses Drops")),
    ("mir_extra_owner_callback", "client-mir", "audit_native_client",
     mutate_client_block_extra_call),
    ("mir_fake_header", "native-mir-headers", "audit_native_mir",
     mutate_mir_header),

    # Shared-vtable callback source/MIR and production clone binding.
    ("source_shared_clone_acquire_load", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "data.load(Ordering::Relaxed);\n    shallow_clone_arc", "data.load(Ordering::Acquire);\n    shallow_clone_arc", "Shared clone ordering")),
    ("source_shared_clone_uses_promotable_callback", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "let shared = data.load(Ordering::Relaxed);\n    shallow_clone_arc(shared as _, ptr, len)", "let shared = data.load(Ordering::Relaxed);\n    promotable_even_clone(data, ptr, len)", "Shared clone helper")),
    ("source_shared_vtable_clone_redirected", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "clone: shared_clone,", "clone: promotable_even_clone,", "Shared vtable clone route")),
    ("source_shared_vtable_drop_redirected", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "drop: shared_drop,", "drop: promotable_even_drop,", "Shared vtable drop route")),
    ("mir_shared_clone_missing_relaxed_load", "native-mir", "audit_native_mir",
     mir("shared_clone", "_6 = core::sync::atomic::Ordering::Relaxed;", "_6 = core::sync::atomic::Ordering::Acquire;", "Shared clone load order")),
    ("mir_shared_clone_redirects_to_vec_helper", "native-mir", "audit_native_mir",
     mir("shared_clone", "_0 = shallow_clone_arc(move _7, move _10, move _11)", "_0 = shallow_clone_vec(move _7, move _10, move _11)", "Shared clone helper target")),
    ("mir_shared_clone_pointer_cas_added", "native-mir", "audit_native_mir",
     mir("shared_clone", "_0 = shallow_clone_arc(move _7, move _10, move _11)", "_8 = Atomic::<*mut ()>::compare_exchange(move _5, move _4, const 0_usize, const 1_usize, const AcqRel, const Acquire);\n        _0 = shallow_clone_arc(move _7, move _10, move _11)", "Shared callback CAS")),
    ("mir_shared_clone_wrong_entrypoint", "native-mir", "audit_native_mir",
     mir("shared_clone", "fn shared_clone(", "fn promotable_even_clone(", "Shared clone header")),
    ("source_clone_impl_not_dispatching_vtable", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "unsafe { (self.vtable.clone)(&self.data, self.ptr, self.len) }", "unsafe { shared_clone(&self.data, self.ptr, self.len) }", "Bytes clone dispatcher")),

    # Guarded refcount/free and callback plumbing remain the production source
    # and MIR that make repeated Shared clones and final normal Drops meaningful.
    ("source_shared_count_initialization_wrong", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "ref_cnt: AtomicUsize::new(2)", "ref_cnt: AtomicUsize::new(1)", "Shared refcount init")),
    ("source_shared_cas_made_weak", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire)", "atom.compare_exchange_weak(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire)", "first promotion CAS")),
    ("source_shared_cas_expected_changed", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire)", "atom.compare_exchange(shared as _, shared as _, Ordering::AcqRel, Ordering::Acquire)", "first promotion expected")),
    ("source_increment_ordering_changed", "refcount-source", "audit_production_sources",
     src("ref_count_source", "ref_count_source_sha256", "Ordering::Relaxed,\n        Ordering::Relaxed,", "Ordering::Acquire,\n        Ordering::Relaxed,", "guarded increment ordering")),
    ("source_release_ordering_changed", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "if (*ptr).ref_cnt.fetch_sub(1, Ordering::Release) != 1 {", "if (*ptr).ref_cnt.fetch_sub(1, Ordering::Relaxed) != 1 {", "Shared release ordering")),
    ("source_release_acquire_fence_removed", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "(*ptr).ref_cnt.load(Ordering::Acquire);", "(*ptr).ref_cnt.load(Ordering::Relaxed);", "last-release Acquire")),
    ("source_buffer_free_removed", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "dealloc(buf, Layout::from_size_align(cap, 1).unwrap());", "", "payload free")),
    ("source_control_free_removed", "production-source", "audit_production_sources",
     src("production_source", "production_source_sha256", "dealloc(ptr.cast(), Layout::new::<Shared>());", "", "control free")),
    ("mir_refcount_increment_call_removed", "native-mir", "audit_native_mir",
     mir("ref_count_increment", "_4 = try_increment(move _5)", "_4 = const ()", "guarded increment MIR")),
    ("mir_last_release_acquire_removed", "native-mir", "audit_native_mir",
     mir("release_shared", "_10 = core::sync::atomic::Ordering::Acquire;", "_10 = core::sync::atomic::Ordering::Relaxed;", "release Acquire MIR")),
    ("mir_free_shared_deallocation_removed", "native-mir", "audit_native_mir",
     mir("free_shared", "_4 = std::alloc::dealloc(move _5, move _6) -> [return: bb3, unwind continue];", "StorageDead(_5);", "free Shared MIR")),
    ("source_atomicmut_callback_changed", "refcount-source", "audit_production_sources",
     src("loom_source", "loom_source_sha256", "f(self.get_mut())", "f(self.get_mut().wrapping_add(0))", "AtomicMut callback")),

    # Global input/profile/path controls: refreshed artifact metadata cannot
    # substitute for checking the captured 63-source and default field set.
    ("capture_source_route_alias", "receipt-route", "audit_paths_and_receipt", mutate_receipt_source_alias),
    ("capture_mir_route_alias", "receipt-route", "audit_paths_and_receipt", mutate_receipt_mir_alias),
    ("native_manifest_alias", "manifest-route", "audit_paths_and_receipt", mutate_manifest_alias),
    ("native_dependency_route_changed", "manifest-route", "audit_paths_and_receipt", mutate_dependency_path),
    ("mir_capture_optimization_changed", "capture-script", "audit_paths_and_receipt", mutate_capture_command),
    ("mir_selected_set_incomplete", "receipt", "audit_paths_and_receipt", mutate_receipt_selected_set),
    ("reviewed_production_ancestry_changed", "production-inputs", "audit_reviewed_production_inputs", mutate_reviewed_source_manifest),
    ("reviewed_bytes_source_changed", "production-inputs", "audit_reviewed_production_inputs", mutate_reviewed_input),
    ("bytes_record_field_added", "terminal-field-profile", "audit_terminal_field_profile",
     lambda d: d.__setitem__("bytes_record_source", d["bytes_record_source"].replace("    len: usize,", "    len: usize,\n    unreviewed: String,", 1))),
    ("native_field_profile_execution_changed", "terminal-field-profile", "audit_terminal_field_profile",
     lambda d: (d.__setitem__("native_field_profile_log", "independent drop glue present\n"),
                d["capture"].__setitem__("native_field_profile_log_sha256", CHECKER.sha(b"independent drop glue present\n")))),
    ("production_record_include_redirect", "production-global-binding", "audit_reviewed_production_inputs",
     src("production_source", "production_source_sha256", 'include!("bytes/bytes_record.rs")', 'include!("bytes/unreviewed_record.rs")', "record include")),
    ("production_atomic_import_redirect", "production-global-binding", "audit_reviewed_production_inputs",
     src("production_source", "production_source_sha256", "use crate::loom::sync::atomic::{", "use crate::unreviewed_atomic::{", "atomic import")),
    ("native_manifest_package_changed", "manifest-route", "audit_paths_and_receipt",
     src("native_manifest", "native_manifest_sha256", "bytes-shared-finite-owners-native", "another-package", "probe package")),
]


def run() -> dict[str, Any]:
    baseline = CHECKER.audit_bundle(CHECKER.load_bundle())
    fixture = json.loads(FIXTURE_PATH.read_text())
    expected = fixture.get("controls")
    actual = [{"id": name, "class": mutation_class, "audit": audit_name,
               "expected": "rejected"}
              for name, mutation_class, audit_name, _mutate in CONTROL_CASES]
    if expected != actual or fixture.get("control_count") != len(actual):
        raise RuntimeError("AP native checker control fixture does not exactly match the runner cases")
    results = []
    for name, mutation_class, audit_name, mutate in CONTROL_CASES:
        data = copy.deepcopy(CHECKER.load_bundle())
        mutate(data)
        try:
            getattr(CHECKER, audit_name)(data)
        except (CHECKER.AuditError, OSError, ValueError, KeyError, TypeError, RuntimeError) as exc:
            results.append({"name": name, "class": mutation_class,
                            "rejected": True, "reason": str(exc)})
        else:
            results.append({"name": name, "class": mutation_class,
                            "rejected": False, "reason": "mutated bundle was accepted"})
    failures = [item["name"] for item in results if not item["rejected"]]
    return {
        "status": "pass" if not failures else "fail",
        "baseline_status": baseline["status"],
        "control_count": len(results),
        "rejected_count": sum(1 for item in results if item["rejected"]),
        "controls": results,
        "cargo_or_rust_build_invoked": False,
        "solver_invoked": False,
        "control_scope": "native source/MIR/config/profile structural mutations only; no proof or semantic theorem claim",
        "checker_scope": baseline["checker_scope"],
        "selected_mir_count_including_client": baseline["paths_and_capture"]["selected_mir_count"],
        "selected_production_mir_count": baseline["paths_and_capture"]["selected_production_mir_count"],
        "limitations": baseline["limitations"],
    }


def main() -> int:
    try:
        result = run()
    except (CHECKER.AuditError, OSError, ValueError, KeyError, TypeError, RuntimeError) as exc:
        print(f"AP native checker controls failed to run: {exc}", file=sys.stderr)
        return 2
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
