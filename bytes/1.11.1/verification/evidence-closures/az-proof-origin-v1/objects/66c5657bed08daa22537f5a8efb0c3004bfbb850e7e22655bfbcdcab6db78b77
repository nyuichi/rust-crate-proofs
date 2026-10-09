#!/usr/bin/env python3
"""AZ operational source/native joins; exact proof/Cargo receipts are separate.

This reviewed bounded mapping is not a general Rust equivalence verifier.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

ROOT=Path(__file__).resolve().parent
AX=ROOT.parent/'original-raw-suffix-drop-2026-10-09'
AX_CHECKER_SHA='f63fcc1b6382be16eff6cf31a3ea4eb05a5648c615f2d681d1069817878c0922'
PREFIX_SHA='9440ac43f7a116e4bd36b5affb449334be8872e34f9e9cb7a9dc0c923311d414'
PACKAGE='bytes-original-root-phase-clone'

def require(ok,message):
    if not ok:raise RuntimeError(message)

def sha(raw):return hashlib.sha256(raw).hexdigest()

def compact(source):return re.sub(r'\s+','',source)

def load_module(path,name,pin=None):
    require(path.is_file() and not path.is_symlink(),f'missing regular module {path}')
    if pin:require(sha(path.read_bytes())==pin,f'changed module {path}')
    spec=importlib.util.spec_from_file_location(name,path);require(spec and spec.loader,'module loader unavailable')
    module=importlib.util.module_from_spec(spec);sys.modules[name]=module;spec.loader.exec_module(module);return module

def ordered(source,tokens,label):
    position=0
    for token in tokens:
        found=source.find(token,position);require(found>=0,f'{label}: missing ordered operation {token}');position=found+len(token)

def audit_source(mapping,native,ax):
    reviewed_raw=(ROOT/'AZ_REVIEWED_SOURCE.json').read_bytes()
    require(sha(reviewed_raw)=='3f6d7e6cb4d115991f16a7c7d4e6e95cd1ef58bfa93d183ed8faf56fcfaf237e','reviewed AZ pre-proof source declaration changed')
    reviewed=json.loads(reviewed_raw)
    parent=ROOT.parent/'original-root-phase-view-2026-10-09'
    prefix=(parent/'src/promotion.rs').read_bytes();raw=(ROOT/'src/promotion.rs').read_bytes()
    require(sha(prefix)=='21f1b7acb2267f5e38938691003d81fdc25dea93a2016be3b74178b98c87a6ed' and raw.startswith(prefix),'AZ lost immutable AY full-positive prefix')
    require(sha(raw)==reviewed['source_sha256'] and sha((ROOT/'src/root_clone_extension.rs').read_bytes())==reviewed['mirror_sha256'],'AZ source differs from reviewed pre-proof freeze')
    require(raw[len(prefix):]==b'\n'+(ROOT/'src/root_clone_extension.rs').read_bytes(),'AZ mirror does not identify exact appended source')
    require(mapping['selected_source_sha256']==sha(raw),'mapping does not identify current AZ source')
    appendix=raw[len(prefix):].decode()
    def body(name):return ax.extract_function(appendix,name)[1]
    def normalized(name):return compact(body(name))
    # Stronger new summary is proved over the exact unchanged operational body.
    inherited_body=ax.extract_function(prefix.decode(),'shallow_clone_suffix_checked')[1]
    require(body('shallow_clone_root_suffix_checked')==inherited_body,'AZ raw helper operation body differs from immutable AV body')
    require(appendix.count('#[trusted]')==2,'AZ adds unreviewed trusted body')
    clone=normalized('clone_root_view')
    ordered(clone,['descriptor.base.raw_pointer()','ifaddress&1usize==0usize','even_root_view_clone_registration()','odd_root_view_clone_registration()','letnative=source.vtable.clone','erased_call::invoke3(native,(&source.data,source.ptr,source.len)'],'common Root Clone')
    require(clone.count('erased_call::invoke3(')==1 and '(*descriptor,scope.view,&mut**scope)' in clone,'common Clone does not use one actual vtable call and exact descriptor/view/scope input')
    require_ghost(body('clone_root_view'),['descriptor.base.raw_pointer()','even_root_view_clone_registration()','odd_root_view_clone_registration()'],'immutable-base Clone registration selection')
    for parity in ('even','odd'):
        name=parity+'_root_view_clone_checked';code=body(name);c=compact(code)
        ordered(c,['owned_pointer::load_acquire(data,ghost!{','letkind=crate::provenance_specs::pointer_addr(stored)&1usize','ifkind==0usize','shallow_clone_owned_view_checked(stored.cast(),offset,len','}else{','shallow_clone_root_suffix_checked(data,stored,base,offset,len'],'Root callback '+parity)
        require(c.count('owned_pointer::load_acquire(')==1 and 'get_mut_finish(' not in c and 'load_relaxed(' not in c,'Clone does not preserve one Acquire/owned permission')
        require_ghost(code,['Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mutraw.current);}','Phase::Shared(shared)=>{c.shoot_load(&shared.own,&mutshared.current);}'],'phase-specific owned Acquire callback')
        match=re.search(r'\bif\s+kind\s*==\s*0usize\s*\{',code);require(match is not None,'native Clone kind guard absent')
        shared,end=braced_region(code,match.end()-1);tail=code[end+1:];other=re.match(r'\s*else\s*\{',tail);require(other is not None,'Clone Raw else arm absent')
        raw_arm,_=braced_region(tail,other.end()-1)
        require('shallow_clone_owned_view_checked(' in shared and 'shallow_clone_root_suffix_checked(' not in shared and 'shallow_clone_root_suffix_checked(' in raw_arm and 'shallow_clone_owned_view_checked(' not in raw_arm,'Raw/Shared native Clone operations cross kind arms')
        require_ghost(shared,['Phase::Shared(shared)=>shared'],'Shared scope phase borrow')
        require_ghost(shared,['(&shared.root,view.into_inner(),&mutshared.cursor)'],'Shared clone exact root/current-view/cursor package')
        require_ghost(raw_arm,['(*descriptor,view.into_inner(),&mut**scope)'],'Raw promotion exact original descriptor/current-view/scope package')
        decoder='tag_specs::clear_low_bit(stored,ghost!{&descriptor.base})' if parity=='even' else 'stored.cast::<u8>()'
        require(decoder in compact(raw_arm),'Root Clone Raw base decode/cast differs')
        position=appendix.index('fn '+parity+'_root_view_clone_registration');start=appendix.rfind('#[trusted]',0,position);contract=compact(appendix[start:position])
        require('erased_call::registered3('+parity+'_table().clone,result.inner_logic())' in contract,'Clone registration points at wrong vtable slot')
        require('result.inner_logic().precondition((data,ptr,len,input))=='+name+'.precondition((data,ptr,len,input))' in contract and 'result.inner_logic().postcondition((data,ptr,len,input),output)=='+name+'.postcondition((data,ptr,len,input),output)' in contract,'Clone registration lacks exact proved pre/post')
    client=body('root_clone_steps');c=compact(client)
    ordered(c,['from_box_scoped(input)','SuffixScope::new(scope)','letmutindex=0usize','whileindex<steps.len(){','letamount=core::cmp::min(steps[index],value.len())','advance_root_view(&mutvalue,amount,suffix.borrow_mut())','letpeer=clone_root_view(&value,suffix.borrow_mut())','bytes_root_peer_terminal_drop(peer,suffix.borrow_mut(),peer_receipt.borrow_mut())','index+=1','chunk_root_view(&value,suffix.borrow()).to_vec()','letsaved_return=result','bytes_root_view_terminal_drop(value,suffix,receipt.borrow_mut())','saved_return'],'runtime Root loop')
    guard=re.search(r'\bwhile\s+index\s*<\s*steps\.len\(\)\s*\{',client);require(guard is not None,'runtime steps loop guard absent')
    loop,end=braced_region(client,guard.end()-1);continuation=client[end+1:]
    require(all(token in compact(loop) for token in ['advance_root_view(','clone_root_view(','bytes_root_peer_terminal_drop(','index+=1']) and all(token not in loop for token in ['chunk_root_view(','bytes_root_view_terminal_drop(']),'loop/current final read/Root Drop boundaries differ')
    require('chunk_root_view(' in continuation and 'bytes_root_view_terminal_drop(' in continuation,'normal suffix read/Root Drop are not after loop')
    # The inner lexical block must own the peer and finish before index increment.
    peer_at=loop.index('let peer=');opening=loop.rfind('{',0,peer_at);peer_block,peer_end=braced_region(loop,opening)
    require('clone_root_view(' in peer_block and 'bytes_root_peer_terminal_drop(' in peer_block and 'index+=' not in compact(peer_block) and 'index+=1' in compact(loop[peer_end+1:]),'peer ownership is not consumed at inner block end before backedge increment')
    require(c.count('advance_root_view(')==c.count('clone_root_view(')==c.count('bytes_root_peer_terminal_drop(')==c.count('bytes_root_view_terminal_drop(')==1,'runtime loop duplicates native lifecycle operation')
    require('steps[index]' in c and 'core::cmp::min(steps[index],value.len())' in c and not re.search(r'index<\d',c),'runtime loop has an added quota or different cap')
    return dict(status='pass',source_sha256=sha(raw),appendix_sha256=sha(raw[len(prefix):]),immutable_AY_prefix=True,strong_raw_helper_operational_body_byte_exact=True,one_Acquire_then_native_kind_branch=True,Ghost_only_specification_phase_resource_selection=True,runtime_loop_and_peer_scope_join=True,native_client=native['native_audit']['client'],generic_min_alias='source core::cmp::min is the same usize operation as compiler MIR std::cmp::min reexport, independently checked by native gate',full_original_admitted=False)


def expected_public_records():
    """Reconstruct current records directly, without ancestry checker replay."""
    production=ROOT.parents[2]
    reviewed=json.loads((ROOT/'reviewed-production-inputs.json').read_bytes())
    require(reviewed['schema']=='reviewed-original-production-inputs-v1','production review schema changed')
    input_hashes={}
    for relative,expected in reviewed['files'].items():
        path=production/relative;require(path.is_file() and not path.is_symlink(),f'missing production input {relative}')
        require(sha(path.read_bytes())==expected,f'production input differs from reviewed identity: {relative}')
    # The same exact production declaration boundaries used by the extractor,
    # reconstructed independently without executing its top-level writer.
    records=''
    for name in ('bytes_record.rs','vtable_record.rs'):
        path=production/'src/bytes'/name;raw=path.read_bytes();input_hashes['../../../src/bytes/'+name]=sha(raw);records+=raw.decode()+'\n'
    mutable=(production/'src/bytes_mut.rs').read_bytes();input_hashes['../../../src/bytes_mut.rs']=sha(mutable)
    text=mutable.decode()
    def declaration(start):
        require(text.count(start)==1,'production mutable declaration not unique')
        a=text.index(start);b=text.index('\n}',a)+2;return text[a:b]
    records+='mod mutable_record {\nuse alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n'+declaration('struct Shared {')+'\n'+declaration('pub struct BytesMut {')+'\n}\nuse mutable_record::BytesMut;\n'
    input_hashes['../../../src/bytes.rs']=sha((production/'src/bytes.rs').read_bytes())
    input_hashes['extract_public.py']=sha((ROOT/'extract_public.py').read_bytes())
    require((ROOT/'generated/public_records.rs').read_bytes()==records.encode(),'generated public records differ from direct reconstruction')
    return None,records.encode(),input_hashes

def cargo_adapter(ax):
    """Reuse only published AX artifact joins, never its old theorem/audit."""
    ax.ROOT=ROOT;ax.PACKAGE=PACKAGE;ax.CAPTURE_DIR=ROOT/'generated/compiled-inputs'
    ax.expected_public_records=expected_public_records
    return ax

def braced_region(source,opening):
    """Balanced source region, ignoring Rust comments and string literals."""
    depth=0;string=False;escaped=False;line=False;block=0;i=opening
    while i<len(source):
        c=source[i];n=source[i+1] if i+1<len(source) else ''
        if line:
            if c=='\n':line=False
        elif block:
            if c=='/' and n=='*':block+=1;i+=1
            elif c=='*' and n=='/':block-=1;i+=1
        elif string:
            if escaped:escaped=False
            elif c=='\\':escaped=True
            elif c=='"':string=False
        elif c=='/' and n=='/':line=True;i+=1
        elif c=='/' and n=='*':block=1;i+=1
        elif c=='"':string=True
        elif c=='{':depth+=1
        elif c=='}':
            depth-=1
            if depth==0:return source[opening+1:i],i
        i+=1
    raise RuntimeError('unbalanced source region')

def ghost_regions(body):
    return [compact(braced_region(body,m.end()-1)[0]) for m in re.finditer(r'\bghost!\s*\{',body)]

def require_ghost(body,tokens,label):
    regions=ghost_regions(body)
    require(any(all(token in region for token in tokens) for region in regions),label+' not contained in one Ghost region')

def audit_ghost_and_branch_placement(appendix,ax):
    def raw(name):return ax.extract_function(appendix,name)[1]
    for name in ('inc_start_root_view','root_view_as_slice'):
        require_ghost(raw(name),['Phase::Raw(raw)=>&raw.physical','Phase::Shared(shared)','full.borrow(&shared.root.ticket.token)'],name+' phase physical selection')
    terminal=raw('bytes_root_view_terminal_drop')
    require_ghost(terminal,['descriptor.base.raw_pointer()','ifaddress&1usize==0usize','even_root_view_drop_registration()','odd_root_view_drop_registration()'],'Root terminal immutable-base spec selection')
    peer=raw('bytes_root_peer_terminal_drop')
    require_ghost(peer,['cursor_shared_drop_registration()'],'peer registration selection')
    require_ghost(peer,['Phase::Shared(shared)=>&mutshared.cursor'],'peer Shared cursor reborrow')
    for parity in ('even','odd'):
        body=raw(parity+'_root_view_drop_checked')
        match=re.search(r'\bif\s+kind\s*==\s*0usize\s*\{',body);require(match is not None,'callback native kind guard missing')
        shared,end=braced_region(body,match.end()-1)
        tail=body[end+1:];other=re.match(r'\s*else\s*\{',tail);require(other is not None,'callback raw else arm missing')
        raw_arm,_=braced_region(tail,other.end()-1)
        require('release_core(' in shared and 'free_raw_suffix_checked(' not in shared and 'free_raw_suffix_checked(' in raw_arm and 'release_core(' not in raw_arm,'callback effects are in incompatible kind arms')
        require_ghost(shared,['RootDropRemainder::Shared(root,cursor)=>(root,cursor)'],'Shared affine remainder match')
        require_ghost(raw_arm,['RootDropRemainder::Raw(recovery,physical)=>(recovery,physical)'],'Raw affine remainder match')
    return {'ghost_placement':'phase physical selection, immutable-base spec, peer cursor/spec, affine remainder matches','native_branch_placement':'Shared release only kind==0; raw free only else'}

def audit_full_proof(mapping):
    prefix='verif/bytes_original_root_phase_clone_rlib/'
    current=sorted(p.relative_to(ROOT).as_posix() for p in (ROOT/'verif').rglob('*.coma'))
    require(current and len(current)==len(set(current)),'no complete current translation inventory')
    inherited=json.loads((ROOT/'inherited-targets.json').read_bytes())
    old={prefix+p for p in inherited['relative_targets']}
    require(len(old)==186 and old<=set(current),'inherited AY target disappeared')
    new=set(current)-old
    require(new and all(p.startswith(prefix+'promotion/') for p in new),'new target outside reviewed appendix')
    for target in new:
        require(target[len(prefix+'promotion/'):-5] in mapping['new_functions'],'new translated target has no appendix function')
    policy=json.loads((ROOT/'generated/proof-targets.json').read_bytes())
    require(set(policy['translated'])==set(current) and set(policy['included'])==set(current) and policy['excluded']=={},'current full policy omits targets')
    require(policy['features']==[] and policy['terminal_feature']==policy['source_control']=='' and policy.get('requested_development_targets',[])==[],'full policy enables feature/control/focus selection')
    require(policy['diagnostic'] is True and policy['correspondence_exit_status']==2,'origin must retain explicit diagnostic checker-skip status')
    receipt_raw=(ROOT/'evidence/AZ_FULL_PROOF_RUN.json').read_bytes();receipt=json.loads(receipt_raw)
    require(receipt['status']=='complete_positive_diagnostic' and receipt['prover_process_exit']==0 and receipt['features']==[] and receipt['excluded']=={} and receipt['policy_diagnostic'] is True and receipt['policy_correspondence_exit']==2,'full receipt does not describe one complete positive diagnostic')
    rows=receipt['proof_file_identity'];expected=set(current)|{p[:-5]+'/proof.json' for p in current}
    require(len(rows)==len(expected) and len({r['path'] for r in rows})==len(rows) and {r['path'] for r in rows}==expected,'full proof output identity inventory is incomplete')
    proofs=[]
    for row in rows:
        p=ROOT/row['path'];require(p.is_file() and not p.is_symlink(),'missing proof output')
        raw=p.read_bytes();require((sha(raw),len(raw))==(row['sha256'],row['size']),'proof output differs from full-run origin')
        if row['path'].endswith('/proof.json'):proofs.append(json.loads(raw))
    stats=dict(files=len(proofs),prover=0,null=0,structural=0)
    def visit(node):
        if node is None:stats['null']+=1
        elif isinstance(node,dict) and 'children' in node:
            require(isinstance(node['children'],list),'proof children are not a list')
            if not node['children']:stats['structural']+=1
            for child in node['children']:visit(child)
        elif isinstance(node,dict) and 'prover' in node:stats['prover']+=1
        else:raise RuntimeError('unknown full proof tree node')
    for proof in proofs:
        require(isinstance(proof.get('proofs',{}).get('Coma'),dict),'full proof lacks Coma tree')
        for node in proof['proofs']['Coma'].values():visit(node)
    require(stats==receipt['statistics'] and stats['files']==len(current) and stats['null']==stats['structural']==0,'full proof statistics do not represent complete success')
    log_raw=(ROOT/'evidence/full-diagnostic-v1.log').read_bytes()
    require(sha(log_raw)==receipt['log_sha256'],'full origin log changed')
    input_rows=receipt['probe_input_identity'];require(input_rows and len({r['path'] for r in input_rows})==len(input_rows),'full origin input inventory empty or duplicate')
    for row in input_rows:
        path=Path(row['path']);require(not path.is_absolute() and '..' not in path.parts,'unsafe origin input path')
        raw=(ROOT/path).read_bytes();require((sha(raw),len(raw))==(row['sha256'],row['size']),'full origin source/input identity changed')
    require('src/promotion.rs' in {r['path'] for r in input_rows},'full origin does not bind selected Rust source')
    return dict(status='complete_positive_diagnostic_not_admitted',statistics=stats,inherited_AY_targets=186,new_AZ_targets=len(new),proof_output_files=len(rows),origin_receipt_sha256=sha(receipt_raw),log_sha256=sha(log_raw),prover_reexecuted=False,diagnostic_policy_exit=2)



def audit(*,capture_compiled=False,captured_only=False,source_only=False):
    ancestor=ROOT.parents[1]/'evidence-closures/ay-admitted-reuse-v1/manifest.json'
    require(sha(ancestor.read_bytes())=='71ed49af75822b9281ec87b7d0009c13463cf4989ebbcf71f36705cc586139d8','published AY ancestor raw manifest changed')
    require(sha(((ROOT.parent/'original-root-phase-view-2026-10-09')/'evidence/AY_FULL_PROOF_RUN.json').read_bytes())=='c10232a3db94caa520eeabf25b9be1387c71fb1e41cd17cab5aecf27d09b3dc4','published AY full positive receipt changed')
    ax=load_module(AX/'check_correspondence.py','az_pinned_AX_utilities',AX_CHECKER_SHA)
    mapping=json.loads((ROOT/'generated/mapping.json').read_bytes())
    for row in mapping['source_inventory']:
        raw=(ROOT/row['path']).read_bytes();require((sha(raw),len(raw))==(row['sha256'],row['size']),'current source inventory differs from mapping')
    native=load_module(ROOT/'check_native.py','az_pinned_native','eb6413cef39d685cd06b96878483f29c653170d3680a076608a16ae6ac257974')
    native_report=native.audit_bundle(native.load_bundle());require(native_report['status']=='pass','AZ independent native audit rejected')
    require(mapping['native_report']==json.loads(json.dumps(native_report)),'mapping native facts differ from current independent audit')
    source=audit_source(mapping,native_report,ax)
    report=dict(status='pass',checker_scope='AZ connected Root Clone lifecycle: arbitrary finite capped steps; one Acquire and kind-specific promotion/shared Clone; lexical peer Drop before loop backedge; common final view read and Root Drop',full_original_admitted=False,source_native=source,ancestor_raw_manifest_sha256='71ed49af75822b9281ec87b7d0009c13463cf4989ebbcf71f36705cc586139d8',ancestor_checker_replayed=False)
    if source_only:return report|dict(proof_inventory='not checked',Cargo_snapshot='not checked')
    require(sha((ROOT/'build.rs').read_bytes())==sha((AX/'build.rs').read_bytes()) and sha((ROOT/'extract_public.py').read_bytes())==sha((AX/'extract_public.py').read_bytes()),'production build/extractor route changed')
    adapter=cargo_adapter(ax)
    # Snapshot only current source/native and completed Cargo outputs. The proof
    # origin and policy disposition are intentionally not prerequisites here.
    if capture_compiled:
        cargo=adapter.audit_capture(live_capture=True)
        return report|dict(Cargo_snapshot=cargo,proof_inventory='not checked; source/native/Cargo snapshot only')
    cargo=adapter.audit_capture(live_capture=False)
    if captured_only:return report|dict(Cargo_snapshot=cargo,proof_inventory='not checked; captured Cargo-only audit')
    proof=audit_full_proof(mapping)
    return report|dict(Cargo_snapshot=cargo,proof_inventory=proof)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--shadow',type=Path,default=ROOT/'src/promotion.rs')
    parser.add_argument('--mapping',type=Path,default=ROOT/'generated/mapping.json')
    parser.add_argument('--capture-compiled-inputs',action='store_true')
    parser.add_argument('--audit-compiled-capture-only',action='store_true')
    parser.add_argument('--source-native-only',action='store_true')
    parser.add_argument('--output',type=Path)
    args=parser.parse_args()
    try:
        require(args.shadow.resolve()==(ROOT/'src/promotion.rs').resolve() and args.mapping.resolve()==(ROOT/'generated/mapping.json').resolve(),'checker input redirects selected source/mapping')
        require(sum((args.capture_compiled_inputs,args.audit_compiled_capture_only,args.source_native_only))<=1,'audit modes conflict')
        report=audit(capture_compiled=args.capture_compiled_inputs,captured_only=args.audit_compiled_capture_only,source_only=args.source_native_only)
    except Exception as exc:report=dict(status='reject',reason=f'{type(exc).__name__}: {exc}',full_original_admitted=False)
    raw=json.dumps(report,indent=2,ensure_ascii=False)+'\n'
    if args.output:args.output.write_text(raw)
    print(raw,end='');return 0 if report['status']=='pass' else 1

if __name__=='__main__':raise SystemExit(main())
