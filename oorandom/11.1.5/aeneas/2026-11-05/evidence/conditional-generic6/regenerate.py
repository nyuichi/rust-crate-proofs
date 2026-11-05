"""Reproduce original core-body extraction without changing Rust, LLBC or generated Lean."""
from pathlib import Path
import subprocess,tempfile,gzip,hashlib,json,os,shutil
r=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def call(argv,cwd=r):subprocess.run(argv,cwd=cwd,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='1'))
prov=json.load(open(r/'tool/provenance.json'));source=json.load(open(r/'original-source-provenance.json'))
for p,h in source['files'].items():assert sha((r/'source/oorandom'/p).read_bytes())==h
core=json.load(open(r/'original-core-bodies.json'));assert 'commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8' in subprocess.check_output(['rustc','-vV'],text=True)
sysroot=Path(subprocess.check_output(['rustc','--print','sysroot'],text=True).strip());src=sysroot/'lib/rustlib/src/rust/library/core/src/convert/mod.rs';assert sha(src.read_bytes())==core['original_source_sha256']
blob=(r/'tool/aeneas-original-core-export.gz').read_bytes();assert sha(blob)==prov['archive_sha256'];binary=gzip.decompress(blob);assert sha(binary)==prov['binary_sha256']
with tempfile.TemporaryDirectory(prefix='oor-core-regenerate-') as tmp:
 tmp=Path(tmp);tool=tmp/'aeneas';tool.write_bytes(binary);tool.chmod(0o700)
 for filename,roots,custom,namespace,dest in [('CoreConversions.llbc',['core_into','core_try_into','core_try_from'],False,None,'Generated'),('CoreDelegations.llbc',['core_into','core_try_into'],True,'oorandom_core_delegations','GeneratedDelegations')]:
  args=['charon','cargo','--preset=aeneas','--sysroot','default','--dest-file',filename]
  for root in roots:args+=['--start-from','crate::'+root]
  args+=['--include','core::convert'];call(args)
  output=tmp/dest;args=[str(tool) if custom else 'aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(output)]
  if namespace:args+=['-namespace',namespace]
  if custom:args+=['-emit-original-core-conversion-bodies']
  args+=[filename];call(args)
  lean=Path(filename).with_suffix('.lean').name
  assert (output/lean).read_bytes()==(r/'proofs'/dest/lean).read_bytes(),lean
  assert json.load(open(output/'translation.json'))==json.load(open(r/'proofs'/dest/'translation.json')),dest
 call(['python3','check_core_bodies.py'])
print('PASS: original source/body regeneration, both actual generated modules and metadata exact')
