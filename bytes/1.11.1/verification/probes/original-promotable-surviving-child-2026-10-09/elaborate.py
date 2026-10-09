#!/usr/bin/env python3
"""AO proof-only normal Drop elaboration. Independent correspondence is required."""
from pathlib import Path
import argparse
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent
AN = ROOT.parent / 'original-promotable-reclone-2026-10-09'
BASE_SHA = '35c37cca2f01ddc1e12e57de6a2ee5959ea42744ddd7bb6ee9f716c85c65e94c'
FEATURES = ('', 'omit_original', 'omit_survivor', 'omit_handoff', 'duplicate_original',
            'duplicate_survivor', 'duplicate_scope', 'early_survivor', 'swap_adapters')

def sha(data):
    return hashlib.sha256(data).hexdigest()

def frozen_inputs():
    base = (ROOT / 'src/promotion.rs').read_bytes()
    if sha(base) != BASE_SHA or base != (AN / 'generated/positive.rs').read_bytes():
        raise ValueError('AN complete positive prefix changed')
    for old in (AN / 'src').glob('*.rs'):
        if old.name == 'promotion.rs':
            continue
        if (ROOT / 'src' / old.name).read_bytes() != old.read_bytes():
            raise ValueError('frozen AN support changed: ' + old.name)
    return base.decode()

def extension_source(feature):
    extension = (ROOT / 'src/surviving_extension.rs').read_text()
    if feature == 'omit_handoff':
        call = '        ghost! {**detached=Some(DetachedScope {cursor:cursor.into_inner()});};'
        assert extension.count(call) == 1
        extension = extension.replace(call, '')
    return extension

def client_source(feature):
    original = '    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());\n'
    survivor = '    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());\n'
    client = '''
/// Inner original Drop publishes recovery; the surviving child's final Drop
/// recovers it. No original owner or root permission survives the handoff.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_surviving_child_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
''' + original + '''    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!(!(*detached.observation()).0.contains(*root_id));
    proof_assert!((*detached.observation()).0.contains(*child_id));
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!(detached.accepts(survivor));
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<Completion>};
''' + survivor + '''    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
'''
    calls = {'original': original, 'survivor': survivor}
    if feature in ('omit_original', 'omit_survivor'):
        client = client.replace(calls[feature.removeprefix('omit_')], '')
    elif feature in ('duplicate_original', 'duplicate_survivor'):
        call = calls[feature.removeprefix('duplicate_')]
        client = client.replace(call, call + call)
    elif feature == 'duplicate_scope':
        client = client.replace(original, original + '    let _retained_scope=scope;\n')
    elif feature == 'early_survivor':
        declaration = '    let mut child_receipt=ghost! {None::<Completion>};\n'
        client = client.replace(survivor, '').replace(declaration, '')
        client = client.replace('    let observed=borrowed.to_vec();\n', declaration + survivor + '    let observed=borrowed.to_vec();\n')
    elif feature == 'swap_adapters':
        client = client.replace('bytes_root_detaching_terminal_drop(original,', 'bytes_detached_child_terminal_drop(original,')
        client = client.replace('bytes_detached_child_terminal_drop(survivor,', 'bytes_root_detaching_terminal_drop(survivor,')
    return client

def native_mapping():
    native = ROOT / 'native.rs'
    paths = sorted((ROOT / 'native-mir').glob('*ElaborateDrops.after.mir'))
    clients = [p for p in paths if '.promoted_surviving_child_scope.' in p.name]
    places, edges = {}, []
    if len(clients) == 1:
        text = clients[0].read_text()
        places = dict(re.findall(r'debug\s+(\w+)\s*=>\s*(_\d+)\s*;', text))
        for block, cleanup, body in re.findall(r'\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}', text, re.S):
            if cleanup:
                continue
            for place, successor, unwind in re.findall(r'drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]', body):
                owner = next((x for x in ('original', 'survivor') if places.get(x) == place), None)
                if owner:
                    edges.append(dict(block=block, place=place, owner=owner, successor=successor, unwind=unwind.strip(),
                                      scope='scope' if owner == 'original' else 'detached',
                                      output='root_receipt' if owner == 'original' else 'child_receipt'))
    return dict(native_source='native.rs', native_source_sha256=sha(native.read_bytes()) if native.is_file() else None,
                native_client_mir=clients[0].relative_to(ROOT).as_posix() if len(clients) == 1 else None,
                native_mir_ready=[e['owner'] for e in edges] == ['original', 'survivor'],
                debug_places=places, normal_edges=edges,
                mir=[dict(path=p.relative_to(ROOT).as_posix(), sha256=sha(p.read_bytes())) for p in paths])

def generate(feature):
    base = frozen_inputs()
    extension, client = extension_source(feature), client_source(feature)
    active = base + '\n' + extension + client
    output = ROOT / 'generated'
    output.mkdir(exist_ok=True)
    for name, text in (('active.rs', active), ('terminal-helper.rs', extension),
                       ('surviving-extension.rs', extension), ('elaborated-client.rs', client)):
        (output / name).write_text(text)
    if not feature:
        (output / 'positive.rs').write_text(active)
    mapping = dict(feature=feature, status='generated_unchecked', stage='after-ElaborateDrops',
                   base_source='src/promotion.rs', base_source_sha256=sha(base.encode()),
                   terminal_helpers_sha256=sha(extension.encode()), extension_source='src/surviving_extension.rs',
                   extension_sha256=sha(extension.encode()), client_sha256=sha(client.encode()),
                   active='generated/active.rs', active_sha256=sha(active.encode()),
                   helpers=['bytes_root_detaching_terminal_drop', 'bytes_detached_child_terminal_drop'],
                   callbacks=['detaching_root_drop_checked', 'child_drop_checked'],
                   ownership_handoff='consume root core and owned pointer permission; export only actual affine cursor',
                   return_evaluation=dict(shadow='let saved_return=observed', survivor_effect_after_evaluation=True),
                   excluded=['unwind completion', 'concurrent first-promotion loser', 'arbitrary concurrent closure', 'whole crate'],
                   tcb=['native compiler/MIR and normal terminal-place elaboration', 'address nonobservation and no independent field-drop glue', 'inherited generic pointer/field/physical/erased-callback boundaries'],
                   **native_mapping())
    (output / 'mapping.json').write_text(json.dumps(mapping, indent=2) + '\n')
    print(json.dumps({k: mapping[k] for k in ('feature', 'active_sha256', 'extension_sha256', 'client_sha256', 'native_mir_ready')}))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--feature', default='', choices=FEATURES)
    generate(parser.parse_args().feature)
