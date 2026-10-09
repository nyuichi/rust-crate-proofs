#!/usr/bin/env python3
"""Independent AO checker for surviving-child normal Drop correspondence."""
from __future__ import annotations
import argparse, hashlib, importlib.util, json, pathlib, re, sys
from typing import Any

ROOT=pathlib.Path(__file__).resolve().parent
PROBES=ROOT.parent
AN_ROOT=PROBES/"original-promotable-reclone-2026-10-09"
AM_ROOT=PROBES/"original-promotable-automatic-drop-2026-10-09"
AL_ROOT=PROBES/"original-promotable-first-clone-2026-10-09"
AI_ROOT=PROBES/"original-shared-scoped-client-2026-10-09"
ACTIVE=ROOT/"generated/active.rs"
MAPPING=ROOT/"generated/mapping.json"
CLIENT=ROOT/"generated/elaborated-client.rs"
NATIVE_MIR=ROOT/"native-mir/bytes_promotable_surviving_child_native.promoted_surviving_child_scope.2-2-004.ElaborateDrops.after.mir"
AN_CHECKER_SHA="28f66c83e8c900aad0e5c9b2e937ff9d661bf384d354b66e695a06cb2493b89d"
PUBLISHED_PYTHON_INPUTS={
    AM_ROOT/"check_correspondence.py":"77acbf4e9aefd0d68decec909a456b823644debc74475d57d5cdeea792765968",
    AM_ROOT/"check_native.py":"e83283feb8d546a5c9e0e2a91188b6f9032c86bc7bcc37a32c83ad2ecf780b68",
    AL_ROOT/"check_correspondence.py":"9d876d21f712441d71cc309411a853874c852c139219363b0c71a621da480867",
    AL_ROOT/"check_native.py":"91adb92c5ed5ff27f5c6169ff770ee01cf368febf1dd5b38b5371167cd446d88",
    AI_ROOT/"check_correspondence.py":"decb502490fdd1c534e02b5ac0798b070eeb8ed7be4d6aa1779e05cee5e08bb7",
}

def assert_published_python_lineage(overrides:dict[pathlib.Path,bytes]|None=None)->dict[str,str]:
    overrides=overrides or {}; result={}
    for checker_path,expected_sha in PUBLISHED_PYTHON_INPUTS.items():
        data=overrides.get(checker_path,checker_path.read_bytes() if checker_path.is_file() else b"")
        require(checker_path.is_file() and sha(data)==expected_sha,
            f"published ancestor checker input changed before import: {checker_path.name}")
        result[checker_path.name]=expected_sha
    return result
AN_PREFIX_SHA="35c37cca2f01ddc1e12e57de6a2ee5959ea42744ddd7bb6ee9f716c85c65e94c"
AO_EXTENSION_SHA="768dcce2a6d539921360668622c1de8a4d11c3259c7a92365c4b44fb581b3ec0"
AO_CLIENT_SHA="22798c3acd302c74463a895d94637afea99ebb908c2192da073e3044638d38fb"
AO_NATIVE_CHECKER_SHA="efbc05ff8643606a822da10dae8eed139679a51e2c67e2613a608c88f6d7506b"
REVIEWED_PRODUCTION_SHA="e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306"

EXPECTED_CLIENT=r"""/// Inner original Drop publishes recovery; the surviving child's final Drop
/// recovers it. No original owner or root permission survives the handoff.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_surviving_child_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!(!(*detached.observation()).0.contains(*root_id));
    proof_assert!((*detached.observation()).0.contains(*child_id));
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!(detached.accepts(survivor));
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}"""

EXPECTED_NATIVE=r"""use bytes::Bytes;

/// Original owner retires first; child survives, reads and finally retires.
pub fn promoted_surviving_child_scope(input: Box<[u8]>) -> Vec<u8> {
    let survivor = {
        let original = Bytes::from(input);
        original.clone()
    };
    let observed = AsRef::<[u8]>::as_ref(&survivor).to_vec();
    observed
}"""

class CheckError(RuntimeError): pass
def require(ok:bool,message:str)->None:
    if not ok: raise CheckError(message)
def sha(data:bytes)->str: return hashlib.sha256(data).hexdigest()

require(sha((AN_ROOT/"check_correspondence.py").read_bytes())==AN_CHECKER_SHA,
        "published AN checker changed before reuse")
assert_published_python_lineage()
spec=importlib.util.spec_from_file_location("ao_frozen_an_checker",AN_ROOT/"check_correspondence.py")
require(spec is not None and spec.loader is not None,"cannot load pinned AN checker")
AN=importlib.util.module_from_spec(spec); sys.modules[spec.name]=AN; spec.loader.exec_module(AN)


def derive_terminal_edges(mir:str)->tuple[list[dict[str,str]],dict[str,str],dict[str,str]]:
    debug=dict(re.findall(r"(?m)^\s*debug\s+(\w+)\s*=>\s*(_\d+)\s*;",mir))
    require("original" in debug and "survivor" in debug,
            "AO native MIR has no source-local identities for original and survivor")
    blocks={}
    for match in re.finditer(r"(?m)^\s*bb(\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)^\s*\}",mir,re.S):
        blocks[("bb"+match.group(1),bool(match.group(2)))]=match.group(3)
    edges=[]; returns=[]
    for (block,cleanup),body in blocks.items():
        if cleanup: continue
        for place,successor,unwind in re.findall(
            r"drop\((_[0-9]+)\)\s*->\s*\[return:\s*(bb[0-9]+),\s*unwind:\s*([^\]]+)\]",body):
            owner=next((name for name in ("original","survivor") if debug[name]==place),None)
            if owner: edges.append({"block":block,"place":place,"owner":owner,
                "successor":successor,"unwind":unwind.strip()})
        for stmt in re.findall(r"(?m)^\s*(_0\s*=\s*move\s+_[0-9]+;)\s*$",body):
            returns.append({"block":block,"statement":stmt})
    require([x["owner"] for x in edges]==["original","survivor"],
            "native normal Drop edges must retire original first and the surviving child last")
    require(len(returns)==1,"native return Vec must be evaluated exactly once")
    require(edges[0]["place"]=="_3" and edges[0]["block"]=="bb2" and edges[0]["successor"]=="bb3" and
            edges[1]["place"]=="_2" and edges[1]["block"]=="bb6" and edges[1]["successor"]=="bb7" and
            returns[0]=={"block":"bb5","statement":"_0 = move _6;"},
            "AO native drop places/order or return-before-final-child point changed")
    return edges,debug,returns[0]


def audit_native_prefix()->dict[str,Any]:
    prefix=AN_ROOT/"generated/positive.rs"
    require(prefix.is_file() and sha(prefix.read_bytes())==AN_PREFIX_SHA,
            "published AN full active prefix changed")
    require(NATIVE_MIR.is_file(),"AO selected native ElaborateDrops MIR is missing")
    mir=NATIVE_MIR.read_text()
    edges,debug,returned=derive_terminal_edges(mir)
    capture=json.loads((ROOT/"native-mir/capture.json").read_text())
    require(capture.get("native_source_sha256")==sha((ROOT/"native.rs").read_bytes()),
            "AO native capture does not bind the selected client source")
    matches=[x for x in capture.get("selected",[]) if x.get("path")==NATIVE_MIR.relative_to(ROOT).as_posix()]
    require(len(matches)==1 and matches[0].get("sha256")==sha(NATIVE_MIR.read_bytes()),
            "AO capture does not uniquely bind selected client MIR bytes")
    return {"frozen_AN_prefix_sha256":AN_PREFIX_SHA,"native_client_source_sha256":sha((ROOT/"native.rs").read_bytes()),
        "native_mir_sha256":sha(NATIVE_MIR.read_bytes()),"normal_drop_edges":edges,
        "debug_places":debug,"return_evaluation":returned,
        "interpretation":{"original":"nonfinal root retirement; surviving child remains readable",
            "survivor":"final child retirement after saved Vec return evaluation"}}


def assert_paired_inherited_sources(an_source_overrides:dict[str,bytes]|None=None,
                                    ao_source_overrides:dict[str,bytes]|None=None)->dict[str,Any]:
    an_source_overrides=an_source_overrides or {}; ao_source_overrides=ao_source_overrides or {}
    manifest=json.loads(AN.AM.FROZEN_MANIFEST.read_text())
    frozen_names=set(manifest["rust_modules"])
    pinned={name:an_source_overrides.get(name,(AN_ROOT/"src"/name).read_bytes()) for name in frozen_names}
    AN.AM.assert_frozen_al_sources(pinned)
    expected={p.name for p in (AN_ROOT/"src").glob("*.rs")}
    require(expected=={p.name for p in (ROOT/"src").glob("*.rs") if p.name!="surviving_extension.rs"},
            "AO inherited module inventory changed or an expected source module is missing")
    for name in sorted(expected):
        old=an_source_overrides.get(name,(AN_ROOT/"src"/name).read_bytes())
        current=ao_source_overrides.get(name,(ROOT/"src"/name).read_bytes())
        if name!="promotion.rs": require(current==old,f"AO inherited AN proof/support module changed: {name}")
    return {"inherited_module_count":len(expected),"AM_frozen_modules_anchored":len(frozen_names),
        "all_AO_descendant_modules_match_AN":True}


def audit_inherited_an_sources(an_source_overrides:dict[str,bytes]|None=None,
                              ao_source_overrides:dict[str,bytes]|None=None,
                              lib_override:str|None=None,
                              active_override:str|None=None)->dict[str,Any]:
    an_source_overrides=an_source_overrides or {}
    ao_source_overrides=ao_source_overrides or {}
    if not an_source_overrides and not ao_source_overrides: AN.assert_frozen_prefix()
    paired=assert_paired_inherited_sources(an_source_overrides,ao_source_overrides)
    prefix=(AN_ROOT/"generated/positive.rs").read_bytes()
    require((ROOT/"src/promotion.rs").read_bytes()==prefix,
            "AO selected promotion prefix differs from complete positive AN active source")
    require(sha(prefix)==AN_PREFIX_SHA,"frozen AN active-prefix identity changed")
    lib=lib_override if lib_override is not None else (ROOT/"src/lib.rs").read_text()
    require(lib==(AN_ROOT/"src/lib.rs").read_text(),"AO lib.rs diverged from inherited AN module and support routes")
    active=active_override if active_override is not None else ACTIVE.read_text()
    module_route=AN.assert_module_route(lib,active)
    imported=AN.assert_imported_support_sources(lib)
    return {"whole_AN_active_prefix_sha256":AN_PREFIX_SHA,"inherited_module_count":paired["inherited_module_count"],
        "all_inherited_modules_byte_exact":True,"frozen_prefix_anchored_to_AM":True,
        "frozen_ancestor_module_count":paired["AM_frozen_modules_anchored"],
        "inherited_module_route_rechecked":True,"route_audit":module_route,"imported_support_sha256":imported}


def assert_client_tokens(client:str)->dict[str,Any]:
    require(sha(client.encode())==AO_CLIENT_SHA and AN.AI.rust_tokens(client)==AN.AI.rust_tokens(EXPECTED_CLIENT),
            "AO generated client differs from its complete reviewed token stream, including macros")
    names=re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b",AN.AI.mask_noncode(client))
    require(names==["promoted_surviving_child_scope"],"AO client has an open executable function surface")
    require(AN.AI.function_outer_attributes(client,"promoted_surviving_child_scope")==[
        AN.AI.rust_tokens("#[requires(input@.len()>0)]"),AN.AI.rust_tokens("#[ensures(result@==input@)]")],
        "AO proof client admission/result contract changed")
    return {"whole_client_token_stream_pinned":True,"macro_shadowing_excluded":True,
        "original_drop_then_survivor_read_and_final_drop":True}


def assert_extension(extension:str)->dict[str,Any]:
    require(sha(extension.encode())==AO_EXTENSION_SHA,
            "AO detached-scope extension differs from Astra-reviewed source")
    names=re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b",AN.AI.mask_noncode(extension))
    expected=["observation","model","public","accepts","detaching_root_drop_checked",
        "even_detaching_root_registration","odd_detaching_root_registration",
        "bytes_root_detaching_terminal_drop","read_surviving_child","bytes_detached_child_terminal_drop"]
    require(names==expected,"AO surviving-child extension function surface changed")
    detached=AN.AI.extract_struct_source(extension,"struct DetachedScope {","DetachedScope")
    require(AN.AI.rust_tokens(detached)==AN.AI.rust_tokens("struct DetachedScope { cursor:Cursor }"),
            "DetachedScope must carry only the actual affine ScopeCursor")
    trusted={name for name in expected if AN.AI.function_outer_attributes(extension,name) and
             AN.AI.rust_tokens("#[trusted]") in AN.AI.function_outer_attributes(extension,name)}
    require(trusted=={"even_detaching_root_registration","odd_detaching_root_registration"},
            "AO added trust outside the two exact registered native root-drop callbacks")
    require("get_mut_finish(data,own)" in extension and "release_core(word.cast(),root,cursor.borrow_mut(),output)" in extension and
            "DetachedScope {cursor:cursor.into_inner()}" in extension and
            "value.vtable.drop" in extension and "erased_call::invoke3(native,(&mut value.data,value.ptr,value.len)" in extension,
            "AO root retirement no longer consumes the root core and exports only the cursor through the native drop route")
    require("full.borrow(&child.core.ticket.token)" in extension and
            "physical_projection::borrow(value.ptr,value.len,ghost! {&child.core.bound},region)" in extension,
            "AO surviving-child read no longer uses the child's own physical ticket/bound pointer")
    require("child_drop_registration()" in extension and
            "erased_call::invoke3(native,(&mut value.data,value.ptr,value.len)" in extension,
            "AO final child retirement is not wired to the inherited native drop callback")
    return {"sha256":sha(extension.encode()),"function_inventory":names,
        "detached_scope_cursor_only":True,"root_drop_consumes_core_exports_cursor":True,
        "child_own_ticket_read_and_final_drop_checked":True}


def assert_active_composition(prefix:str,extension:str,client:str,active:str)->dict[str,Any]:
    require(active==prefix+"\n"+extension+client,
            "AO active module is not the exact complete AN prefix plus checked detached extension and client")
    require(not re.search(r"\bimpl\s+Drop\s+for\s+Bytes\b",AN.AI.mask_noncode(active)),
            "AO proof shadow invented an implicit Bytes destructor")
    return {"active_composition_exact":True,"no_shadow_Bytes_Drop":True}


def assert_ao_module_route(lib:str,active:str)->dict[str,Any]:
    expected=(AN_ROOT/"src/lib.rs").read_text()
    route='# [cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;'
    # Compare raw source to inherited AN root; this pins every cfg/path/support
    # declaration, including literal paths and module attributes.
    require(lib==expected,"AO lib.rs changed the inherited AN route or support-module set")
    require(AN.AI.rust_path_for_module(lib,"promotion")=="../generated/active.rs" and
            ACTIVE.resolve()==(ROOT/"src/../generated/active.rs").resolve(),
            "AO selected promotion source is not the live generated/active.rs file")
    return {"lib_byte_exact_to_AN":True,"active_route":"../generated/active.rs"}


def assert_mapping(mapping:dict[str,Any],prefix:str,extension:str,client:str,active:str,
                   capture:dict[str,Any],mir:str,native:Any)->dict[str,Any]:
    keys={"feature","status","stage","base_source","base_source_sha256","terminal_helpers_sha256",
        "extension_source","extension_sha256","client_sha256","active","active_sha256","helpers","callbacks",
        "ownership_handoff","return_evaluation","excluded","tcb","native_source","native_source_sha256",
        "native_client_mir","native_mir_ready","debug_places","normal_edges","mir"}
    require(set(mapping)==keys,"AO mapping fields changed")
    require(mapping["feature"]=="" and mapping["status"]=="generated_unchecked" and mapping["stage"]=="after-ElaborateDrops",
            "AO mapping is not the positive default feature after drop elaboration")
    require(mapping["base_source"]=="src/promotion.rs" and mapping["base_source_sha256"]==sha(prefix.encode()) and
        mapping["terminal_helpers_sha256"]==sha(extension.encode()) and mapping["extension_source"]=="src/surviving_extension.rs" and
        mapping["extension_sha256"]==sha(extension.encode()) and mapping["client_sha256"]==sha(client.encode()) and
        mapping["active"]=="generated/active.rs" and mapping["active_sha256"]==sha(active.encode()),
        "AO mapping hashes do not bind the selected exact source composition")
    require(mapping["helpers"]==["bytes_root_detaching_terminal_drop","bytes_detached_child_terminal_drop"] and
        mapping["callbacks"]==["detaching_root_drop_checked","child_drop_checked"] and
        mapping["ownership_handoff"]=="consume root core and owned pointer permission; export only actual affine cursor",
        "AO callback/handoff mapping changed")
    require(mapping["return_evaluation"]=={"shadow":"let saved_return=observed","survivor_effect_after_evaluation":True} and
        mapping["excluded"]==["unwind completion","concurrent first-promotion loser","arbitrary concurrent closure","whole crate"] and
        mapping["tcb"]==["native compiler/MIR and normal terminal-place elaboration","address nonobservation and no independent field-drop glue",
                         "inherited generic pointer/field/physical/erased-callback boundaries"],
        "AO map return point, exclusions, or TCB changed")
    edges,debug,returned=derive_terminal_edges(mir)
    expected=[dict(edges[0],scope="scope",output="root_receipt"),dict(edges[1],scope="detached",output="child_receipt")]
    require(mapping["normal_edges"]==expected and mapping["debug_places"]==debug,
        "AO original/survivor Drop places or scope/output mapping differ from actual native MIR")
    require(returned=={"block":"bb5","statement":"_0 = move _6;"} and mapping["native_mir_ready"] is True,
        "AO saved return point or native MIR readiness changed")
    require(mapping["native_source"]=="native.rs" and mapping["native_source_sha256"]==capture.get("native_source_sha256") and
        mapping["native_client_mir"]==NATIVE_MIR.relative_to(ROOT).as_posix(),"AO native source/client MIR route mismatch")
    selected=capture.get("selected"); require(isinstance(selected,list) and len(selected)==19,"AO capture must list 19 MIR sources")
    rows=[]
    for row in selected:
        require(set(row)=={"label","path","sha256"} and row["path"].startswith("native-mir/"),"AO MIR receipt row malformed")
        path=(ROOT/row["path"]).resolve()
        require(path.parent==(ROOT/"native-mir").resolve() and path.is_file() and sha(path.read_bytes())==row["sha256"],
            f"AO selected MIR hash/path mismatch: {row['path']}")
        rows.append({"path":row["path"],"sha256":row["sha256"]})
    require(mapping["mir"]==sorted(rows,key=lambda r:r["path"]),"AO mapping MIR list differs from actual selected capture")
    return {"normal_edges_rederived":expected,"return_evaluation_rederived":returned,"selected_mir_count":len(rows)}


def assert_ao_probe_manifest(probe:dict[str,Any])->dict[str,Any]:
    expected={"package":{"name":"bytes-original-promotable-surviving-child","version":"0.1.0","edition":"2021","publish":False},
        "dependencies":{"creusot-std":"=0.13.0"},"workspace":{},
        "features":{"negative_missing_acquire":[],"negative_missing_payload_free":[],"negative_missing_control_free":[]}}
    require(probe==expected,"AO Cargo manifest/default dependency and target route changed")
    return {"package_and_default_dependency_route_exact":True,"only_reviewed_negative_features_present":True}


def assert_compiled_records(bundle:dict[str,Any])->dict[str,Any]:
    import os, tomllib
    try:
        probe=tomllib.loads((ROOT/"Cargo.toml").read_text())
        production=tomllib.loads((AN.CRATE_ROOT/"Cargo.toml").read_text())
    except tomllib.TOMLDecodeError as exc: raise CheckError(f"AO Cargo manifest parse failed: {exc}") from exc
    assert_ao_probe_manifest(probe)
    require(production.get("package",{}).get("name")=="bytes" and production.get("package",{}).get("version")=="1.11.1" and
        production.get("package",{}).get("build") is False and production.get("lib",{}).get("path")=="src/lib.rs",
        "production Cargo target route changed")
    build=(ROOT/"build.rs").read_bytes(); extractor=(ROOT/"extract_public.py").read_bytes()
    rerun=AN.AL.assert_build_script_surface(build); AN.AL.assert_extractor_source_surface(extractor)
    expected_paths=["../../../src/bytes.rs","../../../src/bytes/bytes_record.rs","../../../src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs","extract_public.py"]
    require(rerun==expected_paths,"AO build script rerun paths changed")
    inputs={
        "../../../src/bytes.rs":AN.CRATE_ROOT/"src/bytes.rs",
        "../../../src/bytes/bytes_record.rs":AN.CRATE_ROOT/"src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs":AN.CRATE_ROOT/"src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs":AN.CRATE_ROOT/"src/bytes_mut.rs",
        "extract_public.py":ROOT/"extract_public.py",
    }
    source_hashes={}
    for literal,path in inputs.items():
        resolved=(ROOT/literal).resolve()
        require(resolved==path.resolve() and resolved.is_file(),f"AO build path resolves outside reviewed input: {literal}")
        source_hashes[literal]=sha(resolved.read_bytes())
    record_source=(AN.CRATE_ROOT/"src/bytes/bytes_record.rs").read_text()
    vtable_source=(AN.CRATE_ROOT/"src/bytes/vtable_record.rs").read_text()
    mutable_source=(AN.CRATE_ROOT/"src/bytes_mut.rs").read_text()
    AN.AI.audit_generated_extractions(bundle["production_source"],ROOT/"generated",record_source,vtable_source,mutable_source)
    shared=AN.AI.extract_struct_source(mutable_source,"struct Shared {","BytesMut::Shared")
    bytesmut=AN.AI.extract_struct_source(mutable_source,"pub struct BytesMut {","BytesMut")
    expected=(record_source+"\n"+vtable_source+"\nmod mutable_record {\nuse alloc::vec::Vec;\n"
        "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n"+shared+"\n"+bytesmut+"\n}\nuse mutable_record::BytesMut;\n").encode()
    generated_path=ROOT/"generated/public_records.rs"; map_path=ROOT/"generated/source-map.json"
    generated=generated_path.read_bytes(); smap=json.loads(map_path.read_text())
    require(generated==expected and smap.get("generated/public_records.rs",{}).get("sha256")==sha(generated),
        "AO translated production-record bytes differ from independent reconstruction/source-map")
    target=pathlib.Path(os.environ["CARGO_TARGET_DIR"]).resolve() if os.environ.get("CARGO_TARGET_DIR") else \
        (ROOT.parents[5]/"bytes-proof-tools/targets/bytes").resolve()
    fingerprints=list((target/"debug/.fingerprint").glob(
        "bytes-original-promotable-surviving-child-*/run-build-script-build-script-build.json"))
    require(len(fingerprints)==1,"AO requires exactly one actual Cargo build fingerprint")
    fp_path=fingerprints[0]; fp_bytes=fp_path.read_bytes(); fp=json.loads(fp_bytes)
    rows=[r["RerunIfChanged"] for r in fp.get("local",[]) if isinstance(r,dict) and "RerunIfChanged" in r]
    require(len(rows)==1 and rows[0].get("paths")==expected_paths,"AO Cargo fingerprint does not pin reviewed build inputs")
    output_rel=pathlib.Path(rows[0].get("output",""))
    require(not output_rel.is_absolute() and output_rel.parts[:2]==("debug","build"),"AO build output escaped Cargo target")
    output=(target/output_rel).resolve(); outdir=output.parent
    require(fp_path.parent.name==outdir.name and output.is_file(),"AO Cargo build fingerprint/output identity mismatch")
    expected_directives=(
        ["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)","cargo:rustc-cfg=bytes_original_shared_gate"]+
        ["cargo:rerun-if-changed="+p for p in expected_paths])
    output_bytes=output.read_bytes()
    require(output_bytes==("\n".join(expected_directives)+"\n").encode(),"AO build output directives changed")
    root_output=outdir/"root-output"; compiled=outdir/"out/public_records.rs"
    root_bytes=root_output.read_bytes(); compiled_bytes=compiled.read_bytes()
    require(root_bytes==str(outdir/"out").encode() and compiled_bytes==expected==generated,
        "AO actual Cargo OUT_DIR/public_records.rs differs from reviewed source reconstruction")
    capture_dir=ROOT/"generated/compiled-inputs"; capture_dir.mkdir(parents=True,exist_ok=True)
    artifacts={"public_records.rs":compiled_bytes,"cargo-run-build-fingerprint.json":fp_bytes,
        "cargo-build-output.txt":output_bytes,"cargo-root-output.txt":root_bytes}
    for name,data in artifacts.items(): (capture_dir/name).write_bytes(data)
    receipt={"status":"pass","probe_package":"bytes-original-promotable-surviving-child",
        "build_script_sha256":sha(build),"extractor_sha256":sha(extractor),"cargo_target_dir":str(target),
        "cargo_build_fingerprint":str(fp_path),"cargo_build_fingerprint_sha256":sha(fp_bytes),
        "captured_cargo_build_fingerprint_path":"generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_cargo_build_fingerprint_sha256":sha(artifacts["cargo-run-build-fingerprint.json"]),
        "build_output_path":str(output),"build_output_sha256":sha(output_bytes),
        "captured_build_output_path":"generated/compiled-inputs/cargo-build-output.txt",
        "captured_build_output_sha256":sha(artifacts["cargo-build-output.txt"]),
        "root_output_path":str(root_output),"root_output_sha256":sha(root_bytes),
        "captured_root_output_path":"generated/compiled-inputs/cargo-root-output.txt",
        "captured_root_output_sha256":sha(artifacts["cargo-root-output.txt"]),
        "actual_out_dir":str(compiled.parent),"compiled_input_path":str(compiled),"compiled_input_sha256":sha(compiled_bytes),
        "captured_input_path":"generated/compiled-inputs/public_records.rs","captured_input_sha256":sha(compiled_bytes),
        "reconstructed_generated_sha256":sha(expected),"source_map_sha256":sha(map_path.read_bytes()),
        "production_rerun_input_sha256":source_hashes}
    (capture_dir/"public-records-build-receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    return {"actual_Cargo_OUT_DIR_record_matches_reconstruction":True,"compiled_input_receipt":receipt}


def audit()->dict[str,Any]:
    native=audit_native_prefix()
    inherited=audit_inherited_an_sources()
    require(ACTIVE.is_file() and CLIENT.is_file() and MAPPING.is_file(),
            "AO generated client/source/mapping inputs are not ready for correspondence audit")
    prefix=(ROOT/"src/promotion.rs").read_text()
    extension=(ROOT/"src/surviving_extension.rs").read_text()
    generated_extension=(ROOT/"generated/surviving-extension.rs").read_text()
    helper=(ROOT/"generated/terminal-helper.rs").read_text()
    active=ACTIVE.read_text(); client=CLIENT.read_text(); mapping=json.loads(MAPPING.read_text())
    require(AN.AI.rust_tokens((ROOT/"native.rs").read_text())==AN.AI.rust_tokens(EXPECTED_NATIVE),
            "AO native client differs from the complete reviewed inner-drop/read/survivor-drop witness")
    client_check=assert_client_tokens(client)
    extension_check=assert_extension(extension)
    require(generated_extension==extension and helper==extension,
            "AO generated terminal extension is not the reviewed single source module")
    composition=assert_active_composition(prefix,extension,client,active)
    route=assert_ao_module_route((ROOT/"src/lib.rs").read_text(),active)
    require(sha((ROOT/"check_native.py").read_bytes())==AO_NATIVE_CHECKER_SHA,
            "AO native correspondence checker changed after its independent-controls freeze")
    native_checker=AN.load_module("ao_native_correspondence",ROOT/"check_native.py")
    bundle=native_checker.load_bundle()
    native_audit=native_checker.audit_bundle(bundle)
    require(native_audit.get("status")=="pass","AO native source/MIR correspondence audit failed")
    reviewed=native_audit.get("reviewed_production_inputs",{})
    require(reviewed=={"manifest_sha256":REVIEWED_PRODUCTION_SHA,"base_commit":"361c7cd261507ac0a705b3b836f73240070891c6",
        "source_files":61,"manifest_and_lock_pinned":True,"global_import_macro_and_include_bindings_frozen":True},
        "AO native audit omitted complete reviewed production source/import/include binding")
    require(native_audit.get("terminal_field_profile",{}).get("compile_time_no_independent_field_drop_glue") is True,
        "AO native Bytes field profile did not rule out independent field drop glue")
    reviewed_modules=AN.AL.assert_reviewed_token_surfaces()
    support_adapters=AN.AL.assert_support_adapters((ROOT/"src/lib.rs").read_text(),active,bundle)
    AN.AL.assert_proof_state_types(prefix)
    mapping_check=assert_mapping(mapping,prefix,extension,client,active,bundle["capture"],
        bundle["mir_sources"]["client"],native_checker)
    compiled=assert_compiled_records(bundle)
    return {"status":"pass","scope":"AO actual native source/MIR and proof-shadow correspondence for the original-nonfinal/readable-child/final-child normal-return trace",
        "published_checker_lineage":assert_published_python_lineage(),
        "native_trace":native,"inherited_an_sources":inherited,"native_audit":native_audit,
        "reviewed_production_bindings":reviewed,"generic_support_audit":{"reviewed_module_surfaces":reviewed_modules,
            "support_adapters":support_adapters,"inherited_proof_prefix_exactly_pinned":True},
        "native_client":client_check,"extension":extension_check,"active_composition":composition,
        "module_route":route,"mapping_reconstruction":mapping_check,"compiled_inputs":compiled,
        "limitations":["Does not prove Creusot/Why3 obligations.",
            "MIR terminal-place adequacy, Rust drop semantics, Ghost erasure, allocator behavior, and pointer provenance remain TCB.",
            "Unwind completion, concurrent execution, whole-crate behavior, and arbitrary callback closure are excluded."]}


def main()->int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", "--active", dest="shadow", type=pathlib.Path, default=ACTIVE)
    parser.add_argument("--output",type=pathlib.Path)
    args=parser.parse_args()
    try:
        require(args.shadow.resolve()==ACTIVE.resolve(),"shadow override must be the live generated/active.rs route")
        result=audit()
    except Exception as exc: result={"status":"reject","checker_scope":"AO surviving-child correspondence","reason":str(exc)}
    rendered=json.dumps(result,indent=2)+"\n"
    if args.output: args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_text(rendered)
    print(rendered,end="")
    return 0 if result.get("status")=="pass" else 1

if __name__=="__main__": raise SystemExit(main())
