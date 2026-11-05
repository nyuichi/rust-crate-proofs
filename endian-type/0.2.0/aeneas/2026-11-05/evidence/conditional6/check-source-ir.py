from pathlib import Path
import json,hashlib,tarfile
p=Path(__file__).resolve().parent
for name,h in json.loads((p/'INPUT-PINS.json').read_text()).items():assert hashlib.sha256((p/name).read_bytes()).hexdigest()==h,name
with tarfile.open(p/'upstream.crate') as tf:
 for m in tf.getmembers():
  if m.isfile():assert (p/'original'/Path(*Path(m.name).parts[1:])).read_bytes()==tf.extractfile(m).read(),m.name
s=(p/'std09/convert-mod.rs').read_text()
for body in ['U::from(self)','U::try_from(self)','Ok(U::into(value))','type Error = !;']:assert body in s
raw=json.loads((p/'endian_conditional_original.llbc').read_text());assert not raw['has_errors']
fs=[f for f in raw['translated']['fun_decls'] if f and isinstance(f['body'],dict)];assert len(fs)==6
for b in json.loads((p/'api-bindings6.json').read_text()):
 name=b['caller'];f=next(f for f in fs if f['item_meta']['name'][-1]=={'Ident':[name,0]});assert len(f['generics']['types'])==(3 if name.endswith('try_into') else 2);assert len(f['generics']['trait_clauses'])==1
 calls=[]
 def walk(v):
  if isinstance(v,dict):
   if 'Call' in v:calls.append(v['Call'])
   for x in v.values():walk(x)
  elif isinstance(v,list):
   for x in v:walk(x)
 walk(f['body']);assert len(calls)==1
 owner='BigEndian' if name.startswith('big') else 'LittleEndian';method=name.split('_',1)[1];trait={'into':'Into','try_into':'TryInto','try_from':'TryFrom'}[method];target=f'<endian_type::{owner}<T> as std::convert::{trait}<U>>::{method}'
 before=list((p/'mir-inline-evidence').glob('*.'+name+'.*.Inline.before.mir'));after=list((p/'mir-inline-evidence').glob('*.'+name+'.*.Inline.after.mir'));assert len(before)==len(after)==1
 assert target in before[0].read_text() and target in after[0].read_text() and '(inlined' in after[0].read_text()
assert 'axiom ' not in (p/'generated/EndianConditionalOriginal.lean').read_text()
print('PASS archive/pins/std09source/6typedInlineScopes/6singlecallbackIR/noAddedAxiom')
