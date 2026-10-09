#!/usr/bin/env python3
"""One reviewed AY lexical peer-Drop omission, no compiler/prover invocation."""
import argparse, hashlib, importlib.util, json, pathlib, shutil, sys
PIN='d0afa9123d6187b463c827118645a145cbd84b5e7eb9edfb4bd58ffd3f690372'
def sha(raw): return hashlib.sha256(raw).hexdigest()
def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--probe',type=pathlib.Path,required=True)
    ap.add_argument('--fixture',type=pathlib.Path,required=True)
    ap.add_argument('--output',type=pathlib.Path,required=True)
    args=ap.parse_args(); p=args.probe.resolve(); f=args.fixture.resolve()
    if f.exists(): raise RuntimeError('fixture destination already exists; no overwrite')
    checker=p/'check_native.py'; raw=checker.read_bytes()
    if sha(raw)!=PIN: raise RuntimeError('native checker authority changed')
    spec=importlib.util.spec_from_file_location('ay_reviewed_peer_control',checker)
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module;spec.loader.exec_module(module)
    data=module.load_bundle()
    positive=json.loads((p/'generated/correspondence.json').read_bytes())
    if positive['status']!='pass': raise RuntimeError('current positive is required')
    mapping=json.loads((p/'generated/mapping.json').read_bytes())
    baseline=mapping['native_report']
    if baseline['status']!='pass': raise RuntimeError('captured positive native report is required')
    ancestor=baseline['ancestor_av']
    f.mkdir(parents=True); shutil.copytree(p/'native-mir',f/'native-mir')
    capture=data['capture']; client_row=next(r for r in capture['selected'] if r['label']=='client')
    client=f/client_row['path']; original=client.read_bytes()
    old=b'drop(_12) -> [return: bb5, unwind: bb13];';new=b'goto -> bb5;'
    if original.count(old)!=1: raise RuntimeError('actual peer Drop anchor is not unique')
    mutated=original.replace(old,new);client.write_bytes(mutated)
    client_row['sha256']=sha(mutated)
    capture_raw=(json.dumps(capture,indent=2)+'\n').encode();(f/'native-mir/capture.json').write_bytes(capture_raw)
    data['capture_raw']=capture_raw;data['mir_sources']['client']=mutated.decode()
    module.ROOT=f;module.MIR_DIR=f/'native-mir';module.CAPTURE_PATH=f/'native-mir/capture.json'
    module.EXPECTED_CAPTURE_SHA256=sha(capture_raw);module.EXPECTED_CLIENT_MIR_SHA256=sha(mutated)
    integrity=module.audit_capture(data,ancestor)
    if integrity['capture_sha256']!=sha(capture_raw): raise RuntimeError('refreshed capture not checked')
    try:module.audit_client_mir(data)
    except module.AuditError as exc:
        reason=str(exc)
        if reason!='AY client bb4 contains missing, extra, or reordered operations':
            raise RuntimeError('unexpected rejection stage: '+reason)
    else:raise RuntimeError('peer-Drop omission accepted')
    receipt={'schema':'ay-peer-drop-control-v1','status':'pass','gap':'lexical peer Drop must occur inside promote branch before common continuation',
      'checker_sha256':PIN,'current_positive_sha256':sha((p/'generated/correspondence.json').read_bytes()),
      'source_mapping_sha256':sha((p/'generated/mapping.json').read_bytes()),
      'original_capture_sha256':sha((p/'native-mir/capture.json').read_bytes()),'refreshed_capture_sha256':sha(capture_raw),
      'original_client_sha256':sha(original),'mutated_client_sha256':sha(mutated),'client_path':client_row['path'],
      'mutation':{'block':'bb4','old':old.decode(),'new':new.decode(),'owner_place':'_12'},
      'capture_integrity':'pass','capture_integrity_report':integrity,
      'operational_rejection':{'status':'reject','stage':'audit_client_mir exact bb4 operations','reason':reason},
      'ancestor_audit_reexecuted':False,'positive_report_reused':True,'compiler_invoked':False,'prover_invoked':False,
      'scope':'structural correspondence sensitivity; no failed Rust VC or native counterexample',
      'fixture_files':[{'path':q.relative_to(f).as_posix(),'sha256':sha(q.read_bytes()),'size':q.stat().st_size} for q in sorted(f.rglob('*')) if q.is_file()]}
    args.output.write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({'status':'pass','integrity':'pass','reason':reason,'receipt_sha256':sha(args.output.read_bytes())}))
if __name__=='__main__':main()
