from pathlib import Path
import os,subprocess,tempfile,json,hashlib,tomllib
p=Path(__file__).resolve().parent;r=p.parent;sha=lambda b:hashlib.sha256(b).hexdigest();env=os.environ.copy();env['RUSTUP_TOOLCHAIN']='nightly-2026-09-17';edition=tomllib.loads((r/'original/Cargo.toml').read_text())['package'].get('edition','2015');source_sha=sha((r/'original/src/lib.rs').read_bytes());compiler=subprocess.check_output(['rustc','-vV'],env=env,text=True);records=[]
with tempfile.TemporaryDirectory(prefix='bitfield-empty-native-') as temporary:
 tmp=Path(temporary)
 for name,flags in [('debug',['-Copt-level=0','-Coverflow-checks=on','-Cdebug-assertions=on']),('release',['-Copt-level=3','-Coverflow-checks=off','-Cdebug-assertions=off'])]:
  lib=tmp/('libbit_field_'+name+'.rlib');exe=tmp/('observer_'+name);commands=[['rustc','--crate-name','bit_field','--crate-type=rlib','--edition='+edition,*flags,str(r/'original/src/lib.rs'),'-o',str(lib)],['rustc','--edition=2021',*flags,str(p/'main.rs'),'--extern','bit_field='+str(lib),'-o',str(exe)]]
  for i,cmd in enumerate(commands):
   run=subprocess.run(cmd,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30);(p/(name+'-compile'+str(i)+'.log')).write_text(run.stdout+run.stderr);assert run.returncode==0,(name,i,run.stderr)
  run=subprocess.run([str(exe)],env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5);(p/(name+'-stdout.log')).write_text(run.stdout);(p/(name+'-stderr.log')).write_text(run.stderr);assert run.returncode==0
  records.append({'profile':name,'flags':flags,'edition_original':edition,'commands':commands,'executable_sha256':sha(exe.read_bytes()),'library_sha256':sha(lib.read_bytes()),'exit':run.returncode,'stdout':run.stdout,'stderr':run.stderr,'runtime_timeout_seconds':5})
assert sha((r/'original/src/lib.rs').read_bytes())==source_sha
assert records[0]['stdout'].strip()=='outcome=caught_panic state=0' and records[1]['stdout'].strip()=='outcome=normal state=0'
x={'scope':'one exact native original-call observation per profile, not all-input release theorem','input':'old0u8,range0usize..0usize,replacement1u8','compiler':compiler,'source_sha256_before_after':source_sha,'original_source_unchanged':True,'observations':records,'compilation_and_runtime_bins':'temporary owned dir cleaned after hashes; logs/source retained','relation_to_Lean':'debug observed shift-overflow panic agrees with modeled integerOverflow; no panic payload/unwind equivalence theorem or general release claim'};(p/'results.json').write_text(json.dumps(x,indent=2)+'\n');print('\n'.join(q['profile']+': '+q['stdout'].strip() for q in records))
