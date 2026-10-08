import pathlib,sys,shutil,hashlib,json,tarfile
p=pathlib.Path(__file__).resolve().parent; out=p/'evidence'/sys.argv[1];out.mkdir(parents=True,exist_ok=False)
for n in ['src','verif','generic-boundary-source']:
 if (p/n).exists():shutil.copytree(p/n,out/n)
for n in ['PLAN.md','README.md','capture.py','Cargo.toml','Cargo.lock','run-proof.sh','why3find.json']:
 if (p/n).exists():shutil.copy2(p/n,out/n)
for n in ['src/ownership_proof/raw_vec.rs','src/ownership_proof/bound_ptr.rs','src/ownership_proof/owned_region.rs','src/provenance_specs.rs','src/allocation_ops.rs']:
 q=out/'physical-source'/n;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p.parents[2]/n,q)
std=pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std')
for n in ['src/std/sync/atomic.rs','src/std/sync/committer.rs','src/std/sync/view.rs','src/ghost/invariant.rs','src/ghost/fn_ghost.rs','src/ghost/resource.rs','src/ghost/resource/auth.rs']:
 q=out/'std'/n;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(std/n,q)
for n in ['src/ghost/shared.rs','src/cell/predcell.rs','src/ghost/perm.rs']:
 q=out/'std'/n;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(std/n,q)
for n in sys.argv[2:]:shutil.copy2(n,out/pathlib.Path(n).name)
h={str(f.relative_to(out)):hashlib.sha256(f.read_bytes()).hexdigest() for f in out.rglob('*') if f.is_file()};(out/'hashes.json').write_text(json.dumps(h,indent=2))
with tarfile.open(str(out)+'.tar.gz','w:gz') as t:t.add(out,arcname=out.name)
print(out)
