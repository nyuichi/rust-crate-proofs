from pathlib import Path
import subprocess,json,tarfile,hashlib,re,os
r=Path(__file__).resolve().parents[1]
archive=r/'evidence/upstream.crate'
def source_guard():
 with tarfile.open(archive) as t:
  src=t.extractfile('nohash-hasher-0.2.0/src/lib.rs').read();manifest=t.extractfile('nohash-hasher-0.2.0/Cargo.toml').read()
 assert (r/'source/src/lib.rs').read_bytes()==src
 assert (r/'source/Cargo.toml').read_bytes()==manifest
 suffix=(r/'tools/observer-suffix.rs').read_bytes()
 assert hashlib.sha256(suffix).hexdigest()=='906995097ae0b73d1f84ed3ba07dcdb9c43b8967e7b23fbfcfec7598b5a1cde0'
 assert (r/'observed-source/src/lib.rs').read_bytes()==src+suffix
 assert (r/'observed-source/Cargo.toml').read_bytes()==manifest
 print('Original unchanged; observed fixture contains exact original byte-prefix and only fixed read-only getters',flush=True)
source_guard();rows=[]
for stage in ['original-stage','observed-stage']:
 for std in [True,False]:
  for debug in [True,False]:
   label=f"{stage.split('-')[0]}-{'std' if std else 'no-std'}-{'debug' if debug else 'release'}"
   features=(['dependency-std'] if std else [])+([] if debug else ['expect-no-debug'])
   flags=['--no-default-features']+(['--features',','.join(features)] if features else [])
   meta=subprocess.check_output(['cargo','metadata','--offline','--format-version','1']+flags,cwd=r/stage)
   (r/f'evidence/{label}-metadata.json').write_bytes(meta)
   x=json.loads(meta);p=next(p for p in x['packages'] if p['name']=='nohash-hasher');n=next(n for n in x['resolve']['nodes'] if n['id']==p['id'])
   assert n['features']==(['std'] if std else []),(label,n['features'])
   env=dict(os.environ,CARGO_PROFILE_DEV_DEBUG_ASSERTIONS='true' if debug else 'false')
   log=r/f'evidence/{label}.log'
   with log.open('w') as f:subprocess.run(['cargo','kani']+flags,cwd=r/stage,env=env,stdout=f,stderr=subprocess.STDOUT,check=True)
   s=log.read_text();assert 'VERIFICATION:- SUCCESSFUL' in s and '1 successfully verified harnesses, 0 failures, 1 total.' in s and 'Status: FAILURE' not in s
   m=re.search(r'\*\* 0 of (\d+) failed(?: \((\d+) unreachable\))?',s);assert m
   rows.append({'stage':stage,'dependency_features':n['features'],'debug_assertions':debug,'checks':int(m[1]),'unreachable':int(m[2] or 0),'failed':0,'log':str(log.relative_to(r)),'seconds':float(re.search(r'Verification Time: ([\d.]+)s',s)[1])})
   print(label,rows[-1]['checks'],'checks passed',flush=True)
source_guard()
(r/'results.json').write_text(json.dumps({'crate':'nohash-hasher','version':'0.2.0','tool':'Kani0.68.0','instantiation':'T=u64 only','reachable_states':'Default or one write_u64(any u64), independently for source/destination','original_stage':'unmodified original path dependency, finish() value observed','observed_stage':'byte-identical original prefix copy plus fixed read-only getter observes flag; original std clone_from body called directly','existing_Lean_model_composition':'separate evidence, not used here','generic_all_T':False,'configurations':rows},indent=2)+'\n')
