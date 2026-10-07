#!/usr/bin/env python3
"""Audit the diagnostic evidence, not a new API or synchronization theorem."""
import pathlib,tarfile,json,hashlib,subprocess
probe=pathlib.Path(__file__).resolve().parent
root=probe.parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
def nulls(v):
    if v is None:return 1
    if isinstance(v,dict):return sum(nulls(x) for x in v.values())
    if isinstance(v,list):return sum(nulls(x) for x in v)
    return 0
receipts=[]
for name,counts,status in [('positive-34',(34,34,0),0),('recovery-rejected',(0,0,0),1)]:
    path=probe/'evidence'/(name+'.tar.gz')
    with tarfile.open(path) as tar:
        files={m.name:tar.extractfile(m).read() for m in tar.getmembers() if m.isfile()}
    prefix=name+'/'
    hashes=json.loads(files[prefix+'hashes.json'])
    for rel,digest in hashes.items():assert sha(files[prefix+rel])==digest,(name,rel)
    trees=[n for n in files if n.endswith('/proof.json') and '/verif/' in n]
    tasks=[n for n in files if n.endswith('.coma') and '/verif/' in n]
    assert {n[:-5] for n in tasks}=={n.rsplit('/',1)[0] for n in trees}
    actual=(len(tasks),len(trees),sum(nulls(json.loads(files[n]).get('proofs',{})) for n in trees))
    assert actual==counts,(name,actual)
    source_matches={}
    for rel in ['src/bytes.rs','src/bytes_mut.rs','src/provenance_specs.rs','src/ownership_proof/raw_vec.rs','src/ownership_proof/owned_region.rs']:
        source_matches[rel]=files[prefix+'crate/'+rel]==(root/rel).read_bytes()
    assert all(source_matches.values())
    assert files[prefix+'src/lib.rs']==(probe/'src/lib.rs').read_bytes()
    if name=='positive-34':
        log=files[prefix+'bytes-tokenless-positive.log'].decode()
        assert 'Proved (34 files)' in log
        body=files[prefix+'verif/bytes_tokenless_sharing_diagnostic_rlib/actual_two_reads/proof.json']
        assert 'vc_actual_two_reads' in body.decode()
    else:
        log=files[prefix+'bytes-tokenless-recover.log'].decode()
        assert 'error[E0507]' in log and 'cannot move out of a shared reference' in log
    receipts.append({'archive':str(path.relative_to(root)),'sha256':sha(path.read_bytes()),
     'hashed_members':len(hashes),'coma':actual[0],'proof_json':actual[1],'null_leaves':actual[2],
     'exit_code':status,'prover_executed':name=='positive-34','current_source_matches':source_matches})
# The diagnostic must not mutate either actual Bytes implementation.
base='8bed373b'
for rel in ['src/bytes.rs','src/bytes_mut.rs']:
    original=subprocess.check_output(['git','show',base+':bytes/1.11.1/'+rel],cwd=root)
    assert original==(root/rel).read_bytes(),rel
std=pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std')
std_rel=['src/ghost/shared.rs','src/ghost/resource.rs','src/ghost/resource/auth.rs','src/ghost/invariant.rs',
 'src/ghost/perm.rs','src/cell/permcell.rs','src/cell/predcell.rs','src/std/sync/atomic.rs','src/std/sync/atomic_sc.rs',
 'src/std/sync/committer.rs','src/std/sync/view.rs','src/std/rc.rs','src/std/sync.rs','src/std/thread.rs']
receipt={'baseline_commit':base,'scope':'Diagnostic: immutable observation with retained storage; ownership extraction rejected. Original Clone not proved; no new project trusted law.',
 'archives':receipts,'active_std_sha256':{p:sha((std/p).read_bytes()) for p in std_rel},
 'production_bytes_unchanged':True}
(probe/'audit.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('Audited34 complete positive proofs; recovery E0507/0Coma; production Bytes unchanged.')
