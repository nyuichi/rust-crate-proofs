from pathlib import Path
import tarfile,hashlib,json
r=Path(__file__).resolve().parents[1];rows={}
with tarfile.open(r/'evidence/upstream.crate') as t:
 for m in t.getmembers():
  if m.isfile():
   rel=Path(*Path(m.name).parts[1:]);b=(r/'source'/rel).read_bytes();assert b==t.extractfile(m).read()
   rows[str(rel)]=hashlib.sha256(b).hexdigest()
(r/'evidence/source-hashes.json').write_text(json.dumps(rows,indent=2)+'\n')
print('All original source/manifest/archive files identical; no adaptation')
