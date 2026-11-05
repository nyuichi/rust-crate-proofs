from pathlib import Path
import json,hashlib,tarfile
r=Path(__file__).resolve().parents[1];source=r.parents[1]
archive=r.parent/'upstream.crate'
expected=json.loads((r/'evidence/source-hashes.json').read_text())
with tarfile.open(archive) as t:
 for f,h in expected.items():
  b=(source/f).read_bytes();assert hashlib.sha256(b).hexdigest()==h
  assert b==t.extractfile('rand_pcg-0.10.2/'+f).read()
print('Unchanged original path dependency source/manifest equal release archive')
