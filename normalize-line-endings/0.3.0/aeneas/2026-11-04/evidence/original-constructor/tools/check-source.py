from pathlib import Path
import tarfile,json
r=Path(__file__).resolve().parents[1]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for f in ['src/lib.rs','Cargo.toml']:
  assert (r/'source'/f).read_bytes()==t.extractfile('normalize-line-endings-0.3.0/'+f).read()
p=json.loads((r/'evidence/normalize_line_endings.llbc').read_text())['translated']
assert p['options']['start_from']==['crate::normalized']
a=[f for f in p['fun_decls'] if f and f['item_meta']['is_local']]
assert len(a)==1 and a[0]['item_meta']['name'][-1]['Ident'][0]=='normalized'
print('Archive-identical source/manifest, one original constructor body only')
