from pathlib import Path
import json,re
r=Path(__file__).resolve().parents[1]
s=(r/'evidence/kani.log').read_text()
assert 'VERIFICATION:- SUCCESSFUL' in s
assert '1 successfully verified harnesses, 0 failures, 1 total.' in s
assert not re.search(r'Status: FAILURE',s)
p=json.loads((r/'investigation/constructor/normalize_line_endings.llbc').read_text())['translated']
assert p['options']['start_from']==['crate::normalized']
local=[f for f in p['fun_decls'] if f and f['item_meta']['is_local']]
assert len(local)==1 and local[0]['item_meta']['name'][-1]['Ident'][0]=='normalized'
assert 'Unsupported switch scrutinee' in (r/'investigation/scalar-mono/aeneas.log').read_text()
a=(r/'evidence/constructor-audit.log').read_text()
assert 'sorryAx' not in a and 'axioms' in a
print('Original constructor-only extraction and bounded Kani checks passed')
