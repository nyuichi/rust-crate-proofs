from pathlib import Path
import json,hashlib,tomllib,re
r=Path(__file__).resolve().parents[1];o=r.parent/'candidate-typed-vectors'
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for p in (o/'proofs').glob('*.lean'):
 if p.name!='lakefile.lean':assert h(p)==h(r/'proofs'/p.name),p
old=tomllib.loads((o/'proofs.toml').read_text())['tool']['proofs'];cur=tomllib.loads((r/'proofs.toml').read_text())['tool']['proofs'];assert cur[:41]==old and len(cur)==51
# Every complete old generated declaration block must remain byte-identical.
a=(o/'proofs/Generated/Ascii.lean').read_text();b=(r/'proofs/Generated/Ascii.lean').read_text()
blocks=re.split(r'(?=\n/--)',a)
assert all(x in b for x in blocks if x.strip()),'old generated block changed'
m=json.loads((r/'proofs/Generated/translation.json').read_text());assert not any(x['is_opaque'] for x in m['functions'])
selected=[x for x in m['functions'] if 'CoreConvertAsRef' in x['lean_name'] or 'CoreConvertAsMutSlice' in x['lean_name']]
expected=[('src/ascii_str.rs',376,378),('src/ascii_str.rs',382,384),('src/ascii_str.rs',418,420),('src/ascii_string.rs',602,604),('src/ascii_string.rs',630,632)]
assert len(selected)==5 and [(x['source']['file'],x['source']['begin_line'],x['source']['end_line']) for x in selected]==expected
assert all(x['is_local'] and not x['is_opaque'] for x in selected)
(r/'evidence/borrow-union-guard.json').write_text(json.dumps({'old41_proof_and_mapping_bytes_preserved':True,'old_generated_blocks_preserved':len(blocks),'new5':cur[41:46],'new5_exact_generated_source':selected,'opaque':0,'publication_ready':False},indent=2)+'\n')
