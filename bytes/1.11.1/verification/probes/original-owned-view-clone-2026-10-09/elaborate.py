#!/usr/bin/env python3
"""AT: immutable AS prefix plus owned-view Clone and a normal-MIR replacement loop."""
from pathlib import Path
import argparse,hashlib,json,re
ROOT=Path(__file__).resolve().parent
AS=ROOT.parent/'original-nonnull-view-boundaries-2026-10-09'
AS_COMMIT='2e3525dfa642c809c4d8612efd8a2cecb0067500'
PREFIX_SHA='9fbe1698c4a1a338c7bc30eeb60af8b69bc4077304b63acab9b3be34a7650744'
FEATURES=('', 'zero_to_static','missing_registration','wrong_offset','wrong_capacity',
          'omit_replacement_drop','omit_final_drop','duplicate_value','early_drop')
def sha(data):return hashlib.sha256(data).hexdigest()
def frozen_inputs():
    ancestor=(AS/'generated/positive.rs').read_bytes()
    assert sha(ancestor)==PREFIX_SHA
    assert (ROOT/'src/promotion.rs').read_bytes()==ancestor
    inherited={p.name:p for p in (AS/'src').glob('*.rs')}
    selected={p.name:p for p in (ROOT/'src').glob('*.rs')}
    assert set(selected)==set(inherited)|{'owned_clone_extension.rs'}
    for name,path in inherited.items():
        expected=ancestor if name=='promotion.rs' else path.read_bytes()
        assert selected[name].read_bytes()==expected,name
    return ancestor.decode()
def extension_source(feature):
    text=(ROOT/'src/owned_clone_extension.rs').read_text()
    if feature=='zero_to_static':
        old='fn clone_owned_api(source:&Bytes,mut scope:Ghost<&mut DetachedScope>)->Bytes {\n'
        assert text.count(old)==1
        text=text.replace(old,old+'    if source.len==0 { return new_empty_view(source.ptr); }\n')
    if feature=='missing_registration':
        start=text.index('    let old=field_event::increment_owned')
        end=text.index('    let mut pointer_view=',start)
        text=text[:start]+text[end:]
    if feature=='wrong_offset':
        old='        },bound.into_inner())'
        assert text.count(old)==1
        text=text.replace(old,'        },source.bound)')
    if feature=='wrong_capacity':
        old='core:SharedCore {shared:source.shared,bound:source.bound,capacity:source.capacity,'
        assert text.count(old)==1
        text=text.replace(old,'core:SharedCore {shared:source.shared,bound:source.bound,capacity:len,')
    return text

CLIENT='''
/// Arbitrary finite clone succession. The current ticket/fraction may change;
/// the actual allocation and exact suffix view do not.
#[requires(input@.len()>0)]
#[requires(a<b && b@<=input@.len() && advance_by@<=b@-a@)]
#[ensures(result@==input@.subsequence(a@+advance_by@,b@))]
pub(crate) fn owned_view_clone_scope(input:Box<[u8]>,a:usize,b:usize,advance_by:usize,rounds:usize)->Vec<u8> {
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
    advance_api(&mut value,advance_by);
    let allocation=snapshot!(value);
    let bound=snapshot!(value.view_bound());
    let ptr=snapshot!(value.ptr);
    let mut i=0usize;
    #[invariant(i<=rounds)]
    #[invariant(value.api_view_valid() && value.view_owned() && value.view_accepts(detached.inner_logic()))]
    #[invariant(value.shares_view_allocation(*allocation) && value.view_public().3==*metadata)]
    #[invariant(value.view_bound()==*bound && value.ptr==*ptr && value.len@==b@-a@-advance_by@)]
    #[invariant(value.view_content()==(*expected).subsequence(a@+advance_by@,b@))]
    #[invariant((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())))]
    #[variant(rounds@-i@)]
    while i<rounds {
        let old_id=snapshot!(value.view_id());
        let old_fraction=snapshot!(value.view_fraction());
        let next=clone_owned_api(&value,detached.borrow_mut());
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
    text=CLIENT
    replacement='        bytes_cursor_terminal_drop(value,detached.borrow_mut(),retired.borrow_mut());\n'
    final='    bytes_cursor_terminal_drop(value,detached.borrow_mut(),final_receipt.borrow_mut());\n'
    if feature=='omit_replacement_drop':text=text.replace(replacement,'')
    if feature=='omit_final_drop':text=text.replace(final,'')
    if feature=='duplicate_value':text=text.replace(final,final+final)
    if feature=='early_drop':
        text=text.replace('        let next=clone_owned_api(&value,detached.borrow_mut());\n',
            '        let mut too_early=ghost! {None::<ViewEffect>};\n'
            '        bytes_cursor_terminal_drop(value,detached.borrow_mut(),too_early.borrow_mut());\n'
            '        let next=clone_owned_api(&value,detached.borrow_mut());\n')
    return text

def native_mapping():
    paths=sorted((ROOT/'native-mir').glob('*ElaborateDrops.after.mir'))
    clients=[p for p in paths if '.owned_view_clone_scope.' in p.name]
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
    return dict(native_source='native.rs',native_source_sha256=sha((ROOT/'native.rs').read_bytes()),
        native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients)==1 else None,
        native_mir_ready=len(clients)==1 and len(edges)==4 and assignment is not None and saved_return is not None,
        debug_places=places,normal_edges=edges,mir_blocks=blocks,native_assignment=assignment,native_saved_return=saved_return,
        mir=[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p.read_bytes())) for p in paths])
def generate(feature):
    prefix=frozen_inputs();extension=extension_source(feature);client=client_source(feature)
    active=prefix+'\n'+extension+client
    output=ROOT/'generated';output.mkdir(exist_ok=True)
    for name,text in [('active.rs',active),('owned-clone-extension.rs',extension),('terminal-helper.rs',extension),('elaborated-client.rs',client)]:
        (output/name).write_text(text)
    if not feature:(output/'positive.rs').write_text(active)
    mapping=dict(feature=feature,status='generated_unchecked',stage='after-ElaborateDrops',full_original_admitted=False,
        ancestor_commit=AS_COMMIT,ancestor_source='../'+AS.name+'/generated/positive.rs',ancestor_sha256=PREFIX_SHA,
        base_source='src/promotion.rs',base_source_sha256=sha(prefix.encode()),selected_prefix_sha256=PREFIX_SHA,
        extension_source='src/owned_clone_extension.rs',extension_sha256=sha(extension.encode()),
        client_sha256=sha(client.encode()),active='generated/active.rs',active_sha256=sha(active.encode()),
        support_inventory={p.relative_to(ROOT).as_posix():sha(p.read_bytes()) for p in sorted((ROOT/'src').glob('*.rs'))},
        helpers=['shallow_clone_owned_view_checked','shared_owned_view_clone_checked','owned_view_clone_registration','clone_owned_api'],
        logic_helpers=['singleton_replacement','same_allocation','has_allocation','shares_view_allocation'],
        terminal_helpers=['bytes_root_detaching_terminal_drop','bytes_view_terminal_drop','bytes_cursor_terminal_drop'],
        ownership_frame='same_allocation/has_allocation/shares_view_allocation: complete allocation identity; current ticket/fraction dynamic',
        loop_inventory='cursor map exactly singleton current actual ticket id/fraction; no iteration quota',
        assignment_effect=dict(order=['evaluate next clone','drop old value','install next','increment i'],native_address_nonobserving=True),
        return_evaluation=dict(shadow='let saved_return=observed',before_value_drop=True),
        excluded=['Root and general Static Clone','arbitrary escaping/concurrent ownership','unwind','whole crate'],
        **native_mapping())
    (output/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
    print(json.dumps({k:mapping[k] for k in ('feature','active_sha256','extension_sha256','native_mir_ready')}))
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--feature',default='',choices=FEATURES)
    generate(parser.parse_args().feature)
