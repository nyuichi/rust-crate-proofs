from pathlib import Path
import json,tarfile,hashlib,re
base=Path(__file__).resolve().parent;src=base/'source';p=json.load(open(base/'provenance.json'));E=json.load(open(base/'api-bindings.json'))
patches={x['file']:x for x in p['instrumented_files']}
with tarfile.open(base/'upstream.crate') as t:
 original_files=set()
 for m in t.getmembers():
  parts=Path(m.name).parts[1:]
  if not m.isfile() or not parts:continue
  rel='/'.join(parts);original_files.add(rel)
  if not (rel=='Cargo.toml' or rel.startswith('src/') and rel.endswith('.rs')):continue
  original=t.extractfile(m).read();actual=src.joinpath(*parts).read_bytes()
  if rel in patches:
   change=patches[rel];assert len(original)==change['original_length'];assert hashlib.sha256(original).hexdigest()==change['original_sha256']
   assert actual[:len(original)]==original and actual[len(original):]==change['observer_suffix'].encode(),rel
  else:assert actual==original,rel
 extra={str(f.relative_to(src)) for f in (src/'src').rglob('*.rs')}-original_files
 assert extra=={'src/composition_harnesses.rs'},extra
harness=(src/'src/composition_harnesses.rs').read_text();assert 'kani::assume' not in harness and '.try_next_u32(' not in harness
assert harness.count('#[kani::proof]')==5
for e in E:
 assert e['api']=='<rand_xoshiro::'+e['type']+' as rand_core::TryRng>::try_next_u64'
 assert e['word_count'] in [2,4] and e['word_width']==32
 assert 'let words:[u32;'+str(e['word_count'])+']=kani::any();' in harness
 assert 'fn original_'+e['module']+'_next64_matches_two_steps()' in harness
 assert 'crate::'+e['module']+'::proof_from_words(words)' in harness
 assert 'crate::'+e['module']+'::proof_words(&rng)' in harness
print('All original Rust/Cargo bytes retained; only exact observer suffixes and five full-state harnesses added')
