from pathlib import Path
p=Path(__file__).resolve().parents[1]
bad=[]
for x in p.rglob('*'):
 if not x.is_file() or any(a in {'target','.lake','.git','__pycache__'} for a in x.relative_to(p).parts):continue
 if '.olean' in x.name or '.ilean' in x.name or x.name.endswith(('.ir','.ir.sig')) or x.read_bytes()[:5]==b'olean':bad.append(str(x.relative_to(p)))
assert not bad,bad
print('All committable files source-only: no Lean compiled extensions or olean-header binaries')
