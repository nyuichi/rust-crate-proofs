from pathlib import Path
import re,tomllib,sys
p=Path(__file__).resolve().parents[1];s=(p/sys.argv[1]).read_text();proofs=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs']
assert len(proofs)==27
for name in [v['theorem'] for v in proofs]+[f'Proofs.{t}_length_spec' for t in ['u8','u16','u32','u64','u128','usize','i8','i16','i32','i64','i128','isize']]:
 rows=re.findall(r"'"+re.escape(name)+r"' depends on axioms:\s*\[(.*?)\]",s,re.S);assert rows,name
 for row in rows:assert {v.strip() for v in row.split(',')}<={'propext','Classical.choice','Quot.sound'},(name,row)
print('All 27 mapped contracts and twelve width invariants passed transitive axiom audits')
