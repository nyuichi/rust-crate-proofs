from pathlib import Path
import os,subprocess,time,signal,json
p=Path(__file__).resolve().parent
items=[('inline-route','generated','NohashPrefixInline','PrefixProofs'),('inline-wide-panic','generated-dce','NohashWidePanicInline','WidePanicProofs'),('inline-nostd-debug','generated','NohashNostdDebug','PrefixNoStdDebug'),('inline-nostd-release','generated','NohashNostdRelease','PrefixNoStdRelease'),('inline-std-release-all','generated','NohashStdReleaseAll','PrefixStdRelease')]
paths=[str(p/x) for x,_,_,_ in items]+[str(p/x/sub) for x,sub,_,_ in items]
os.environ['LEAN_PATH']=':'.join(paths)+':'+os.environ.get('LEAN_PATH','')
jobs=[]
for folder,sub,generated,proof in items:
 q=p/folder;jobs.extend([(folder+'-generated',q/sub,q/sub/f'{generated}.lean',q/sub/f'{generated}.olean'),(folder+'-proof',q,q/f'{proof}.lean',q/f'{proof}.olean')])
jobs.extend([('family-coimport',p,p/'FinalFamilyAudit.lean',p/'FinalFamilyAudit.olean')])
if os.environ.get('PREFIX_AUDIT_ONLY'):
 jobs=[jobs[-1]]
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
