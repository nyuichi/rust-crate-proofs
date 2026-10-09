#!/usr/bin/env python3
"""AU: immutable AT prefix plus a separately proved borrowed-input/owning-return summary."""
from pathlib import Path
import argparse,hashlib,json,re
ROOT=Path(__file__).resolve().parent
AT=ROOT.parent/'original-owned-view-clone-2026-10-09'
AT_COMMIT='e3a9b0f8003c1dd2eb5de55cd36695cc2e7f74e0'
PREFIX_SHA='b5790f53182c689186442dc54428693a0c5144bb46febdb1a0824e8ccffee945'
FEATURES=('',)
def sha(data):return hashlib.sha256(data).hexdigest()
def frozen_inputs():
    ancestor=(AT/'generated/positive.rs').read_bytes()
    assert sha(ancestor)==PREFIX_SHA
    assert (ROOT/'src/promotion.rs').read_bytes()==ancestor
    inherited={p.name:p for p in (AT/'src').glob('*.rs')}
    selected={p.name:p for p in (ROOT/'src').glob('*.rs')}
    assert set(selected)==set(inherited)|{'call_summary_extension.rs'}
    for name,path in inherited.items():
        expected=ancestor if name=='promotion.rs' else path.read_bytes()
        assert selected[name].read_bytes()==expected,name
    return ancestor.decode()
def extension_source(feature):
    assert not feature
    return (ROOT/'src/call_summary_extension.rs').read_text()

CLIENT='''
/// A caller uses a separately proved returned-owner summary. Neither the
/// helper body nor its ghost implementation is inlined at the call site.
#[requires(input@.len()>0)]
#[requires(a<b && b@<=input@.len())]
#[ensures(result@==input@.subsequence(a@+consumed(steps@,steps@.len(),b@-a@),b@))]
pub(crate) fn owned_view_call_scope(input:Box<[u8]>,a:usize,b:usize,steps:&[usize])->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let owner=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let root_fraction=snapshot!(scope.root_fraction());
    let owner_id=snapshot!(owner.child_id());
    let owner_fraction=snapshot!(owner.child_fraction());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(owner.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*owner_id));
    proof_assert!({singleton_replacement(*root_id,*root_fraction,*owner_id,*owner_fraction);true});
    proof_assert!((*detached.observation()).0==FMap::singleton(*owner_id,Excl(*owner_fraction)));
    let mut value=slice_cursor_entry(&owner,a..b,detached.borrow_mut());
    let mut owner_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());
    proof_assert!(owner_receipt.inner_logic()!=None && !owner_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!({singleton_replacement(*owner_id,*owner_fraction,value.view_id(),value.view_fraction());true});
    proof_assert!((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())));
    let allocation=snapshot!(value);
    let mut i=0usize;
    #[invariant(i@<=steps@.len())]
    #[invariant(value.api_view_valid() && value.view_owned() && value.view_accepts(detached.inner_logic()))]
    #[invariant(value.shares_view_allocation(*allocation) && value.view_public().3==*metadata)]
    #[invariant(value.len@==b@-a@-consumed(steps@,i@,b@-a@))]
    #[invariant(value.view_content()==(*expected).subsequence(a@+consumed(steps@,i@,b@-a@),b@))]
    #[invariant((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())))]
    #[variant(steps@.len()-i@)]
    while i<steps.len() {
        let old_id=snapshot!(value.view_id());
        let old_fraction=snapshot!(value.view_fraction());
        let amount=core::cmp::min(steps[i],remaining_api(&value));
        let next=clone_suffix_checked(&value,amount,detached.borrow_mut());
        proof_assert!(next.view_id()!=*old_id);
        proof_assert!((*detached.observation()).0.len()==2);
        let mut retired=ghost! {None::<ViewEffect>};
        bytes_cursor_terminal_drop(value,detached.borrow_mut(),retired.borrow_mut());
        proof_assert!(retired.inner_logic()!=None && retired.inner_logic().unwrap_logic().valid(*metadata));
        proof_assert!(!retired.inner_logic().unwrap_logic().reclaimed());
        proof_assert!({singleton_replacement(*old_id,*old_fraction,next.view_id(),next.view_fraction());true});
        proof_assert!((*detached.observation()).0==FMap::singleton(next.view_id(),Excl(next.view_fraction())));
        value=next;
        i+=1;
    }
    let observed=chunk_api(&value).to_vec();
    let saved_return=observed;
    let mut final_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(value,detached.borrow_mut(),final_receipt.borrow_mut());
    proof_assert!(final_receipt.inner_logic()!=None && final_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(final_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
'''
def client_source(feature):
    assert not feature
    return CLIENT

def named_call_summary(paths,client_path):
    native_name='clone_suffix';shadow_name='clone_suffix_checked'
    helpers=[p for p in paths if '.'+native_name+'.' in p.name]
    if len(helpers)!=1 or client_path is None:return None
    helper=helpers[0];text=helper.read_text();caller=client_path.read_text()
    parsed=re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}',text,re.S)
    normal={block:body for block,cleanup,body in parsed if not cleanup}
    assert not any(re.search(r'\bdrop\(',body) for body in normal.values())
    returns=[dict(block=block,source_place=m.group(1),result_place='_0')
             for block,body in normal.items()
             for m in [re.search(r'_0\s*=\s*move\s+(_\d+)\s*;',body)] if m]
    assert len(returns)==1
    callsites=[]
    for block,cleanup,body in re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}',caller,re.S):
        if cleanup:continue
        match=re.search(r'(_\d+)\s*=\s*'+native_name+r'\(move (_\d+), move (_\d+)\) -> \[return: (bb\d+), unwind: (bb\d+)\]',body)
        if match:
            callsites.append(dict(block=block,result_place=match[1],argument_places=[match[2],match[3]],
                successor=match[4],unwind=match[5],body_sha256=sha(body.encode())))
    assert len(callsites)==1
    extension=(ROOT/'src/call_summary_extension.rs').read_text()
    header_start=extension.index('#[requires(')
    body_start=extension.index(' {',extension.index('fn '+shadow_name+'('))
    contract=extension[header_start:body_start]
    opening=body_start+1
    assert extension[opening]=='{'
    depth=0
    item_end=None
    for index in range(opening,len(extension)):
        if extension[index]=='{':depth+=1
        elif extension[index]=='}':
            depth-=1
            if depth==0:
                item_end=index+1
                break
    assert item_end is not None
    # This source-bound extractor supports the reviewed expression-only body.
    # Reject strings/comments rather than interpreting braces inside them.
    body=extension[opening:item_end]
    assert not any(marker in body for marker in ('\"', "'", '//', '/*'))
    source_item=extension[header_start:item_end]
    return dict(schema='named_direct_call_v1',native_callee=native_name,shadow_callee=shadow_name,
        native_signature='fn clone_suffix(source: &Bytes, amount: usize) -> Bytes',
        shadow_signature='fn clone_suffix_checked(source:&Bytes,amount:usize,mut scope:Ghost<&mut DetachedScope>)->Bytes',
        native_argument_modes=['shared_borrow','copy'],native_result_mode='move_owner',
        erased_arguments=[dict(index=2,name='scope',type='Ghost<&mut DetachedScope>',
            caller_expression='detached.borrow_mut()',mode='exclusive_ghost_reborrow')],
        shadow_source='src/call_summary_extension.rs',shadow_source_sha256=sha(extension.encode()),
        contract_sha256=sha(contract.encode()),source_item_sha256=sha(source_item.encode()),
        source_item_hash_format='exact_utf8_first_attribute_through_matching_closing_brace_inclusive',
        contract_hash_format='exact_utf8_first_attribute_through_signature_excluding_space_before_body',
        caller_function='owned_view_call_scope',
        caller_mir=client_path.relative_to(ROOT).as_posix(),caller_mir_sha256=sha(client_path.read_bytes()),
        proof_coma_path='verif/bytes_original_owned_view_call_summaries_rlib/promotion/clone_suffix_checked.coma',
        proof_hash_binding='fresh_post_translation_checker_receipt',
        proof_target='promotion::clone_suffix_checked',trusted_summary=False,caller_inlines_callee=False,
        callee_mir=helper.relative_to(ROOT).as_posix(),callee_mir_sha256=sha(helper.read_bytes()),
        callee_debug_places=dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;',text)),
        callee_blocks=[dict(block=block,cleanup=bool(cleanup),body_sha256=sha(body.encode()))
                       for block,cleanup,body in parsed],
        normal_owner_returns=returns,normal_owner_drops=[],callsites=callsites,
        native_operations=['Clone::clone(source)','Buf::advance(result,amount)','move result to return'],
        shadow_operations=['clone_owned_api(source,ghost_reborrow(scope))','advance_api(result,amount)','move result to return'],
        excluded=['unwind','indirect or unreviewed callees','untracked escape'])

def native_mapping():
    paths=sorted((ROOT/'native-mir').glob('*ElaborateDrops.after.mir'))
    clients=[p for p in paths if '.owned_view_call_scope.' in p.name]
    places,edges,blocks={},[],[]
    assignment=None
    saved_return=None
    if len(clients)==1:
        source=clients[0].read_text()
        places=dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;',source))
        parsed=re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}',source,re.S)
        normal_bodies={block:body for block,cleanup,body in parsed if not cleanup}
        for block,cleanup,body in parsed:
            blocks.append(dict(block=block,cleanup=bool(cleanup),body_sha256=sha(body.encode())))
            if cleanup:continue
            for place,successor,unwind in re.findall(r'drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]',body):
                owner=next((n for n in ('original','owner','value','next') if places.get(n)==place),None)
                if owner:
                    edge=dict(block=block,place=place,owner=owner,successor=successor,unwind=unwind.strip(),repeated=False,
                        scope='scope' if owner=='original' else 'detached',role='scope_exit')
                    if owner=='value':
                        install=re.search(re.escape(place)+r'\s*=\s*move\s+(_\d+)\s*;',normal_bodies[successor])
                        if install:
                            temporary=install.group(1)
                            stash=f'{temporary} = move {places["next"]};'
                            assert stash in body,'native next must be stashed before dropping old value'
                            assert assignment is None,'one repeated assignment edge'
                            edge.update(repeated=True,role='replacement')
                            assignment=dict(next_place=places['next'],temporary_place=temporary,value_place=place,
                                stash_block=block,stash_statement=stash,drop_block=block,
                                install_block=successor,install_statement=f'{place} = move {temporary};',
                                normal_successor=successor,unwind_successor=unwind.strip())
                        else:edge['role']='final_return'
                    edges.append(edge)
        for block,body in normal_bodies.items():
            match=re.search(r'_0\s*=\s*[^;\n]*::to_vec\([^\n]*\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:\s*(bb\d+)\]',body)
            if match:
                assert saved_return is None
                saved_return=dict(block=block,result_place='_0',successor=match.group(1),unwind=match.group(2))
        finals=[e for e in edges if e['role']=='final_return']
        assert len(finals)==1 and saved_return is not None
        assert saved_return['successor']==finals[0]['block']
    summary=named_call_summary(paths,clients[0] if len(clients)==1 else None)
    return dict(named_call_summaries=[] if summary is None else [summary],native_source='native.rs',native_source_sha256=sha((ROOT/'native.rs').read_bytes()),
        native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients)==1 else None,
        native_mir_ready=len(clients)==1 and len(edges)==4 and assignment is not None and saved_return is not None and summary is not None,
        debug_places=places,normal_edges=edges,mir_blocks=blocks,native_assignment=assignment,native_saved_return=saved_return,
        mir=[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p.read_bytes())) for p in paths])
def generate(feature):
    prefix=frozen_inputs();extension=extension_source(feature);client=client_source(feature)
    active=prefix+'\n'+extension+client
    output=ROOT/'generated';output.mkdir(exist_ok=True)
    for name,text in [('active.rs',active),('call-summary-extension.rs',extension),('terminal-helper.rs',extension),('elaborated-client.rs',client)]:
        (output/name).write_text(text)
    if not feature:(output/'positive.rs').write_text(active)
    mapping=dict(feature=feature,status='generated_unchecked',stage='after-ElaborateDrops',full_original_admitted=False,
        ancestor_commit=AT_COMMIT,ancestor_source='../'+AT.name+'/generated/positive.rs',ancestor_sha256=PREFIX_SHA,
        base_source='src/promotion.rs',base_source_sha256=sha(prefix.encode()),selected_prefix_sha256=PREFIX_SHA,
        extension_source='src/call_summary_extension.rs',extension_sha256=sha(extension.encode()),
        client_sha256=sha(client.encode()),active='generated/active.rs',active_sha256=sha(active.encode()),
        support_inventory={p.relative_to(ROOT).as_posix():sha(p.read_bytes()) for p in sorted((ROOT/'src').glob('*.rs'))},
        helpers=['clone_suffix_checked'],
        logic_helpers=[],
        terminal_helpers=['bytes_root_detaching_terminal_drop','bytes_view_terminal_drop','bytes_cursor_terminal_drop'],
        ownership_frame='same_allocation/has_allocation/shares_view_allocation: complete allocation identity; current ticket/fraction dynamic',
        loop_inventory='cursor map exactly singleton current actual ticket id/fraction; no iteration quota',
        assignment_effect=dict(order=['evaluate named clone_suffix return','drop old value','install next','increment i'],native_address_nonobserving=True),
        return_evaluation=dict(shadow='let saved_return=observed',before_value_drop=True),
        excluded=['Root and general Static Clone','arbitrary escaping/concurrent ownership','unwind','whole crate'],
        **native_mapping())
    (output/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
    print(json.dumps({k:mapping[k] for k in ('feature','active_sha256','extension_sha256','native_mir_ready')}))
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--feature',default='',choices=FEATURES)
    generate(parser.parse_args().feature)
