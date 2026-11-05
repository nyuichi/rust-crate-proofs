from pathlib import Path
import re,json
r=Path(__file__).resolve().parents[1];x=json.loads((r/'results.json').read_text());assert x['harnesses']==5 and x['checks']==1484
for a in x['APIs']:
 s=(r/a['log']).read_text();assert 'VERIFICATION:- SUCCESSFUL' in s and '1 successfully verified harnesses, 0 failures, 1 total.' in s
 assert 'Status: FAILURE' not in s
 m=re.search(r'\*\* 0 of (\d+) failed \((\d+) unreachable\)',s);assert int(m[1])==a['checks'] and int(m[2])==a['unreachable']
code=(r/'src/lib.rs').read_text();assert 'kani::assume' not in code and 'kani::stub' not in code
print('All five separate original API Kani results match the fixed-size/domain ledger')
