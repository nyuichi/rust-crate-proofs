"""Read-only archive audit; no compiler or prover invocation."""
from pathlib import Path
import hashlib,json,tarfile
p=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def null_paths(v,path=()):
 if v is None:return [list(path)]
 if isinstance(v,dict):return sum((null_paths(x,path+(k,)) for k,x in v.items()),[])
 if isinstance(v,list):return sum((null_paths(x,path+(i,)) for i,x in enumerate(v)),[])
 return []
expected={'positive-restored-8':(8,8,0),'negative-bind-ward-empty':(11,11,3),'negative-double-commit':(8,8,1),'negative-duplicate-ticket':(0,0,0),'negative-ghost-reentry':(0,0,0)}
reports=[]
for archive in sorted((p/'evidence').glob('*.tar.gz')):
 with tarfile.open(archive) as t:
  files={m.name:t.extractfile(m).read() for m in t.getmembers() if m.isfile()}
  mname=next(n for n in files if n.endswith('/hashes.json'));prefix=mname.removesuffix('hashes.json')
  hashes=json.loads(files[mname]); assert all(sha(files[prefix+n])==h for n,h in hashes.items())
  comas=[n for n in files if n.endswith('.coma')]
  proofs={n:json.loads(b) for n,b in files.items() if n.endswith('/proof.json')}
  nulls={n:null_paths(j.get('proofs',{})) for n,j in proofs.items() if null_paths(j.get('proofs',{}))}
  count=sum(map(len,nulls.values()));name=archive.name.removesuffix('.tar.gz')
  report={'name':name,'sha256':sha(archive.read_bytes()),'hashed_members':len(hashes),'coma':len(comas),'proof_json':len(proofs),'null_leaves':count,'null_paths':nulls}
  if name in expected:assert (len(comas),len(proofs),count)==expected[name]
  if name=='positive-restored-8':
   logs='\n'.join(b.decode(errors='replace') for n,b in files.items() if n.endswith('.log'))
   assert 'Proved (8 files)' in logs and '2 passed; 0 failed' in logs
   report['current_source_matches']={n:(p/n).read_bytes()==files[prefix+n] for n in ['src/lib.rs','src/event.rs','Cargo.toml','Cargo.lock','run-proof.sh']}
   assert all(report['current_source_matches'].values())
  reports.append(report)
result={'scope':'NEW generic invariant TCB + body-checked Relaxed registration. No physical access, retirement, Acquire, Bytes/Clone/vtable/Drop integration.','archives':reports,
 'restored_positive_exit':0,'vc_negative_exit':1,'typing_and_purity_negative_exit':1,
 'missing_acquire_control':'Not applicable to Relaxed-only registration; no synchronization/recovery theorem was added.',
 'control_classifications':{'wrong_bind':'new bind native/model ward precondition','wrong_ward':'stock shoot_store ward precondition','empty_commit':'new operation callback completion: shot_store(final c)','double_commit':'second stock shoot_store requires !shot_store(current c)','duplicate_ticket':'Rust E0382; no Coma','reentry':'non-ghost Registry::clone_registration inside ghost callback; no Coma'}}
(p/'evidence/audit.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
