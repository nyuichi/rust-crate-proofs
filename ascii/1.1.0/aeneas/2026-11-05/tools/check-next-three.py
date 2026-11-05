from pathlib import Path
import json,hashlib,tomllib,re
p=Path(__file__).resolve().parents[1];o=p.parent/'candidate-borrow-views'
def h(f):return hashlib.sha256(f.read_bytes()).hexdigest()
for f in (o/'proofs').glob('*.lean'):
 if f.name!='lakefile.lean':assert h(f)==h(p/'proofs'/f.name),f
old=tomllib.loads((o/'proofs.toml').read_text())['tool']['proofs'];cur=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs'];assert cur[:46]==old and len(cur)==49
blocks=re.split(r'(?=\n/--)',(o/'proofs/Generated/Ascii.lean').read_text());assert all(x in (p/'proofs/Generated/Ascii.lean').read_text() for x in blocks if x.strip())
m=json.loads((p/'proofs/Generated/translation.json').read_text());assert not any(x['is_opaque'] for x in m['functions'])
selected=[x for x in m['functions'] if x['lean_name'].endswith('CoreConvertAsMutAsciiStr.as_mut') or x['lean_name'].endswith('CoreErrorError.description')];assert len(selected)==3 and all(x['is_local'] for x in selected)
assert sorted((x['source']['file'],x['source']['begin_line'],x['source']['end_line']) for x in selected)==[('src/ascii_char.rs',824,826),('src/ascii_str.rs',424,426),('src/ascii_str.rs',837,839)]
(p/'evidence/next-three-guard.json').write_text(json.dumps({'old46_proof_mapping_blocks_preserved':True,'old_blocks':len(blocks),'new3':selected,'opaque':0,'newmodels':0,'publication_ready':False},indent=2)+'\n')
