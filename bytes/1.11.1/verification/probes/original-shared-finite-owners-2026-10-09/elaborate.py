#!/usr/bin/env python3
"""AP proof-only cyclic normal Drop elaboration. Independent correspondence is required."""
from pathlib import Path
import argparse
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent
AO = ROOT.parent / 'original-promotable-surviving-child-2026-10-09'
BASE_SHA = 'e607bc1c5587b3d5e0a8260a3891807db3ade7ab331c264a3e49d1de443ca984'
FEATURES = ('', 'missing_registration', 'omit_peer', 'skip_drain', 'forget_peer',
            'duplicate_peer', 'duplicate_survivor', 'early_survivor', 'nonempty_vec_drop', 'snapshot_extract')

def sha(data):
    return hashlib.sha256(data).hexdigest()

def frozen_inputs():
    base = (ROOT / 'src/promotion.rs').read_bytes()
    if sha(base) != BASE_SHA or base != (AO / 'generated/positive.rs').read_bytes():
        raise ValueError('AO complete positive prefix changed')
    for old in (AO / 'src').glob('*.rs'):
        if old.name == 'promotion.rs':
            continue
        if (ROOT / 'src' / old.name).read_bytes() != old.read_bytes():
            raise ValueError('frozen AO support changed: ' + old.name)
    return base.decode()

def extension_source(feature):
    return (ROOT / 'src/finite_extension.rs').read_text()

def client_source(feature):
    peer_drop = '        bytes_detached_child_terminal_drop(peer,detached.borrow_mut(),peer_receipt.borrow_mut());\n'
    survivor_drop = '    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());\n'
    drain = """    #[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]
    #[invariant(survivor.child_content()==*expected)]
    #[invariant(survivor.child_public().3==*metadata)]
    #[variant(owners@.len())]
    while let Some(peer)=owners.pop() {
        proof_assert!(finite_inventory(owners@.push_back(peer),survivor,detached.inner_logic()));
        let mut peer_receipt=ghost! {None::<Completion>};
""" + peer_drop + """        proof_assert!(peer_receipt.inner_logic()!=None && !peer_receipt.inner_logic().unwrap_logic().reclaimed());
        proof_assert!(peer_receipt.inner_logic().unwrap_logic().valid(*metadata));
    }
"""
    client = """
/// Runtime count creates an arbitrary finite inventory. Every popped peer is
/// retired by its actual lexical Drop; empty Vec Drop precedes final survivor.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn finite_shared_scope(input:Box<[u8]>,count:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1);
    let mut owners:Vec<Bytes>=Vec::new();
    let mut made=0usize;
    #[invariant(made<=count)]
    #[invariant(owners@.len()==made@)]
    #[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]
    #[invariant(survivor.child_content()==*expected)]
    #[invariant(survivor.child_public().3==*metadata)]
    #[variant(count@-made@)]
    while made<count {
        let old_owners=snapshot!(owners@);
        let old_scope=snapshot!(detached.inner_logic());
        let next=clone_surviving_child(&survivor,detached.borrow_mut());
        ghost! {prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),
            old_scope,snapshot!(detached.inner_logic()));};
        owners.push(next);
        made+=1;
    }
""" + drain + """    proof_assert!(owners@.len()==0 && (*detached.observation()).0.len()==1);
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    empty_vec_terminal_drop(owners);
    let mut child_receipt=ghost! {None::<Completion>};
""" + survivor_drop + """    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
"""
    if feature == 'omit_peer':
        client = client.replace(peer_drop, '')
    elif feature == 'skip_drain':
        client = client.replace(drain, '')
    elif feature == 'forget_peer':
        client = client.replace(peer_drop, '        core::mem::forget(peer);\n')
    elif feature == 'duplicate_peer':
        client = client.replace(peer_drop, peer_drop + peer_drop)
    elif feature == 'duplicate_survivor':
        client = client.replace(survivor_drop, survivor_drop + survivor_drop)
    elif feature == 'early_survivor':
        decl = '    let mut child_receipt=ghost! {None::<Completion>};\n'
        client = client.replace(survivor_drop, '').replace(decl, '')
        client = client.replace('    let observed=borrowed.to_vec();\n', decl + survivor_drop + '    let observed=borrowed.to_vec();\n')
    elif feature == 'snapshot_extract':
        marker = '    let survivor=clone_root(&original,scope.borrow_mut());\n'
        assert client.count(marker) == 1
        client = client.replace(marker, marker + '    let _extracted:Ghost<Bytes>=snapshot!(survivor).into_ghost();\n')
    elif feature == 'nonempty_vec_drop':
        client = client.replace(drain, '')
        client = client.replace('    proof_assert!(owners@.len()==0 && (*detached.observation()).0.len()==1);\n', '')
    return client


def native_mapping():
    native = ROOT / 'native.rs'
    paths = sorted((ROOT / 'native-mir').glob('*ElaborateDrops.after.mir'))
    clients = [p for p in paths if '.finite_shared_scope.' in p.name]
    places, edges, blocks = {}, [], []
    if len(clients) == 1:
        text = clients[0].read_text()
        places = dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;', text))
        for block, cleanup, body in re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}', text, re.S):
            blocks.append(dict(block=block, cleanup=bool(cleanup), body_sha256=sha(body.encode())))
            if cleanup:
                continue
            for place, successor, unwind in re.findall(r'drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]', body):
                owner = next((x for x in ('original', 'peer', 'owners', 'survivor') if places.get(x) == place), None)
                if owner:
                    edges.append(dict(block=block, place=place, owner=owner, successor=successor, unwind=unwind.strip(),
                                      repeated=owner == 'peer',
                                      scope='scope' if owner == 'original' else ('empty_container' if owner == 'owners' else 'detached')))
    return dict(native_source='native.rs', native_source_sha256=sha(native.read_bytes()) if native.is_file() else None,
                native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients) == 1 else None,
                native_mir_ready=[e['owner'] for e in edges] == ['original', 'peer', 'owners', 'survivor'],
                debug_places=places, normal_edges=edges, mir_blocks=blocks,
                loop_regions=dict(creation_header='bb5', creation_backedge=['bb9', 'bb5'],
                    drain_header='bb11', pop_branch='bb12', some_payload_place='_26', some_payload_drop='bb13',
                    some_residual_path=['bb15', 'bb46', ['bb43', 'bb44'], 'bb40', 'bb16', 'bb11'],
                    none_residual_path=['bb14', 'bb38', ['bb35', 'bb36'], 'bb32', 'bb17'],
                    interpretation='cyclic CFG with actual pop-Some payload move; not a finite unrolled trace'),
                mir=[dict(path=p.relative_to(ROOT).as_posix(), sha256=sha(p.read_bytes())) for p in paths])

def generate(feature):
    base = frozen_inputs()
    extension, client = extension_source(feature), client_source(feature)
    selected_base = base
    if feature == 'missing_registration':
        call = '''            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());'''
        assert selected_base.count(call) == 1
        selected_base = selected_base.replace(call, '            let _=state; let _=c;')
    active = selected_base + '\n' + extension + client
    output = ROOT / 'generated'
    output.mkdir(exist_ok=True)
    for name, text in (('active.rs', active), ('terminal-helper.rs', extension),
                       ('finite-extension.rs', extension), ('elaborated-client.rs', client)):
        (output / name).write_text(text)
    if not feature:
        (output / 'positive.rs').write_text(active)
    mapping = dict(feature=feature, status='generated_unchecked', stage='after-ElaborateDrops',
                   base_source='src/promotion.rs', base_source_sha256=sha(base.encode()),
                   selected_prefix_sha256=sha(selected_base.encode()),
                   terminal_helpers_sha256=sha(extension.encode()), extension_source='src/finite_extension.rs',
                   extension_sha256=sha(extension.encode()), client_sha256=sha(client.encode()),
                   active='generated/active.rs', active_sha256=sha(active.encode()),
                   helpers=['bytes_root_detaching_terminal_drop', 'bytes_detached_child_terminal_drop', 'empty_vec_terminal_drop'],
                   callbacks=['shared_child_clone_checked', 'detaching_root_drop_checked', 'child_drop_checked'],
                   inventory='body-proved exact live-map domain and fractions over actual Vec elements plus survivor',
                   return_evaluation=dict(shadow='let saved_return=observed', empty_vector_then_survivor=True),
                   excluded=['unwind completion', 'successful completion for all counts', 'arbitrary concurrent closure', 'whole crate'],
                   tcb=['native compiler/MIR and cyclic normal terminal-place elaboration', 'address nonobservation and no independent Bytes field-drop glue', 'generic empty Vec element-drop and storage-deallocation semantics', 'inherited generic pointer/field/physical/erased-callback boundaries'],
                   **native_mapping())
    (output / 'mapping.json').write_text(json.dumps(mapping, indent=2) + '\n')
    print(json.dumps({k: mapping[k] for k in ('feature', 'active_sha256', 'extension_sha256', 'client_sha256', 'native_mir_ready')}))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--feature', default='', choices=FEATURES)
    generate(parser.parse_args().feature)
