from pathlib import Path
import subprocess,time,json,os
p=Path('/tmp/either-union44-candidate');records=[]
for target in ['Generated.Either','CallbackProofs','IntoEitherProofs','ResultProofs','DefaultProofs','BothProofs','MapInspectProofs','IteratorProofs','NextBackProofs','FactorProofs','IntoIterProofs','IterSharedProofs','Proofs','PrimaryCoimport']:
 with (p/'evidence'/('union44-build-'+target+'.log')).open('w') as log:
  proc=subprocess.Popen(['lake','build','+'+target],cwd=p/'proofs',stdout=log,stderr=subprocess.STDOUT);start=time.monotonic();peak=0
  while proc.poll() is None:
   table=[tuple(map(int,r.split())) for r in subprocess.check_output(['ps','-eo','pid=,ppid=,rss='],text=True).splitlines()];ids={proc.pid}
   for _ in range(8):ids|={pid for pid,ppid,rss in table if ppid in ids}
   rss=sum(rss for pid,ppid,rss in table if pid in ids);peak=max(peak,rss)
   if rss>4500*1024 or time.monotonic()-start>30:
    for pid in ids:
     try:os.kill(pid,9)
     except ProcessLookupError:pass
    raise RuntimeError('RSS/time cap '+target)
   time.sleep(.1)
  code=proc.returncode
 records.append({'target':target,'exit':code,'seconds':time.monotonic()-start,'peak_tree_RSS_KiB':peak,'limit_seconds':30,'RSS_limit_MiB':4500});(p/'evidence/union44-build-results.json').write_text(json.dumps(records,indent=2));assert code==0,target
