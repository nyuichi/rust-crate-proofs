from pathlib import Path
import hashlib,json,tarfile,tomllib,subprocess,os
p=Path(__file__).resolve().parent
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
status=json.loads((p/'status.json').read_text())
assert sha(p/'upstream.crate')==status['original_archive_sha256']
for name,h in status['original_archive_files'].items():assert sha(p/'original'/name)==h,name
assert sha(p/'Cargo.lock')==status['cargo_lock_sha256'] and sha(p/'src/lib.rs')==status['harness_sha256']
core=status['rand_core'];assert sha(p/'rand_core-0.10.0.crate')==core['checksum']
meta=json.loads(subprocess.check_output(['cargo','metadata','--locked','--offline','--format-version','1'],cwd=p,env=dict(os.environ,RUSTUP_TOOLCHAIN='nightly-2026-08-21'),text=True))
deps=[x for x in meta['packages']if x['name']=='rand_core'];assert len(deps)==1 and deps[0]['version']=='0.10.0';root=Path(deps[0]['manifest_path']).parent;n=0
with tarfile.open(p/'rand_core-0.10.0.crate')as t:
 for m in t.getmembers():
  if m.isfile():
   rel=Path(m.name).relative_to('rand_core-0.10.0');assert(root/rel).read_bytes()==t.extractfile(m).read(),rel;n+=1
(p/'source-provenance.json').write_text(json.dumps({'original_target_archive_files_unchanged':len(status['original_archive_files']),'actual_dependency_archive_files_unchanged':n,'core_archive_sha256':sha(p/'rand_core-0.10.0.crate'),'original_target_archive_sha256':sha(p/'upstream.crate'),'observer_or_model':False},indent=2))
print('Original source, actual resolved dependency, lock and harness hashes verified')
