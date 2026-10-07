#!/usr/bin/env python3
import pathlib,sys,shutil,hashlib,json,tarfile
probe=pathlib.Path(__file__).resolve().parent
root=probe.parents[2]
run=probe/'evidence'/sys.argv[1]
run.mkdir(parents=True,exist_ok=False)
for rel in ['src','verif']:
    if (probe/rel).exists():shutil.copytree(probe/rel,run/rel)
for rel in ['Cargo.toml','Cargo.lock','README.md','why3find.json','capture.py']:
    shutil.copy2(probe/rel,run/rel)
for rel in ['src/bytes.rs','src/bytes_mut.rs','src/provenance_specs.rs','src/ownership_proof/raw_vec.rs','src/ownership_proof/owned_region.rs','scripts/verify-bytes.sh','.cargo/config.toml','verification/std-support/manifest.json']:
    dest=run/'crate'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/rel,dest)
std=pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std')
for rel in ['Cargo.toml','src/ghost/shared.rs','src/ghost/resource.rs','src/ghost/resource/auth.rs','src/ghost/invariant.rs','src/ghost/perm.rs','src/cell/permcell.rs','src/cell/predcell.rs','src/std/sync/atomic.rs','src/std/sync/atomic_sc.rs','src/std/sync/committer.rs','src/std/sync/view.rs','src/std/rc.rs','src/std/sync.rs','src/std/thread.rs','src/logic/ra/excl.rs','src/logic/ra/positive.rs','src/std/vec.rs']:
    dest=run/'private-std'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(std/rel,dest)
for logfile in sys.argv[2:]:shutil.copy2(logfile,run/pathlib.Path(logfile).name)
hashes={str(p.relative_to(run)):hashlib.sha256(p.read_bytes()).hexdigest() for p in run.rglob('*') if p.is_file()}
(run/'hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
with tarfile.open(str(run)+'.tar.gz','w:gz') as archive:archive.add(run,arcname=run.name)
shutil.rmtree(run)
print(str(run)+'.tar.gz')
