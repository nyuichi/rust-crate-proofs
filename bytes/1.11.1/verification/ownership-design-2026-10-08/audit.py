#!/usr/bin/env python3
"""Read-only evidence audit; never invokes a compiler/prover or grants authority."""
import hashlib,json,pathlib,subprocess,tarfile
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def digest(data): return hashlib.sha256(data).hexdigest()
def nulls(value):
    if value is None:return 1
    if isinstance(value,dict):return sum(nulls(v) for v in value.values())
    if isinstance(value,list):return sum(nulls(v) for v in value)
    return 0
receipts=[]
for rel,expected in [
 ('verification/probes/original-freeze-read-2026-10-08/evidence/positive-byte-load-61.tar.gz',(61,61,0)),
 ('verification/probes/original-freeze-read-2026-10-08/evidence/negative-retarget-pointer.tar.gz',(62,62,1)),
 ('verification/probes/original-unique-write-2026-10-08/evidence/regression-after-freeze-51.tar.gz',(51,51,0)),
]:
    path=ROOT/rel
    with tarfile.open(path) as archive:
        files={m.name:archive.extractfile(m).read() for m in archive.getmembers() if m.isfile()}
        manifests=[name for name in files if name.endswith('/hashes.json')]
        assert len(manifests)==1
        manifest=manifests[0];prefix=manifest.rsplit('/',1)[0]+'/'
        hashes=json.loads(files[manifest])
        for name,sha in hashes.items():assert digest(files[prefix+name])==sha,(rel,name)
        tasks=[name for name in files if name.endswith('.coma') and '/verif/' in name]
        trees=[name for name in files if name.endswith('/proof.json') and '/verif/' in name]
        assert {name[:-5] for name in tasks}=={name.rsplit('/',1)[0] for name in trees},rel
        failed=[name for name in trees if nulls(json.loads(files[name]).get('proofs',{}))]
        total_null=sum(nulls(json.loads(files[name]).get('proofs',{})) for name in trees)
        assert (len(tasks),len(trees),total_null)==expected,(rel,len(tasks),len(trees),total_null)
        source_matches={}
        for source in ['src/bytes.rs','src/bytes_mut.rs','src/ownership_proof/raw_vec.rs']:
            key=prefix+'source/'+source
            if key in files:
                source_matches[source]=files[key]==(ROOT/source).read_bytes()
        assert source_matches and all(source_matches.values()),(rel,source_matches)
        receipts.append({'archive':rel,'sha256':digest(path.read_bytes()),'hashed_members':len(hashes),
          'coma':len(tasks),'proof_json':len(trees),'null_leaves':total_null,'null_files':failed,
          'current_production_source_matches':source_matches})
sources=['src/bytes.rs','src/bytes_mut.rs','src/ownership_proof/raw_vec.rs',
 'src/ownership_proof/scalable_tickets.rs','src/ownership_proof/owned_region.rs',
 'src/storage_ops.rs','verification/ARCHITECTURE_DECISIONS.md',
 'verification/probes/native-integration-frontier/logs/shared-receiver-fraction.log']
sources += ['verification/ownership-design-2026-10-08/'+name for name in ['STRONG_SPEC_JA.md','REVIEW_JA.md','README.md','audit.py']]
source_hashes={p:digest((ROOT/p).read_bytes()) for p in sources}
# Compare preserved tool facts with the active pinned source rather than replaying frozen failures.
tool_root=pathlib.Path('/workspace/bytes-proof-tools/creusot-source')
tool_facts=[]
for archived,active,needle in [
 ('creusot-drop-translation.rs','creusot/src/translation/function/terminator.rs','Drop { target, .. } => Terminator::Goto(*target)'),
 ('creusot-invariant-contracts.rs','creusot-std/src/ghost/invariant.rs','Tokens'),
 ('creusot-tokens-new-validator.rs','creusot/src/validate/tokens_new.rs','can only be called in `main`'),
]:
    old=ROOT/'verification/architecture-evidence'/archived
    new=tool_root/active
    assert needle in old.read_text() and needle in new.read_text(),active
    tool_facts.append({'archived':str(old.relative_to(ROOT)),'active_tool_file':active,
     'archived_sha256':digest(old.read_bytes()),'active_sha256':digest(new.read_bytes()),
     'byte_identical':old.read_bytes()==new.read_bytes(),'required_fact_present':True})
receipt={'baseline_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'scope':'Evidence/source audit and contract design only. No new body proof, no full shared invariant admission.',
 'archives':receipts,'source_sha256':source_hashes,'tool_facts':tool_facts}
(HERE/'audit.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('Audited positive61, negative62/one failure, regression51; matching production sources and pinned tool facts.')
