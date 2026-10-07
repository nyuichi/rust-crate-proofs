import pathlib,sys,shutil,hashlib,json,tarfile
p=pathlib.Path(__file__).resolve().parent; r=p.parents[2]; dest=p/'evidence'/sys.argv[1];dest.mkdir(parents=True,exist_ok=False)
for rel in ['src/bytes_mut.rs','src/ownership_proof/raw_vec.rs','src/ownership_proof/owned_region.rs','src/storage_ops.rs','src/provenance_specs.rs']:
 q=dest/'source'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(r/rel,q)
for rel in ['generated','verif','src']:
 if (p/rel).exists():shutil.copytree(p/rel,dest/rel)
for rel in ['extract.py','build.rs','Cargo.toml','Cargo.lock','README.md','why3find.json']:
 if (p/rel).exists():shutil.copy2(p/rel,dest/rel)
for rel in ['scripts/verify-bytes.sh','scripts/prepare-proof-std.py','.cargo/config.toml']:
 q=dest/'support'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(r/rel,q)
std=pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std')
for rel in ['Cargo.toml','src/std/vec.rs','src/std/ptr.rs','src/std/ptr/nonnull.rs','src/ghost/resource.rs','src/invariant.rs']:
 if (std/rel).is_file():
  q=dest/'private-std'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(std/rel,q)
for log in sys.argv[2:]:shutil.copy2(log,dest/pathlib.Path(log).name)
manifest={str(f.relative_to(dest)):hashlib.sha256(f.read_bytes()).hexdigest() for f in dest.rglob('*') if f.is_file()};(dest/'hashes.json').write_text(json.dumps(manifest,indent=2))
with tarfile.open(str(dest)+'.tar.gz','w:gz') as t:t.add(dest,arcname=dest.name)
print(dest)
