from pathlib import Path
p=Path(__file__).resolve().parents[1]
bad=[str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() and not any(a in {'target','.lake','.git','__pycache__'} for a in x.relative_to(p).parts) and ('.olean' in x.name or '.ilean' in x.name)]
assert not bad,bad
print('No Lean compiled sidecars in committable source tree')
