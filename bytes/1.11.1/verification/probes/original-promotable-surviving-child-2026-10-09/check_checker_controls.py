#!/usr/bin/env python3
"""Replay adversarial AO source, mapping, route, and MIR mutations."""
from __future__ import annotations
import argparse, copy, importlib.util, json, pathlib, sys, tempfile, tomllib
from typing import Any

ROOT=pathlib.Path(__file__).resolve().parent
MANIFEST=ROOT/"fixtures/checker-controls.json"
OUTPUT=ROOT/"generated/checker-controls-receipt.json"
spec=importlib.util.spec_from_file_location("ao_checker_control_target",ROOT/"check_correspondence.py")
if spec is None or spec.loader is None: raise RuntimeError("cannot load AO correspondence checker")
CHECKER=importlib.util.module_from_spec(spec); sys.modules[spec.name]=CHECKER; spec.loader.exec_module(CHECKER)

def set_path(data:dict[str,Any],path:str,value:Any)->None:
    keys=path.split("."); parent:Any=data
    for key in keys[:-1]: parent=parent[int(key)] if isinstance(parent,list) else parent[key]
    last=keys[-1]
    if isinstance(parent,list): parent[int(last)]=value
    else: parent[last]=value

def replace_once(text:str,old:str,new:str,label:str)->str:
    count=text.count(old)
    if count!=1: raise RuntimeError(f"{label}: expected exactly one mutation anchor, got {count}: {old!r}")
    return text.replace(old,new,1)

def mutate_text(text:str,c:dict[str,Any])->str:
    kind=c["kind"]; label=c["id"]
    if kind=="replace_once": return replace_once(text,c["old"],c["new"],label)
    if kind=="insert_before": return replace_once(text,c["old"],c["text"]+c["old"],label)
    if kind=="append": return text+c["text"]
    if kind=="move_before":
        text=replace_once(text,c["needle"],"",label+" remove")
        return replace_once(text,c["before"],c["needle"]+c["before"],label+" insert")
    raise RuntimeError(f"{label}: unsupported text mutation {kind!r}")

def load_inputs(native:Any,bundle:dict[str,Any])->dict[str,Any]:
    source={"prefix":(ROOT/"src/promotion.rs").read_text(),
        "extension":(ROOT/"src/surviving_extension.rs").read_text(),
        "client":(ROOT/"generated/elaborated-client.rs").read_text(),
        "active":(ROOT/"generated/active.rs").read_text(),
        "helper":(ROOT/"generated/terminal-helper.rs").read_text(),
        "lib":(ROOT/"src/lib.rs").read_text(),
        "native":(ROOT/"native.rs").read_text(),
        "mir":bundle["mir_sources"]["client"],
        "cargo":tomllib.loads((ROOT/"Cargo.toml").read_text()),
        "mapping":json.loads((ROOT/"generated/mapping.json").read_text()),
        "capture":bundle["capture"],"bundle":bundle,"native_checker":native}
    for name,(literal,_) in CHECKER.AN.IMPORTED_SUPPORT.items():
        source["support_"+name]=(ROOT/"src"/literal).resolve().read_text()
    return source

def validate(c:dict[str,Any],value:Any,s:dict[str,Any])->None:
    target=c["target"]
    if target=="client": CHECKER.assert_client_tokens(value)
    elif target=="extension": CHECKER.assert_extension(value)
    elif target=="active": CHECKER.assert_active_composition(s["prefix"],s["extension"],s["client"],value)
    elif target=="helper": CHECKER.require(value==s["extension"],"generated terminal helper diverged from source extension")
    elif target=="lib": CHECKER.assert_ao_module_route(value,s["active"])
    elif target=="native":
        CHECKER.require(CHECKER.AN.AI.rust_tokens(value)==CHECKER.AN.AI.rust_tokens(CHECKER.EXPECTED_NATIVE),
            "native client token stream changed")
    elif target=="mir": CHECKER.derive_terminal_edges(value)
    elif target=="cargo": CHECKER.assert_ao_probe_manifest(value)
    elif target=="mapping":
        CHECKER.assert_mapping(value,s["prefix"],s["extension"],s["client"],s["active"],
            s["capture"],s["mir"],s["native_checker"])
    elif target.startswith("support_"):
        module=target.removeprefix("support_")
        CHECKER.AN.assert_imported_support_sources(s["lib"],{module:value})
    else: raise RuntimeError(f"{c['id']}: unsupported checker target {target!r}")

def validate_macro_bundle(client:str,s:dict[str,Any])->None:
    active=s["prefix"]+"\n"+s["extension"]+client
    mapping=copy.deepcopy(s["mapping"])
    mapping["client_sha256"]=CHECKER.sha(client.encode())
    mapping["active_sha256"]=CHECKER.sha(active.encode())
    # First establish that source and receipt agree. Then the complete-client
    # token pin must reject the proof_assert! shadowing macro itself.
    CHECKER.assert_mapping(mapping,s["prefix"],s["extension"],client,active,
        s["capture"],s["mir"],s["native_checker"])
    CHECKER.assert_active_composition(s["prefix"],s["extension"],client,active)
    CHECKER.assert_client_tokens(client)

def validate_paired_support_clone(c:dict[str,Any])->str:
    # Mutate independent temporary descendant copies, not the live AO/AN trees.
    source=(ROOT/"src/field_event.rs").read_text()
    with tempfile.TemporaryDirectory(prefix="ao-paired-source-control-") as td:
        base=pathlib.Path(td); an_dir=base/"an"/"src"; ao_dir=base/"ao"/"src"
        an_dir.mkdir(parents=True); ao_dir.mkdir(parents=True)
        for dest in (an_dir,ao_dir): (dest/"field_event.rs").write_text(source)
        mutated=mutate_text(source,c)
        (an_dir/"field_event.rs").write_text(mutated)
        (ao_dir/"field_event.rs").write_text(mutated)
        an_bytes=(an_dir/"field_event.rs").read_bytes(); ao_bytes=(ao_dir/"field_event.rs").read_bytes()
        try:
            CHECKER.assert_paired_inherited_sources({"field_event.rs":an_bytes},{"field_event.rs":ao_bytes})
        except Exception as exc: return str(exc)
        raise RuntimeError("matching mutated AN/AO temporary source copies were unexpectedly accepted")

def validate_checker_clone(c:dict[str,Any])->str:
    checker_path=next(iter(CHECKER.PUBLISHED_PYTHON_INPUTS))
    with tempfile.TemporaryDirectory(prefix="ao-ancestor-checker-control-") as td:
        clone=pathlib.Path(td)/checker_path.name
        original=checker_path.read_text(); clone.write_text(original)
        mutated=mutate_text(clone.read_text(),c); clone.write_text(mutated)
        try: CHECKER.assert_published_python_lineage({checker_path:clone.read_bytes()})
        except Exception as exc: return str(exc)
        raise RuntimeError("mutated ancestor checker temporary clone was unexpectedly accepted")

def run()->dict[str,Any]:
    manifest=json.loads(MANIFEST.read_text()); controls=manifest.get("controls")
    if manifest.get("version")!=1 or not isinstance(controls,list) or not controls:
        raise RuntimeError("AO checker-controls manifest is invalid")
    ids=[c.get("id") for c in controls]
    if any(not isinstance(x,str) for x in ids) or len(ids)!=len(set(ids)):
        raise RuntimeError("AO control IDs must be unique strings")
    baseline=CHECKER.audit()
    native=CHECKER.AN.load_module("ao_controls_native",ROOT/"check_native.py")
    bundle=native.load_bundle(); native_audit=native.audit_bundle(bundle)
    if baseline.get("status")!="pass" or native_audit.get("status")!="pass":
        raise RuntimeError("AO checker/native baseline failed before structural controls")
    sources=load_inputs(native,bundle); results=[]
    for c in controls:
        target=c["target"]
        if target=="paired_support_snapshot":
            try: reason=validate_paired_support_clone(c)
            except Exception as exc: results.append({"id":c["id"],"status":"control_harness_error","reason":str(exc)})
            else: results.append({"id":c["id"],"status":"rejected_as_expected","reason":reason})
            continue
        if target=="checker_lineage":
            try: reason=validate_checker_clone(c)
            except Exception as exc: results.append({"id":c["id"],"status":"control_harness_error","reason":str(exc)})
            else: results.append({"id":c["id"],"status":"rejected_as_expected","reason":reason})
            continue
        if target=="paired_route":
            original=(ROOT/"src/lib.rs").read_text()
            mutated=mutate_text(original,c)
            try:
                CHECKER.assert_ao_module_route(mutated,sources["active"])
                # The AN route validator independently compares against AL's
                # reviewed root, closing matching descendant route edits.
                CHECKER.AN.assert_module_route(mutated,sources["active"])
            except Exception as exc: results.append({"id":c["id"],"status":"rejected_as_expected","reason":str(exc)})
            else: results.append({"id":c["id"],"status":"UNEXPECTEDLY_ACCEPTED"})
            continue
        if target=="macro_bundle":
            value=mutate_text(sources["client"],c)
            try: validate_macro_bundle(value,sources)
            except Exception as exc: results.append({"id":c["id"],"status":"rejected_as_expected","reason":str(exc)})
            else: results.append({"id":c["id"],"status":"UNEXPECTEDLY_ACCEPTED"})
            continue
        if target not in sources: raise RuntimeError(f"{c['id']}: unknown target {target!r}")
        original=sources[target]
        if target in ("mapping","cargo"):
            value=copy.deepcopy(original)
            kind=c["kind"]
            if kind=="mapping_set": set_path(value,c["path"],c["value"])
            elif kind=="mapping_pop": value[c["path"]].pop()
            else: raise RuntimeError(f"{c['id']}: unsupported mapping mutation {kind!r}")
        else: value=mutate_text(original,c)
        try: validate(c,value,sources)
        except Exception as exc: results.append({"id":c["id"],"status":"rejected_as_expected","reason":str(exc)})
        else: results.append({"id":c["id"],"status":"UNEXPECTEDLY_ACCEPTED"})
    rejected=sum(r["status"]=="rejected_as_expected" for r in results)
    return {"status":"pass" if rejected==len(results) else "fail","control_count":len(results),
        "rejected_as_expected":rejected,"baseline_correspondence_status":baseline["status"],
        "native_baseline_status":native_audit["status"],"controls":results,
        "source_mutated_on_disk":False,"cargo_or_rust_build_invoked":False,"solver_invoked":False,
        "scope":"AO structural correspondence checker controls; not semantic proof controls"}

def main()->int:
    parser=argparse.ArgumentParser(); parser.add_argument("--output",type=pathlib.Path,default=OUTPUT)
    args=parser.parse_args(); result=run(); rendered=json.dumps(result,indent=2)+"\n"
    args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_text(rendered); print(rendered,end="")
    return 0 if result["status"]=="pass" else 1

if __name__=="__main__": raise SystemExit(main())
