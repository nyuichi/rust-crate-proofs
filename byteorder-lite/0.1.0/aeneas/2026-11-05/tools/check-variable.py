from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
fs=json.loads((p/'proofs/VariableGenerated/translation.json').read_text())['functions']
expected={'byteorder_lite::pack_size','byteorder_lite::pack_size128'}
for endian in ['BigEndian','LittleEndian']:
 for suffix in ['','128']:expected.add(f'byteorder_lite::{{impl byteorder_lite::ByteOrder for byteorder_lite::{endian}}}::write_uint{suffix}')
assert {f['rust_name'] for f in fs}==expected
for f in fs:
 assert not f['is_opaque'] and f['is_local'],f
 assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
print('All four original variable-width methods and both actual helper bodies are present and non-opaque')
