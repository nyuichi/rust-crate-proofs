#!/usr/bin/env python3
"""AR cursor closure: staged ghost-dispatch prerequisite, then normal-MIR client."""
from pathlib import Path
import argparse,hashlib,json,re
ROOT=Path(__file__).resolve().parent
AQ=ROOT.parent/'original-shared-slice-views-2026-10-09'
ANCESTOR_SHA='167f08c84980ff5ab80c7db50909a99879294c98ed4bc11ff7453d05c267e42b'
BASE_SHA='92f6115507d4648dca6507737b120a8bea48cc42ed7fa334897fda7bda420fb9'
OLD='enum OriginalSharedProof { Root(RootDescriptor), Child(ChildProof), View(ChildProof,raw_vec::BoundPtr), Empty(EmptyViewProof) }'
NEW='enum OriginalSharedProof { Root(RootDescriptor), Child(ChildProof), View(ChildProof,raw_vec::BoundPtr), Empty(EmptyViewProof), Vacant }'
FEATURES=('', 'wrong_offset', 'wrong_length', 'lose_empty_owner', 'len_dispatch', 'base_read', 'omit_owner', 'omit_value', 'duplicate_value', 'early_advance', 'snapshot_extract', 'zero_without_lease', 'reset_bound', 'wrong_capacity', 'change_identity', 'change_fraction', 'spurious_registration')
def sha(data):return hashlib.sha256(data).hexdigest()
def source_transform():return dict(ancestor_source='../original-shared-slice-views-2026-10-09/generated/positive.rs',ancestor_sha256=ANCESTOR_SHA,old=OLD,new=NEW,transformed_sha256=BASE_SHA)
def frozen_inputs():
    ancestor=(AQ/'generated/positive.rs').read_bytes()
    assert sha(ancestor)==ANCESTOR_SHA and ancestor.decode().count(OLD)==1
    base=(ROOT/'src/promotion.rs').read_bytes()
    assert base==ancestor.decode().replace(OLD,NEW).encode() and sha(base)==BASE_SHA
    for path in (AQ/'src').glob('*.rs'):
        if path.name=='promotion.rs':continue
        expected=path.read_bytes()
        if path.name=='lib.rs':expected+=b'\n#[cfg(creusot)] mod cursor_pointer;\n'
        assert (ROOT/'src'/path.name).read_bytes()==expected,path.name
    return base.decode()
def extension_source(feature):
    text=(ROOT/'src/cursor_extension.rs').read_text()
    if feature=='wrong_offset':
        text=text.replace('cursor_pointer::add(value.ptr,cursor_by,bound.borrow(),lease)','cursor_pointer::add(value.ptr,0,bound.borrow(),lease)')
    if feature=='wrong_length':text=text.replace('value.len-=cursor_by;','value.len=0;')
    if feature=='lose_empty_owner':
        anchor='    value.ptr=ptr;\n'
        assert text.count(anchor)==1
        text=text.replace(anchor,anchor+'    if value.len==0 { *value=new_empty_view(value.ptr); }\n')
    if feature=='len_dispatch':
        old='''    let spec:Ghost<CursorDropSpec>=ghost! {match &*value.original_shared {
        OriginalSharedProof::Child(_)|OriginalSharedProof::View(_,_)=>cursor_shared_drop_registration().into_inner(),
        OriginalSharedProof::Empty(_)=>cursor_static_drop_registration().into_inner(),
        _=>{proof_assert!(false);panic!()},
    }};'''
        new='''    let spec:Ghost<CursorDropSpec>=ghost! {if value.len==0 {
        cursor_static_drop_registration().into_inner()
    } else {cursor_shared_drop_registration().into_inner()}};'''
        assert text.count(old)==1;text=text.replace(old,new)
    if feature=='base_read':
        text=text.replace('physical_projection::borrow(value.ptr,value.len,bound,region)',
            'physical_projection::borrow(value.ptr,value.len,ghost! {&proof.core.bound},region)')
    if feature=='zero_without_lease':
        text=text.replace('cursor_pointer::add(value.ptr,cursor_by,bound.borrow(),lease)',
            'cursor_pointer::add(value.ptr,cursor_by,bound.borrow(),ghost! {crate::cursor_pointer::AdvanceLease::Zero})')
    if feature in ('reset_bound','wrong_capacity','change_identity','change_fraction'):
        old='''            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>
                OriginalSharedProof::View(p,shifted.into_inner()),'''
        assert text.count(old)==1
        mutation={
            'reset_bound':'let base=p.core.bound; OriginalSharedProof::View(p,base)',
            'wrong_capacity':'p.core.capacity=value.len; OriginalSharedProof::View(p,shifted.into_inner())',
            'change_identity':'p.core.ticket.id+=Int::new(1).into_inner(); OriginalSharedProof::View(p,shifted.into_inner())',
            'change_fraction':'let _unregistered=p.core.ticket.token.split_off(); OriginalSharedProof::View(p,shifted.into_inner())',
        }[feature]
        new='''            OriginalSharedProof::Child(mut p)|OriginalSharedProof::View(mut p,_)=>{
                '''+mutation+'''},'''
        text=text.replace(old,new)
    return text

def client_source(feature):
    owner_drop='    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());\n'
    value_drop='    bytes_cursor_terminal_drop(value,detached.borrow_mut(),value_receipt.borrow_mut());\n'
    client='\n/// A runtime sequence of public advances preserves ownership, including the\n/// owned zero-length final state. Only the actual final vtable Drop retires it.\n#[requires(input@.len()>0)]\n#[requires(a<=b && b@<=input@.len())]\n#[ensures(result@==input@.subsequence(a@+consumed(steps@,steps@.len(),b@-a@),b@))]\npub(crate) fn cursor_scope(input:Box<[u8]>,a:usize,b:usize,steps:&[usize])->Vec<u8> {\n    let expected=snapshot!(input@);\n    let (original,mut scope)=from_box_scoped(input);\n    let owner=clone_root(&original,scope.borrow_mut());\n    let root_id=snapshot!(scope.root_id());\n    let owner_id=snapshot!(owner.child_id());\n    let before=snapshot!((*scope.observation()).0);\n    let metadata=snapshot!(owner.child_public().3);\n    let mut detached_output=ghost! {None::<DetachedScope>};\n    let mut root_receipt=ghost! {None::<Completion>};\n    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());\n    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());\n    let mut detached=ghost! {detached_output.into_inner().unwrap()};\n    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));\n    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*owner_id));\n    let mut value=slice_cursor_entry(&owner,a..b,detached.borrow_mut());\n    let value_id=snapshot!(value.view_id());\n    let value_fraction=snapshot!(value.view_fraction());\n    proof_assert!(value.view_owned() ==> *value_id!=*owner_id);\n    let mut owner_receipt=ghost! {None::<ViewEffect>};\n    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());\n    proof_assert!(owner_receipt.inner_logic()!=None && owner_receipt.inner_logic().unwrap_logic().valid(*metadata));\n    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()==(a==b));\n    proof_assert!((*detached.observation()).0.len()==(if a<b {1int}else{0int}));\n    let mut i=0usize;\n    #[invariant(i@<=steps@.len())]\n    #[invariant(value.api_view_valid() && value.view_accepts(detached.inner_logic()))]\n    #[invariant(value.view_owned()==(a<b))]\n    #[invariant(value.view_owned() ==> value.view_id()==*value_id && value.view_fraction()==*value_fraction && value.view_public().3==*metadata)]\n    #[invariant(value.view_content()==(*expected).subsequence(a@+consumed(steps@,i@,b@-a@),b@))]\n    #[invariant(value.len@==b@-a@-consumed(steps@,i@,b@-a@))]\n    #[variant(steps@.len()-i@)]\n    while i<steps.len() {\n        let cursor_by=core::cmp::min(steps[i],remaining_api(&value));\n        advance_api(&mut value,cursor_by);\n        i+=1;\n    }\n    let observed=chunk_api(&value).to_vec();\n    let rest=remaining_api(&value);\n    advance_api(&mut value,rest);\n    assert_eq!(remaining_api(&value),0);\n    assert!(chunk_api(&value).is_empty());\n    proof_assert!(value.view_owned()==(a<b));\n    proof_assert!(value.view_owned() ==> (*detached.observation()).0.get(value.view_id())==Some(Excl(value.view_fraction())));\n    let saved_return=observed;\n    let mut value_receipt=ghost! {None::<ViewEffect>};\n    bytes_cursor_terminal_drop(value,detached.borrow_mut(),value_receipt.borrow_mut());\n    proof_assert!(value_receipt.inner_logic()!=None && value_receipt.inner_logic().unwrap_logic().valid(*metadata));\n    proof_assert!(value_receipt.inner_logic().unwrap_logic().reclaimed()==(a<b));\n    proof_assert!(value_receipt.inner_logic().unwrap_logic().was_static()==(a==b));\n    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()!=value_receipt.inner_logic().unwrap_logic().reclaimed());\n    proof_assert!((*detached.observation()).0.len()==0);\n    saved_return\n}\n'
    if feature=='omit_owner':client=client.replace(owner_drop,'')
    if feature=='omit_value':client=client.replace(value_drop,'')
    if feature=='duplicate_value':client=client.replace(value_drop,value_drop+value_drop)
    if feature=='snapshot_extract':
        marker='    let value_id=snapshot!(value.view_id());\n'
        client=client.replace(marker,marker+'    let _extracted:Ghost<Bytes>=snapshot!(value).into_ghost();\n')
    if feature=='spurious_registration':
        marker='    let mut owner_receipt=ghost! {None::<ViewEffect>};\n'
        assert client.count(marker)==1
        client=client.replace(marker,
            '    if a<b { let _extra=clone_shared_view(&value,detached.borrow_mut()); }\n'+marker)
    if feature=='early_advance':
        old='    let observed=chunk_api(&value).to_vec();\n    let rest=remaining_api(&value);\n    advance_api(&mut value,rest);\n'
        new='    let borrowed=chunk_api(&value);\n    let rest=remaining_api(&value);\n    advance_api(&mut value,rest);\n    let observed=borrowed.to_vec();\n'
        assert client.count(old)==1;client=client.replace(old,new)
    return client
def native_mapping():
    paths=sorted((ROOT/'native-mir').glob('*ElaborateDrops.after.mir'))
    clients=[p for p in paths if '.cursor_scope.' in p.name]
    places,edges,blocks={},[],[]
    if len(clients)==1:
        text=clients[0].read_text()
        places=dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;',text))
        for block,cleanup,body in re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}',text,re.S):
            blocks.append(dict(block=block,cleanup=bool(cleanup),body_sha256=sha(body.encode())))
            if cleanup:continue
            for place,successor,unwind in re.findall(r'drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]',body):
                owner=next((x for x in ('original','owner','value') if places.get(x)==place),None)
                if owner:edges.append(dict(block=block,place=place,owner=owner,successor=successor,unwind=unwind.strip(),repeated=False,scope='scope' if owner=='original' else 'detached'))
    return dict(native_source='native.rs',native_source_sha256=sha((ROOT/'native.rs').read_bytes()),
        native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients)==1 else None,
        native_mir_ready=len(edges)==3,debug_places=places,normal_edges=edges,mir_blocks=blocks,
        mir=[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p.read_bytes())) for p in paths])
def generate(feature):
    base=frozen_inputs();ext=extension_source(feature);client=client_source(feature);active=base+'\n'+ext+client
    output=ROOT/'generated';output.mkdir(exist_ok=True)
    for n,s in [('active.rs',active),('cursor-extension.rs',ext),('terminal-helper.rs',ext),('elaborated-client.rs',client)]: (output/n).write_text(s)
    if not feature:(output/'positive.rs').write_text(active)
    mapping=dict(feature=feature,status='generated_unchecked',stage='after-ElaborateDrops',full_original_admitted=False,
        base_source='src/promotion.rs',base_source_sha256=sha(base.encode()),selected_prefix_sha256=sha(base.encode()),source_transform=source_transform(),native_alpha_renaming={'inc_start':{'by':'cursor_by'},'cursor_scope':{'by':'cursor_by'},'slice_cursor_entry':{'begin':'view_begin'}},
        extension_source='src/cursor_extension.rs',extension_sha256=sha(ext.encode()),terminal_helpers_sha256=sha(ext.encode()),
        client_sha256=sha(client.encode()),active='generated/active.rs',active_sha256=sha(active.encode()),
        cursor_pointer_support=dict(path='src/cursor_pointer.rs',sha256=sha((ROOT/'src/cursor_pointer.rs').read_bytes())),
        helpers=['bytes_root_detaching_terminal_drop','bytes_view_terminal_drop','bytes_cursor_terminal_drop'],callbacks=['cursor_shared_drop_checked','cursor_static_drop_checked'],
        cursor_methods=['slice_cursor_entry','inc_start_api','advance_api','remaining_api','read_api','chunk_api'],
        ownership_frame='same_api_owner; original data/vtable and actual ticket are preserved independently of byte length',
        fold='consumed(steps,count,capacity): resource-free bounded prefix fold, no unrolling or quota',
        return_evaluation=dict(shadow='let saved_return=observed',after_final_drain=True,before_value_drop=True),
        excluded=['Root cursor mutation','arbitrary concurrent or escaping ownership','unwind','whole crate'],
        **native_mapping())
    (output/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
    print(json.dumps({k:mapping[k] for k in ('stage','feature','active_sha256','extension_sha256','native_mir_ready')}))
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--feature',default='',choices=FEATURES);generate(p.parse_args().feature)
