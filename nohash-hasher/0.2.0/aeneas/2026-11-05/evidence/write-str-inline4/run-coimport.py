from pathlib import Path
import os,subprocess,time,signal,json
p=Path(__file__).resolve().parent
family=p.parent/'nohash-length-prefix'
paths=[p/'std-debug',p/'std-debug/generated-reference']+[p/folder/sub for folder in ['nostd-debug','nostd-release','std-release'] for sub in ['', 'generated']]+[family]+[family/folder/sub for folder,sub in [('inline-route',''),('inline-route','generated'),('inline-wide-panic',''),('inline-wide-panic','generated-dce'),('inline-nostd-debug',''),('inline-nostd-debug','generated'),('inline-nostd-release',''),('inline-nostd-release','generated'),('inline-std-release-all',''),('inline-std-release-all','generated')]]
os.environ['LEAN_PATH']=':'.join(map(str,paths))+':'+os.environ.get('LEAN_PATH','')
jobs=[('whole-family',p,p/'WriteStrFamilyAudit.lean',p/'WriteStrFamilyAudit.olean')]
for label,root,source,out in jobs:
 cmd=['lean','-R',str(root),'-o',str(out),str(source)];start=time.monotonic();peak=0;stop=None
 with (p/(label+'-bounded.log')).open('w') as log:
  child=subprocess.Popen(cmd,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
  while child.poll() is None:
   lines=subprocess.check_output(['ps','-eo','pgid=,rss='],text=True).splitlines();rss=sum(int(x.split()[1])*1024 for x in lines if len(x.split())==2 and int(x.split()[0])==child.pid);peak=max(peak,rss)
   if time.monotonic()-start>30:stop='timeout30s'
   if rss>4500*1024*1024:stop='RSS4500MiB'
   if stop:os.killpg(child.pid,signal.SIGKILL);break
   time.sleep(.05)
  code=child.wait()
 result={'argv':cmd,'exit':code,'seconds':time.monotonic()-start,'peak_processgroup_RSS_bytes':peak,'stop':stop};(p/(label+'-bounded.json')).write_text(json.dumps(result,indent=2)+'\n');print(label,result,flush=True)
 if code:raise SystemExit(code)
