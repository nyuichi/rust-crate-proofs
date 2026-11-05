import pathlib,subprocess,hashlib,json,shutil,os
out=pathlib.Path('/workspace/full-api/generic6-cli-orchestrator-checkpoint/local-accept');out.mkdir(exist_ok=True)
src=pathlib.Path('/workspace/full-api/enum/oorandom-generic6-binding-cli');new='/tmp/oorandom-generic6-check/debug/scanner-probe';old='/workspace/full-api/enum/ascii-next2-source-cli/bin/scanner-probe';test='/tmp/oorandom-generic6-check/debug/build/cargo-proofs/6af6c1fa08f9ec82/out/scanner_probe-6af6c1fa08f9ec82';records=[]
def run(label,argv,env=None):
 r=subprocess.run(argv,cwd=src,env=env,capture_output=True,text=True,timeout=90);(out/(label+'.stdout')).write_text(r.stdout);(out/(label+'.stderr')).write_text(r.stderr);records.append({'label':label,'argv':argv,'cwd':str(src),'exit':r.returncode,'binary_sha256':hashlib.sha256(pathlib.Path(argv[0]).read_bytes()).hexdigest()});(out/'commands.json').write_text(json.dumps(records,indent=2));return r
fixture='/tmp/oorandom-generic6-capture-artifact/Cargo.toml'
r=run('some20',[new,'init','--manifest-path',fixture,'--tool','aeneas']);assert r.returncode==0,r.stderr
rows=[json.loads(l) for l in r.stdout.splitlines()];assert len(rows)==20
public='/workspace/publish-full-api/oorandom/11.1.5/aeneas/2026-11-05/Cargo.toml'
a=run('none14-old',[old,'init','--manifest-path',public,'--tool','aeneas']);b=run('none14-new',[new,'init','--manifest-path',public,'--tool','aeneas']);assert a.returncode==b.returncode==0 and a.stdout==b.stdout
oldrows=[json.loads(l) for l in a.stdout.splitlines()];assert rows[:14]==oldrows
r=run('none-serialization',[test,'aeneas::tests::exact_binding_opt_in_is_narrow_and_fail_closed','--exact','--nocapture']);assert r.returncode==0
r=run('negative16',['/usr/bin/python3','/tmp/run-oor-cheap.py']);assert r.returncode==0
x=json.load(open('/workspace/full-api/generic6-cli-orchestrator-checkpoint/cheap-negatives/RESULTS.json'));assert all(c['early_reject'] for c in x['cases'])
env=os.environ.copy();env['PROOFS_OOR_GENERIC_FIXTURE']=str(pathlib.Path(fixture).parent)
r=run('focused-retained',[test,'oorandom_generic::capture::focused_tests::capture_evidence_and_five_semantic_negatives','--exact','--ignored','--nocapture'],env);assert r.returncode==0,r.stderr
hashes={str(p.relative_to(src)):hashlib.sha256(p.read_bytes()).hexdigest() for p in src.joinpath('src').rglob('*') if p.is_file()}
(out/'source-hashes.json').write_text(json.dumps(hashes,indent=2));(out/'RESULT.json').write_text(json.dumps({'some_rows':20,'old14_paths_byte_equal':True,'prefix14_equal':True,'six_keys':rows[14:],'negative16_rejected':True,'none_serialization_test_passed':True,'build_recipe':'source /workspace/aeneas-tools/activate.sh; CARGO_TARGET_DIR=/tmp/oorandom-generic6-check cargo check --offline --bins; cargo build --offline --bin scanner-probe; cargo test --offline --bin scanner-probe --no-run','scope_sha256':hashes['src/oorandom_generic_scope.json'],'focused_stdout':r.stdout},indent=2))
