#!/usr/bin/env python3
"""Replay AN source/mapping mutations against the independent checker."""
from __future__ import annotations
import argparse
import copy
import importlib.util
import json
import pathlib
import sys
from typing import Any

ROOT=pathlib.Path(__file__).resolve().parent
MANIFEST=ROOT/"fixtures/checker-controls.json"
OUTPUT=ROOT/"generated/checker-controls-receipt.json"
spec=importlib.util.spec_from_file_location("an_checker_control_target",ROOT/"check_correspondence.py")
if spec is None or spec.loader is None: raise RuntimeError("cannot load AN correspondence checker")
CHECKER=importlib.util.module_from_spec(spec); sys.modules[spec.name]=CHECKER; spec.loader.exec_module(CHECKER)


def set_path(data:dict[str,Any],path:str,value:Any)->None:
    keys=path.split("."); parent:Any=data
    for key in keys[:-1]: parent=parent[int(key)] if isinstance(parent,list) else parent[key]
    last=keys[-1]
    if isinstance(parent,list): parent[int(last)]=value
    else: parent[last]=value


def replace_once(text:str,old:str,new:str,label:str)->str:
    n=text.count(old)
    if n!=1: raise RuntimeError(f"{label}: expected one source anchor, found {n}: {old!r}")
    return text.replace(old,new,1)


def mutate_text(text:str,c:dict[str,Any])->str:
    kind=c["kind"]; label=c["id"]
    if kind=="replace_once": return replace_once(text,c["old"],c["new"],label)
    if kind=="replace_all":
        count=text.count(c["old"])
        if count<1: raise RuntimeError(f"{label}: replacement anchor is absent")
        return text.replace(c["old"],c["new"])
    if kind=="insert_before": return replace_once(text,c["old"],c["text"]+c["old"],label)
    if kind=="append": return text+c["text"]
    if kind=="move_before":
        text=replace_once(text,c["needle"],"",label+" remove")
        return replace_once(text,c["before"],c["needle"]+c["before"],label+" insert")
    raise RuntimeError(f"{label}: unsupported mutation kind {kind!r}")


def load_inputs()->dict[str,Any]:
    data={
        "base":(ROOT/"src/promotion.rs").read_text(),
        "helper":(ROOT/"src/terminal_helpers.rs").read_text(),
        "extension":(ROOT/"src/reclone_extension.rs").read_text(),
        "client":(ROOT/"generated/elaborated-client.rs").read_text(),
        "active":(ROOT/"generated/active.rs").read_text(),
        "lib":(ROOT/"src/lib.rs").read_text(),
        "lifecycle":(ROOT/"src/lifecycle.rs").read_text(),
        "mapping":json.loads((ROOT/"generated/mapping.json").read_text()),
        "cargo":(ROOT/"Cargo.toml").read_text(),
        "production_cargo":(CHECKER.CRATE_ROOT/"Cargo.toml").read_text(),
        "build":(ROOT/"build.rs").read_text(),
        "extractor":(ROOT/"extract_public.py").read_text(),
    }
    for name,(literal,_) in CHECKER.IMPORTED_SUPPORT.items():
        data["support_"+name]=(ROOT/"src"/literal).resolve().read_text()
    return data


def validate(c:dict[str,Any],value:Any,s:dict[str,Any],native:Any,bundle:dict[str,Any])->None:
    target=c["target"]
    if target=="client": CHECKER.assert_client(value)
    elif target=="helper": CHECKER.assert_helpers(value,s["base"]+"\n"+value+s["extension"]+s["client"])
    elif target=="extension": CHECKER.assert_extension(value,(ROOT/"generated/reclone-extension.rs").read_text())
    elif target=="active": CHECKER.assert_active_composition(s["base"],s["helper"],s["extension"],s["client"],value)
    elif target=="lib": CHECKER.assert_module_route(value,s["active"])
    elif target=="lifecycle": CHECKER.AM.assert_frozen_al_sources({"lifecycle.rs":value.encode()})
    elif target=="mapping": CHECKER.assert_mapping(value,s["base"],s["helper"],s["extension"],s["client"],s["active"],
        bundle["capture"],bundle["mir_sources"]["client"],native)
    elif target=="cargo": CHECKER.assert_cargo_probe_manifest(value)
    elif target=="build": CHECKER.AL.assert_build_script_surface(value.encode())
    elif target=="extractor": CHECKER.AL.assert_extractor_source_surface(value.encode())
    elif target.startswith("support_"):
        module=target.removeprefix("support_")
        CHECKER.assert_imported_support_sources(s["lib"],{module:value})
    else: raise RuntimeError(f"{c['id']}: unsupported control target {target!r}")


def validate_macro_bundle(client:str,s:dict[str,Any],native:Any,bundle:dict[str,Any])->None:
    active=s["base"]+"\n"+s["helper"]+s["extension"]+client
    mapping=copy.deepcopy(s["mapping"])
    mapping["client_sha256"]=CHECKER.sha(client.encode())
    mapping["active_sha256"]=CHECKER.sha(active.encode())
    # Keep the composed source and receipt mutually consistent first. The
    # following complete-client token check must still reject the injected
    # proof-erasing macro.
    CHECKER.assert_mapping(mapping,s["base"],s["helper"],s["extension"],client,active,
        bundle["capture"],bundle["mir_sources"]["client"],native)
    CHECKER.assert_active_composition(s["base"],s["helper"],s["extension"],client,active)
    CHECKER.assert_client(client)


def run()->dict[str,Any]:
    manifest=json.loads(MANIFEST.read_text())
    controls=manifest.get("controls")
    if manifest.get("version")!=1 or not isinstance(controls,list) or not controls:
        raise RuntimeError("AN checker-controls manifest is invalid")
    ids=[c.get("id") for c in controls]
    if any(not isinstance(x,str) for x in ids) or len(ids)!=len(set(ids)):
        raise RuntimeError("AN control IDs must be unique strings")
    baseline=CHECKER.audit()
    native=CHECKER.load_module("an_controls_native",CHECKER.NATIVE_CHECKER)
    bundle=native.load_bundle(); native_result=native.audit_bundle(bundle)
    if baseline.get("status")!="pass" or native_result.get("status")!="pass":
        raise RuntimeError("AN checker baseline failed before controls")
    sources=load_inputs(); results=[]
    for c in controls:
        target=c["target"]
        if target=="macro_bundle":
            value=mutate_text(sources["client"],c)
            try: validate_macro_bundle(value,sources,native,bundle)
            except Exception as exc: results.append({"id":c["id"],"status":"rejected_as_expected","reason":str(exc)})
            else: results.append({"id":c["id"],"status":"UNEXPECTEDLY_ACCEPTED"})
            continue
        if target not in sources: raise RuntimeError(f"{c['id']}: unknown target {target!r}")
        original=sources[target]
        if target=="mapping":
            value=copy.deepcopy(original)
            if c["kind"]=="mapping_set": set_path(value,c["path"],c["value"])
            elif c["kind"]=="mapping_pop": value[c["path"]].pop()
            elif c["kind"]=="mapping_duplicate": value[c["path"]].append(copy.deepcopy(value[c["path"]][0]))
            else: raise RuntimeError(f"{c['id']}: unsupported mapping mutation {c['kind']!r}")
        else: value=mutate_text(original,c)
        try: validate(c,value,sources,native,bundle)
        except Exception as exc: results.append({"id":c["id"],"status":"rejected_as_expected","reason":str(exc)})
        else: results.append({"id":c["id"],"status":"UNEXPECTEDLY_ACCEPTED"})
    rejected=sum(r["status"]=="rejected_as_expected" for r in results)
    return {"status":"pass" if rejected==len(results) else "fail","control_count":len(results),
        "rejected_as_expected":rejected,"baseline_correspondence_status":baseline["status"],
        "native_baseline_status":native_result["status"],"controls":results,
        "source_mutated_on_disk":False,"cargo_or_rust_build_invoked":False,"solver_invoked":False,
        "scope":"structural checker controls; no semantic proof claim"}


def main()->int:
    parser=argparse.ArgumentParser(); parser.add_argument("--output",type=pathlib.Path,default=OUTPUT)
    args=parser.parse_args(); result=run(); rendered=json.dumps(result,indent=2)+"\n"
    args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_text(rendered); print(rendered,end="")
    return 0 if result["status"]=="pass" else 1

if __name__=="__main__": raise SystemExit(main())
