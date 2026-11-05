from pathlib import Path
import json,re
r=Path(__file__).resolve().parents[1];x=json.loads((r/'results.json').read_text());assert x['harnesses']==2 and x['checks']==909
for a in x['APIs']:
 s=(r/a['log']).read_text();assert 'VERIFICATION:- SUCCESSFUL' in s and 'Status: FAILURE' not in s
 assert '1 successfully verified harnesses, 0 failures, 1 total.' in s
 m=re.search(r'\*\* 0 of (\d+) failed \((\d+) unreachable\)',s);assert int(m[1])==a['checks']
code=(r/'src/lib.rs').read_text();assert code.count('kani::assume(len<=4)')==2 and 'kani::assume(target' not in code
print('Both original Iterator search contracts passed independently for full Unicode target/prefix domains')
