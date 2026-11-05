from pathlib import Path
import tarfile,hashlib
r=Path(__file__).resolve().parents[1]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for f in ('src/lib.rs','Cargo.toml'):
  assert (r/'source'/f).read_bytes()==t.extractfile('normalize-line-endings-0.3.0/'+f).read()
assert hashlib.sha256((r/'source/src/lib.rs').read_bytes()).hexdigest()=='1ae29ba0393192db102a26bf8db05c15ed05adc7c4474ad1bde05ef2bc937225'
print('Original source/manifest identical to release archive')
