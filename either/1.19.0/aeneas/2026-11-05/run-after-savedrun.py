from pathlib import Path
import subprocess,json,hashlib,time,os,shutil
p=Path(__file__).resolve().parent
assert os.environ.get('EITHER45_WINDOW')=='approved','parent/publisher Savedrun window required'
plan=json.loads((p/'evidence/union45-SOURCE-PREPARED.json').read_text());baseline=Path(plan['baseline']);sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
for n,h in plan['pins'].items():assert sha(baseline/n)==h,n
for name in ['charon','aeneas']:
 exe=Path(shutil.which(name)).resolve();assert sha(exe)==json.loads((p/'BUILD-PLAN.json').read_text())['official_tools'][name+'_sha256']
env=dict(os.environ,CARGO_NET_OFFLINE='true',RUSTUP_TOOLCHAIN='nightly-2026-09-17')
for name in ['RUSTFLAGS','RUSTDOCFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_ENCODED_RUSTDOCFLAGS','RUSTC','RUSTDOC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:env.pop(name,None)
records=[]
def bounded(label,argv,cwd):
 with (p/'evidence'/('union45-'+label+'.log')).open('w') as log:
  proc=subprocess.Popen(argv,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT);start=time.monotonic();peak=0
  while proc.poll()is None:
   table=[tuple(map(int,r.split()))for r in subprocess.check_output(['ps','-eo','pid=,ppid=,rss='],text=True).splitlines()];ids={proc.pid}
   for _ in range(8):ids|={pid for pid,ppid,rss in table if ppid in ids}
   rss=sum(rss for pid,ppid,rss in table if pid in ids);peak=max(peak,rss)
   if rss>4500*1024 or time.monotonic()-start>30:
    for pid in ids:
     try:os.kill(pid,9)
     except ProcessLookupError:pass
    raise RuntimeError('RSS/time cap '+label)
   time.sleep(.1)
  code=proc.returncode
 records.append({'label':label,'argv':argv,'cwd':str(cwd),'exit':code,'seconds':time.monotonic()-start,'peak_RSS_KiB':peak,'RSS_limit_MiB':4500,'seconds_limit':30});(p/'evidence/union45-executed.json').write_text(json.dumps(records,indent=2));assert code==0,label
bounded('extract',['python3','tools/extract.py'],p)
for target in ['Generated.Either','CallbackProofs','IntoEitherProofs','ResultProofs','DefaultProofs','BothProofs','MapInspectProofs','IteratorProofs','NextBackProofs','FactorProofs','IntoIterProofs','IterSharedProofs','FactorIterSharedProofs','Proofs','PrimaryCoimport']:bounded('build-'+target,['lake','build','+'+target],p/'proofs')
bounded('machine45',['python3','tools/audit-all.py','.','evidence/machine45-audit'],p)
bounded('all148',['lake','env','lean','../evidence/all148-audit.lean'],p/'proofs')
print('kernel/sourcegeneration phase complete; source/scanner/regeneration/test/freeze still required')
