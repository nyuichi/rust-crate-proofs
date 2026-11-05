from pathlib import Path
import json,hashlib,re,tomllib
p=Path(__file__).resolve().parents[1];old=Path('/workspace/full-api/bits/oorandom-range-partial');sha=lambda b:hashlib.sha256(b).hexdigest()
preserved={}
for f in [old/'Cargo.toml',old/'Cargo.lock',old/'src/lib.rs',*old.glob('proofs/*.lean'),*old.glob('proofs/Oorandom/*')]:
 if not f.is_file() or f.name=='lakefile.lean':continue
 rel=f.relative_to(old);assert f.read_bytes()==(p/rel).read_bytes(),str(rel);preserved[str(rel)]=sha(f.read_bytes())
results={}
for f in sorted((p/'evidence').glob('*-whole-bounded.json')):
 r=json.loads(f.read_text());assert r['status']=='success' and r['exit']==0 and r['seconds']<30 and r['peak_rss']<=4500*1024**2,(f,r);results[f.name]=r
assert len(results)==22,len(results)
allowed={'propext','Classical.choice','Quot.sound'};audits={}
for f in (p/'evidence').glob('*-whole-bounded.log'):
 log=f.read_text();assert 'error:' not in log and 'sorryAx' not in log,str(f)
 for name,axioms in re.findall(r"'([^']+)' depends on axioms: \[([^]]*)\]",log):
  found={x.strip() for x in axioms.split(',') if x.strip()};assert found<=allowed,(name,found);audits[name]=sorted(found)
for name in ['ExactRange.rand32_contract','ExactRange.rand64_contract','TraceAudit.first_or_none','PublicTrace32.public_first_accept','PublicTrace64.public_first_accept','PublicTrace32.public_all_rejected','PublicTrace64.public_all_rejected']:assert name in audits,name
for f in p.rglob('*'):
 if not f.is_file():continue
 assert not re.search(r'\.(olean|ilean|ir|pyc)(\.|$)',f.name) and '__pycache__' not in f.parts and f.read_bytes()[:5]!=b'olean',str(f)
 if f.suffix=='.lean':assert not re.search(r'\b(sorry|admit|axiom)\b',f.read_text()),str(f)
rows=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs'];assert [(r['api'],r['module'],r['theorem']) for r in rows]==[('Rand32::rand_range','ExactRange','ExactRange.rand32_contract'),('Rand64::rand_range','ExactRange','ExactRange.rand64_contract')]
assert 'opaque0; generated Funs/Types byte-identical' in (p/'evidence/portable-regeneration.log').read_text()
out={'unchanged_prior_Rust_generated_and_proof_files':preserved,'bounded_modules':results,'audits':audits,'new_models':0,'opaque_functions':0,'public_method_keys':2,'general_termination_claimed':False,'uniformity_claimed':False,'exact_semantics_cases':'invalid assertion / leastaccept exact normal / allreject div','case_exhaustiveness':'kernel theorem TraceAudit.first_or_none','source_only':True};(p/'evidence/trace-guard.json').write_text(json.dumps(out,indent=2)+'\n');print('trace guard PASS',len(preserved),'preserved files,',len(results),'modules,',len(audits),'audited declarations')
