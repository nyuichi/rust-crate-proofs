from pathlib import Path
import tomllib
p=Path(__file__).resolve().parents[1]
rows=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs']
text=''.join(f'import {m}\n' for m in dict.fromkeys(x['module'] for x in rows))
text+=''.join(f"#print axioms {x['theorem']}\n" for x in rows)
(p/'evidence/UnionAudit.lean').write_text(text)
