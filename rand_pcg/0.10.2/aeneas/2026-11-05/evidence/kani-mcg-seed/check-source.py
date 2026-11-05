from pathlib import Path
import tarfile,json,hashlib
b=Path(__file__).resolve().parent
with tarfile.open(b/'upstream.crate') as t:
 for m in t.getmembers():
  if m.isfile() and (m.name.endswith('.rs') or m.name.endswith('/Cargo.toml')):
   assert (b/'original'/Path(m.name).relative_to('rand_pcg-0.10.2')).read_bytes()==t.extractfile(m).read(),m.name
s=(b/'src/lib.rs').read_bytes();assert hashlib.sha256(s).hexdigest()==json.loads((b/'provenance.json').read_text())['harness_sha256'];assert b'kani::assume' not in s and b'kani::stub' not in s
print('Original Rust/Cargo unchanged; exact one independent Mcg seed harness, no assumptions/stubs')

p=json.loads((b/"provenance.json").read_text());assert hashlib.sha256((b/"Cargo.lock").read_bytes()).hexdigest()==p["cargo_lock_sha256"]
