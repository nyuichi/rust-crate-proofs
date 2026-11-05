from pathlib import Path
import sys,json,hashlib,tarfile,tomllib
p=Path(__file__).parent
root=(p/tomllib.loads((p/'Cargo.toml').read_text())['dependencies']['crate_under_test']['path']).resolve()
rows=[]
with tarfile.open(p/'upstream.crate') as tf:
 for m in tf.getmembers():
  rel=Path(*Path(m.name).parts[1:])
  if m.isfile() and (str(rel)=='Cargo.toml' or rel.suffix=='.rs'):
   data=(root/rel).read_bytes();assert data==tf.extractfile(m).read(),rel
   rows.append({'path':str(rel),'sha256':hashlib.sha256(data).hexdigest()})
if sys.argv[1]=='before':(p/'reproduction-source-before.json').write_text(json.dumps(rows,indent=2))
else:assert rows==json.loads((p/'reproduction-source-before.json').read_text())
print('Original archive source and manifest match; source fingerprint '+sys.argv[1]+' checked')
