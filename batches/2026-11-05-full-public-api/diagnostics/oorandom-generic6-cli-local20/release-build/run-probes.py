import pathlib,subprocess,json,hashlib,tomllib
out=pathlib.Path('/workspace/full-api/generic6-cli-orchestrator-checkpoint/release-build');src=pathlib.Path('/workspace/full-api/enum/oorandom-generic6-binding-cli');b=pathlib.Path('/tmp/oorandom-generic6-release/release');records=[]
def run(label,argv):
 r=subprocess.run(argv,cwd=src,capture_output=True,text=True,timeout=90);(out/(label+'.stdout')).write_text(r.stdout);(out/(label+'.stderr')).write_text(r.stderr);records.append({'label':label,'argv':list(map(str,argv)),'cwd':str(src),'exit':r.returncode,'binary_sha256':hashlib.sha256(pathlib.Path(argv[0]).read_bytes()).hexdigest()});assert r.returncode==0,r.stderr;return r.stdout
some=run('release-some20',[str(b/'scanner-probe'),'init','--manifest-path','/tmp/oorandom-generic6-capture-artifact/Cargo.toml','--tool','aeneas'])
none=run('release-none14',[str(b/'scanner-probe'),'init','--manifest-path','/workspace/publish-full-api/oorandom/11.1.5/aeneas/2026-11-05/Cargo.toml','--tool','aeneas'])
assert some==pathlib.Path('/workspace/full-api/generic6-cli-orchestrator-checkpoint/local-accept/some20.stdout').read_text();assert none==pathlib.Path('/workspace/full-api/generic6-cli-orchestrator-checkpoint/local-accept/none14-old.stdout').read_text()
scope=json.load(open(src/'src/oorandom_generic_scope.json'));actual=tomllib.loads(pathlib.Path('/workspace/publish-full-api/oorandom/11.1.5/aeneas/2026-11-05/proofs.toml').read_text());assert scope['configured_bindings20'][:14]==actual['tool']['proofs']
for name in ['cargo-proofs','scanner-probe']:run(name+'-version',[str(b/name),'--version'])
(out/'PROBE-RESULT.json').write_text(json.dumps({'pass':True,'some20_byte_equal_local':True,'none14_byte_equal_baseline51':True,'old14_all_fields_including_preconditions_equal':True,'commands':records,'bins':{name:hashlib.sha256((b/name).read_bytes()).hexdigest() for name in ['cargo-proofs','scanner-probe']}},indent=2)+'\n')
