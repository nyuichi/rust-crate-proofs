from pathlib import Path
import hashlib,json,sys,tarfile
p=Path(__file__).resolve().parent;src=p.parents[1]
data={str(f.relative_to(src)):hashlib.sha256(f.read_bytes()).hexdigest() for f in [src/'Cargo.toml',*(src/'src').rglob('*.rs')]}
assert data==json.loads((p/'source-before.json').read_text())
with tarfile.open(p.parent/'upstream.crate') as archive:
 for rel in data:assert archive.extractfile('byteorder-lite-0.1.0/'+rel).read()==(src/rel).read_bytes()
print('Original source and Cargo.toml unchanged and archive-identical')
