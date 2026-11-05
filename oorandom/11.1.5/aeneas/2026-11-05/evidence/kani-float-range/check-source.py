from pathlib import Path
import tarfile,hashlib,json
r=Path(__file__).resolve().parent
with tarfile.open(r/'upstream.crate') as t:
 for m in t.getmembers():
  if m.isfile():
   p=r/'original'/Path(m.name).relative_to('oorandom-11.1.5')
   assert p.read_bytes()==t.extractfile(m).read(),str(p)
s=(r/'src/lib.rs').read_text()
assert 'assume' not in s and 'stub' not in s
assert s.count('#[kani::proof]')==2
print('Original dependency: all archive files byte-identical. Two direct API range harnesses, no assumptions/stubs.')
