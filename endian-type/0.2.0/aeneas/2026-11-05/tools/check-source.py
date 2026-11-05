from pathlib import Path
import tarfile,hashlib
r=Path(__file__).resolve().parents[1]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for m in t.getmembers():
  if m.isfile():
   p=Path(*Path(m.name).parts[1:]); data=t.extractfile(m).read()
   assert (r/p).read_bytes()==data, str(p)
print('Every original archive file remains byte-identical')
for p in ['src/lib.rs','Cargo.toml','evidence/upstream.crate']:
 print(hashlib.sha256((r/p).read_bytes()).hexdigest(),p)
