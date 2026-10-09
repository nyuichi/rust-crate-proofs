#!/usr/bin/env python3
"""Replay no-solver mutations against the independent native-client checker."""
from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import argparse
import hashlib
from typing import Callable

ROOT = pathlib.Path(__file__).resolve().parent
FIXTURES = ROOT / "fixtures" / "checker-controls.json"
CLIENT = ROOT / "src" / "native_client.rs"
CHECKER = ROOT / "check_correspondence.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("scope_client_correspondence", CHECKER)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load independent correspondence checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def replace_once(source: str, old: str, new: str) -> str:
    if source.count(old) != 1:
        raise RuntimeError(f"fixture mutation anchor must occur once: {old!r}")
    return source.replace(old, new, 1)


def replace_in_marker(source: str, marker: str, old: str, new: str) -> str:
    begin = f"// ORIGINAL_SHARED_BEGIN {marker}\n"
    end = f"// ORIGINAL_SHARED_END {marker}\n"
    if source.count(begin) != 1 or source.count(end) != 1:
        raise RuntimeError(f"fixture marker must occur once: {marker}")
    a = source.index(begin)
    b = source.index(end, a) + len(end)
    chunk = source[a:b]
    changed = replace_once(chunk, old, new)
    return source[:a] + changed + source[b:]


def mutations(source: str) -> dict[str, str]:
    changed: dict[str, str] = {}
    changed["unretired_extra_clone"] = replace_once(
        source, "    let fourth = second.clone();",
        "    let fourth = second.clone();\n    let extra = first.clone();")
    changed["escaped_bytes_owner"] = replace_once(source, "    observed\n}", "    fourth\n}")
    changed["unfinished_callback"] = replace_once(
        source, "    let fourth = second.clone();",
        "    let fourth = second.clone();\n    unknown_callback(&fourth);")
    changed["thread_escape"] = replace_once(
        source, "    let fourth = second.clone();",
        "    let fourth = second.clone();\n    std::thread::spawn(move || fourth.clone());")
    changed["raw_alias"] = replace_once(
        source, "    let fourth = second.clone();",
        "    let fourth = second.clone();\n    let raw = &fourth as *const Bytes;")
    changed["forgotten_handle"] = replace_once(
        source, "    let fourth = second.clone();",
        "    let fourth = second.clone();\n    core::mem::forget(fourth);")
    changed["wrong_clone_source"] = replace_once(source, "let third = first.clone();", "let third = second.clone();")
    changed["borrow_from_wrong_owner"] = replace_once(
        source, "let borrowed = AsRef::<[u8]>::as_ref(&fourth);",
        "let borrowed = AsRef::<[u8]>::as_ref(&second);")
    changed["copy_before_peer_cleanup"] = replace_once(
        source,
        "    second.cleanup();\n    let observed = borrowed.to_vec();",
        "    let observed = borrowed.to_vec();\n    second.cleanup();")
    changed["retire_borrow_owner_before_copy"] = replace_once(
        source,
        "    second.cleanup();\n    let observed = borrowed.to_vec();\n    fourth.cleanup();",
        "    second.cleanup();\n    fourth.cleanup();\n    let observed = borrowed.to_vec();")
    changed["duplicate_cleanup"] = replace_once(
        source, "    fourth.cleanup();\n    observed",
        "    fourth.cleanup();\n    fourth.cleanup();\n    observed")
    changed["omitted_cleanup"] = replace_once(source, "    first.cleanup();\n", "")
    changed["wrong_native_order"] = replace_once(
        source,
        "    third.cleanup();\n    first.cleanup();\n    let fourth = second.clone();",
        "    let fourth = second.clone();\n    third.cleanup();\n    first.cleanup();")
    changed["comment_cannot_replace_event"] = replace_once(source, "    third.cleanup();", "    // third.cleanup();")
    changed["string_cannot_replace_event"] = replace_once(
        source, "    third.cleanup();", '    let fake = "third.cleanup();";')
    return changed


def shadow_mutations(source: str) -> dict[str, str]:
    """Adversarial edits to the proof-facing client and selected call wiring."""
    changed: dict[str, str] = {}
    changed["shadow_extra_live_owner"] = replace_once(
        source, "    let borrowed=original_shared_as_slice(&fourth);",
        "    let borrowed=original_shared_as_slice(&fourth);\n"
        "    let unretired=original_shared_clone(&fourth,cursor.borrow_mut());")
    changed["shadow_escaped_owner"] = replace_once(source, "    observed\n}", "    fourth\n}")
    changed["shadow_unknown_callback"] = replace_once(
        source, "    let fourth=original_shared_clone(&second,cursor.borrow_mut());",
        "    let fourth=original_shared_clone(&second,cursor.borrow_mut());\n    unknown_callback(&fourth);")
    changed["shadow_omitted_clone_event"] = replace_once(
        source, "    let third=original_shared_clone(&first,cursor.borrow_mut());\n", "")
    changed["shadow_wrong_clone_source"] = replace_once(
        source, "    let third=original_shared_clone(&first,cursor.borrow_mut());",
        "    let third=original_shared_clone(&second,cursor.borrow_mut());")
    changed["shadow_duplicate_cleanup"] = replace_once(
        source,
        "    original_shared_cleanup(fourth,cursor.borrow_mut(),fourth_receipt.borrow_mut());",
        "    original_shared_cleanup(fourth,cursor.borrow_mut(),fourth_receipt.borrow_mut());\n"
        "    original_shared_cleanup(fourth,cursor.borrow_mut(),fourth_receipt.borrow_mut());")
    changed["shadow_omitted_cleanup_event"] = replace_once(
        source,
        "    original_shared_cleanup(second,cursor.borrow_mut(),second_receipt.borrow_mut());\n", "")
    changed["shadow_borrow_wrong_owner"] = replace_once(
        source, "let borrowed=original_shared_as_slice(&fourth);",
        "let borrowed=original_shared_as_slice(&second);")
    changed["shadow_copy_before_peer_cleanup"] = replace_once(
        source,
        "    original_shared_cleanup(second,cursor.borrow_mut(),second_receipt.borrow_mut());\n"
        "    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());\n"
        "    let observed=borrowed.to_vec();",
        "    let observed=borrowed.to_vec();\n"
        "    original_shared_cleanup(second,cursor.borrow_mut(),second_receipt.borrow_mut());\n"
        "    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());")
    changed["shadow_forged_final_receipt"] = replace_once(
        source,
        "fourth_receipt.inner_logic().unwrap_logic().reclaimed());",
        "!fourth_receipt.inner_logic().unwrap_logic().reclaimed());")
    changed["shadow_missing_clone_cursor"] = replace_once(
        source,
        "ghost! {(*source.invariant).to_ref()},cursor,",
        "ghost! {(*source.invariant).to_ref()},invariant,")
    changed["shadow_omitted_registration_event"] = replace_once(
        source, "lifecycle::State::on_register(", "lifecycle::State::on_release(")
    changed["shadow_missing_release_event"] = replace_once(
        source, "lifecycle::State::on_release(", "lifecycle::State::on_register(")
    changed["shadow_missing_acquire"] = replace_once(
        source, "field_event::acquire_owned::<Shared,lifecycle::State<Payload>,_>",
        "field_event::decrement_owned::<Shared,lifecycle::State<Payload>,_>")
    changed["shadow_missing_payload_free"] = replace_once(
        source,
        "let buffer=physical_projection::deallocate(shared.buf,shared.cap,bound,capabilities);",
        "let buffer=ghost! { Ghost::<physical_projection::FreeReceipt>::conjure().into_inner() };")
    changed["shadow_missing_control_free"] = replace_once(
        source,
        "let control=free_effect::deallocate_typed_box(pointer,owner);",
        "let control=ghost! { Ghost::<free_effect::TypedFreeReceipt<Shared>>::conjure().into_inner() };")
    changed["shadow_wrong_control_free_owner"] = replace_once(
        source,
        "free_effect::deallocate_typed_box(pointer,owner)",
        "free_effect::deallocate_typed_box(shared,owner)")
    changed["shadow_clone_wrong_native_arguments"] = replace_once(
        source,
        "erased_call::invoke3(native,(&source.data,source.ptr,source.len),",
        "erased_call::invoke3(native,(source.ptr,&source.data,source.len),")
    changed["shadow_cleanup_missing_receipt_channel"] = replace_once(
        source,
        "ghost! {(proof.into_inner(),&mut **cursor,&mut **output)},spec)",
        "ghost! {(proof.into_inner(),&mut **cursor)},spec)")
    changed["shadow_wrong_registration_target"] = replace_once(
        source,
        "fn shared_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {\n"
        "    // The native branch names the exact extracted production getter. The\n"
        "    // closed table and target/helper checks are in generated/native_bindings.rs\n"
        "    // and extract_public.py. Reification/ghost erasure is explicit generic TCB.\n"
        "    #[cfg(not(creusot))]\n    { (original_shared_table_native(), Ghost::conjure()) }",
        "fn shared_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {\n"
        "    // The native branch names the exact extracted production getter. The\n"
        "    // closed table and target/helper checks are in generated/native_bindings.rs\n"
        "    // and extract_public.py. Reification/ghost erasure is explicit generic TCB.\n"
        "    #[cfg(not(creusot))]\n    { (wrong_table(), Ghost::conjure()) }")
    changed["shadow_fake_comment_event"] = replace_once(
        source, "lifecycle::State::on_register(", "// lifecycle::State::on_register(")
    changed["shadow_fake_string_event"] = replace_once(
        source,
        "    let third=original_shared_clone(&first,cursor.borrow_mut());",
        '    let fake = "original_shared_clone(&first,cursor.borrow_mut())";')
    changed["shadow_trusted_helper_false_summary"] = replace_once(
        source, "fn original_shared_from_vec(input:Vec<u8>)->(Bytes,Ghost<Cursor>) {",
        "#[trusted]\n#[ensures(false)]\n"
        "fn original_shared_from_vec(input:Vec<u8>)->(Bytes,Ghost<Cursor>) {")
    changed["shadow_driver_false_precondition"] = replace_once(
        source,
        "#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]\n"
        "#[ensures(result@ == input@)]\npub(crate) fn actual_public_shared_driver(",
        "#[requires(false)]\n#[ensures(result@ == input@)]\n"
        "pub(crate) fn actual_public_shared_driver(")
    changed["shadow_constructor_wrong_native_pointer"] = replace_once(
        source, "    let ptr = base.as_ptr();", "    let ptr = input.as_ptr();")
    changed["shadow_constructor_wrong_refcount_init"] = replace_once(
        source, "    let (ref_cnt, count_permission) = field_event::new(1, current.borrow_mut());",
        "    let (ref_cnt, count_permission) = field_event::new(2, current.borrow_mut());")
    changed["shadow_as_slice_wrong_range"] = replace_once(
        source, "physical_projection::borrow(value.ptr,value.len,",
        "physical_projection::borrow(value.ptr,value.cap,")
    changed["shadow_clone_extra_event"] = replace_once(
        source, "    let mut pointer_view=ghost! {SyncView::new().into_inner()};",
        "    let _extra=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>();\n"
        "    let mut pointer_view=ghost! {SyncView::new().into_inner()};")
    changed["shadow_cleanup_wrong_target"] = replace_once(
        source, "let receipts=free_recovered(shared,recovered);",
        "let receipts=free_recovered(proof.shared,recovered);")
    return changed


def module_mutations(source: str) -> dict[str, str]:
    return {
        "wrong_public_shared_cfg": replace_once(
            source, "#[cfg(creusot)] mod public_shared;",
            "#[cfg(not(creusot))] mod public_shared;"),
        "wrong_lifecycle_source_path": replace_once(
            source,
            '#[path = "../../scoped-issuance-cursor-2026-10-09/src/lifecycle.rs"] mod lifecycle;',
            '#[path = "../../original-shared-lifecycle-2026-10-08/src/lifecycle.rs"] mod lifecycle;'),
    }


def native_harness_mutations(source: str, manifest: str,
                             client_path: pathlib.Path,
                             crate_root: pathlib.Path) -> dict[str, tuple[str, str]]:
    return {
        "harness_wrong_client_source": (
            replace_once(source, str(client_path.resolve()),
                         str(client_path.with_name("other_client.rs").resolve())), manifest),
        "harness_omitted_capacity_premise": (
            replace_once(source, "        assert!(input.len() < input.capacity());\n", ""), manifest),
        "harness_unchecked_output": (
            replace_once(source,
                         "        assert_eq!(native_client::actual_public_shared_driver(input), expected);",
                         "        let _ = native_client::actual_public_shared_driver(input);"), manifest),
        "harness_wrong_production_dependency": (
            source, replace_once(manifest, str(crate_root.resolve()),
                                 str(crate_root.parent / "wrong-bytes-crate"))),
    }


def production_mutations(source: str) -> dict[str, str]:
    """Adversarial edits to real production spans; the checker reparses them."""
    changed: dict[str, str] = {}
    changed["wrong_vtable_clone_target"] = replace_in_marker(
        source, "shared_vtable", "    clone: shared_clone,", "    clone: static_clone,")
    changed["wrong_clone_helper_target"] = replace_in_marker(
        source, "shared_clone", "shallow_clone_arc(shared as _, ptr, len)", "shared_clone(shared as _, ptr, len)")
    changed["duplicate_release_callback"] = replace_in_marker(
        source, "shared_drop", "        release_shared(shared.cast());", "        release_shared(shared.cast());\n        release_shared(shared.cast());")
    changed["extra_clone_refcount_event"] = replace_in_marker(
        source, "shallow_clone_arc",
        "    crate::ref_count_ops::increment(&(*shared).ref_cnt);",
        "    crate::ref_count_ops::increment(&(*shared).ref_cnt);\n"
        "    crate::ref_count_ops::increment(&(*shared).ref_cnt);")
    changed["extra_release_free_effect"] = replace_in_marker(
        source, "release_shared", "    free_shared(ptr);",
        "    free_shared(ptr);\n    free_shared(ptr);")
    changed["extra_free_shared_deallocation"] = replace_in_marker(
        source, "free_shared",
        "    dealloc(ptr.cast(), Layout::new::<Shared>());",
        "    dealloc(ptr.cast(), Layout::new::<Shared>());\n"
        "    dealloc(ptr.cast(), Layout::new::<Shared>());")
    changed["wrong_native_event_order"] = replace_in_marker(
        source, "release_shared", "    (*ptr).ref_cnt.load(Ordering::Acquire);\n\n    // Explicit cleanup has the same payload/control effects as dropping the Box.",
        "    free_shared(ptr);\n    (*ptr).ref_cnt.load(Ordering::Acquire);\n\n    // Explicit cleanup has the same payload/control effects as dropping the Box.")
    changed["missing_acquire"] = replace_in_marker(
        source, "release_shared", "    (*ptr).ref_cnt.load(Ordering::Acquire);\n", "")
    changed["omitted_payload_free"] = replace_in_marker(
        source, "free_shared", "    dealloc(buf, Layout::from_size_align(cap, 1).unwrap());\n", "")
    changed["omitted_control_free"] = replace_in_marker(
        source, "free_shared", "    dealloc(ptr.cast(), Layout::new::<Shared>());\n", "")
    changed["unconditional_from_requirement"] = replace_in_marker(
        source, "bytes_from_vec_impl", "    fn from(vec: Vec<u8>) -> Bytes {",
        "    #[requires(vec@.len() < creusot_std::std::vec::capacity_model(vec))]\n"
        "    fn from(vec: Vec<u8>) -> Bytes {")
    changed["constructor_wrong_selected_vtable"] = replace_in_marker(
        source, "bytes_from_vec_impl", "{ &SHARED_VTABLE }", "{ &STATIC_VTABLE }")
    changed["constructor_wrong_len_capacity_branch"] = replace_in_marker(
        source, "bytes_from_vec_impl", "if len == cap {", "if len != cap {")
    changed["constructor_wrong_shared_initial_count"] = replace_in_marker(
        source, "bytes_from_vec_impl", "{ AtomicUsize::new(1) }", "{ AtomicUsize::new(2) }")
    changed["constructor_wrong_native_fallback_cfg"] = replace_in_marker(
        source, "bytes_from_vec_impl",
        "#[cfg(not(all(creusot, any(bytes_original_shared_gate, bytes_original_constructor_gate))))]",
        "#[cfg(all(creusot, any(bytes_original_shared_gate, bytes_original_constructor_gate)))]")
    changed["wrong_clone_signature"] = replace_in_marker(
        source, "bytes_clone_impl", "fn clone(&self) -> Bytes {", "fn clone(&mut self) -> Bytes {")
    changed["fake_comment_native_free"] = replace_in_marker(
        source, "release_shared", "    free_shared(ptr);\n", "    // free_shared(ptr);\n")
    changed["fake_string_native_free"] = replace_in_marker(
        source, "release_shared", "    free_shared(ptr);\n", '    let fake = "free_shared(ptr);";\n')
    return changed


def production_profile_mutations(source: str, shared_record: str) -> dict[str, tuple[str, str]]:
    changed: dict[str, tuple[str, str]] = {}
    changed["missing_atomic_needs_drop_guard"] = (
        replace_in_marker(
            source, "free_shared",
            "const _: [(); 0] = [(); mem::needs_drop::<AtomicUsize>() as usize];\n",
            ""),
        shared_record,
    )
    changed["changed_shared_destructor"] = (
        replace_once(
            source,
            "dealloc(self.buf, Layout::from_size_align(self.cap, 1).unwrap())",
            "dealloc(self.buf, Layout::from_size_align(self.cap + 1, 1).unwrap())"),
        shared_record,
    )
    changed["added_independent_drop_field"] = (
        source,
        replace_once(
            shared_record,
            "    pub(crate) ref_cnt: AtomicUsize,",
            "    pub(crate) ref_cnt: AtomicUsize,\n    owner: Vec<u8>,"),
    )
    changed["unexpected_shared_drop_glue"] = (
        replace_in_marker(
            source, "free_shared",
            "    dealloc(ptr.cast(), Layout::new::<Shared>());",
            "    core::ptr::drop_in_place(ptr);\n    dealloc(ptr.cast(), Layout::new::<Shared>());"),
        shared_record,
    )
    return changed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path,
                        default=ROOT / "fixtures" / "checker-control-results.json")
    parser.add_argument("--native-harness", type=pathlib.Path,
                        default=ROOT / "generated/native-harness/src/main.rs")
    parser.add_argument("--native-manifest", type=pathlib.Path,
                        default=ROOT / "generated/native-harness/Cargo.toml")
    args = parser.parse_args()
    spec = json.loads(FIXTURES.read_text())
    checker = load_checker()
    source = CLIENT.read_text()
    shadow_source = checker.SHADOW.read_text()
    lib_source = checker.LIB_SOURCE.read_text()
    production_source = checker.BYTES_SOURCE.read_text()
    shared_record_source = checker.SHARED_RECORD_SOURCE.read_text()
    refcount_source = checker.REFCOUNT_SOURCE.read_text()
    try:
        positive = checker.audit_all(
            driver_source=source,
            shadow_source=shadow_source,
            lib_source=lib_source,
            bytes_source=production_source,
            refcount_source=refcount_source,
            shared_record_source=shared_record_source,
            bytes_record_source=(checker.CRATE_ROOT / "src/bytes/bytes_record.rs").read_text(),
            vtable_record_source=(checker.CRATE_ROOT / "src/bytes/vtable_record.rs").read_text(),
            bytes_mut_source=(checker.CRATE_ROOT / "src/bytes_mut.rs").read_text(),
            generated_dir=checker.GENERATED,
            native_harness_source=args.native_harness.read_text(),
            native_manifest_source=args.native_manifest.read_text(),
            native_harness_path=args.native_harness,
            native_manifest_path=args.native_manifest,
        )
    except checker.CorrespondenceError as exc:
        print(json.dumps({"status": "controls_failed", "error": f"positive correspondence rejected: {exc}"}, indent=2))
        return 2

    try:
        production_positive = checker.audit_production_chain(
            production_source, refcount_source, shared_record_source)
    except checker.CorrespondenceError as exc:
        print(json.dumps({"status": "controls_failed", "error": f"positive production source rejected: {exc}"}, indent=2))
        return 2

    rows = []
    client_controls = mutations(source)
    shadow_controls = shadow_mutations(shadow_source)
    wiring_controls = module_mutations(lib_source)
    source_controls = production_mutations(production_source)
    profile_controls = production_profile_mutations(production_source, shared_record_source)
    harness_source = args.native_harness.read_text()
    harness_manifest = args.native_manifest.read_text()
    harness_controls = native_harness_mutations(
        harness_source, harness_manifest, CLIENT, checker.CRATE_ROOT)
    by_name = {**client_controls, **shadow_controls, **wiring_controls,
               **source_controls, **profile_controls, **harness_controls}
    expected_names = {row["name"] for row in spec["controls"]}
    if set(by_name) != expected_names:
        print(json.dumps({"status": "controls_failed", "error": "fixture names and mutations differ"}, indent=2))
        return 2
    for row in spec["controls"]:
        try:
            if row["name"] in client_controls:
                checker.audit_native_client(client_controls[row["name"]])
            elif row["name"] in shadow_controls:
                checker.audit_shadow_source(shadow_controls[row["name"]], lib_source)
            elif row["name"] in wiring_controls:
                checker.audit_module_wiring(wiring_controls[row["name"]], shadow_source)
            elif row["name"] in profile_controls:
                mutated_bytes, mutated_record = profile_controls[row["name"]]
                checker.audit_production_chain(mutated_bytes, refcount_source, mutated_record)
            elif row["name"] in harness_controls:
                mutated_harness, mutated_manifest = harness_controls[row["name"]]
                checker.audit_native_harness(
                    mutated_harness, mutated_manifest, args.native_harness,
                    CLIENT, checker.CRATE_ROOT)
            else:
                checker.audit_production_chain(source_controls[row["name"]], refcount_source,
                                               shared_record_source)
        except checker.CorrespondenceError as exc:
            rows.append({"name": row["name"], "class": row["class"], "rejected": True,
                         "reason": str(exc)})
        else:
            rows.append({"name": row["name"], "class": row["class"], "rejected": False,
                         "reason": "mutated client passed"})
    passed = all(row["rejected"] for row in rows)
    result = {
        "status": "controls_pass" if passed else "controls_failed",
        "checker_sha256": hashlib.sha256(CHECKER.read_bytes()).hexdigest(),
        "native_client_sha256": hashlib.sha256(CLIENT.read_bytes()).hexdigest(),
        "shadow_sha256": hashlib.sha256(checker.SHADOW.read_bytes()).hexdigest(),
        "lib_source_sha256": hashlib.sha256(checker.LIB_SOURCE.read_bytes()).hexdigest(),
        "shared_record_sha256": hashlib.sha256(checker.SHARED_RECORD_SOURCE.read_bytes()).hexdigest(),
        "native_harness_sha256": hashlib.sha256(args.native_harness.read_bytes()).hexdigest(),
        "native_manifest_sha256": hashlib.sha256(args.native_manifest.read_bytes()).hexdigest(),
        "positive_status": positive["status"],
        "positive_events": len(positive["native_client"]["events"]),
        "production_native_chain": positive["production_native_chain"]["native_clone_target"],
        "generated_extractions": positive["generated_extractions"],
        "controls": rows,
        "control_count": len(rows),
        "prover_invoked": False,
        "fixture_boundary": spec["boundary"],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if passed else 2


if __name__ == "__main__":
    raise SystemExit(main())
