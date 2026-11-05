"""Mandatory, uncached audits in six isolated Lean workspaces.
Only the official dependency directory is shared; generated modules remain isolated.
"""
from pathlib import Path
import hashlib,json,os,re,subprocess,sys
root=Path(__file__).resolve().parent.parent
base=root/'evidence/standard-range-workspaces'
config=json.loads((base/'WORKSPACES.json').read_text())
packages=(root/'proofs/.lake/packages').resolve(strict=True)
seen=set()
for row in config['workspaces']:
 width=row['width']; leaf=base/width; proof=leaf/'proofs'; module=row['module']
 for name,digest in row['source_files'].items():
  p=leaf/name
  if p.is_symlink() or hashlib.sha256(p.read_bytes()).hexdigest()!=digest:
   raise RuntimeError(f'{width}: changed frozen source {name}')
 own=proof/'.lake';own.mkdir(exist_ok=True)
 link=own/'packages'
 if not link.exists():link.symlink_to(packages,target_is_directory=True)
 if not link.is_symlink() or link.resolve()!=packages:raise RuntimeError('unexpected dependency directory')
 # Parent Lake's environment must not leak same-named modules into another package.
 env={k:v for k,v in os.environ.items() if k not in ['LEAN_PATH','LEAN_SRC_PATH','LAKE_PACKAGE_URL_MAP']}
 def run(args):
  p=subprocess.run(args,cwd=proof,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
  print(p.stdout,flush=True)
  if p.returncode:raise RuntimeError(f'{width}: command failed ({p.returncode}): {args}')
  return p.stdout
 print(f'proofs.rs:range-workspace:BEGIN:{width}:{row["expected_count"]}:{row["audit_sha256"]}',flush=True)
 run(['lake','build','+'+module])
 # Direct elaboration executes thmInfo/collectAxioms checks even on cache hits.
 text=run(['lake','env','lean',module+'.lean'])
 names=re.findall(r'proofs.rs:checked:([^\s]+)',text)
 if len(names)!=row['expected_count'] or set(names)!=set(row['expected_names']):
  raise RuntimeError(f'{width}: missing, duplicate or unexpected audit marker')
 for name in names:
  key=(width,name)
  if key in seen:raise RuntimeError(f'duplicate {key}')
  seen.add(key)
 print(f'proofs.rs:range-workspace:END:{width}:{len(names)}',flush=True)
if len(seen)!=178:raise RuntimeError(f'expected178 audit markers, got {len(seen)}')
print('proofs.rs:range-workspaces:SUCCESS:178',flush=True)
