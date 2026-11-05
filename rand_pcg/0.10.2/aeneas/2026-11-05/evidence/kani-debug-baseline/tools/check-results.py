from pathlib import Path
import json,re
r=Path(__file__).resolve().parents[1];x=json.loads((r/'results.json').read_text());assert len(x['APIs'])==4
for a in x['APIs']:
 s=(r/a['log']).read_text();assert 'VERIFICATION:- SUCCESSFUL' in s and 'Status: FAILURE' not in s
 assert '1 successfully verified harnesses, 0 failures, 1 total.' in s
 m=re.search(r'\*\* 0 of (\d+) failed \((\d+) unreachable\)',s);assert int(m[1])==a['checks'] and int(m[2])==a['unreachable']
assert 'kani::assume' not in (r/'src/lib.rs').read_text()
print('All four constructor/literal/error contracts independently succeeded; scoped Kani supplement only')
