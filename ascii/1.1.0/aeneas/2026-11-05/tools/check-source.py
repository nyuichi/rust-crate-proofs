from pathlib import Path
import tarfile,hashlib
r=Path(__file__).resolve().parents[1]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for f in [r/'Cargo.toml',*sorted((r/'src').rglob('*.rs')),*sorted((r/'tests').rglob('*.rs')),*sorted(r.glob('tests.rs'))]:
  orig=t.extractfile('ascii-1.1.0/'+str(f.relative_to(r))).read()
  assert orig==f.read_bytes(), f
  print(hashlib.sha256(orig).hexdigest(),f.relative_to(r))
print('All published Rust source files and Cargo.toml are byte-identical to crates.io archive')
