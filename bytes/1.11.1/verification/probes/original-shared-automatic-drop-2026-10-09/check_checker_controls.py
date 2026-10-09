#!/usr/bin/env python3
"""No-solver adversarial mutations for the automatic-Drop source checker."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import pathlib
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
FIXTURES = ROOT / "fixtures" / "checker-controls.json"
CHECKER_PATH = ROOT / "check_correspondence.py"


def replace_once(source: str, old: str, new: str) -> str:
    if source.count(old) != 1:
        raise RuntimeError(f"mutation anchor must occur exactly once: {old!r}")
    return source.replace(old, new, 1)


def load_checker():
    spec = importlib.util.spec_from_file_location("automatic_drop_correspondence", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load automatic-Drop correspondence checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    module.AI = module.load_ai_checker()
    return module


def source_mutations(native: str) -> dict[str, str]:
    return {
        "native_explicit_drop_call": replace_once(
            native, "    observed\n}", "    core::mem::drop(second);\n    observed\n}"),
        "native_extra_clone": replace_once(
            native, "    let observed = AsRef::<[u8]>::as_ref(&second).to_vec();",
            "    let third = second.clone();\n    let observed = AsRef::<[u8]>::as_ref(&second).to_vec();"),
        "native_wrong_drop_order_source": replace_once(
            native, "    let second = first.clone();", "    let first = first.clone();"),
    }


def shadow_mutations(active: str) -> dict[str, str]:
    changed: dict[str, str] = {}
    caller_marker = "/// Saved return value followed by certified native terminal normal Drop edges."
    if active.count(caller_marker) != 1:
        raise RuntimeError("positive shadow must contain one terminal caller marker")
    split = active.index(caller_marker)
    prefix, client = active[:split], active[split:]
    second = "    bytes_terminal_drop(second,cursor.borrow_mut(),second_receipt.borrow_mut());\n"
    first = "    bytes_terminal_drop(first,cursor.borrow_mut(),first_receipt.borrow_mut());\n"
    second_kept = "    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());\n"
    changed["omit_second_terminal_drop"] = prefix + replace_once(client, second, "")
    changed["omit_first_terminal_drop"] = prefix + replace_once(client, first, "")
    changed["duplicate_second_terminal_drop"] = prefix + replace_once(client, second, second + second)
    changed["swap_terminal_drop_places"] = prefix + replace_once(
        replace_once(client, second, "    bytes_terminal_drop(first,cursor.borrow_mut(),second_receipt.borrow_mut());\n"),
        first, "    bytes_terminal_drop(second,cursor.borrow_mut(),first_receipt.borrow_mut());\n")
    changed["wrong_terminal_drop_place"] = prefix + replace_once(
        client, second, "    bytes_terminal_drop(first,cursor.borrow_mut(),second_receipt.borrow_mut());\n")
    # Move the existing completion write before the last borrow read. This is
    # a source-only structural control; exact caller tokens must reject it.
    second_effect = "    let mut second_receipt=ghost! {None::<Completion>};\n" + second + second_kept
    changed["early_terminal_drop_before_last_read"] = prefix + replace_once(
        replace_once(client, second_effect, ""),
        "    let observed=borrowed.to_vec();\n",
        second_effect + "    let observed=borrowed.to_vec();\n")
    changed["return_saved_after_drop"] = prefix + replace_once(
        replace_once(client, "    let saved_return=observed;\n", ""),
        first, first + "    let saved_return=observed;\n")
    changed["wrong_terminal_cursor"] = prefix + replace_once(
        client, second, "    bytes_terminal_drop(second,wrong_cursor.borrow_mut(),second_receipt.borrow_mut());\n")
    changed["wrong_receipt_output"] = prefix + replace_once(
        client, second, "    bytes_terminal_drop(second,cursor.borrow_mut(),first_receipt.borrow_mut());\n")
    changed["terminal_helper_wrong_callback"] = replace_once(
        prefix, "original_shared_cleanup(value,cursor,output)",
        "unknown_native_callback(value,cursor,output)")
    changed["terminal_helper_observes_bytes_address"] = replace_once(
        prefix, "    original_shared_cleanup(value,cursor,output)",
        "    observe_address(&value as *const Bytes);\n"
        "    original_shared_cleanup(value,cursor,output)")
    changed["terminal_helper_escapes_data_address"] = replace_once(
        prefix, "    original_shared_cleanup(value,cursor,output)",
        "    let address=core::ptr::addr_of!(value.data);\n"
        "    unknown_sink(address);\n"
        "    original_shared_cleanup(value,cursor,output)")
    changed["terminal_helper_false_trusted_summary"] = replace_once(
        prefix, "fn bytes_terminal_drop(",
        "#[trusted]\n#[ensures(false)]\nfn bytes_terminal_drop(")
    changed["terminal_shadow_hidden_drop_impl"] = (
        active + "\nimpl Drop for Bytes { fn drop(&mut self) { unknown_native_drop(self) } }\n")
    changed["terminal_shadow_hidden_native_call"] = prefix + replace_once(
        client, "    let second=original_shared_clone(&first,cursor.borrow_mut());",
        "    let second=original_shared_clone(&first,cursor.borrow_mut());\n"
        "    unknown_native_callback(&second);")
    changed["terminal_shadow_false_precondition"] = prefix + replace_once(
        client, "#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]",
        "#[requires(false)]")
    changed["terminal_shadow_move_or_dropflag"] = prefix + replace_once(
        client, "    let saved_return=observed;",
        "    let saved_return=observed;\n    if true { drop(first); }")
    return changed


def mir_mutations(sources: dict[str, str], names: dict[str, str]) -> dict[str, dict[str, str]]:
    scope = names["scope"]
    bytes_drop = names["bytes_drop"]
    closure = names["shared_drop_closure"]
    changed: dict[str, dict[str, str]] = {}

    def with_change(key: str, filename: str, old: str, new: str) -> None:
        updated = dict(sources)
        updated[filename] = replace_once(updated[filename], old, new)
        changed[key] = updated

    with_change("mir_changed_terminal_edge", scope,
                "drop(_4) -> [return: bb6, unwind: bb10];",
                "drop(_4) -> [return: bb7, unwind: bb10];")
    with_change("mir_omitted_terminal_drop", scope,
                "StorageDead(_6);\n        drop(_4) -> [return: bb6, unwind: bb10];",
                "StorageDead(_6);\n        goto -> bb6;")
    with_change("mir_swapped_terminal_drop_places", scope,
                "drop(_4) -> [return: bb6, unwind: bb10];",
                "drop(_2) -> [return: bb6, unwind: bb10];")
    with_change("mir_changed_drop_unwind_successor", scope,
                "drop(_2) -> [return: bb7, unwind: bb12];",
                "drop(_2) -> [return: bb7, unwind: bb10];")
    with_change("mir_added_branch_dropflag", scope,
                "_0 = move _6;\n        goto -> bb5;",
                "_0 = move _6;\n        switchInt(const true) -> [0: bb5, otherwise: bb6];")
    with_change("mir_wrong_bytes_vtable_slot", bytes_drop,
                "(*_7).4: for<'a> unsafe fn", "(*_7).3: for<'a> unsafe fn")
    with_change("mir_wrong_bytes_drop_argument", bytes_drop,
                "_5 = copy ((*_1).0: *const u8);", "_5 = copy ((*_1).1: usize);")
    with_change("mir_shared_closure_wrong_target", closure,
                "bytes::release_shared(move _4)", "bytes::free_shared(move _4)")
    with_change("mir_shared_closure_observes_data_address", closure,
                "_5 = copy (*_2);", "_5 = copy _2;")
    return changed


def production_mutations(ai: Any, bytes_source: str, refcount: str,
                         shared_record: str, bytes_record: str, vtable_record: str,
                         loom: str) -> dict[str, tuple[str, str, str, str, str, str]]:
    changed: dict[str, tuple[str, str, str, str, str, str]] = {}

    def row(k: str, bs: str = bytes_source, sr: str = shared_record,
            br: str = bytes_record, vr: str = vtable_record, ls: str = loom) -> None:
        changed[k] = (bs, refcount, sr, br, vr, ls)

    row("production_bytes_drop_wrong_vtable_field", replace_once(
        bytes_source, "(self.vtable.drop)(&mut self.data, self.ptr, self.len)",
        "(self.vtable.clone)(&self.data, self.ptr, self.len)"))
    row("production_bytes_drop_observes_self_address", replace_once(
        bytes_source, "    fn drop(&mut self) {\n        unsafe { (self.vtable.drop)",
        "    fn drop(&mut self) {\n        let _address=self as *mut Bytes;\n"
        "        unsafe { (self.vtable.drop)"))
    row("production_vtable_field_order_changed", vr=replace_once(
        vtable_record,
        "pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),",
        "pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),\n    pub hidden: unsafe fn(),"))
    row("production_native_vtable_drop_target_changed", bs=replace_once(
        bytes_source, "drop: shared_drop,", "drop: shared_clone,"))
    row("production_bytes_added_field_glue",
        br=replace_once(bytes_record, "    vtable: &'static Vtable,",
                        "    vtable: &'static Vtable,\n    hidden: alloc::boxed::Box<u8>,"))
    row("production_shared_added_field_glue",
        sr=replace_once(shared_record, "    pub(crate) ref_cnt: AtomicUsize,",
                        "    pub(crate) ref_cnt: AtomicUsize,\n    hidden: alloc::vec::Vec<u8>,"))
    row("production_loom_callback_sees_atomic_address", loom,
        ls=replace_once(loom, "f(self.get_mut())", "f(&mut *self.get_mut())"))
    row("production_shared_callback_extra_effect", replace_once(
        bytes_source,
        "unsafe fn shared_drop(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {\n"
        "    data.with_mut(|shared| {\n        release_shared(shared.cast());",
        "unsafe fn shared_drop(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {\n"
        "    data.with_mut(|shared| {\n        release_shared(shared.cast());\n"
        "        release_shared(shared.cast());"))
    return changed


def mapping_mutations(mapping: dict[str, Any]) -> dict[str, dict[str, Any]]:
    changed: dict[str, dict[str, Any]] = {}
    for name, mutate in (
        ("mapping_wrong_terminal_place", lambda m: m["normal_edges"][0].__setitem__("place", "_2")),
        ("mapping_wrong_unwind_successor", lambda m: m["normal_edges"][1].__setitem__("unwind", "bb10")),
        ("mapping_stale_mir_hash", lambda m: m["mir"][0].__setitem__("sha256", "0" * 64)),
        ("mapping_omitted_tcb_premise", lambda m: m["tcb"].remove("generic terminal-place/non-address-observing correspondence")),
    ):
        result = json.loads(json.dumps(mapping))
        mutate(result)
        changed[name] = result
    return changed


def provenance_mutations(*, rustc_text: str, cargo_text: str, manifest_text: str,
                         lock_text: str, harness_source: str, module_source: str,
                         native_run_log: str,
                         support_paths: dict[str, str]) -> dict[str, dict[str, str]]:
    positive = {
        "rustc_text": rustc_text,
        "cargo_text": cargo_text,
        "manifest_text": manifest_text,
        "lock_text": lock_text,
        "harness_source": harness_source,
        "module_source": module_source,
        "native_run_log": native_run_log,
    }
    changed: dict[str, dict[str, str]] = {}

    def row(name: str, key: str, old: str, new: str) -> None:
        item = dict(positive)
        item[key] = replace_once(item[key], old, new)
        changed[name] = item

    row("provenance_changed_rustc_toolchain", "rustc_text",
        "91fe22da8084a1c9e993d78d4a56f22ab8396236", "0000000000000000000000000000000000000000")
    row("provenance_changed_cargo_toolchain", "cargo_text",
        "cargo 1.98.0-nightly (a595d0da2 2026-06-20)",
        "cargo 1.97.0-nightly (000000000 2026-01-01)")
    row("provenance_native_manifest_wrong_target", "manifest_text",
        "native.rs", "alternate.rs")
    row("provenance_native_manifest_added_feature", "manifest_text",
        "[dependencies]", "[features]\nloom = []\n[dependencies]")
    row("provenance_native_lock_changed_std_version", "lock_text",
        'name = "creusot-std"\nversion = "0.13.0"',
        'name = "creusot-std"\nversion = "0.14.0"')
    row("provenance_native_lock_changed_bytes_version", "lock_text",
        'name = "bytes"\nversion = "1.11.1"',
        'name = "bytes"\nversion = "1.11.2"')
    row("provenance_native_lock_changed_registry_checksum", "lock_text",
        "c9530985d4aff8273851f018c36422944f70717d7aa9e0c724c3deab004bd6db",
        "0" * 64)
    row("provenance_harness_wrong_atomic_profile", "harness_source",
        "AtomicPtr<()>", "AtomicUsize")
    row("provenance_module_wrong_active_route", "module_source",
        "../generated/active.rs", "../generated/positive.rs")
    row("provenance_module_added_route_attribute", "module_source",
        '#[cfg(creusot)] #[path="../generated/active.rs"] mod public_shared;',
        '#[cfg_attr(creusot, path="../generated/other.rs")]\n'
        '#[cfg(creusot)] #[path="../generated/active.rs"] mod public_shared;')
    row("provenance_native_log_missing_execution", "native_run_log",
        "actual Bytes automatic normal Drop: 5 inputs passed",
        "actual Bytes automatic normal Drop: 4 inputs passed")
    for module_name, path in support_paths.items():
        row(f"provenance_support_path_{module_name}", "module_source",
            path, path + ".unreviewed")
    return changed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path,
                        default=ROOT / "fixtures" / "checker-control-results.json")
    args = parser.parse_args()
    checker = load_checker()
    spec = json.loads(FIXTURES.read_text())
    native = checker.NATIVE_SOURCE.read_text()
    mir_sources = {name: (checker.MIR_DIR / name).read_text()
                   for name in checker.MIR_FILENAMES.values()}
    active = checker.ACTIVE_SHADOW.read_text()
    mapping_path = checker.MAP_DEFAULT
    mapping = json.loads(mapping_path.read_text())
    bytes_source = (checker.CRATE_ROOT / "src/bytes.rs").read_text()
    refcount = (checker.CRATE_ROOT / "src/ref_count_ops.rs").read_text()
    shared_record = (checker.CRATE_ROOT / "src/bytes/shared_record.rs").read_text()
    bytes_record = (checker.CRATE_ROOT / "src/bytes/bytes_record.rs").read_text()
    vtable_record = (checker.CRATE_ROOT / "src/bytes/vtable_record.rs").read_text()
    loom = (checker.CRATE_ROOT / "src/loom.rs").read_text()
    harness_dir = checker.NATIVE_HARNESS
    provenance_inputs = {
        "rustc_text": (checker.MIR_DIR / "rustc-version.txt").read_text(),
        "cargo_text": (checker.MIR_DIR / "cargo-version.txt").read_text(),
        "manifest_text": (harness_dir / "Cargo.toml").read_text(),
        "lock_text": (harness_dir / "Cargo.lock").read_text(),
        "harness_source": (harness_dir / "src/main.rs").read_text(),
        "module_source": (checker.ROOT / "src/lib.rs").read_text(),
        "native_run_log": (checker.GENERATED / "native-run.log").read_text(),
    }

    try:
        positive = checker.audit_all(native_source=native, mir_dir=checker.MIR_DIR,
                                     active_source=active, mapping_path=mapping_path)
    except Exception as exc:
        print(json.dumps({"status": "controls_failed", "error": f"positive rejected: {exc}"}, indent=2))
        return 2
    native_controls = source_mutations(native)
    shadow_controls = shadow_mutations(active)
    mir_controls = mir_mutations(mir_sources, checker.MIR_FILENAMES)
    prod_controls = production_mutations(checker.AI, bytes_source, refcount,
                                         shared_record, bytes_record, vtable_record, loom)
    map_controls = mapping_mutations(mapping)
    provenance_controls = provenance_mutations(
        **provenance_inputs,
        support_paths=json.loads(checker.EXPECTED_NATIVE_PROVENANCE.read_text())["support_module_paths"],
    )
    control_names = {**native_controls, **shadow_controls, **mir_controls,
                     **prod_controls, **map_controls, **provenance_controls}
    expected_names = {row["name"] for row in spec["controls"]}
    if set(control_names) != expected_names:
        print(json.dumps({"status": "controls_failed", "error": {
            "unmatched_fixture_names": sorted(expected_names - set(control_names)),
            "unmatched_mutation_names": sorted(set(control_names) - expected_names)}}, indent=2))
        return 2

    mir_facts = positive["native_mir"]
    shadow_facts = positive["proof_shadow"]
    base_source = (checker.AI_PROBE / "src/public_shared.rs").read_text()
    rows = []
    for row in spec["controls"]:
        name = row["name"]
        try:
            if name in native_controls:
                checker.audit_native_source(native_controls[name])
            elif name in shadow_controls:
                checker.audit_shadow(shadow_controls[name], checker.AI)
            elif name in mir_controls:
                checker.audit_mir_sources(mir_controls[name])
            elif name in map_controls:
                checker.audit_mapping_data(
                    map_controls[name], json.dumps(map_controls[name]).encode(),
                    f"mutation:{name}", mir_facts, shadow_facts,
                    native_source=native, base_source=base_source, active_source=active,
                    ai=checker.AI, mir_dir=checker.MIR_DIR)
            elif name in provenance_controls:
                checker.audit_native_provenance_data(**provenance_controls[name], ai=checker.AI)
            else:
                bs, rc, sr, br, vr, ls = prod_controls[name]
                checker.audit_production_drop_sources(checker.AI, bs, rc, sr, br, vr, ls)
        except Exception as exc:
            rows.append({"name": name, "class": row["class"], "rejected": True, "reason": str(exc)})
        else:
            rows.append({"name": name, "class": row["class"], "rejected": False,
                         "reason": "mutation passed correspondence check"})
    result = {
        "status": "controls_pass" if all(r["rejected"] for r in rows) else "controls_failed",
        "checker_sha256": hashlib.sha256(checker.CHECKER_PATH.read_bytes()).hexdigest()
        if hasattr(checker, "CHECKER_PATH") else hashlib.sha256((ROOT / "check_correspondence.py").read_bytes()).hexdigest(),
        "native_source_sha256": hashlib.sha256(native.encode()).hexdigest(),
        "active_source_sha256": hashlib.sha256(active.encode()).hexdigest(),
        "mapping_sha256": hashlib.sha256(mapping_path.read_bytes()).hexdigest(),
        "controls": rows,
        "control_count": len(rows),
        "all_controls_rejected": all(r["rejected"] for r in rows),
        "prover_invoked": False,
        "positive_status": positive["status"],
        "boundary": spec["boundary"],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "controls_pass" else 2


if __name__ == "__main__":
    raise SystemExit(main())
