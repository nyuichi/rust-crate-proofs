from pathlib import Path
import sys,re,tomllib
p=Path(__file__).resolve().parents[1]
proofs=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs']
s=(p/sys.argv[1]).read_text()
assert len(proofs)==22
for proof in proofs:
 name=proof['theorem'];rows=re.findall(r"'"+re.escape(name)+r"' depends on axioms:\s*\[(.*?)\]",s,re.S)
 assert rows,name
 for row in rows:assert {a.strip() for a in row.split(',')}<={'propext','Classical.choice','Quot.sound'},(name,row)
print('All 22 mapped original-body contracts passed transitive axiom audits; TOML mappings parsed')
