from pathlib import Path
import json,re
r=Path(__file__).resolve().parents[1];x=json.loads((r/'results.json').read_text());assert len(x['APIs'])==3 and x['totalChecks']==1162
for a in x['APIs']:
 s=(r/a['log']).read_text();assert 'VERIFICATION:- SUCCESSFUL' in s and 'Status: FAILURE' not in s
 assert '1 successfully verified harnesses, 0 failures, 1 total.' in s
 m=re.search(r'\*\* 0 of (\d+) failed \((\d+) unreachable\)',s);assert int(m[1])==a['checks']
code=(r/'src/lib.rs').read_text();assert code.count('kani::assume(len<=4)')==3 and 'kani::assume(index' not in code
print('Three original Iterator-default bounded contracts, including every usize nth index, match separate successful logs')
