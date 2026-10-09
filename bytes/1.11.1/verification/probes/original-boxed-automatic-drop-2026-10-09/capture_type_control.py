import pathlib,sys,tarfile,hashlib,json
r=pathlib.Path(__file__).resolve().parent
label,log,error,control=sys.argv[1:];folder=r/'evidence';folder.mkdir(exist_ok=True)
a=folder/(label+'.tar.gz');m=folder/(label+'.json');assert not a.exists() and not m.exists()
text=pathlib.Path(log).read_text();assert error in text;assert not list((r/'verif').rglob('*.coma'))
files={}
for p in r.rglob('*'):
 if p.is_file() and not set(p.relative_to(r).parts)&{'evidence','verif','__pycache__','.proof-config'} and p.relative_to(r).as_posix()!='generated/proof-targets.json':files['probe/'+p.relative_to(r).as_posix()]=p
files['run.log']=pathlib.Path(log)
rows=[]
with tarfile.open(a,'w:gz') as tar:
 for name,p in sorted(files.items()):
  tar.add(p,arcname=name,recursive=False);rows.append(dict(path=name,sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
reference=json.loads((folder/'boxed-positive-diagnostic-v4.json').read_text())
out=dict(reference_input_archive=reference['archive'],reference_input_archive_sha256=reference['archive_sha256'],archive=a.name,archive_sha256=hashlib.sha256(a.read_bytes()).hexdigest(),stage='Rust type/loan rejection before VC generation',rust_error=error,coma_files=0,prover_invoked=False,members=rows,
 source_control=control,terminal_feature=json.loads((r/'generated/mapping.json').read_text())['terminal_feature'],
 correspondence='Semantic/type development control; structured correspondence is separately tested by final checker suite',
 scope='Type rejection only; no destructor/full-crate theorem')
m.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:out[k] for k in ['archive','archive_sha256','stage','rust_error','coma_files']}))
