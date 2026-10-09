#!/usr/bin/env python3
"""External elaboration of the exact normal terminal owner from pinned native MIR."""
from pathlib import Path
import argparse,json,hashlib
r=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
p=argparse.ArgumentParser();p.add_argument('--feature',default='');a=p.parse_args()
s=(r/'driver-template.rs').read_text()
call='    bytes_terminal_drop(bytes,completion.borrow_mut());\n'
if a.feature=='omit_drop':s=s.replace(call,'')
elif a.feature=='duplicate_drop':s=s.replace(call,call+call)
elif a.feature=='early_drop':
 s=s.replace('    let observed=original_bytes_as_slice(&bytes).to_vec();','    let borrowed=original_bytes_as_slice(&bytes);\n    let mut completion=ghost! {None::<BoxedCompletion>};\n'+call+'    let observed=borrowed.to_vec();')
 s=s.replace('    let mut completion=ghost! {None::<BoxedCompletion>};\n'+call+'    proof_assert!','    proof_assert!')
elif a.feature:raise SystemExit('unknown terminal control')
(r/'generated/boxed_client.rs').write_text(s)
capture=json.loads((r/'native-mir/capture.json').read_text())
mapping=dict(stage='before Creusot borrow/liveness; external consuming terminal-place',
 native_source='native.rs',native_source_sha256=sha((r/'native.rs').read_bytes()),
 native_capture=capture,normal_edges=[dict(block='bb4',place='_2',owner='bytes',successor='bb5',unwind='bb9',helper='bytes_terminal_drop',output='completion')],
 return_evaluation=dict(block='bb3',operation='_0 = move _4'),
 source_inputs=[dict(path=f,sha256=sha((r/f).read_bytes())) for f in ['src/public_constructor.rs','src/boxed_drop.rs','src/tag_specs.rs','driver-template.rs']],
 shadow=dict(path='generated/boxed_client.rs',sha256=sha(s.encode())),terminal_feature=a.feature,
 representational_routes=['empty Static -> static_drop no allocation','nonempty even PromotableRaw -> clear low bit -> free_boxed_slice','nonempty odd PromotableRaw -> raw pointer -> free_boxed_slice'],
 unreachable_native_branch='KIND_ARC -> release_shared retained in native source/MIR; readonly binding proves kind == KIND_VEC in each checked callback',
 exclusions=['unwind','Clone/promotion','arbitrary move/drop glue','non-default cfg','full-crate admission'],
 generic_tcb=['terminal-place address non-observation and no field glue','native source/MIR/shadow mapping and erasure','assumed exposed-provenance tag roundtrip','equal pointer distance','existing physical read/free and readonly atomic binding'])
(r/'generated/mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
print(json.dumps(dict(feature=a.feature,shadow_sha256=sha(s.encode()),normal_edges=1)))
