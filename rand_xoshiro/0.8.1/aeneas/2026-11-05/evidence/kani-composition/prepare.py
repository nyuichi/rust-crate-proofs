from pathlib import Path
import tarfile,json,hashlib,shutil
base=Path(__file__).resolve().parent;src=base/'source';archive=base/'upstream.crate'
with tarfile.open(archive) as t:
 for m in t.getmembers():
  rel=Path(m.name).parts[1:]
  if m.isfile() and rel:
   p=src.joinpath(*rel);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read())
E=json.load(open(base/'api-bindings.json'));prefixes=[]
for e in E:
 n=2 if e['module'].startswith('xoroshiro') else 4;t=e['type'];p=src/'src'/(e['module']+'.rs');original=p.read_bytes()
 make='{s0:words[0],s1:words[1]}' if n==2 else '{s:words}'
 read='[s.s0,s.s1]' if n==2 else 's.s'
 suffix=f'''\n#[cfg(kani)]
pub(crate) fn proof_from_words(words:[u32;{n}])->{t}{{{t}{make}}}
#[cfg(kani)]
pub(crate) fn proof_words(s:&{t})->[u32;{n}]{{{read}}}
'''.encode()
 p.write_bytes(original+suffix);prefixes.append({'file':str(p.relative_to(src)),'original_length':len(original),'original_sha256':hashlib.sha256(original).hexdigest(),'observer_suffix':suffix.decode()})
p=src/'src/lib.rs';original=p.read_bytes();suffix=b'\n#[cfg(kani)]\nmod composition_harnesses;\n';p.write_bytes(original+suffix);prefixes.append({'file':'src/lib.rs','original_length':len(original),'original_sha256':hashlib.sha256(original).hexdigest(),'observer_suffix':suffix.decode()})
lines=['#[allow(unused_imports)] use rand_core::TryRng;']
for e in E:
 m,t=e['module'],e['type'];n=2 if m.startswith('xoroshiro') else 4;names=list('abcd')[:n];params=','.join(names)
 if m=='xoroshiro64star':out='a.wrapping_mul(0x9E3779BB)'
 elif m=='xoroshiro64starstar':out='a.wrapping_mul(0x9E3779BB).rotate_left(5).wrapping_mul(5)'
 elif m=='xoshiro128plus':out='a.wrapping_add(d)'
 elif m=='xoshiro128plusplus':out='a.wrapping_add(d).rotate_left(7).wrapping_add(a)'
 else:out='b.wrapping_mul(5).rotate_left(7).wrapping_mul(9)'
 transition='let q=b^a; [a.rotate_left(26)^q^(q<<9),q.rotate_left(13)]' if n==2 else 'let p=c^a;let q=d^b;[a^q,b^p,p^(b<<9),q.rotate_left(11)]'
 lines.append(f'''fn reference_{m}(words:[u32;{n}])->(u32,[u32;{n}]){{
 let [{params}]=words;
 let output={out};
 let next={{ {transition} }};
 (output,next)
}}
#[kani::proof]
fn original_{m}_next64_matches_two_steps(){{
 let words:[u32;{n}]=kani::any();
 let mut rng=crate::{m}::proof_from_words(words);
 let (low,s1)=reference_{m}(words);
 let (high,s2)=reference_{m}(s1);
 let expected=(u64::from(high)<<32)|u64::from(low);
 let result=match rng.try_next_u64(){{Ok(v)=>v,Err(e)=>match e{{}}}};
 assert_eq!(result,expected);
 let actual=crate::{m}::proof_words(&rng);
'''+''.join(f' assert_eq!(actual[{k}],s2[{k}]);\n' for k in range(n))+'}\n')
(src/'src/composition_harnesses.rs').write_text('\n'.join(lines))
(base/'provenance.json').write_text(json.dumps({'crate':'rand_xoshiro','version':'0.8.1','archive_reference':'upstream.crate','archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'instrumented_files':prefixes,'all_other_files':'byte-identical to original archive','instrumentation_scope':'Observers construct and read private scalar state only; no original method body is changed. Harnesses call the original try_next_u64 implementation.','input_domain':'Every two-u32 or four-u32 internal state, including all zero, with no arbitrary length bound.'},indent=2))
if archive.resolve()!=(base/'upstream.crate').resolve():shutil.copy(archive,base/'upstream.crate')
print('Prepared five original API harnesses with original source byte-prefix retention')
