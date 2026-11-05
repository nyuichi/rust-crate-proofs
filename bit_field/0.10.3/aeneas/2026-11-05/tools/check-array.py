from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
fs=json.loads((p/'proofs/BitArrayGenerated/translation.json').read_text())['functions']
assert len(fs)==3
for method in ['bit_length','get_bit','set_bit']:
 rows=[f for f in fs if f['rust_name'].endswith('::'+method) and 'BitArray<T>' in f['rust_name']]
 assert len(rows)==1,(method,rows)
 f=rows[0];assert f['is_local'] and not f['is_opaque'],f
 assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
print('All three original generic slice bodies are non-opaque/source-traced; no callback body is substituted or claimed proved')
