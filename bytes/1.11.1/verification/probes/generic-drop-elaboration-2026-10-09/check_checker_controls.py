#!/usr/bin/env python3
"""Run non-prover correspondence/control mutations against checker fixtures."""
from __future__ import annotations
import copy, hashlib, importlib.util, json, pathlib, shutil, tempfile

ROOT=pathlib.Path(__file__).resolve().parent
FIX=ROOT/'fixtures/checker-controls.json'
CHECKER_PATH=ROOT/'check_correspondence.py'
spec=importlib.util.spec_from_file_location('dropcorr',CHECKER_PATH)
corr=importlib.util.module_from_spec(spec); spec.loader.exec_module(corr)

def sha(b:bytes)->str: return hashlib.sha256(b).hexdigest()
def mutate(s:str,old:str,new:str)->str:
    n=s.count(old)
    if n!=1: raise AssertionError(f'mutation expected one occurrence, found {n}: {old!r}')
    return s.replace(old,new,1)

def seed(root:pathlib.Path)->tuple[pathlib.Path,pathlib.Path]:
    for rel in ['native.rs','elaborate.py','native-mir/native-rustc-input.rs','native-mir/source-erasure.json','native-mir/rustc-version.txt','native-mir/command.txt']:
        dst=root/rel; dst.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(ROOT/rel,dst)
    for eff in corr.EFFECTS:
        rel='native-mir/'+eff['mir']; dst=root/rel; dst.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(ROOT/rel,dst)
    (root/'generated').mkdir(exist_ok=True)
    shadow=root/'generated/shadow.rs'; mapping=root/'generated/mapping.json'
    shutil.copy2(ROOT/'generated/shadow.rs',shadow); shutil.copy2(ROOT/'generated/mapping.json',mapping)
    return shadow,mapping

def refresh_shadow_receipts(shadow:str,m:dict)->None:
    """Refresh output receipts independently so tests cannot pass by stale hashes."""
    m['variant']['shadow_sha256']=sha(shadow.encode())
    for e in m['effects']:
        helper=e['shadow_helper']['name']; hf=corr.fn_data(shadow,helper)
        h=e['shadow_helper']; h['body']=hf['body']; h['body_sha256']=sha(hf['body'].encode()); h['item_sha256']=sha(hf['item'].encode())
        h['span_bytes']=[hf['start'],hf['end']]
        h['span_lines']=[shadow[:hf['start']].count('\n')+1,shadow[:hf['end']].count('\n')+1]
        sf=corr.fn_data(shadow,e['scope'],visibility='pub')
        e['injection']['generated_function_body_sha256']=sha(sf['body'].encode())
        cs=corr.call_occurrences(sf['body'],helper,sf['body_start'])
        rows=[]
        for i,c in enumerate(cs):
            g=c['byte_span'][0]; last=shadow.rfind('\n',0,g)
            rows.append({'line':shadow[:g].count('\n')+1,'column':g-last,'byte_span':c['byte_span'],'text':c['text'],'argument':c['argument'],'order_in_scope':i,'text_sha256':c['text_sha256']})
        e['injection']['observed_generated_calls']=rows; e['injection']['expected_call_count']=len(cs)

def run_mutation(case:dict)->dict:
    with tempfile.TemporaryDirectory(prefix='drop-check-control-') as td:
        root=pathlib.Path(td); shadow_path,mapping_path=seed(root)
        source_path=root/'native.rs'; source=source_path.read_text(); shadow=shadow_path.read_text(); mp=json.loads(mapping_path.read_text())
        if case['id']=='fake_drop_impl_header_ignored':
            fake="/* impl Drop for SetTrueOnDrop<'_> { fn drop(&mut self) { unknown(); } } */\n"+source
            found=corr.source_drop_items(fake)
            valid=(set(found)=={'SetTrueOnDrop','ToggleOnDrop'} and found['SetTrueOnDrop']['tokens']==['set_true','(','self','.','0',')',';'] and found['ToggleOnDrop']['tokens']==['*','self','.','0','=','!','*','self','.','0',';'])
            status='parser_control_pass' if valid else 'coverage_failure'
            return {'id':case['id'],'expected':case['expected'],'actual':status,'pass':status==case['expected'],'error':None if valid else 'executable Drop impl set changed after masking comment text','mapping_receipts_refreshed':False,'prover_invoked':False}
        if case['id']=='shadow_drop_body_tamper_receipt_refreshed':
            shadow=mutate(shadow,'*guard.0 = !*guard.0;','*guard.0 = false;')
            refresh_shadow_receipts(shadow,mp); shadow_path.write_text(shadow); mapping_path.write_text(json.dumps(mp,indent=2,sort_keys=True)+'\n')
        elif case['id']=='unknown_callback': shadow_path.write_text(mutate(shadow,'set_true(guard.0);','unknown_callback(guard.0);'))
        elif case['id']=='guard_escape': shadow_path.write_text(mutate(shadow,'    set_true_drop_effect(&mut _guard);','    core::mem::forget(_guard);\n    set_true_drop_effect(&mut _guard);'))
        elif case['id']=='unsupported_field_drop': shadow_path.write_text(mutate(shadow,"pub struct ToggleOnDrop<'a>(pub &'a mut bool);","pub struct ToggleOnDrop<'a>(pub String);"))
        elif case['id']=='extra_drop_glue': shadow_path.write_text(shadow+'\nstruct ExtraDrop;\nimpl Drop for ExtraDrop { fn drop(&mut self) {} }\n')
        elif case['id']=='panic_only_normal_edge':
            p=root/'native-mir'/corr.EFFECTS[1]['mir']; p.write_text(mutate(p.read_text(),'drop(_2) -> [return: bb1, unwind: bb2];','drop(_2) -> [return: bb2, unwind: bb1];'))
        elif case['id']=='thread_escape': shadow_path.write_text(mutate(shadow,'    let mut _guard = ToggleOnDrop(flag);','    let mut _guard = ToggleOnDrop(flag);\n    std::thread::spawn(|| {});'))
        elif case['id']=='guard_move': shadow_path.write_text(mutate(shadow,'    let mut _guard = SetTrueOnDrop(flag);','    let mut _guard = SetTrueOnDrop(flag);\n    let moved = _guard;'))
        elif case['id']=='extra_normal_successor_statement':
            p=root/'native-mir'/corr.EFFECTS[0]['mir']; p.write_text(mutate(p.read_text(),'        StorageDead(_2);\n        return;','        StorageDead(_2);\n        _0 = const ();\n        return;'))
        elif case['id']=='extra_unwind_successor_statement':
            p=root/'native-mir'/corr.EFFECTS[0]['mir']; p.write_text(mutate(p.read_text(),'    bb2 (cleanup): {\n        resume;','    bb2 (cleanup): {\n        _0 = const ();\n        resume;'))
        elif case['id']=='set_true_trusted_false_summary':
            shadow=mutate(shadow,'#[ensures(^flag == true)]\nfn set_true(flag: &mut bool)', '#[trusted]\n#[ensures(^flag == false)]\nfn set_true(flag: &mut bool)')
            refresh_shadow_receipts(shadow,mp); shadow_path.write_text(shadow); mapping_path.write_text(json.dumps(mp,indent=2,sort_keys=True)+'\n')
        elif case['id']=='commented_out_drop_call': shadow_path.write_text(mutate(shadow,'    set_true_drop_effect(&mut _guard);','    // set_true_drop_effect(&mut _guard);'))
        elif case['id']=='string_literal_pretend_call': shadow_path.write_text(mutate(shadow,'    set_true_drop_effect(&mut _guard);','    let marker = "set_true_drop_effect(&mut _guard);";'))
        elif case['id']=='fake_fn_header_and_whitespace_real':
            shadow='/* pub fn set_true_scope(flag: &mut bool) {} */\n'+mutate(shadow,'pub fn set_true_scope(','pub\nfn set_true_scope(')
            refresh_shadow_receipts(shadow,mp); shadow_path.write_text(shadow); mapping_path.write_text(json.dumps(mp,indent=2,sort_keys=True)+'\n')
        elif case['id']=='comment_between_contract_and_function':
            shadow=mutate(shadow,'#[ensures(^flag == false)]\npub fn toggle_after_write','#[ensures(^flag == false)]\n// A comment-only line between the contract and callable.\npub fn toggle_after_write')
            refresh_shadow_receipts(shadow,mp); shadow_path.write_text(shadow); mapping_path.write_text(json.dumps(mp,indent=2,sort_keys=True)+'\n')
        elif case['id']=='source_shadow_changed_without_new_native_mir':
            source=mutate(source,'*self.0 = !*self.0;','*self.0 = ! *self.0;'); source_path.write_text(source)
            shadow=mutate(shadow,'*guard.0 = !*guard.0;','*guard.0 = ! *guard.0;'); shadow_path.write_text(shadow)
        else: raise AssertionError(f'unknown fixture id {case["id"]}')
        try:
            result=corr.run_check(root,shadow_path,mapping_path,set(),verify_receipt=case['id'] in {'shadow_drop_body_tamper_receipt_refreshed','set_true_trusted_false_summary','fake_fn_header_and_whitespace_real','comment_between_contract_and_function'})
            status=result['status']; error=None
        except corr.AuditError as e:
            status=e.category; error=str(e)
        return {'id':case['id'],'expected':case['expected'],'actual':status,'pass':status==case['expected'],'error':error,'mapping_receipts_refreshed':case['id'] in {'shadow_drop_body_tamper_receipt_refreshed','set_true_trusted_false_summary'},'prover_invoked':False}

def main()->int:
    controls=json.loads(FIX.read_text()); results=[]
    for row in controls['generator_diagnostics']:
        f=row['feature']; base=ROOT/'generated/controls'/f
        try:
            got=corr.run_check(ROOT,base/'shadow.rs',base/'mapping.json',{f},verify_receipt=True)
            status='correspondence_pass_semantic_vc_required' if got['semantic_vc_controls'] else 'correspondence_pass'
            error=None
        except corr.AuditError as e:
            status=e.category; error=str(e)
        results.append({'id':f,'expected':row['expected'],'actual':status,'pass':status==row['expected'],'error':error,'prover_invoked':False})
    for row in controls['checker_mutations']: results.append(run_mutation(row))
    report={'schema_version':1,'status':'controls_pass' if all(x['pass'] for x in results) else 'controls_failed','checker_sha256':sha(CHECKER_PATH.read_bytes()),'native_source_sha256':sha((ROOT/'native.rs').read_bytes()),'positive_shadow_sha256':sha((ROOT/'generated/shadow.rs').read_bytes()),'positive_mapping_sha256':sha((ROOT/'generated/mapping.json').read_bytes()),'controls':results,'boundary':controls['semantic_boundary'],'prover_invoked':False}
    out=ROOT/'fixtures/checker-control-results.json'; out.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'status':report['status'],'report':str(out),'controls':len(results),'failures':[x for x in results if not x['pass']]},indent=2))
    return 0 if report['status']=='controls_pass' else 2
if __name__=='__main__': raise SystemExit(main())
