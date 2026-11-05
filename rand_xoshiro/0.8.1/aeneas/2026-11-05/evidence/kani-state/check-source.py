from pathlib import Path
import tarfile,json,hashlib
b=Path(__file__).resolve().parent;src=b/'source';p=json.load(open(b/'provenance.json'));changes={x['file']:x for x in p['instrumented_files']}
with tarfile.open(b/'upstream.crate') as t:
 original_files=set()
 for m in t.getmembers():
  parts=Path(m.name).parts[1:]
  if not m.isfile() or not parts:continue
  rel='/'.join(parts);original_files.add(rel)
  if not (rel=='Cargo.toml' or rel.startswith('src/') and rel.endswith('.rs')):continue
  original=t.extractfile(m).read();actual=src.joinpath(*parts).read_bytes()
  if rel in changes:
   c=changes[rel];assert len(original)==c['original_length'] and hashlib.sha256(original).hexdigest()==c['original_sha256']
   assert actual==original+c['observer_suffix'].encode(),rel
  else:assert actual==original,rel
 assert {str(f.relative_to(src)) for f in (src/'src').rglob('*.rs')}-original_files=={'src/state_harnesses.rs'}
h=(src/'src/state_harnesses.rs').read_text();assert hashlib.sha256(h.encode()).hexdigest()==p['harness_sha256'];assert 'kani::assume' not in h and 'kani::stub' not in h;assert h.count('#[kani::proof]')==15
for a in json.load(open(b/'api-bindings.json')):
 assert 'let words:[u'+str(a['word_width'])+';'+str(a['word_count'])+']=kani::any();' in h
 assert 'crate::'+a['module']+'::proof_state_from_words(words)' in h
 assert 'let bytes=rng.state()' in h
print('All original Rust source prefixes and Cargo manifest retained; fifteen direct original state harness calls, constructor observers, no assumptions/stubs')
