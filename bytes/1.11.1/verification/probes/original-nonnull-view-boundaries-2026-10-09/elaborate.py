#!/usr/bin/env python3
"""AS: exact nonnull-interface repair of the complete published AR positive."""
from pathlib import Path
import argparse,hashlib,json
ROOT=Path(__file__).resolve().parent
AR=ROOT.parent/'original-bytes-cursor-closure-2026-10-09'
AR_COMMIT='ccca44a694d6553226149f089996bf16943c9f25'
AR_POSITIVE_SHA='b30f0128f9b6a2ec935359930af4c8f835d24ab1c031f3618f5829f5f504c113'
FEATURES=('', 'omit_view_nonnull', 'weak_empty_boundary', 'weak_wrapping_boundary')
VIEW_OLD='#[logic(prophetic)] fn view_valid(self)->bool {pearlite! {match self.original_shared.inner_logic() {'
VIEW_NEW='#[logic(prophetic)] fn view_valid(self)->bool {pearlite! {!self.ptr.is_null_logic() && match self.original_shared.inner_logic() {'
EMPTY_OLD="""#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer == bound.inner_logic().raw_pointer() as *const u8)]
#[ensures(result@.len() == 0)]
pub unsafe fn borrow_empty"""
EMPTY_NEW=EMPTY_OLD.replace('#[trusted]','#[trusted]\n#[requires(!pointer.is_null_logic())]',1)
WRAP_OLD="""#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
#[requires(match bound.inner_logic()@ {
    Some((_,cap,offset))=>offset+count@<=cap,None=>count==0usize})]"""
WRAP_NEW=WRAP_OLD.replace('#[trusted]','#[trusted]\n#[requires(!pointer.is_null_logic())]',1)
TRANSFORMS=(
 ('generated/positive.rs','src/promotion.rs',AR_POSITIVE_SHA,VIEW_OLD,VIEW_NEW),
 ('src/physical_projection.rs','src/physical_projection.rs','b4890cf7e9df7e008770264d7f978fdbd32522684d5c93f9e50bc2c9dcd7730f',EMPTY_OLD,EMPTY_NEW),
 ('src/view_pointer.rs','src/view_pointer.rs','11560870cfe83e8d8ace30ebe27c6efa0c618d1368a749f4ace773b89961c6dd',WRAP_OLD,WRAP_NEW),
)
def sha(data):return hashlib.sha256(data).hexdigest()
def transformations():
 rows=[]
 for source,output,expected,old,new in TRANSFORMS:
  before=(AR/source).read_bytes();assert sha(before)==expected,source
  text=before.decode();assert text.count(old)==1,source
  after=text.replace(old,new,1).encode()
  rows.append(dict(ancestor_source='../'+AR.name+'/'+source,ancestor_sha256=expected,
   output=output,old=old,new=new,replacements=1,transformed_sha256=sha(after),native_bodies_unchanged=True))
 return rows

def frozen_inputs():
 rows=transformations()
 for row in rows:
  before=(ROOT/row['ancestor_source']).read_text()
  assert (ROOT/row['output']).read_text()==before.replace(row['old'],row['new'],1),row['output']
 changed={Path(row['output']).name for row in rows}
 inherited={p.name:p for p in (AR/'src').glob('*.rs')}
 selected={p.name:p for p in (ROOT/'src').glob('*.rs')}
 assert set(inherited)==set(selected),'complete inherited Rust source inventory'
 for name,path in inherited.items():
  if name not in changed:assert selected[name].read_bytes()==path.read_bytes(),name
 return (ROOT/'src/promotion.rs').read_text(),rows

WEAK_EMPTY="""
// Diagnostic only: deliberately omit the native nonnull premise.
#[requires(bound.inner_logic().invariant())]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
#[ensures(result@.len()==0)]
unsafe fn weak_empty_boundary<'a>(pointer:*const u8,bound:Ghost<&'a raw_vec::BoundPtr>)->&'a [u8] {
    unsafe {physical_projection::borrow_empty(pointer,bound)}
}
"""
WEAK_WRAP="""
// Diagnostic only: old unbound/count-zero metadata premises, no nonnull fact.
#[requires(bound.inner_logic().invariant() && bound.inner_logic()@==None)]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
fn weak_wrapping_boundary(pointer:*const u8,bound:Ghost<&raw_vec::BoundPtr>)->(*const u8,Ghost<raw_vec::BoundPtr>) {
    crate::view_pointer::wrapping_bounded(pointer,0,bound)
}
"""
def selected_source(feature):
 source,rows=frozen_inputs()
 if feature=='omit_view_nonnull':
  assert source.count(VIEW_NEW)==1;source=source.replace(VIEW_NEW,VIEW_OLD,1)
 elif feature=='weak_empty_boundary':source+=WEAK_EMPTY
 elif feature=='weak_wrapping_boundary':source+=WEAK_WRAP
 return source,rows

def generate(feature):
 source,rows=selected_source(feature)
 output=ROOT/'generated';output.mkdir(exist_ok=True)
 (output/'active.rs').write_text(source)
 if not feature:(output/'positive.rs').write_text(source)
 ancestor_mapping_bytes=(AR/'generated/mapping.json').read_bytes()
 ancestor_mapping=json.loads(ancestor_mapping_bytes)
 # The native client, callback bodies, normal edges and MIR captures are unchanged.
 native_keys=('native_alpha_renaming','helpers','callbacks','cursor_methods','ownership_frame','fold',
  'return_evaluation','excluded','native_source','native_source_sha256','native_client_mir',
  'native_mir_ready','debug_places','normal_edges','mir_blocks','mir')
 native={key:ancestor_mapping[key] for key in native_keys}
 assert sha((ROOT/native['native_source']).read_bytes())==native['native_source_sha256']
 for item in native['mir']:assert sha((ROOT/item['path']).read_bytes())==item['sha256'],item['path']
 mapping=dict(feature=feature,status='generated_unchecked',stage='after-ElaborateDrops',
  full_original_admitted=False,ancestor_commit=AR_COMMIT,
  ancestor_mapping='../'+AR.name+'/generated/mapping.json',ancestor_mapping_sha256=sha(ancestor_mapping_bytes),
  source_transformations=rows,base_source='src/promotion.rs',
  base_source_sha256=sha((ROOT/'src/promotion.rs').read_bytes()),
  active='generated/active.rs',active_sha256=sha(source.encode()),
  diagnostic_added_targets=int(feature in ('weak_empty_boundary','weak_wrapping_boundary')),
  support_inventory={p.relative_to(ROOT).as_posix():sha(p.read_bytes()) for p in sorted((ROOT/'src').glob('*.rs'))},
  **native)
 (output/'mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')
 print(json.dumps({k:mapping[k] for k in ('feature','active_sha256','diagnostic_added_targets')}))
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--feature',default='',choices=FEATURES)
 generate(parser.parse_args().feature)
