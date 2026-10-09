#!/usr/bin/env python3
"""Bounded terminal-place shadow; independent checker rederives correspondence."""
from pathlib import Path
import argparse,hashlib,json,re
R=Path(__file__).resolve().parent
BASE=R.parent/'original-shared-scoped-client-2026-10-09'
def sha(b):return hashlib.sha256(b).hexdigest()

def generate(feature):
 base=(BASE/'src/public_shared.rs').read_text()
 a=base.index('#[requires(value.original_shared_valid() && value.accepts(*cursor.inner_logic()))]')
 b=base.index('\nfn original_shared_cleanup(',a)
 contract=base[a:b]
 terminal=contract+'''
fn bytes_terminal_drop(value:Bytes,cursor:Ghost<&mut Cursor>,output:Ghost<&mut Option<Completion>>) {
    original_shared_cleanup(value,cursor,output)
}
'''
 second='    bytes_terminal_drop(second,cursor.borrow_mut(),second_receipt.borrow_mut());\n'
 first='    bytes_terminal_drop(first,cursor.borrow_mut(),first_receipt.borrow_mut());\n'
 driver='''
/// Saved return value followed by certified native terminal normal Drop edges.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result@ == input@)]
pub(crate) fn shared_automatic_scope(input:Vec<u8>)->Vec<u8> {
    let (first,mut cursor)=original_shared_from_vec(input);
    let second=original_shared_clone(&first,cursor.borrow_mut());
    let first_id=snapshot!(first.ticket_id());
    let second_id=snapshot!(second.ticket_id());
    let live_before=snapshot!((*cursor.inner_logic().observation()).0);
    let borrowed=original_shared_as_slice(&second);
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut second_receipt=ghost! {None::<Completion>};
'''+second+'''    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0 == (*live_before).remove(*second_id));
    proof_assert!((*cursor.inner_logic().observation()).0.contains(*first_id));
    proof_assert!(!(*cursor.inner_logic().observation()).0.contains(*second_id));
    let mut first_receipt=ghost! {None::<Completion>};
'''+first+'''    proof_assert!(first_receipt.inner_logic() != None && first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0.len() == 0);
    saved_return
}
'''
 if feature=='omit_second':driver=driver.replace(second,'')
 elif feature=='omit_first':driver=driver.replace(first,'')
 elif feature=='swap_places':
  driver=driver.replace(second,second.replace('(second,','(first,')).replace(first,first.replace('(first,','(second,'))
 elif feature=='duplicate_second':driver=driver.replace(second,second+second)
 elif feature=='wrong_place':driver=driver.replace(second,second.replace('(second,','(first,'))
 elif feature=='early_second':
  driver=driver.replace(second,'').replace('    let mut second_receipt=ghost! {None::<Completion>};\n','')
  driver=driver.replace('    let observed=borrowed.to_vec();\n','    let mut second_receipt=ghost! {None::<Completion>};\n'+second+'    let observed=borrowed.to_vec();\n')
 elif feature:raise ValueError(feature)
 generated=R/'generated';generated.mkdir(exist_ok=True)
 active=base+'\n'+terminal+driver
 (generated/'active.rs').write_text(active)
 (generated/'terminal-helper.rs').write_text(terminal)
 (generated/'elaborated-client.rs').write_text(driver)
 mirpaths=sorted((R/'native-mir').glob('*ElaborateDrops.after.mir'))
 native=(R/'native.rs').read_bytes()
 mapping=dict(feature=feature,stage='2-2-004.ElaborateDrops.after.mir',
  native_source='native.rs',native_source_sha256=sha(native),
  base_source='../original-shared-scoped-client-2026-10-09/src/public_shared.rs',base_source_sha256=sha(base.encode()),
  active='generated/active.rs',active_sha256=sha(active.encode()),
  helper='bytes_terminal_drop',helper_body='ordinary forward call to body-proved original_shared_cleanup; expanded native projected dispatch checked independently',
  normal_edges=[dict(block='bb5',place='_4',owner='second',successor='bb6',unwind='bb10',cursor='cursor',output='second_receipt'),dict(block='bb6',place='_2',owner='first',successor='bb7',unwind='bb12',cursor='cursor',output='first_receipt')],
  mir=[dict(path=p.relative_to(R).as_posix(),sha256=sha(p.read_bytes())) for p in mirpaths],
  return_evaluation=dict(native_block='bb4',native='_0 = move _6',shadow='let saved_return=observed',effects_after_evaluation=True),
  excluded=['unwind cleanup correctness','other representations','arbitrary Drop/move equivalence','arbitrary concurrent closure','whole crate'],
  tcb=['native MIR/compiler interpretation','generic terminal-place/non-address-observing correspondence','ghost erasure and native callback reification','AI scoped field/physical primitives'])
 (generated/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
 print(json.dumps(dict(feature=feature,active_sha256=mapping['active_sha256'],normal_edges=2)))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--feature',default='');a=p.parse_args();generate(a.feature)
