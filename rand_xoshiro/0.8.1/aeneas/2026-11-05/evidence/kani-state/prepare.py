from pathlib import Path
import json,tarfile,hashlib
b=Path(__file__).resolve().parent;s=b/'source';a=b/'upstream.crate'
with tarfile.open(a) as t:
 for m in t.getmembers():
  if m.isfile():
   p=s/Path(m.name).relative_to('rand_xoshiro-0.8.1');p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read())
lines=[];changes=[];apis=[]
for e in json.loads((b/'engines.json').read_text()):
 m,ty,w,n=e['module'],e['type'],e['word_width'],e['word_count'];W='u'+str(w);N=w*n//8
 shape='{x:words[0]}' if ty=='SplitMix64' else '{s0:words[0],s1:words[1]}' if m.startswith('xoroshiro') else '{s:words}'
 p=s/'src'/(m+'.rs');raw=p.read_bytes();suffix=f'\n#[cfg(kani)]\npub(crate) fn proof_state_from_words(words:[{W};{n}])->{ty}{{{ty}{shape}}}\n';p.write_bytes(raw+suffix.encode());changes.append({'file':str(p.relative_to(s)),'original_length':len(raw),'original_sha256':hashlib.sha256(raw).hexdigest(),'observer_suffix':suffix})
 name=f'original_{m}_state_bytes';call='rng.state().0' if N==64 else 'rng.state()'
 lines+=['#[kani::proof]',f'fn {name}(){{',f' let words:[{W};{n}]=kani::any();',f' let rng=crate::{m}::proof_state_from_words(words);',f' let bytes={call};']
 for k in range(n):
  for j in range(w//8):lines+=[f' assert_eq!(bytes[{k*(w//8)+j}],((words[{k}]>>{j*8})&255) as u8);']
 lines+=['}'];apis.append(dict(e,api=f'rand_xoshiro::{ty}::state',harness='state_harnesses::'+name,state_bytes=N,domain=f'every {n}x{w}-bit internal state including zero',reference='independent shift/mask extraction of all words in little-endian order',status='pending'))
p=s/'src/lib.rs';raw=p.read_bytes();suffix='\n#[cfg(kani)]\nmod state_harnesses;\n';p.write_bytes(raw+suffix.encode());changes.append({'file':'src/lib.rs','original_length':len(raw),'original_sha256':hashlib.sha256(raw).hexdigest(),'observer_suffix':suffix});(s/'src/state_harnesses.rs').write_text('\n'.join(lines)+'\n');(b/'api-bindings.json').write_text(json.dumps(apis,indent=2));(b/'provenance.json').write_text(json.dumps({'archive_sha256':hashlib.sha256(a.read_bytes()).hexdigest(),'instrumented_files':changes,'guarantee':'Only cfg(kani) private-state constructors appended; original complete Rust byte prefixes and method bodies retained. All target calls are original state methods.','domain':'Every full internal word state, zero included, no assumptions.'},indent=2))

p=b/'provenance.json';data=json.loads(p.read_text());data['harness_sha256']=hashlib.sha256((s/'src/state_harnesses.rs').read_bytes()).hexdigest();p.write_text(json.dumps(data,indent=2))
