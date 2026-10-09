#!/usr/bin/env python3
"""No-solver adversarial controls for the boxed automatic-Drop checker."""
from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import pathlib
import sys
from typing import Any, Callable

ROOT = pathlib.Path(__file__).resolve().parent
FIXTURES = ROOT / "fixtures"
CHECKER_PATH = ROOT / "check_correspondence.py"
MANIFEST = FIXTURES / "checker-controls.json"
RESULTS = FIXTURES / "checker-control-results.json"


def replace_once(source: str, old: str, new: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"control mutation requires one anchor, found {count}: {old!r}")
    return source.replace(old, new, 1)


def replace_occurrence(source: str, old: str, new: str, occurrence: int, expected_count: int) -> str:
    starts = []
    cursor = 0
    while True:
        found = source.find(old, cursor)
        if found < 0:
            break
        starts.append(found)
        cursor = found + len(old)
    if len(starts) != expected_count or occurrence >= len(starts):
        raise RuntimeError(f"control mutation requires {expected_count} occurrences, found {len(starts)}: {old!r}")
    start = starts[occurrence]
    return source[:start] + new + source[start + len(old):]


def load_checker():
    spec = importlib.util.spec_from_file_location("boxed_drop_correspondence", CHECKER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load boxed automatic-Drop correspondence checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def changed_data(base: dict[str, Any]) -> dict[str, Any]:
    data = dict(base)
    for key in ("mapping", "source_map", "generated_sources", "production_sources",
                "mir_sources", "capture_data", "provenance_data"):
        data[key] = copy.deepcopy(base[key])
    return data


def mutate_text(base: dict[str, Any], field: str, old: str, new: str,
                subfield: str | None = None, occurrence: int | None = None,
                expected_count: int | None = None) -> dict[str, Any]:
    data = changed_data(base)
    replace = replace_once if occurrence is None else lambda s, o, n: replace_occurrence(
        s, o, n, occurrence, expected_count if expected_count is not None else occurrence + 1)
    if subfield is None:
        data[field] = replace(data[field], old, new)
    else:
        data[field][subfield] = replace(data[field][subfield], old, new)
    return data


def mutate_mapping(base: dict[str, Any], name: str, fn: Callable[[dict[str, Any]], None]) -> dict[str, Any]:
    data = changed_data(base)
    fn(data["mapping"])
    return data


def mutate_source_map(base: dict[str, Any], name: str, fn: Callable[[dict[str, Any]], None]) -> dict[str, Any]:
    data = changed_data(base)
    fn(data["source_map"])
    return data


def mutate_provenance(base: dict[str, Any], name: str, key: str, old: str, new: str) -> dict[str, Any]:
    return mutate_text(base, "provenance_data", old, new, subfield=key)


def mutate_mir(base: dict[str, Any], name: str, filename: str, old: str, new: str) -> dict[str, Any]:
    return mutate_text(base, "mir_sources", old, new, subfield=filename)


def mutate_production(base: dict[str, Any], name: str, filename: str, old: str, new: str) -> dict[str, Any]:
    return mutate_text(base, "production_sources", old, new, subfield=filename)


def mutate_generated(base: dict[str, Any], name: str, filename: str, old: str, new: str) -> dict[str, Any]:
    return mutate_text(base, "generated_sources", old, new, subfield=filename)


def controls(base: dict[str, Any]) -> list[tuple[str, str, Callable[[], dict[str, Any]]]]:
    rows: list[tuple[str, str, Callable[[], dict[str, Any]]]] = []

    def add(name: str, group: str, factory: Callable[[], dict[str, Any]]) -> None:
        rows.append((name, group, factory))

    # Closed native source client.
    add("native_explicit_drop", "native client", lambda: mutate_text(base, "native_source",
        "    observed\n}", "    core::mem::drop(bytes);\n    observed\n}"))
    add("native_extra_clone", "native client", lambda: mutate_text(base, "native_source",
        "    let observed = AsRef", "    let extra = bytes.clone();\n    let observed = AsRef"))
    add("native_forget_handle", "native client", lambda: mutate_text(base, "native_source",
        "    observed\n}", "    core::mem::forget(bytes);\n    observed\n}"))
    add("native_unknown_callback", "native client", lambda: mutate_text(base, "native_source",
        "    observed\n}", "    unknown_drop_callback(&bytes);\n    observed\n}"))
    add("native_escape_bytes_address", "native client", lambda: mutate_text(base, "native_source",
        "    let observed = AsRef", "    let raw = &bytes as *const Bytes;\n    let observed = AsRef"))
    add("native_second_constructor", "native client", lambda: mutate_text(base, "native_source",
        "    let observed = AsRef", "    let other = Bytes::from(Vec::new().into_boxed_slice());\n    let observed = AsRef"))
    add("native_wrong_read_target", "native client", lambda: mutate_text(base, "native_source",
        "as_ref(&bytes)", "as_ref(&Bytes::new())"))
    add("native_explicit_return_drop", "native client", lambda: mutate_text(base, "native_source",
        "    observed\n}", "    drop(bytes);\n    observed\n}"))

    # Client-shadow effects and anti-vacuity surface.
    drop_call = "    bytes_terminal_drop(bytes,completion.borrow_mut());\n"
    read_line = "    let observed=original_bytes_as_slice(&bytes).to_vec();\n"
    add("shadow_omitted_drop", "proof client", lambda: mutate_text(base, "shadow", drop_call, ""))
    add("shadow_duplicate_drop", "proof client", lambda: mutate_text(base, "shadow", drop_call, drop_call + drop_call))
    add("shadow_wrong_terminal_owner", "proof client", lambda: mutate_text(base, "shadow", drop_call,
        "    bytes_terminal_drop(input,completion.borrow_mut());\n"))
    add("shadow_early_drop_before_read", "proof client", lambda: mutate_text(base, "shadow", read_line,
        drop_call + read_line))
    add("shadow_false_precondition", "proof client", lambda: mutate_text(base, "shadow",
        "#[ensures(result@ == input@)]", "#[requires(false)]\n#[ensures(result@ == input@)]"))
    add("shadow_trusted_summary", "proof client", lambda: mutate_text(base, "shadow",
        "#[ensures(result@ == input@)]", "#[trusted]\n#[ensures(result@ == input@)]"))
    add("shadow_unknown_callback", "proof client", lambda: mutate_text(base, "shadow", drop_call,
        drop_call + "    unknown_callback(&bytes);\n"))
    add("shadow_no_content_ensures", "proof client", lambda: mutate_text(base, "shadow",
        "#[ensures(result@ == input@)]\n", ""))
    add("shadow_omit_completion_assertion", "proof client", lambda: mutate_text(base, "shadow",
        "    proof_assert!(completion.inner_logic() != None && completion.inner_logic().unwrap_logic().valid(*metadata));\n", ""))
    add("shadow_wrong_metadata", "proof client", lambda: mutate_text(base, "shadow",
        "valid(*metadata)", "valid(None)"))
    add("template_unrestricted_contract_removed", "proof client", lambda: mutate_text(base, "driver_template",
        "#[ensures(result@ == input@)]\n", ""))
    add("template_terminal_drop_omitted", "proof client", lambda: mutate_text(base, "driver_template", drop_call, ""))
    add("template_terminal_drop_duplicated", "proof client", lambda: mutate_text(base, "driver_template", drop_call, drop_call + drop_call))
    add("template_free_before_observation", "proof client", lambda: mutate_text(base, "driver_template", read_line,
        drop_call + read_line))

    # Proof-side terminal transitions and receipt relation.
    add("helper_terminal_trusted", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "#[requires(value.original_bytes_valid() && value.boxed_only())]",
        "#[trusted]\n#[requires(value.original_bytes_valid() && value.boxed_only())]"))
    add("helper_terminal_false_precondition", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "#[requires(value.original_bytes_valid() && value.boxed_only())]",
        "#[requires(false)]\n#[requires(value.original_bytes_valid() && value.boxed_only())]"))
    add("helper_drop_without_boxed_only", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "value.original_bytes_valid() && value.boxed_only()", "value.original_bytes_valid()"))
    add("helper_unknown_native_call", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),", "unknown_native_drop(native,(&mut value.data,value.ptr,value.len),",
        occurrence=0, expected_count=3))
    add("helper_wrong_static_table", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "result.0==static_table()", "result.0==promotable_even_table()"))
    add("helper_static_wrong_completion", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "ghost! {**output=Some(BoxedCompletion::NoAllocation);};",
        "ghost! {**output=Some(BoxedCompletion::Freed(receipt.into_inner()));};", occurrence=0, expected_count=2))
    add("helper_even_untagged_pointer", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let buf=tag_specs::clear_low_bit(stored,ghost! {&proof.base});",
        "let buf=stored.cast::<u8>();"))
    add("helper_even_wrong_parity", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "#[requires(offset.addr_logic() & 1usize == 0usize)]", "#[requires(offset.addr_logic() & 1usize != 0usize)]"))
    add("helper_odd_wrong_pointer", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let buf=stored.cast::<u8>();", "let buf=proof.base.raw_pointer();", occurrence=1, expected_count=2))
    add("helper_arc_branch_admitted", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "proof_assert!(false);", "proof_assert!(true);", occurrence=0, expected_count=4))
    add("helper_force_arc_kind", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;\n    #[cfg(feature=\"negative_raw_as_arc\")] let kind=0usize;",
        "let kind=0usize;"))
    add("helper_boxed_completion_pointer_weak", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "r.pointer()==pointer", "true"))
    add("helper_boxed_completion_size_weak", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "r.size()==size", "true"))
    add("helper_missing_full_recovery_premise", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "proof.inner_logic().capabilities.inner_logic().0.invariant() && proof.inner_logic().capabilities.inner_logic().1.invariant()",
        "proof.inner_logic().capabilities.inner_logic().0.invariant()"))
    add("helper_wrong_completion_namespace", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "r.namespace()==id", "r.namespace()!=id"))
    add("helper_missing_terminal_output_write", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "ghost! {**output=Some(BoxedCompletion::Freed(receipt.into_inner()));};", "", occurrence=0, expected_count=2))

    # Physical free count, target, and layout.
    dealloc_call = "physical_projection::deallocate(buf,cap,ghost! {proof.base},ghost! {proof.into_inner().capabilities.into_inner()})"
    add("helper_free_replaced_by_conjure", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let receipt=unsafe {" + dealloc_call + "};", "let receipt=Ghost::conjure();"))
    add("helper_duplicate_free_effect", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let receipt=unsafe {" + dealloc_call + "};", "let receipt=unsafe {" + dealloc_call + "};\n        let _again=unsafe {" + dealloc_call + "};"))
    add("helper_wrong_free_pointer", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "deallocate(buf,cap,ghost!", "deallocate(buf.wrapping_add(1),cap,ghost!", occurrence=0, expected_count=2))
    add("helper_wrong_free_size", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let cap=distance as usize + len;", "let cap=distance as usize + len + 1;"))
    add("helper_omit_equal_pointer_distance", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "let distance=unsafe {tag_specs::equal_pointer_distance(offset,buf)};", "let distance=0isize;"))
    add("helper_free_false_trusted", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "#[requires(proof.inner_logic().base.invariant())]", "#[trusted]\n#[requires(proof.inner_logic().base.invariant())]"))
    add("helper_duplicate_free_canonical_cfg_removed", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "#[cfg(feature=\"negative_duplicate_free\")]\n        let _second=unsafe {physical_projection::deallocate(buf,cap,ghost! {proof.base},ghost! {proof.into_inner().capabilities.into_inner()})};",
        "let _second=unsafe {physical_projection::deallocate(buf,cap,ghost! {proof.base},ghost! {proof.into_inner().capabilities.into_inner()})};"))
    add("helper_noallocation_receipt_validity_weak", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "(Self::NoAllocation,None)=>true,", "(Self::NoAllocation,Some(_))=>true,"))
    add("helper_static_drop_calls_free", "proof helper", lambda: mutate_text(base, "boxed_drop",
        "ghost! {**output=Some(BoxedCompletion::NoAllocation);};",
        "physical_projection::deallocate(offset as *mut u8,len,ghost!{},ghost!{});", occurrence=0, expected_count=2))

    # Tag interpretation and read/free generic boundaries.
    add("tag_clear_does_not_reconstruct_exact_pointer", "tag boundary", lambda: mutate_text(base, "tag_specs",
        "(tagged as usize & !1) as *mut u8", "tagged.cast::<u8>()"))
    add("tag_clear_ensures_address_only", "tag boundary", lambda: mutate_text(base, "tag_specs",
        "#[ensures(result == base.inner_logic().raw_pointer())]", "#[ensures(result.addr_logic() == base.inner_logic().raw_pointer().addr_logic())]"))
    add("tag_wrong_origin_relation", "tag boundary", lambda: mutate_text(base, "tag_specs",
        "tagged == tagged_data(base.inner_logic().raw_pointer())", "tagged.addr_logic() == base.inner_logic().raw_pointer().addr_logic()"))
    add("tag_bitvector_proof_removed", "tag boundary", lambda: mutate_text(base, "tag_specs",
        "#[bitwise_proof]\n#[ensures(address & 1usize", "#[ensures(address & 1usize"))
    add("tag_distance_wrong_contract", "tag boundary", lambda: mutate_text(base, "tag_specs",
        "#[ensures(result == 0isize)]", "#[ensures(result == 1isize)]"))
    add("physical_deallocator_wrong_alignment", "physical boundary", lambda: mutate_text(base, "physical_projection",
        "Layout::from_size_align(capacity,1)", "Layout::from_size_align(capacity,2)"))
    add("physical_deallocator_wrong_pointer", "physical boundary", lambda: mutate_text(base, "physical_projection",
        "alloc::alloc::dealloc(pointer,", "alloc::alloc::dealloc(pointer.wrapping_add(1),"))
    add("physical_receipt_pointer_weak", "physical boundary", lambda: mutate_text(base, "physical_projection",
        "#[ensures(result.inner_logic().pointer() == pointer)]", "#[ensures(true)]"))
    add("erased_call_wrong_native_arity", "erasure boundary", lambda: mutate_text(base, "erased_call",
        "native(args.0,args.1,args.2)", "native(args.0,args.1)"))
    add("read_projection_wrong_pointer", "read boundary", lambda: mutate_text(base, "read_projection",
        "core::slice::from_raw_parts(pointer, len)", "core::slice::from_raw_parts(core::ptr::null(), len)"))

    # Constructor representation/fork and table identity.
    add("constructor_boxed_only_post_removed", "constructor source", lambda: mutate_text(base, "public_constructor",
        "#[ensures(result.boxed_only())]\n", ""))
    add("constructor_false_trusted_summary", "constructor source", lambda: mutate_text(base, "public_constructor",
        "#[ensures(result.boxed_only())]", "#[trusted]\n#[ensures(result.boxed_only())]"))
    add("constructor_tagged_pointer_uses_raw_cast", "constructor source", lambda: mutate_text(base, "public_constructor",
        "ptr_map(ptr, |address| address | 1).cast()", "ptr.cast()"))
    add("constructor_tagged_data_alias_changed", "constructor source", lambda: mutate_text(base, "public_constructor",
        "#[ensures(result == tag_specs::tagged_data(ptr))]", "#[ensures(result.addr_logic() == ptr.addr_logic())]"))
    add("constructor_raw_even_vtable_swapped", "constructor source", lambda: mutate_text(base, "public_constructor",
        "(promotable_tag_pointer(ptr), promotable_even_table_reification())", "(ptr.cast(), promotable_odd_table_reification())"))
    add("constructor_empty_static_repr_removed", "constructor source", lambda: mutate_text(base, "public_constructor",
        "#[ensures(result.static_repr())]", "#[ensures(result.boxed_only())]"))
    add("constructor_raw_valid_lowbit_weak", "constructor source", lambda: mutate_text(base, "public_constructor",
        "p.base.raw_pointer().addr_logic() & 1usize == 0usize", "true"))
    add("constructor_byte_read_range_weak", "constructor source", lambda: mutate_text(base, "public_constructor",
        "unsafe { read_projection::borrow_any(value.ptr, value.len, lease, expected) }",
        "unsafe { read_projection::borrow_any(value.ptr, 0usize, lease, expected) }"))

    # Selected production implementation and no independent field glue.
    add("production_bytes_drop_wrong_slot", "production source", lambda: mutate_production(base,
        "production_bytes_drop_wrong_slot", "bytes.rs", "(self.vtable.drop)(&mut self.data, self.ptr, self.len)",
        "(self.vtable.clone)(&self.data, self.ptr, self.len)"))
    add("production_bytes_drop_observes_receiver", "production source", lambda: mutate_production(base,
        "production_bytes_drop_observes_receiver", "bytes.rs", "fn drop(&mut self) {\n        unsafe { (self.vtable.drop)",
        "fn drop(&mut self) {\n        let _address=self as *mut Bytes;\n        unsafe { (self.vtable.drop)"))
    add("production_static_drop_extra_free", "production source", lambda: mutate_production(base,
        "production_static_drop_extra_free", "bytes.rs", "// nothing to drop for &'static [u8]", "dealloc(core::ptr::null_mut(), Layout::new::<u8>())"))
    add("production_even_drop_missing_tag", "production source", lambda: mutate_production(base,
        "production_even_drop_missing_tag", "bytes.rs",
        "let buf = ptr_map(shared.cast(), |addr| addr & !KIND_MASK);\n            free_boxed_slice(buf, ptr, len);",
        "let buf = shared.cast();\n            free_boxed_slice(buf, ptr, len);"))
    add("production_even_drop_wrong_free_target", "production source", lambda: mutate_production(base,
        "production_even_drop_wrong_free_target", "bytes.rs", "free_boxed_slice(buf, ptr, len);", "release_shared(buf.cast());"))
    add("production_odd_drop_wrong_pointer", "production source", lambda: mutate_production(base,
        "production_odd_drop_wrong_pointer", "bytes.rs", "free_boxed_slice(shared.cast(), ptr, len);", "free_boxed_slice(ptr as *mut u8, ptr, len);"))
    add("production_free_cap_formula_changed", "production source", lambda: mutate_production(base,
        "production_free_cap_formula_changed", "bytes.rs",
        "let cap = offset.offset_from(buf) as usize + len;\n    dealloc(buf, Layout::from_size_align(cap, 1).unwrap())",
        "let cap = len;\n    dealloc(buf, Layout::from_size_align(cap, 1).unwrap())"))
    add("production_ptr_map_native_provenance_changed", "production source", lambda: mutate_production(base,
        "production_ptr_map_native_provenance_changed", "bytes.rs", "new_addr as *mut u8", "ptr"))
    add("production_ptr_map_cfg_branch_changed", "production source", lambda: mutate_production(base,
        "production_ptr_map_cfg_branch_changed", "bytes.rs", "#[cfg(not(miri))]", "#[cfg(miri)]",))
    add("production_kind_vec_value_changed", "production source", lambda: mutate_production(base,
        "production_kind_vec_value_changed", "bytes.rs", "const KIND_VEC: usize = 0b1;", "const KIND_VEC: usize = 0b0;"))
    add("production_bytes_field_glue_added", "production source", lambda: mutate_production(base,
        "production_bytes_field_glue_added", "bytes_record.rs", "    vtable: &'static Vtable,", "    vtable: &'static Vtable,\n    hidden: alloc::vec::Vec<u8>,"))
    add("production_vtable_slot_added", "production source", lambda: mutate_production(base,
        "production_vtable_slot_added", "vtable_record.rs", "    pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),",
        "    pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),\n    pub hidden: unsafe fn(),"))
    add("production_atomicmut_extra_callback", "production source", lambda: mutate_production(base,
        "production_atomicmut_extra_callback", "loom.rs", "f(self.get_mut())", "f(self.get_mut()); f(self.get_mut())"))

    # Post-ElaborateDrops MIR source and ownership edges.
    client = "bytes_boxed_drop_native.boxed_read_then_drop.2-2-004.ElaborateDrops.after.mir"
    dropmir = "bytes.bytes-{impl#3}-drop.2-2-004.ElaborateDrops.after.mir"
    evenmir = "bytes.bytes-promotable_even_drop-{closure#0}.2-2-004.ElaborateDrops.after.mir"
    oddmir = "bytes.bytes-promotable_odd_drop-{closure#0}.2-2-004.ElaborateDrops.after.mir"
    freemir = "bytes.bytes-free_boxed_slice.2-2-004.ElaborateDrops.after.mir"
    ptrmir = "bytes.bytes-ptr_map.2-2-004.ElaborateDrops.after.mir"
    withmutmir = "bytes.loom-sync-atomic-{impl#0}-with_mut.2-2-004.ElaborateDrops.after.mir"
    add("mir_terminal_drop_omitted", "native MIR", lambda: mutate_mir(base, "mir_terminal_drop_omitted", client,
        "drop(_2) -> [return: bb5, unwind: bb9];", "goto -> bb5;"))
    add("mir_terminal_owner_changed", "native MIR", lambda: mutate_mir(base, "mir_terminal_owner_changed", client,
        "drop(_2) -> [return: bb5, unwind: bb9];", "drop(_1) -> [return: bb5, unwind: bb9];"))
    add("mir_terminal_unwind_successor_changed", "native MIR", lambda: mutate_mir(base,
        "mir_terminal_unwind_successor_changed", client, "unwind: bb9", "unwind: bb8"))
    add("mir_return_eval_after_drop", "native MIR", lambda: mutate_mir(base, "mir_return_eval_after_drop", client,
        "_0 = move _4;\n        goto -> bb4;", "goto -> bb4;"))
    add("mir_drop_vtable_slot_changed", "native MIR", lambda: mutate_mir(base, "mir_drop_vtable_slot_changed", dropmir,
        "(*_7).4: for<'a> unsafe fn", "(*_7).3: for<'a> unsafe fn"))
    add("mir_drop_argument_pointer_changed", "native MIR", lambda: mutate_mir(base, "mir_drop_argument_pointer_changed", dropmir,
        "_5 = copy ((*_1).0: *const u8);", "_5 = copy ((*_1).1: usize);"))
    add("mir_even_arc_branch_removed", "native MIR", lambda: mutate_mir(base, "mir_even_arc_branch_removed", evenmir,
        "const bytes::KIND_ARC", "const bytes::KIND_VEC"))
    add("mir_even_free_target_changed", "native MIR", lambda: mutate_mir(base, "mir_even_free_target_changed", evenmir,
        "free_boxed_slice(move _38, move _39, move _40)", "release_shared(move _38)"))
    add("mir_odd_free_target_changed", "native MIR", lambda: mutate_mir(base, "mir_odd_free_target_changed", oddmir,
        "free_boxed_slice(move _34, move _36, move _37)", "release_shared(move _34)"))
    add("mir_free_layout_changed", "native MIR", lambda: mutate_mir(base, "mir_free_layout_changed", freemir,
        "Layout::from_size_align(move _15, const 1_usize)", "Layout::from_size_align(move _15, const 2_usize)"))
    add("mir_ptr_map_reconstruction_changed", "native MIR", lambda: mutate_mir(base, "mir_ptr_map_reconstruction_changed", ptrmir,
        "PointerExposeProvenance", "PointerWithExposedProvenance"))
    add("mir_with_mut_callback_changed", "native MIR", lambda: mutate_mir(base, "mir_with_mut_callback_changed", withmutmir,
        "FnOnce<(&mut *mut T,)>", "FnOnce<(&mut *const T,)>"))

    # Mapping claims: every field is checked against independently parsed input.
    add("mapping_wrong_terminal_place", "mapping", lambda: mutate_mapping(base, "mapping_wrong_terminal_place",
        lambda m: m["normal_edges"][0].__setitem__("place", "_1")))
    add("mapping_wrong_unwind", "mapping", lambda: mutate_mapping(base, "mapping_wrong_unwind",
        lambda m: m["normal_edges"][0].__setitem__("unwind", "bb8")))
    add("mapping_wrong_return_eval", "mapping", lambda: mutate_mapping(base, "mapping_wrong_return_eval",
        lambda m: m["return_evaluation"].__setitem__("block", "bb4")))
    add("mapping_fake_source_hash", "mapping", lambda: mutate_mapping(base, "mapping_fake_source_hash",
        lambda m: m["source_inputs"][1].__setitem__("sha256", "0" * 64)))
    add("mapping_drop_feature_enabled", "mapping", lambda: mutate_mapping(base, "mapping_drop_feature_enabled",
        lambda m: m.__setitem__("terminal_feature", "omit_drop")))
    add("mapping_raw_as_static_claim", "mapping", lambda: mutate_mapping(base, "mapping_raw_as_static_claim",
        lambda m: m["representational_routes"].__setitem__(1, "nonempty PromotableRaw -> Static -> no allocation")))
    add("mapping_overstates_provenance", "mapping", lambda: mutate_mapping(base, "mapping_overstates_provenance",
        lambda m: m["generic_tcb"].__setitem__(2, "strict-provenance preservation proved")))
    add("mapping_capture_omitted", "mapping", lambda: mutate_mapping(base, "mapping_capture_omitted",
        lambda m: m.pop("native_capture")))

    # Compiler/Cargo provenance, no-drop profile, module routing, and fixture extraction.
    add("toolchain_rustc_changed", "provenance", lambda: mutate_provenance(base, "toolchain_rustc_changed",
        "rustc_text", "91fe22da8084a1c9e993d78d4a56f22ab8396236", "0000000000000000000000000000000000000000"))
    add("toolchain_cargo_changed", "provenance", lambda: mutate_provenance(base, "toolchain_cargo_changed",
        "cargo_text", "cargo 1.98.0-nightly", "cargo 1.99.0-nightly"))
    add("manifest_native_target_changed", "provenance", lambda: mutate_provenance(base, "manifest_native_target_changed",
        "manifest_text", "../../native.rs", "../../wrong.rs"))
    add("manifest_dependency_path_changed", "provenance", lambda: mutate_provenance(base,
        "manifest_dependency_path_changed", "manifest_text", "../../../../../", "../../../../wrong-bytes/"))
    add("manifest_extra_feature_added", "provenance", lambda: mutate_provenance(base, "manifest_extra_feature_added",
        "manifest_text", "bytes = { path = ", "bytes = { features = [\"extra-platforms\"], path = "))
    add("lock_package_changed", "provenance", lambda: mutate_provenance(base, "lock_package_changed",
        "lock_text", 'name = "bytes"\nversion = "1.11.1"', 'name = "bytes"\nversion = "1.11.2"'))
    add("harness_call_target_changed", "provenance", lambda: mutate_provenance(base, "harness_call_target_changed",
        "harness_source", "bytes_boxed_drop_native::boxed_read_then_drop(input)", "Bytes::from(input).to_vec()"))
    add("harness_no_drop_profile_removed", "provenance", lambda: mutate_provenance(base, "harness_no_drop_profile_removed",
        "harness_source", "const _: [(); 0]", "const _: [(); 1]"))
    add("harness_input_coverage_reduced", "provenance", lambda: mutate_provenance(base, "harness_input_coverage_reduced",
        "harness_source", "[0, 1, 7, 63, 1024]", "[1, 7, 63, 1024]"))
    add("harness_success_log_forged", "provenance", lambda: mutate_provenance(base, "harness_success_log_forged",
        "native_run_log", "5 inputs passed", "4 inputs passed"))
    add("module_support_path_override", "module routing", lambda: mutate_text(base, "lib_source",
        "../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs",
        "../../unreviewed/src/pointer_event.rs"))
    add("module_raw_vec_path_override", "module routing", lambda: mutate_text(base, "lib_source",
        "../../../../src/ownership_proof/raw_vec.rs", "../../../../unreviewed/raw_vec.rs"))
    add("native_binding_wrong_target", "generated extraction", lambda: mutate_generated(base,
        "native_binding_wrong_target", "native_constructor_bindings.rs", "drop: promotable_even_drop,", "drop: promotable_odd_drop,"))
    add("extracted_box_from_target_changed", "generated extraction", lambda: mutate_generated(base,
        "extracted_box_from_target_changed", "public_traits.rs", "original_bytes_from_box(slice)", "original_bytes_from_static(slice)"))
    add("extracted_record_hidden_field", "generated extraction", lambda: mutate_generated(base,
        "extracted_record_hidden_field", "public_records.rs", "pub struct Bytes {", "pub struct Bytes { hidden: Vec<u8>,"))
    add("source_map_box_constructor_hash_forged", "generated extraction", lambda: mutate_source_map(base,
        "source_map_box_constructor_hash_forged",
        lambda m: m["bytes_from_box_impl"].__setitem__("sha256", "0" * 64)))
    add("source_map_native_line_changed", "generated extraction", lambda: mutate_source_map(base,
        "source_map_native_line_changed",
        lambda m: m["native/promotable_even_vtable"].__setitem__("line", 1)))

    return rows


def run(base: dict[str, Any]) -> dict[str, Any]:
    checker = load_checker()
    rows = controls(base)
    results = []
    for name, group, factory in rows:
        candidate = factory()
        try:
            checker.audit_all_data(**candidate)
        except Exception as exc:
            results.append({"name": name, "group": group, "result": "rejected", "reason": str(exc)})
        else:
            results.append({"name": name, "group": group, "result": "accepted"})
    accepted = [r["name"] for r in results if r["result"] != "rejected"]
    return {"status": "pass" if not accepted else "fail", "control_count": len(results),
            "rejected_count": len(results) - len(accepted), "accepted_controls": accepted,
            "controls": results, "solver_invocations": 0}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path, default=RESULTS)
    args = parser.parse_args()
    checker = load_checker()
    base = checker.collect_inputs(checker.POSITIVE_SHADOW, checker.POSITIVE_MAPPING)
    result = run(base)
    manifest = [{"name": name, "group": group, "expected": "rejected"}
                for name, group, _ in controls(base)]
    MANIFEST.write_text(json.dumps({"controls": manifest, "solver_invocations": 0}, indent=2) + "\n")
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: result[k] for k in ("status", "control_count", "rejected_count", "accepted_controls", "solver_invocations")}, indent=2))
    return 0 if result["status"] == "pass" else 2


if __name__ == "__main__":
    raise SystemExit(main())
