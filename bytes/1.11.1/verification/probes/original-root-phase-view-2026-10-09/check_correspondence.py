#!/usr/bin/env python3
"""AY operational source/native joins; exact proof/Cargo receipts are separate.

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
PACKAGE='bytes-original-root-phase-view'

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
    reviewed_raw=(ROOT/'AY_REVIEWED_SOURCE.json').read_bytes()
    require(sha(reviewed_raw)=='e57d83e839aca03a16167abfad596c36d668ed8c97c4703a4a3efe03ea86aed1','independently reviewed pre-proof source declaration changed')
    reviewed=json.loads(reviewed_raw)
    prefix=(AX/'src/promotion.rs').read_bytes();raw=(ROOT/'src/promotion.rs').read_bytes()
    require(sha(prefix)==PREFIX_SHA and raw.startswith(prefix),'AY lost its immutable AX source prefix')
    require(sha(raw)==reviewed['source_sha256'] and sha((ROOT/'src/root_phase_extension.rs').read_bytes())==reviewed['mirror_sha256'],'current semantic source differs from independently reviewed freeze')
    require(mapping['selected_source_sha256']==sha(raw),'mapping does not identify current source')
    appendix=raw[len(prefix):].decode();full=raw.decode()
    require(b'\n'+(ROOT/'src/root_phase_extension.rs').read_bytes()==raw[len(prefix):],'AY appendix mirror differs from selected source')
    def body(name):return compact(ax.extract_function(appendix,name)[1])
    client=body('root_phase_scope')
    ordered(client,['from_box_scoped(input)','SuffixScope::new(scope)','advance_root_view(&mutvalue,first,suffix.borrow_mut())','ifpromote{','clone_suffix_root(&value,suffix.borrow_mut())','bytes_root_peer_terminal_drop(peer,suffix.borrow_mut(),peer_receipt.borrow_mut())','advance_root_view(&mutvalue,second,suffix.borrow_mut())','chunk_root_view(&value,suffix.borrow()).to_vec()','letsaved_return=result','bytes_root_view_terminal_drop(value,suffix,receipt.borrow_mut())','saved_return'],'AY witness')
    require(client.count('advance_root_view(')==2 and client.count('clone_suffix_root(')==1 and client.count('bytes_root_peer_terminal_drop(')==1 and client.count('bytes_root_view_terminal_drop(')==1,'AY witness duplicates lifecycle operation')
    # Braces locate the runtime branch itself; a same-order bag of calls is insufficient.
    start=client.index('ifpromote{')+len('ifpromote');depth=0;end=None
    for i in range(start,len(client)):
        if client[i]=='{':depth+=1
        elif client[i]=='}':
            depth-=1
            if depth==0:end=i;break
    require(end is not None,'runtime branch is unbalanced')
    branch=client[start:end+1];continuation=client[end+1:]
    require('clone_suffix_root(' in branch and 'bytes_root_peer_terminal_drop(' in branch and 'advance_root_view(&mutvalue,second' not in branch,'peer Clone/Drop is not contained in runtime promote branch')
    require('advance_root_view(&mutvalue,second' in continuation and 'clone_suffix_root(' not in continuation,'second common advance is not after branch join')
    advance=body('advance_root_view');inc=body('inc_start_root_view')
    ordered(advance,['assert!(amount<=value.len()','inc_start_root_view(value,amount,scope)'],'common advance')
    ordered(inc,['value.len-=amount','cursor_pointer::add(value.ptr,amount,bound,lease)','value.ptr=ptr','scope.view=shifted.into_inner()'],'common pointer increment')
    require(inc.count('cursor_pointer::add(')==1 and 'value.data=' not in inc and 'value.vtable=' not in inc,'common advance mutates owner or duplicates pointer add')
    read=body('root_view_as_slice');chunk=body('chunk_root_view')
    require(chunk=='root_view_as_slice(value,scope)','common chunk no longer delegates to current view read')
    for name,value in [('advance',inc),('read',read)]:
        require('Phase::Raw(raw)=>&raw.physical' in value and 'full.borrow(&shared.root.ticket.token)' in value and 'Phase::Shared(shared)' in value,f'{name} does not select live Raw or root-ticket physical borrow')
    require(read.count('physical_projection::borrow(value.ptr,value.len,bound,region)')==1,'read is not one physical borrow at current pointer/length')
    terminal=body('bytes_root_view_terminal_drop')
    ordered(terminal,['letnative=value.vtable.drop','descriptor.base.raw_pointer()','ifaddress&1usize==0usize','even_root_view_drop_registration()','odd_root_view_drop_registration()','erased_call::invoke3(native,(&mutvalue.data,value.ptr,value.len)'],'Root terminal')
    require(terminal.count('erased_call::invoke3(')==1 and 'value.ptr.addr' not in terminal and '(scope.into_inner(),&mut**output)' in terminal,'Root terminal dispatch key or affine argument package differs')
    peer=body('bytes_root_peer_terminal_drop')
    ordered(peer,['letnative=value.vtable.drop','cursor_shared_drop_registration()','Phase::Shared(shared)=>&mutshared.cursor','erased_call::invoke3(native,(&mutvalue.data,value.ptr,value.len)'],'lexical peer adapter')
    require(peer.count('erased_call::invoke3(')==1 and '(value.original_shared.into_inner(),cursor.into_inner(),&mut**output)' in peer,'peer adapter does not pass its exact owner/cursor/output')
    require(appendix.count('#[trusted]')==2,'AY introduced an unreviewed trusted body')
    for parity in ('even','odd'):
        callback=body(parity+'_root_view_drop_checked')
        ordered(callback,['suffix.scope.phase.take().unwrap()','owned_pointer::get_mut_finish(data,own)','letkind=crate::provenance_specs::pointer_addr(word)&1usize','ifkind==0usize','release_core(word.cast(),root,cursor.borrow_mut(),completion.borrow_mut())','}else{','free_raw_suffix_checked(base,offset,len'],'native kind-branch callback '+parity)
        require(callback.count('owned_pointer::get_mut_finish(')==1 and callback.count('release_core(')==1 and callback.count('free_raw_suffix_checked(')==1,'callback duplicates owned read/release/free')
        require('RootDropRemainder::Raw(raw.recovery,raw.physical)' in callback and 'RootDropRemainder::Shared(shared.root,shared.cursor)' in callback,'callback does not preserve exact phase remainder')
        require('(descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())' in callback,'raw callback free input does not consume exact affine resources')
        decoder='tag_specs::clear_low_bit(word,ghost!{&descriptor.base})' if parity=='even' else 'word.cast::<u8>()'
        require(decoder in callback,'callback tag decoder/cast changed')
        require('RootDropEffect::Shared(DetachedScope{cursor:cursor.into_inner()},completion.into_inner().unwrap())' in callback and 'RootDropEffect::Raw(receipt.into_inner())' in callback,'callback effect package changed')
        registration=compact(ax.extract_function(appendix,parity+'_root_view_drop_registration')[0])
        require('Ghost::conjure()' in registration,'registration body differs from generic TCB instance')
        function_position=appendix.index('fn '+parity+'_root_view_drop_registration')
        contract_start=appendix.rfind('#[trusted]',0,function_position)
        contract=compact(appendix[contract_start:function_position])
        require('erased_call::registered3('+parity+'_table().drop,result.inner_logic())' in contract and 'result.inner_logic().precondition((data,ptr,len,input))=='+parity+'_root_view_drop_checked.precondition((data,ptr,len,input))' in contract and 'result.inner_logic().postcondition((data,ptr,len,input),())=='+parity+'_root_view_drop_checked.postcondition((data,ptr,len,input),())' in contract,'callback registration does not bind precise proved body contracts')
    require('typeRootViewDropInput' in compact(appendix) and '=(SuffixScope,&\'amutOption<RootDropEffect>)' in compact(appendix),'Root callback input package changed')
    placement=audit_ghost_and_branch_placement(appendix,ax)
    client_native=native['native_audit']['client']
    return dict(status='pass',source_sha256=sha(raw),appendix_sha256=sha(raw[len(prefix):]),source_join_scope='runtime promote branch/lexical peer adapter/common continuation; phase-selected physical borrow; one consuming owned-field read followed by native kind branches; immutable-base terminal registration',native_client=client_native,ghost_and_branch_placement=placement,immutable_AX_constructor_advance_free_helpers=True,full_original_admitted=False)


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
    prefix='verif/bytes_original_root_phase_view_rlib/'
    current=sorted(p.relative_to(ROOT).as_posix() for p in (ROOT/'verif').rglob('*.coma'))
    require(current and len(current)==len(set(current)),'no complete current translation inventory')
    inherited=json.loads((ROOT/'inherited-targets.json').read_bytes())
    old={prefix+p for p in inherited['relative_targets']}
    require(len(old)==177 and old<=set(current),'inherited AX target disappeared')
    new=set(current)-old
    require(new and all(p.startswith(prefix+'promotion/') for p in new),'new target outside reviewed appendix')
    for target in new:
        require(target[len(prefix+'promotion/'):-5] in mapping['new_functions'],'new translated target has no appendix function')
    policy=json.loads((ROOT/'generated/proof-targets.json').read_bytes())
    require(set(policy['translated'])==set(current) and set(policy['included'])==set(current) and policy['excluded']=={},'current full policy omits targets')
    require(policy['features']==[] and policy['terminal_feature']==policy['source_control']=='' and policy.get('requested_development_targets',[])==[],'full policy enables feature/control/focus selection')
    require(policy['diagnostic'] is True and policy['correspondence_exit_status']==2,'origin must retain explicit diagnostic checker-skip status')
    receipt_raw=(ROOT/'evidence/AY_FULL_PROOF_RUN.json').read_bytes();receipt=json.loads(receipt_raw)
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
    return dict(status='complete_positive_diagnostic_not_admitted',statistics=stats,inherited_AX_targets=177,new_AY_targets=len(new),proof_output_files=len(rows),origin_receipt_sha256=sha(receipt_raw),log_sha256=sha(log_raw),prover_reexecuted=False,diagnostic_policy_exit=2)


def audit(*,capture_compiled=False,captured_only=False,source_only=False):
    ancestor=ROOT.parents[1]/'evidence-closures/ax-admitted-reuse-v1/manifest.json'
    require(sha(ancestor.read_bytes())=='05cca18b78627f28340e01f86abc52c7e75669734f1143389b832d7a1db5e8a5','published AX ancestor raw manifest changed')
    require(sha((AX/'evidence/AX_FULL_PROOF_RUN.json').read_bytes())=='9499ae885e6339eaca9897bce69ea46c3fd133c9ef28357ffc77ca63cf9a54fd','published AX full positive receipt changed')
    ax=load_module(AX/'check_correspondence.py','ay_pinned_AX_utilities',AX_CHECKER_SHA)
    mapping=json.loads((ROOT/'generated/mapping.json').read_bytes())
    for row in mapping['source_inventory']:
        raw=(ROOT/row['path']).read_bytes();require((sha(raw),len(raw))==(row['sha256'],row['size']),'current source inventory differs from mapping')
    native=load_module(ROOT/'check_native.py','ay_pinned_native','d0afa9123d6187b463c827118645a145cbd84b5e7eb9edfb4bd58ffd3f690372')
    native_report=native.audit_bundle(native.load_bundle());require(native_report['status']=='pass','AY independent native audit rejected')
    require(mapping['native_report']==json.loads(json.dumps(native_report)),'mapping native facts differ from current independent audit')
    source=audit_source(mapping,native_report,ax)
    report=dict(status='pass',checker_scope='AY connected normal Root lifecycle: runtime optional promotion and lexical peer Drop; common view advance/read; consuming raw/shared Root Drop',full_original_admitted=False,source_native=source,ancestor_raw_manifest_sha256='05cca18b78627f28340e01f86abc52c7e75669734f1143389b832d7a1db5e8a5',ancestor_checker_replayed=False)
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
