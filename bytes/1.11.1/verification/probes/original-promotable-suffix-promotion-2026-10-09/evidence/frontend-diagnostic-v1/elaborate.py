#!/usr/bin/env python3
"""AV: immutable AU prefix and exact promotable suffix first-promotion bodies."""
from pathlib import Path
import argparse,hashlib,json,re
ROOT=Path(__file__).resolve().parent
AU=ROOT.parent/'original-owned-view-call-summaries-2026-10-09'
AU_COMMIT='60d70b215fd29150adf149a9ac793eb7f52b7a1d'
PREFIX_SHA='f3627094e2121b4b52d5d2c881adc6cd23d1f80ac8c33cdf0e21a0ed56ecc1e9'
FEATURES=('', 'wrong_capacity')
def sha(data):return hashlib.sha256(data).hexdigest()
def frozen_inputs():
    ancestor=(AU/'generated/positive.rs').read_bytes()
    assert sha(ancestor)==PREFIX_SHA
    inherited={p.name:p for p in (AU/'src').glob('*.rs')}
    selected={p.name:p for p in (ROOT/'src').glob('*.rs')}
    assert set(selected)==set(inherited)|{'suffix_extension.rs','suffix_pointer.rs'}
    for name,path in inherited.items():
        expected=ancestor if name=='promotion.rs' else path.read_bytes()
        if name=='lib.rs':expected+=b'\n#[cfg(creusot)] mod suffix_pointer;\n'
        assert selected[name].read_bytes()==expected,name
    return ancestor.decode()
def extension_source(feature):
    text=(ROOT/'src/suffix_extension.rs').read_text()
    if feature=='wrong_capacity':
        old='    let cap=distance as usize + len;'
        assert text.count(old)==1
        text=text.replace(old,'    let cap=len;')
    return text
CLIENT='''
/// Actual raw suffix advance precedes first promotion. Root drops before the
/// surviving suffix is read; zero-length suffixes still own the allocation.
#[requires(input@.len()>0 && amount@<=input@.len())]
#[ensures(result@==input@.subsequence(amount@,input@.len()))]
pub(crate) fn promotable_suffix_scope(input:Box<[u8]>,amount:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (mut root,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    advance_suffix(&mut root,amount,suffix.borrow_mut());
    let child=clone_suffix_root(&root,suffix.borrow_mut());
    let root_id=snapshot!(suffix.scope.root_id());
    let child_id=snapshot!(child.view_id());
    let before=snapshot!((*suffix.scope.observation()).0);
    let metadata=snapshot!(child.view_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_suffix_root_terminal_drop(root,suffix,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*child_id));
    proof_assert!(child.view_accepts(*detached));
    let observed=chunk_api(&child).to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(child,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
'''
def native_mapping():
    paths=sorted((ROOT/'native-mir').glob('*ElaborateDrops.after.mir'))
    clients=[p for p in paths if '.promotable_suffix_scope.' in p.name]
    places,edges,blocks={},[],[]
    saved_return=None
    if len(clients)==1:
        source=clients[0].read_text()
        places=dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;',source))
        parsed=re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}',source,re.S)
        for block,cleanup,body in parsed:
            blocks.append(dict(block=block,cleanup=bool(cleanup),body_sha256=sha(body.encode())))
            if cleanup:continue
            for place,successor,unwind in re.findall(r'drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]',body):
                owner=next((name for name in ('root','child') if places.get(name)==place),None)
                if owner:
                    edges.append(dict(block=block,place=place,owner=owner,successor=successor,
                        unwind=unwind.strip(),scope='suffix' if owner=='root' else 'detached',
                        role='promoted_root_scope_exit' if owner=='root' else 'surviving_child_final'))
            match=re.search(r'_0\s*=\s*[^;\n]*::to_vec\([^\n]*\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:\s*(bb\d+)\]',body)
            if match:
                assert saved_return is None
                saved_return=dict(block=block,result_place='_0',successor=match.group(1),unwind=match.group(2))
        finals=[e for e in edges if e['owner']=='child']
        assert len(finals)==1 and saved_return is not None
        assert saved_return['successor']==finals[0]['block']
    return dict(native_source='native.rs',native_source_sha256=sha((ROOT/'native.rs').read_bytes()),
        native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients)==1 else None,
        native_mir_ready=len(clients)==1 and len(edges)==2 and saved_return is not None,
        debug_places=places,normal_edges=edges,mir_blocks=blocks,native_saved_return=saved_return,
        mir=[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p.read_bytes())) for p in paths])
def generate(feature):
    prefix=frozen_inputs();extension=extension_source(feature);client=CLIENT
    active=prefix+'\n'+extension+client
    output=ROOT/'generated';output.mkdir(exist_ok=True)
    for name,text in [('active.rs',active),('suffix-extension.rs',extension),('terminal-helper.rs',extension),('elaborated-client.rs',client)]:
        (output/name).write_text(text)
    if not feature:(output/'positive.rs').write_text(active)
    mapping=dict(feature=feature,status='generated_unchecked',stage='after-ElaborateDrops',full_original_admitted=False,
        ancestor_commit=AU_COMMIT,ancestor_source='../'+AU.name+'/generated/positive.rs',ancestor_sha256=PREFIX_SHA,
        base_source='src/promotion.rs',base_source_sha256=sha(prefix.encode()),selected_prefix_sha256=PREFIX_SHA,
        extension_source='src/suffix_extension.rs',extension_sha256=sha(extension.encode()),
        pointer_source='src/suffix_pointer.rs',pointer_sha256=sha((ROOT/'src/suffix_pointer.rs').read_bytes()),
        client_sha256=sha(client.encode()),active='generated/active.rs',active_sha256=sha(active.encode()),
        support_inventory={p.relative_to(ROOT).as_posix():sha(p.read_bytes()) for p in sorted((ROOT/'src').glob('*.rs'))},
        helpers=['SuffixScope::new','inc_start_suffix','advance_suffix','shallow_clone_suffix_checked',
            'even_suffix_clone_checked','odd_suffix_clone_checked','clone_suffix_root','suffix_root_drop_checked',
            'bytes_suffix_root_terminal_drop'],
        registrations=['even_suffix_clone_registration','odd_suffix_clone_registration','even_suffix_drop_registration','odd_suffix_drop_registration'],
        logic_helpers=['SuffixScope::valid','SuffixScope::matches','SuffixScope::root_valid','SuffixScope::content','suffix_clone_result'],
        terminal_helpers=['bytes_suffix_root_terminal_drop','bytes_cursor_terminal_drop'],
        ownership_frame='one PromotionScope authority; pure current BoundPtr; view offset+len==original capacity',
        allocation_recovery='native offset_from(base) as usize+len; Payload retains original initialized capacity',
        ghost_selector='immutable allocation base/table in Ghost; visible pointer parity not used',
        return_evaluation=dict(shadow='let saved_return=observed',before_child_drop=True),
        excluded=['raw suffix Drop without promotion','concurrent CAS loser','general Static/custom-owner representations',
            'arbitrary escaping/concurrent ownership','unwind','whole crate'],
        **native_mapping())
    (output/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
    print(json.dumps({k:mapping[k] for k in ('feature','active_sha256','extension_sha256','native_mir_ready')}))
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--feature',default='',choices=FEATURES)
    generate(parser.parse_args().feature)
