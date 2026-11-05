from pathlib import Path
import json,hashlib,tarfile
p=Path(__file__).resolve().parent
h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
with tarfile.open(p/'upstream.crate') as tf:
 for m in tf.getmembers():
  if m.isfile():assert (p/'original'/Path(*Path(m.name).parts[1:])).read_bytes()==tf.extractfile(m).read(),m.name
assert (p/'first-proof.llbc').read_bytes()==(p/'endian_reflexive_original.llbc').read_bytes()
convert=(p/'std09/convert-mod.rs').read_text();borrow=(p/'std09/borrow.rs').read_text()
assert 'const impl<T> From<T> for T' in convert and 'fn from(t: T) -> T {\n        t\n    }' in convert
assert 'const impl<T: ?Sized> Borrow<T> for T' in borrow and 'fn borrow(&self) -> &T {\n        self\n    }' in borrow
assert 'const impl<T: ?Sized> BorrowMut<T> for T' in borrow and 'fn borrow_mut(&mut self) -> &mut T {\n        self\n    }' in borrow
raw=json.loads((p/'endian_reflexive_original.llbc').read_text());assert not raw['has_errors'];c=raw['translated'];fs=[x for x in c['fun_decls'] if x and isinstance(x['body'],dict)];assert len(fs)==6
for side,owner in [('big','BigEndian'),('little','LittleEndian')]:
 for method,trait in [('from','From'),('borrow','Borrow'),('borrow_mut','BorrowMut')]:
  name=side+'_'+method;f=next(f for f in fs if f['item_meta']['name']==[{'Ident':['endian_reflexive_original',0]},{'Ident':[name,0]}]);assert len(f['generics']['types'])==1 and not f['generics']['trait_clauses'];calls=[]
  def walk(v):
   if isinstance(v,dict):
    if 'Call' in v:calls.append(v)
    for z in v.values():walk(z)
   elif isinstance(v,list):
    for z in v:walk(z)
  walk(f['body']);assert not calls
  before=list((p/'mir-inline-evidence').glob('*.'+name+'.*.Inline.before.mir'));after=list((p/'mir-inline-evidence').glob('*.'+name+'.*.Inline.after.mir'));assert len(before)==len(after)==1
  ns='convert' if method=='from' else 'borrow';target=f'<endian_type::{owner}<T> as std::{ns}::{trait}<endian_type::{owner}<T>>>::{method}'
  assert target in before[0].read_text() and '(inlined '+target+')' in after[0].read_text()
assert 'axiom ' not in (p/'generated/EndianReflexiveOriginal.lean').read_text()
print('PASS fullarchive/std09bodies/6exacttypedMIRscopes/zeroCallIR/noAddedAxiom')
