from pathlib import Path
import hashlib,json,tomllib,re
p=Path(__file__).resolve().parents[1];o=p.parent/'candidate-next-three'
def h(f):return hashlib.sha256(f.read_bytes()).hexdigest()
for f in (o/'proofs').glob('*.lean'):
 if f.name!='lakefile.lean':assert h(f)==h(p/'proofs'/f.name),f
old=tomllib.loads((o/'proofs.toml').read_text())['tool']['proofs'];cur=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs'];assert cur[:49]==old and len(cur)==51
blocks=re.split(r'(?=\n/--)',(o/'proofs/Generated/Ascii.lean').read_text());new=(p/'proofs/Generated/Ascii.lean').read_text();assert all(x in new for x in blocks if x.strip())
m=json.loads((p/'proofs/Generated/translation.json').read_text());assert not any(x['is_opaque'] for x in m['functions'])
sel=[x for x in m['functions'] if x['lean_name'].endswith('AsciiChar.from_ascii') or ('FromAsciiError' in x['lean_name'] and x['lean_name'].endswith('CoreErrorError.description'))];assert len(sel)==2 and all(x['is_local'] for x in sel)
assert sorted((x['source']['file'],x['source']['begin_line'],x['source']['end_line']) for x in sel)==[('src/ascii_char.rs',293,295),('src/ascii_string.rs',788,790)]
(p/'evidence/generic-two-guard.json').write_text(json.dumps({'old49proof_mapping_generated_blocks_preserved':True,'old_blocks':len(blocks),'new2':sel,'opaque':0,'newmodels':0,'configuration':'default std only; no nostd/customtranslator','publication_ready':False},indent=2)+'\n')
