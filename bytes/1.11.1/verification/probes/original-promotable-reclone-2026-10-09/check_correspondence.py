#!/usr/bin/env python3
"""Fail-closed AN source/native mapping for bounded normal-return reclone trace."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
PROBES = ROOT.parent
AL_ROOT = PROBES / "original-promotable-first-clone-2026-10-09"
AM_ROOT = PROBES / "original-promotable-automatic-drop-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
HELPERS = ROOT / "src/terminal_helpers.rs"
EXTENSION = ROOT / "src/reclone_extension.rs"
CLIENT = ROOT / "generated/elaborated-client.rs"
MAPPING = ROOT / "generated/mapping.json"
NATIVE_CHECKER = ROOT / "check_native.py"
BASE_SHA = "2c51555cc0c5eb9da7719609b2b2826e3e354251b802ceb3b440efa2b5f5f3f2"
HELPER_SHA = "f7e410dcf3c2900464249a3d02ca04deae18cfaf59054ba1f20a9bdf9848d0f7"
EXTENSION_SHA = "44088313f920d23491325973d6833eebb6352606cc6367d8f5c193c4db5227ab"
REVIEWED_PRODUCTION_SHA = "e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306"
AM_CHECKER_SHA = "77acbf4e9aefd0d68decec909a456b823644debc74475d57d5cdeea792765968"
NATIVE_CHECKER_SHA = "eaa6b8f98b61541f5f8709293371172627ff1dfb9da4542293a769cedaa7c318"
IMPORTED_SUPPORT = {
    "relaxed": ("../../original-public-shared-gate-2026-10-08/src/relaxed.rs", "e576c17ffa56d581568d80b3454acc2eb39e8e9761b102e960636c369cc79e04"),
    "fraction_map": ("../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs", "21d16972f66f3fe8d546416763a150105afc1c50ce16e523ea1e9c4ebee7e38d"),
    "release": ("../../shared-physical-lifecycle-2026-10-08/src/release.rs", "99d8d83270a61d644960944e0019dd477c965b550e757b06e6e87d2540a2e766"),
    "pointer_event": ("../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs", "5e9288f10072c583c3a7c5870180a04d90c71581f3fbc8452bbf3dbdfe9c2d6e"),
    "promotion_tags": ("../../original-boxed-automatic-drop-2026-10-09/src/tag_specs.rs", "d58c28a3a0480362dd1189ceb7b665dd2aa1334b53b44b80912f473e2918d899"),
    "ref_count_limit": ("../../../../src/ref_count_limit.rs", "4132ec80c39487ded0eae958fb949208a73c7cff50b5af980a53f246b709a63b"),
    "owned_region": ("../../../../src/ownership_proof/owned_region.rs", "ffa44756e4866e05d9b02bfac7db2631b030eb42a45709974700edefd0b06b83"),
    "raw_vec": ("../../../../src/ownership_proof/raw_vec.rs", "0cd8a2cd65a764ba9c3c49b8d2133eb61353eebe6d33c3fa260cfd5d3ebe0ff8"),
    "boxed_alignment": ("../../../../src/ownership_proof/boxed_alignment.rs", "3a7879fc80b9aaad8a72fc8ab8a1cda82038b14f76c0fac0d415d75d35e626f5"),
    "native_ref_count_ops": ("../../../../src/ref_count_ops.rs", "ba377280ab62388662491445f0653701773dba06087c1abf71e450920a3af2a0"),
}


class CheckError(RuntimeError):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def has_tokens(tokens: list[str], snippet: str) -> bool:
    wanted=AI.rust_tokens(snippet)
    return any(tokens[i:i+len(wanted)]==wanted for i in range(len(tokens)-len(wanted)+1))


def load_module(name: str, path: pathlib.Path):
    require(path.is_file(), f"required checker input is missing: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot import checker: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


require(sha((AM_ROOT/"check_correspondence.py").read_bytes())==AM_CHECKER_SHA,
        "published AM correspondence checker changed before reuse")
AM = load_module("an_frozen_am_correspondence", AM_ROOT / "check_correspondence.py")
AI = AM.AI
AL = AM.AL


EXPECTED_CLIENT = r"""/// Native lexical second/first Drop, surviving root read, saved return and
/// final root Drop. Every ledger key comes from the actual returned ticket.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_reclone_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let first=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let first_id=snapshot!(first.child_id());
    let before_reclone=snapshot!((*scope.observation()).0);
    let second=reclone_root(&original,scope.borrow_mut());
    let second_id=snapshot!(second.child_id());
    let second_fraction=snapshot!(second.child_fraction());
    let live_three=snapshot!((*scope.observation()).0);
    proof_assert!(!(*before_reclone).contains(*second_id));
    proof_assert!(*live_three==(*before_reclone).insert(*second_id,Excl(*second_fraction)));
    proof_assert!((*live_three).len()==3);
    let mut second_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(second,scope.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic()!=None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_three).remove(*second_id));
    proof_assert!(!(*scope.observation()).0.contains(*second_id));
    proof_assert!((*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==2);
    let live_two=snapshot!((*scope.observation()).0);
    let mut first_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(first,scope.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_two).remove(*first_id));
    proof_assert!(!(*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}"""


def expected_client_contract() -> list[list[str]]:
    return [AI.rust_tokens("#[requires(input@.len()>0)]"),
            AI.rust_tokens("#[ensures(result@==input@)]")]


def assert_frozen_prefix() -> dict[str, Any]:
    base = (ROOT / "src/promotion.rs").read_bytes()
    helper = HELPERS.read_bytes()
    require(sha(base) == BASE_SHA and base == (AL_ROOT / "src/promotion.rs").read_bytes(),
            "AN AL promotion prefix differs from published AL")
    require(sha(helper) == HELPER_SHA and helper == (AM_ROOT / "generated/terminal-helper.rs").read_bytes(),
            "AN terminal adapters differ from the published AM adapters")
    expected_modules=set(json.loads(AM.FROZEN_MANIFEST.read_text())["rust_modules"])
    require(all((ROOT/"src"/name).is_file() for name in expected_modules),
            "AN is missing a frozen AL support module")
    overrides = {name:(ROOT/"src"/name).read_bytes() for name in expected_modules}
    AM.assert_frozen_al_sources(overrides)
    require((ROOT / "src/terminal_helpers.rs").read_bytes() == helper,
            "AN terminal helper module is not byte-identical to AM's reviewed helper")
    return {"AL_promotion_sha256": sha(base), "AM_terminal_helper_sha256": sha(helper),
            "AL_support_modules_frozen": len(overrides), "AM_helper_reused": True}


def assert_module_route(lib: str, active: str) -> dict[str, Any]:
    original = (AL_ROOT / "src/lib.rs").read_text()
    original_route="#[cfg(creusot)] mod promotion;"
    route = '#[cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;'
    require(original.count(original_route)==1 and lib==original.replace(original_route,route,1),
            "AN crate root changed outside the inherited selected promotion route")
    require(lib.count(route) == 1 and AI.rust_path_for_module(lib, "promotion") == "../generated/active.rs",
            "AN proof module must use the exact selected generated/active.rs route")
    resolved = (ROOT / "src/../generated/active.rs").resolve()
    require(resolved == ACTIVE.resolve() and resolved.is_file(), "AN promotion route does not resolve to active.rs")
    normalized = lib.replace(route, "#[cfg(creusot)] mod promotion;", 1)
    AL.assert_routes(normalized, (AL_ROOT / "src/promotion.rs").read_text())
    AL.assert_selected_include_routes(active, ACTIVE, "generated/active.rs")
    imported=assert_imported_support_sources(lib)
    return {"lib_source_inherited_exactly": True, "selected_active_route": "../generated/active.rs",
            "resolved_active": str(resolved), "support_routes_and_literals_checked": True,
            "imported_support_sources": imported}


def assert_imported_support_sources(lib: str, source_overrides: dict[str, str] | None = None) -> dict[str, str]:
    source_overrides=source_overrides or {}
    result={}
    for module,(literal,expected_sha) in IMPORTED_SUPPORT.items():
        require(AI.rust_path_for_module(lib,module)==literal,
                f"imported support module route changed: {module}")
        path=(ROOT/"src"/literal).resolve()
        require(path.is_file(),f"reviewed support source missing: {module}")
        source=source_overrides[module] if module in source_overrides else path.read_text()
        require(sha(source.encode())==expected_sha,
                f"directly imported proof support source changed: {module}")
        result[module]=expected_sha
    return result


def assert_helpers(helper: str, active: str) -> dict[str, Any]:
    base = (ROOT / "src/promotion.rs").read_text()
    require(AI.rust_tokens(helper) == AI.rust_tokens((AM_ROOT / "generated/terminal-helper.rs").read_text()),
            "AN helper token stream differs from the reviewed AM terminal wrappers")
    require(active.startswith(base + "\n" + helper), "active module does not start with frozen AL/AM prefix")
    require(not re.search(r"\bimpl\s+Drop\s+for\s+Bytes\b", AI.mask_noncode(active)),
            "proof shadow introduced an implicit Bytes Drop implementation")
    return {"two_terminal_adapters_inherited": True, "no_shadow_Bytes_Drop": True}


def assert_active_composition(base: str, helper: str, extension: str, client: str, active: str) -> dict[str, Any]:
    require(active == base + "\n" + helper + extension + client,
            "active source is not the exact AL+AM prefix plus checked extension and client")
    return {"complete_source_composition_pinned": True}


def assert_extension(extension: str, generated_extension: str) -> dict[str, Any]:
    require(sha(extension.encode()) == EXTENSION_SHA,
            "AN reclone extension differs from Astra-reviewed source")
    require(extension == generated_extension,
            "generated reclone extension differs from the checked source module")
    names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", AI.mask_noncode(extension))
    expected_names = ["pointer_history_progresses", "load_visible_snapshot", "shallow_clone_arc_checked",
                      "reclone_result", "even_reclone_checked", "odd_reclone_checked",
                      "even_reclone_registration", "odd_reclone_registration", "reclone_root"]
    require(names == expected_names, "AN extension function surface changed")
    # Pin the executable body at each handoff: native Acquire observation,
    # generic relaxed increment plus fresh child registration, two parity
    # callback adapters, and the selected vtable dispatch.
    expected_bodies = {
        "load_visible_snapshot": "{owned_pointer::load_acquire(field,ghost! {|c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {c.shoot_load(&**own,&mut **current);}})}",
        "even_reclone_registration": "{unreachable!(\"checked closed even-table ARC-clone erasure\")}",
        "odd_reclone_registration": "{unreachable!(\"checked closed odd-table ARC-clone erasure\")}",
    }
    for name, body in expected_bodies.items():
        actual, _, _ = AI.find_function(extension, name)
        require(actual == AI.body_from_braced_snippet(body), f"AN callback handoff body changed: {name}")
    even_attrs=[AI.rust_tokens("#[trusted]"),AI.rust_tokens("#[ensures(result.0==even_table())]"),
        AI.rust_tokens("#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]"),
        AI.rust_tokens("#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>> result.1.inner_logic().precondition((data,ptr,len,input))==even_reclone_checked.precondition((data,ptr,len,input)))]"),
        AI.rust_tokens("#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes> result.1.inner_logic().postcondition((data,ptr,len,input),output)==even_reclone_checked.postcondition((data,ptr,len,input),output))]" )]
    odd_attrs=[AI.rust_tokens("#[trusted]"),AI.rust_tokens("#[ensures(result.0==odd_table())]"),
        AI.rust_tokens("#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]"),
        AI.rust_tokens("#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>> result.1.inner_logic().precondition((data,ptr,len,input))==odd_reclone_checked.precondition((data,ptr,len,input)))]"),
        AI.rust_tokens("#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes> result.1.inner_logic().postcondition((data,ptr,len,input),output)==odd_reclone_checked.postcondition((data,ptr,len,input),output))]" )]
    require(AI.function_outer_attributes(extension, "even_reclone_registration") == even_attrs,
            "even native clone callback registration contract changed")
    require(AI.function_outer_attributes(extension, "odd_reclone_registration") == odd_attrs,
            "odd native clone callback registration contract changed")
    require(extension.count("load_visible_snapshot(data,expected,own,current)")==2 and
            extension.count("shallow_clone_arc_checked(stored.cast(),offset,len") == 2 and
            extension.count("erased_call::invoke3(native,(&source.data,source.ptr,source.len)") == 2 and
            "source.vtable.clone" in extension,
            "reclone callback registration and repeated native-vtable invocation are not fully wired")
    shallow, _, _ = AI.find_function(extension, "shallow_clone_arc_checked")
    shallow_attrs = AI.function_outer_attributes(extension, "shallow_clone_arc_checked")
    require(any(has_tokens(a,"Child(p)=>p.core.accepts(^input.inner_logic().1)") for a in shallow_attrs) and
            "field_event :: increment_owned" in " ".join(shallow),
            "checked ARC-clone helper must export child cursor acceptance and one generic relaxed registration")
    return {"source_sha256": sha(extension.encode()), "function_inventory": names,
            "Acquire_relaxed_clone_and_two_callback_routes_checked": True,
            "trusted_registrations_are_exactly_the_two_vtable_bridges": True}


def assert_client(client: str) -> dict[str, Any]:
    require(AI.rust_tokens(client) == AI.rust_tokens(EXPECTED_CLIENT),
            "generated client token stream differs from the complete reviewed three-owner client (including macros)")
    names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", AI.mask_noncode(client))
    require(names == ["promoted_reclone_scope"], "generated client has an open executable function surface")
    require(AI.function_outer_attributes(client, "promoted_reclone_scope") == expected_client_contract(),
            "AN proof client requires/ensures changed")
    return {"whole_client_token_stream_pinned": True, "three_owner_event_order_pinned": True,
            "macro_shadowing_excluded": True}


def derive_edges(mir: str, native: Any) -> tuple[list[dict[str, str]], dict[str, str], dict[str, str]]:
    places = dict(re.findall(r"(?m)^\s*debug\s+(\w+)\s*=>\s*(_\d+)\s*;", mir))
    require(all(name in places for name in ("second", "first", "original")),
            "native client MIR lacks local bindings for all three owning handles")
    blocks = native.mir_blocks(mir, "client")
    edges = []
    return_evals = []
    for (num, cleanup), body in blocks.items():
        if cleanup:
            continue
        for place, successor, unwind in re.findall(
            r"drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?[ ]*([^\]]+)\]", body):
            owner = next((n for n in ("second", "first", "original") if places[n] == place), None)
            if owner:
                edges.append({"block": f"bb{num}", "place": place, "owner": owner,
                              "successor": successor, "unwind": unwind.strip()})
        for statement in re.findall(r"(?m)^\s*(_0\s*=\s*move\s+_\d+;)\s*$", body):
            return_evals.append({"block": f"bb{num}", "statement": statement})
    require([e["owner"] for e in edges] == ["second", "first", "original"],
            "selected native normal edges are not second Drop, first Drop, original Drop")
    require(len(return_evals) == 1, "native trace must evaluate exactly one saved return")
    return edges, places, return_evals[0]


def assert_cargo_probe_manifest(text: str) -> dict[str, Any]:
    try: manifest=tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc: raise CheckError(f"AN Cargo manifest parse failed: {exc}") from exc
    require(manifest.get("package")=={"name":"bytes-original-promotable-reclone","version":"0.1.0","edition":"2021","publish":False} and
        manifest.get("dependencies")=={"creusot-std":"=0.13.0"} and manifest.get("workspace")=={} and
        "lib" not in manifest and "build" not in manifest.get("package",{}),"AN Cargo package/dependency/target route changed")
    return {"package":"bytes-original-promotable-reclone","default_lib_and_build_routes":True}


def assert_mapping(mapping: dict[str, Any], base: str, helper: str, extension: str,
                   client: str, active: str, capture: dict[str, Any], mir: str, native: Any) -> dict[str, Any]:
    fields = {"feature", "status", "stage", "base_source", "base_source_sha256", "terminal_helpers_sha256",
              "extension_source", "extension_sha256", "client_sha256", "active", "active_sha256", "helpers",
              "callbacks", "second_clone_events", "return_evaluation", "excluded", "tcb", "native_source",
              "native_source_sha256", "native_client_mir", "native_mir_ready", "debug_places", "normal_edges", "mir"}
    require(set(mapping) == fields, "AN mapping receipt field set changed")
    require(mapping["feature"] == "" and mapping["status"] == "generated_unchecked" and
            mapping["stage"] == "after-ElaborateDrops", "AN mapping does not denote the positive default elaboration")
    require(mapping["base_source"] == "src/promotion.rs" and mapping["base_source_sha256"] == sha(base.encode()) and
            mapping["terminal_helpers_sha256"] == sha(helper.encode()) and
            mapping["extension_source"] == "src/reclone_extension.rs" and
            mapping["extension_sha256"] == sha(extension.encode()) and
            mapping["client_sha256"] == sha(client.encode()) and mapping["active"] == "generated/active.rs" and
            mapping["active_sha256"] == sha(active.encode()), "AN mapping hashes do not bind exact selected source")
    require(mapping["helpers"] == ["bytes_child_terminal_drop", "bytes_root_terminal_drop"] and
            mapping["callbacks"] == ["even_reclone_checked", "odd_reclone_checked"] and
            mapping["second_clone_events"] == ["Acquire root pointer load", "Relaxed refcount increment",
                                                "fresh child pointer construction"],
            "AN mapping callback or operation interpretation changed")
    require(mapping["excluded"] == ["unwind completion", "concurrent first-promotion loser",
            "arbitrary concurrent closure", "whole crate"] and
            mapping["tcb"] == ["native compiler/MIR and normal terminal-place elaboration",
                "address nonobservation and no independent field-drop glue",
                "AL generic pointer/field/physical/erased-callback boundaries"],
            "AN mapping scope/TCB disclosures changed")
    edges, places, return_eval = derive_edges(mir, native)
    expected_edges = [dict(e, scope="scope", output=("root_receipt" if e["owner"] == "original" else e["owner"] + "_receipt"))
                      for e in edges]
    require(mapping["normal_edges"] == expected_edges and mapping["debug_places"] == places,
            "AN mapping drop edges/debug locals were not independently rederived from native MIR")
    require(mapping["return_evaluation"] == {"shadow": "let saved_return=observed", "root_effect_after_evaluation": True} and
            return_eval == {"block": "bb7", "statement": "_0 = move _9;"},
            "AN saved return point differs from native MIR")
    require(mapping["native_source"] == "native.rs" and mapping["native_source_sha256"] == capture.get("native_source_sha256") and
            mapping["native_client_mir"] == "native-mir/bytes_promotable_reclone_native.promoted_reclone_scope.2-2-004.ElaborateDrops.after.mir" and
            mapping["native_mir_ready"] is True, "AN mapping native source/MIR binding changed")
    rows = capture.get("selected")
    require(isinstance(rows, list) and len(rows) == 19, "AN capture must select exactly 19 MIR inputs")
    expected_mir=[]
    for row in rows:
        require(set(row)=={"label","path","sha256"} and row["path"].startswith("native-mir/"),
                "AN MIR capture row has an open shape or path")
        path=(ROOT/row["path"]).resolve()
        require(path.parent==(ROOT/"native-mir").resolve() and path.is_file() and sha(path.read_bytes())==row["sha256"],
                f"AN captured MIR differs from selected bytes: {row['path']}")
        expected_mir.append({"path":row["path"],"sha256":row["sha256"]})
    require(mapping["mir"]==sorted(expected_mir,key=lambda x:x["path"]), "AN mapping MIR set differs from captured input")
    return {"drop_edges_rederived": expected_edges, "saved_return_rederived": return_eval,
            "selected_mir_count": len(expected_mir)}


def assert_compiled_records(bundle: dict[str, Any]) -> dict[str, Any]:
    try:
        manifest=tomllib.loads((ROOT/"Cargo.toml").read_text())
        production=tomllib.loads((CRATE_ROOT/"Cargo.toml").read_text())
    except tomllib.TOMLDecodeError as exc:
        raise CheckError(f"Cargo manifest parse failed: {exc}") from exc
    assert_cargo_probe_manifest((ROOT/"Cargo.toml").read_text())
    require(production.get("package",{}).get("name")=="bytes" and production.get("package",{}).get("version")=="1.11.1" and
            production.get("package",{}).get("build") is False and production.get("lib",{}).get("path")=="src/lib.rs",
            "production Cargo package route changed")
    build=(ROOT/"build.rs").read_bytes(); extractor=(ROOT/"extract_public.py").read_bytes()
    rerun=AL.assert_build_script_surface(build)
    AL.assert_extractor_source_surface(extractor)
    required=["../../../src/bytes.rs","../../../src/bytes/bytes_record.rs","../../../src/bytes/vtable_record.rs",
              "../../../src/bytes_mut.rs","extract_public.py"]
    require(rerun==required, "AN build rerun input list changed")
    for literal in required:
        expected=ROOT/literal
        if literal!="extract_public.py":
            expected=expected.resolve()
            require(expected.is_file() and expected==((CRATE_ROOT/literal.removeprefix("../../../")).resolve()),
                    f"AN build literal resolves away from production source: {literal}")
    AI.audit_generated_extractions(bundle["production_source"],ROOT/"generated",
        (CRATE_ROOT/"src/bytes/bytes_record.rs").read_text(),
        (CRATE_ROOT/"src/bytes/vtable_record.rs").read_text(),(CRATE_ROOT/"src/bytes_mut.rs").read_text())
    br=(CRATE_ROOT/"src/bytes/bytes_record.rs").read_text(); vr=(CRATE_ROOT/"src/bytes/vtable_record.rs").read_text()
    bm=(CRATE_ROOT/"src/bytes_mut.rs").read_text()
    shared=AI.extract_struct_source(bm,"struct Shared {","BytesMut::Shared")
    mut=AI.extract_struct_source(bm,"pub struct BytesMut {","BytesMut")
    expected=(br+"\n"+vr+"\nmod mutable_record {\nuse alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n"+
              shared+"\n"+mut+"\n}\nuse mutable_record::BytesMut;\n").encode()
    generated=(ROOT/"generated/public_records.rs").read_bytes()
    require(generated==expected,"AN generated public record differs from independent production reconstruction")
    source_map=json.loads((ROOT/"generated/source-map.json").read_text())
    require(source_map.get("generated/public_records.rs",{}).get("sha256")==sha(generated),
            "AN source-map receipt does not bind generated record")
    target=pathlib.Path(os.environ["CARGO_TARGET_DIR"]).resolve() if os.environ.get("CARGO_TARGET_DIR") else \
        (ROOT.parents[5]/"bytes-proof-tools/targets/bytes").resolve()
    fps=list((target/"debug/.fingerprint").glob("bytes-original-promotable-reclone-*/run-build-script-build-script-build.json"))
    require(len(fps)==1,"expected exactly one AN Cargo build fingerprint")
    fp=json.loads(fps[0].read_text())
    records=[e["RerunIfChanged"] for e in fp.get("local",[]) if isinstance(e,dict) and "RerunIfChanged" in e]
    require(len(records)==1 and records[0].get("paths")==required,"AN compiled build inputs differ from exact source rerun list")
    output_rel=pathlib.Path(records[0].get("output",""))
    require(not output_rel.is_absolute() and output_rel.parts[:2]==("debug","build"),"AN Cargo output route escaped target")
    output=(target/output_rel).resolve(); outdir=output.parent
    require(fps[0].parent.name==outdir.name and output.is_file(),"AN Cargo fingerprint/output identity mismatch")
    directives=(["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)","cargo:rustc-cfg=bytes_original_shared_gate"]+
        ["cargo:rerun-if-changed="+p for p in required])
    require(output.read_bytes()==("\n".join(directives)+"\n").encode(),"AN build output directives changed")
    root_output=outdir/"root-output"; compiled=outdir/"out/public_records.rs"
    root_output_bytes=root_output.read_bytes(); compiled_bytes=compiled.read_bytes()
    require(root_output_bytes==str(outdir/"out").encode() and compiled_bytes==expected==generated,
            "AN actual Cargo OUT_DIR record does not match reconstructed production record")
    capture_dir=ROOT/"generated/compiled-inputs"
    capture_dir.mkdir(parents=True,exist_ok=True)
    artifacts={"public_records.rs":compiled_bytes,"cargo-run-build-fingerprint.json":fps[0].read_bytes(),
        "cargo-build-output.txt":output.read_bytes(),"cargo-root-output.txt":root_output_bytes}
    for name,data in artifacts.items(): (capture_dir/name).write_bytes(data)
    receipt={"status":"pass","probe_package":"bytes-original-promotable-reclone",
        "build_script_sha256":sha((ROOT/"build.rs").read_bytes()),
        "extractor_sha256":sha((ROOT/"extract_public.py").read_bytes()),
        "cargo_target_dir":str(target),"cargo_build_fingerprint":str(fps[0]),
        "cargo_build_fingerprint_sha256":sha(artifacts["cargo-run-build-fingerprint.json"]),
        "captured_cargo_build_fingerprint_path":"generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_cargo_build_fingerprint_sha256":sha(artifacts["cargo-run-build-fingerprint.json"]),
        "build_output_path":str(output),"build_output_sha256":sha(artifacts["cargo-build-output.txt"]),
        "captured_build_output_path":"generated/compiled-inputs/cargo-build-output.txt",
        "captured_build_output_sha256":sha(artifacts["cargo-build-output.txt"]),
        "root_output_path":str(root_output),"root_output_sha256":sha(root_output_bytes),
        "captured_root_output_path":"generated/compiled-inputs/cargo-root-output.txt",
        "captured_root_output_sha256":sha(root_output_bytes),
        "actual_out_dir":str(compiled.parent),"compiled_input_path":str(compiled),
        "compiled_input_sha256":sha(compiled_bytes),
        "captured_input_path":"generated/compiled-inputs/public_records.rs",
        "captured_input_sha256":sha(artifacts["public_records.rs"]),
        "reconstructed_and_generated_input_sha256":sha(expected),
        "source_map_sha256":sha((ROOT/"generated/source-map.json").read_bytes()),
        "production_rerun_inputs":required}
    (capture_dir/"public-records-build-receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    return {"AN_Cargo_identity_and_build_route":True,"production_record_reconstructed":True,
            "actual_OUT_DIR_record_matched":True,"build_fingerprint":str(fps[0]),
            "build_fingerprint_sha256":sha(fps[0].read_bytes()),"compiled_record_sha256":sha(compiled.read_bytes())}


def audit() -> dict[str, Any]:
    frozen=assert_frozen_prefix()
    base=(ROOT/"src/promotion.rs").read_text()
    helper=HELPERS.read_text(); extension=EXTENSION.read_text(); client=CLIENT.read_text(); active=ACTIVE.read_text()
    assert_active_composition(base,helper,extension,client,active)
    helper_check=assert_helpers(helper,active)
    extension_check=assert_extension(extension,(ROOT/"generated/reclone-extension.rs").read_text())
    client_check=assert_client(client)
    route=assert_module_route((ROOT/"src/lib.rs").read_text(),active)
    require(sha(NATIVE_CHECKER.read_bytes())==NATIVE_CHECKER_SHA,
            "AN native checker changed after reviewed native-controls run")
    native=load_module("an_native_correspondence",NATIVE_CHECKER)
    bundle=native.load_bundle(); native_result=native.audit_bundle(bundle)
    require(native_result.get("status")=="pass","AN actual native source/MIR checker did not pass")
    reviewed=native_result.get("reviewed_production_inputs",{})
    require(reviewed=={"manifest_sha256":REVIEWED_PRODUCTION_SHA,"base_commit":"361c7cd261507ac0a705b3b836f73240070891c6",
        "source_files":61,"manifest_and_lock_pinned":True,"global_import_macro_and_include_bindings_frozen":True},
        "AN native audit omitted complete reviewed production input/import/include binding")
    require(native_result.get("terminal_field_profile",{}).get("compile_time_no_independent_field_drop_glue") is True,
            "AN native fields have unreviewed independent drop glue")
    reviewed_modules=AL.assert_reviewed_token_surfaces()
    support_adapters=AL.assert_support_adapters((ROOT/"src/lib.rs").read_text(),active,bundle)
    AL.assert_proof_state_types(base)
    AL.assert_trust_boundary(base)
    AL.assert_closed_function_surface(base)
    mapping_check=assert_mapping(json.loads(MAPPING.read_text()),base,helper,extension,client,active,
        bundle["capture"],bundle["mir_sources"]["client"],native)
    compiled=assert_compiled_records(bundle)
    return {"status":"pass","scope":"AN generated proof-source/native correspondence for selected three-owner normal-return trace",
        "frozen_prefix":frozen,"terminal_adapters":helper_check,"extension":extension_check,"client":client_check,
        "module_route":route,"native_audit":native_result,"reviewed_production_bindings":reviewed,
        "generic_support_audit":{"reviewed_module_surfaces":reviewed_modules,"support_adapters":support_adapters,
            "proof_state_types_and_trust_boundary_checked":True},
        "mapping_reconstruction":mapping_check,"compiled_inputs":compiled,
        "limitations":["does not prove Creusot/Why3 obligations","normal Drop/MIR adequacy, Ghost erasure, pointer provenance, allocator and runtime semantics remain TCB",
                       "unwind completion, concurrent first-promotion loser, arbitrary callback closure, and whole-crate behavior excluded"]}


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow","--active",dest="shadow",type=pathlib.Path,default=ACTIVE)
    parser.add_argument("--output",type=pathlib.Path)
    args=parser.parse_args()
    try:
        require(args.shadow.resolve()==ACTIVE.resolve(),"selected shadow override must be the reviewed live generated/active.rs route")
        result=audit()
    except Exception as exc: result={"status":"reject","scope":"AN source/native correspondence","reason":str(exc)}
    rendered=json.dumps(result,indent=2)+"\n"
    if args.output: args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_text(rendered)
    print(rendered,end="")
    return 0 if result.get("status")=="pass" else 1


if __name__=="__main__": raise SystemExit(main())
