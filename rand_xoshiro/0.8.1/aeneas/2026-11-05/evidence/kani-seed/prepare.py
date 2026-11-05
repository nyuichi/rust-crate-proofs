from pathlib import Path
import tarfile,json,hashlib
b=Path(__file__).resolve().parent;s=b/'source';a=b/'upstream.crate'
with tarfile.open(a) as t:
 for m in t.getmembers():
  if m.isfile():
   p=s/Path(m.name).relative_to('rand_xoshiro-0.8.1');p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read())
fallback=[0xaf,0xcd,0x1d,0x7b,0x39,0xa8,0x20,0xe2,0xf4,0x65,0xb9,0xa1,0x6a,0x9e,0x78,0x6e,0x4f,0x45,0x09,0x80,0x18,0x5d,0xc4,0x06,0xec,0x81,0x4c,0x72,0xa8,0xb8,0x8b,0xf8,0x9b,0x74,0xa8,0x51,0x6a,0x89,0x39,0x1b,0xea,0xa2,0x7e,0x74,0x0c,0x9f,0xcb,0x53,0xe1,0x32,0x45,0x1f,0xbe,0x9a,0x82,0x2c,0x3c,0xab,0x16,0xc9,0x3a,0x13,0x84,0xc5]
lines=['use rand_core::SeedableRng;'];entries=json.loads((b/'engines.json').read_text());changes=[];apis=[]
for e in entries:
 m,ty,word,n=e['module'],e['type'],e['word_width'],e['word_count'];N=word*n//8;W='u'+str(word)
 read='[s.x]' if ty=='SplitMix64' else '[s.s0,s.s1]' if m.startswith('xoroshiro') else 's.s'
 p=s/'src'/(m+'.rs');raw=p.read_bytes();suffix=f'\n#[cfg(kani)]\npub(crate) fn proof_seed_words(s:&{ty})->[{W};{n}]{{{read}}}\n';p.write_bytes(raw+suffix.encode());changes.append({'file':str(p.relative_to(s)),'original_length':len(raw),'original_sha256':hashlib.sha256(raw).hexdigest(),'observer_suffix':suffix})
 seed='crate::Seed512(bytes)' if N==64 else 'bytes';name=f'original_{m}_from_seed_words'
 lines+=['#[kani::proof]',f'fn {name}(){{',f' let bytes:[u8;{N}]=kani::any();',f' let rng=<crate::{ty} as SeedableRng>::from_seed({seed});',f' let words=crate::{m}::proof_seed_words(&rng);']
 if ty=='SplitMix64':lines+=[' let effective=bytes;']
 else:lines+=[f' let effective=if bytes==[0;{N}]{{{str(fallback[:N])}}}else{{bytes}};']
 for k in range(n):
  expr=' | '.join(f'(({W}::from(effective[{k*(word//8)+j}])) << {8*j})' for j in range(word//8));lines+=[f' assert_eq!(words[{k}],{expr});']
 lines+=['}'];apis.append(dict(e,api=f'<rand_xoshiro::{ty} as rand_core::SeedableRng>::from_seed',harness='seed_harnesses::'+name,seed_bytes=N,domain=f'all {8*N}-bit fixed-size seeds',reference='independent LE byte concatenation, all-zero fallback for every non-SplitMix generator',status='pending'))
p=s/'src/lib.rs';raw=p.read_bytes();suffix='\n#[cfg(kani)]\nmod seed_harnesses;\n';p.write_bytes(raw+suffix.encode());changes.append({'file':'src/lib.rs','original_length':len(raw),'original_sha256':hashlib.sha256(raw).hexdigest(),'observer_suffix':suffix})
(s/'src/seed_harnesses.rs').write_text('\n'.join(lines)+'\n');(b/'api-bindings.json').write_text(json.dumps(apis,indent=2));(b/'provenance.json').write_text(json.dumps({'archive_sha256':hashlib.sha256(a.read_bytes()).hexdigest(),'instrumented_files':changes,'guarantee':'Only cfg(kani) read-only private word observers appended; original method bodies and entire original file byte prefixes preserved.','domain':'Every fixed-size seed without assumptions or exclusions, zero fallback included.'},indent=2))

p=b/'provenance.json';data=json.loads(p.read_text());data['harness_sha256']=hashlib.sha256((s/'src/seed_harnesses.rs').read_bytes()).hexdigest();p.write_text(json.dumps(data,indent=2))
