from pathlib import Path
import tomllib
p=Path(__file__).resolve().parents[1];rows=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs']
s=''.join(f'import {m}\n' for m in dict.fromkeys(r['module'] for r in rows))
s+=''.join(f"#print axioms {r['theorem']}\n" for r in rows)
(p/'evidence/UnionAudit.lean').write_text(s)
