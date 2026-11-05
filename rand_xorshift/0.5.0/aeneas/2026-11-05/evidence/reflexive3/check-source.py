from pathlib import Path
import json,tarfile,hashlib
p=Path(__file__).resolve().parent
with tarfile.open(p/'upstream.crate')as a:
 for m in a.getmembers():
  if m.isfile():assert(p/'original'/Path(*Path(m.name).parts[1:])).read_bytes()==a.extractfile(m).read(),m.name
for n,h in json.loads((p/'SOURCE-PINS.json').read_text()).items():assert hashlib.sha256((p/n).read_bytes()).hexdigest()==h,n
r=json.loads((p/'xorshift_reflexive_original.llbc').read_text());assert not r['has_errors'];fs=[f for f in r['translated']['fun_decls']if f and isinstance(f['body'],dict)];assert len(fs)==3
calls=[]
def walk(v):
 if isinstance(v,dict):
  if 'Call'in v:calls.append(v)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
for f in fs:walk(f['body'])
assert not calls
for m,t in [('from','From'),('borrow','Borrow'),('borrow_mut','BorrowMut')]:
 n='original_'+m;before=list((p/'mir').glob('*.'+n+'.*.Inline.before.mir'));after=list((p/'mir').glob('*.'+n+'.*.Inline.after.mir'));assert len(before)==len(after)==1
 ns='convert'if m=='from'else'borrow';target=f'<rand_xorshift::XorShiftRng as std::{ns}::{t}<rand_xorshift::XorShiftRng>>::'+m
 assert target in before[0].read_text()and'(inlined '+target+')'in after[0].read_text()
assert 'axiom 'not in(p/'generated/XorshiftReflexiveOriginal.lean').read_text()
print('PASS archiveallmembers/sourcepins/3typedMIR/zeroIRcalls/noopaqueaxiom')
