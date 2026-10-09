#!/usr/bin/env python3
"""AQ selected public slicing and exact normal Drop proof elaboration."""
from pathlib import Path
import argparse
import hashlib
import json
import re
ROOT=Path(__file__).resolve().parent
AP=ROOT.parent/'original-shared-finite-owners-2026-10-09'
ANCESTOR_SHA='cf05c10ecfe38af6ec999a4577a5de5bd5a8bca1d4422c8b94737b2349ef428b'
BASE_SHA='d2bfc0ab0103303bf9656b7e830a74f6a98b21af169331dd69aee2e3493b5428'
OLD='enum OriginalSharedProof { Root(RootDescriptor), Child(ChildProof) }'
NEW='enum OriginalSharedProof { Root(RootDescriptor), Child(ChildProof), View(ChildProof,raw_vec::BoundPtr), Empty(EmptyViewProof) }'
FEATURES=('', 'wrong_offset', 'wrong_length', 'base_read', 'omit_first', 'omit_owner', 'omit_selected',
          'duplicate_selected', 'early_selected', 'snapshot_extract', 'missing_view_registration', 'view_capacity', 'empty_register')
def sha(data): return hashlib.sha256(data).hexdigest()
def source_transform():
    return dict(ancestor_source='../original-shared-finite-owners-2026-10-09/generated/positive.rs',
                ancestor_sha256=ANCESTOR_SHA,old=OLD,new=NEW,transformed_sha256=BASE_SHA)
def frozen_inputs():
    ancestor=(AP/'generated/positive.rs').read_bytes()
    assert sha(ancestor)==ANCESTOR_SHA and ancestor.decode().count(OLD)==1
    expected=ancestor.decode().replace(OLD,NEW).encode()
    base=(ROOT/'src/promotion.rs').read_bytes()
    assert base==expected and sha(base)==BASE_SHA
    for path in (AP/'src').glob('*.rs'):
        if path.name=='promotion.rs': continue
        data=(ROOT/'src'/path.name).read_bytes()
        expected=path.read_bytes()
        if path.name=='lib.rs': expected+=b'\n#[cfg(creusot)] mod view_pointer;\n'
        assert data==expected,path.name
    return base.decode()
def extension_source(feature):
    text=(ROOT/'src/slice_extension.rs').read_text()
    if feature=='wrong_offset': text=text.replace('add_live(ret.ptr,view_begin,bound,region)','add_live(ret.ptr,0,bound,region)')
    if feature=='wrong_length': text=text.replace('ret.len=end-view_begin;','ret.len=end;')
    if feature=='base_read': text=text.replace('physical_projection::borrow(value.ptr,value.len,bound,region)',
        'physical_projection::borrow(value.ptr,value.len,ghost! {&proof.core.bound},region)')
    if feature=='missing_view_registration':
        call='            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),\n                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());'
        assert text.count(call)==1
        text=text.replace(call,'            let _=state; let _=c;')
    if feature=='view_capacity':
        old='core:SharedCore {shared:source.shared,bound:source.bound,capacity:source.capacity,'
        assert text.count(old)==1
        text=text.replace(old,'core:SharedCore {shared:source.shared,bound:source.bound,capacity:len,')
    if feature=='empty_register':
        old='    if end==view_begin {\n'
        assert text.count(old)==1
        text=text.replace(old,old+'        if source.len>0 {\n            let _phantom=clone_shared_view(source,ghost! {&mut **scope});\n        }\n')
    return text

def client_source(feature):
    first_drop='    bytes_view_terminal_drop(first,detached.borrow_mut(),first_receipt.borrow_mut());\n'
    owner_drop='    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());\n'
    selected_drop='    bytes_view_terminal_drop(selected,detached.borrow_mut(),selected_receipt.borrow_mut());\n'
    client=r'''
/// All valid concrete ranges, including nested and one-past empty endpoints.
#[requires(input@.len()>0)]
#[requires(a<=b && b@<=input@.len() && c<=d && d@<=b@-a@)]
#[ensures(result@==input@.subsequence(a@+c@,a@+d@))]
pub(crate) fn nested_slice_scope(input:Box<[u8]>,a:usize,b:usize,c:usize,d:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let owner=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let owner_id=snapshot!(owner.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(owner.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!((*detached.observation()).0.contains(*owner_id));
    let first=slice_view(&owner,a..b,detached.borrow_mut());
    let first_id=snapshot!(first.view_id());
    proof_assert!(first.view_owned() ==> *first_id!=*owner_id);
    proof_assert!(first.view_content()==(*expected).subsequence(a@,b@));
    let selected=slice_view(&first,c..d,detached.borrow_mut());
    let selected_id=snapshot!(selected.view_id());
    proof_assert!(selected.view_owned() ==> *selected_id!=*owner_id && *selected_id!=*first_id);
    proof_assert!(selected.view_content()==(*expected).subsequence(a@+c@,a@+d@));
    proof_assert!((*detached.observation()).0.len()==1+(if a<b {1int}else{0int})+(if c<d {1int}else{0int}));
    let mut first_receipt=ghost! {None::<ViewEffect>};
'''+first_drop+r'''    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(first_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==1+(if c<d {1int}else{0int}));
    let mut owner_receipt=ghost! {None::<ViewEffect>};
'''+owner_drop+r'''    proof_assert!(owner_receipt.inner_logic()!=None && owner_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()==(c==d));
    proof_assert!((*detached.observation()).0.len()==(if c<d {1int}else{0int}));
    let borrowed=read_view(&selected,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut selected_receipt=ghost! {None::<ViewEffect>};
'''+selected_drop+r'''    proof_assert!(selected_receipt.inner_logic()!=None && selected_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(selected_receipt.inner_logic().unwrap_logic().reclaimed()==(c<d));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()!=selected_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
'''
    if feature=='omit_first': client=client.replace(first_drop,'')
    if feature=='omit_owner': client=client.replace(owner_drop,'')
    if feature=='omit_selected': client=client.replace(selected_drop,'')
    if feature=='duplicate_selected': client=client.replace(selected_drop,selected_drop+selected_drop)
    if feature=='early_selected':
        decl='    let mut selected_receipt=ghost! {None::<ViewEffect>};\n'
        client=client.replace(selected_drop,'').replace(decl,'')
        client=client.replace('    let observed=borrowed.to_vec();\n',decl+selected_drop+'    let observed=borrowed.to_vec();\n')
    if feature=='snapshot_extract':
        client=client.replace('    let selected_id=snapshot!(selected.view_id());\n',
            '    let selected_id=snapshot!(selected.view_id());\n    let _extracted:Ghost<Bytes>=snapshot!(selected).into_ghost();\n')
    return client

def native_mapping():
    native = ROOT / 'native.rs'
    paths = sorted((ROOT / 'native-mir').glob('*ElaborateDrops.after.mir'))
    clients = [p for p in paths if '.nested_slice_scope.' in p.name]
    places, edges, blocks = {}, [], []
    if len(clients) == 1:
        text = clients[0].read_text()
        places = dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;', text))
        for block, cleanup, body in re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}', text, re.S):
            blocks.append(dict(block=block, cleanup=bool(cleanup), body_sha256=sha(body.encode())))
            if cleanup:
                continue
            for place, successor, unwind in re.findall(r'drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]', body):
                owner = next((x for x in ('original', 'first', 'owner', 'selected') if places.get(x) == place), None)
                if owner:
                    edges.append(dict(block=block, place=place, owner=owner, successor=successor, unwind=unwind.strip(),
                                      repeated=False,
                                      scope='scope' if owner == 'original' else 'detached'))
    return dict(native_source='native.rs', native_source_sha256=sha(native.read_bytes()) if native.is_file() else None,
                native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients) == 1 else None,
                native_mir_ready=[e['owner'] for e in edges] == ['original', 'first', 'owner', 'selected'],
                debug_places=places, normal_edges=edges, mir_blocks=blocks,
                mir=[dict(path=p.relative_to(ROOT).as_posix(), sha256=sha(p.read_bytes())) for p in paths])

def generate(feature):
    base=frozen_inputs()
    extension=extension_source(feature)
    client=client_source(feature)
    active=base+'\n'+extension+client
    output=ROOT/'generated'
    output.mkdir(exist_ok=True)
    for name,text in (('active.rs',active),('slice-extension.rs',extension),
                      ('terminal-helper.rs',extension),('elaborated-client.rs',client)):
        (output/name).write_text(text)
    if not feature: (output/'positive.rs').write_text(active)
    mapping=dict(feature=feature,status='generated_unchecked',stage='after-ElaborateDrops',
        base_source='src/promotion.rs',base_source_sha256=sha(base.encode()),selected_prefix_sha256=sha(base.encode()),
        source_transform=source_transform(),native_alpha_renaming={'slice':{'begin':'view_begin'}},extension_source='src/slice_extension.rs',extension_sha256=sha(extension.encode()),
        terminal_helpers_sha256=sha(extension.encode()),client_sha256=sha(client.encode()),
        pointer_support=dict(path='src/view_pointer.rs',sha256=sha((ROOT/'src/view_pointer.rs').read_bytes())),
        active='generated/active.rs',active_sha256=sha(active.encode()),
        helpers=['bytes_root_detaching_terminal_drop','bytes_view_terminal_drop'],
        callbacks=['shared_view_clone_checked','child_drop_checked','static_view_drop_checked'],
        view='full original allocation authority; separate absolute view offset; unbound provenance-free Empty',
        return_evaluation=dict(shadow='let saved_return=observed',before_selected_drop=True),
        excluded=['invalid-range panic and unwind','arbitrary RangeBounds implementations','arbitrary concurrent closure','whole crate'],
        tcb=['native compiler/MIR normal terminal-place elaboration','address nonobservation and no independent Bytes field-drop glue',
             'generic live ptr.add and bounded wrapping_add metadata','native provenance-free empty pointer metadata',
             'inherited generic pointer/field/physical/erased-callback boundaries'],**native_mapping())
    (output/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
    print(json.dumps({k:mapping[k] for k in ('feature','active_sha256','extension_sha256','client_sha256','native_mir_ready')}))
if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--feature',default='',choices=FEATURES)
    generate(parser.parse_args().feature)
