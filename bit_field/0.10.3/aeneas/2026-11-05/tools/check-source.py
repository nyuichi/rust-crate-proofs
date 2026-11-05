from pathlib import Path
import tarfile,hashlib
r=Path(__file__).resolve().parents[1]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for f in [r/'Cargo.toml',*sorted((r/'src').rglob('*'))]:
  if not f.is_file():continue
  original=t.extractfile('bit_field-0.10.3/'+str(f.relative_to(r))).read()
  assert original==f.read_bytes(),f
  print(hashlib.sha256(original).hexdigest(),f.relative_to(r))
print('Original Rust source and Cargo manifest are byte-identical to the archive')
