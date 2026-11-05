from pathlib import Path
import subprocess,os,signal,time,json,sys
p=Path(__file__).resolve().parents[1];cache=Path(sys.argv[1]);name=sys.argv[2];t=time.monotonic();peak=0;status='running';cmd=['lake','env','bash','-c',f'export LEAN_PATH={cache}:{p}/proofs:$LEAN_PATH; cd {p}/proofs; exec lean -M 8192 -o {cache}/{name}.olean {name}.lean']
with (p/'evidence'/(name.replace('/','-')+'-whole-bounded.log')).open('w') as log:
 proc=subprocess.Popen(cmd,cwd='/workspace/full-api/oorandom-range/proofs',stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 while proc.poll() is None:
  rss=0
  for entry in Path('/proc').iterdir():
   if not entry.name.isdigit():continue
   try:
    parts=(entry/'stat').read_text().split()
    if int(parts[4])==proc.pid:rss+=int(parts[23])*os.sysconf('SC_PAGE_SIZE')
   except (OSError,ValueError,IndexError):pass
  peak=max(peak,rss)
  if rss>4500*1024**2 or time.monotonic()-t>30:
   status='RSS_guard_stopped' if rss>4500*1024**2 else 'timeout';os.killpg(proc.pid,signal.SIGTERM);proc.wait();break
  time.sleep(.1)
 if status=='running':status='success' if proc.returncode==0 else 'failed'
(p/'evidence'/(name.replace('/','-')+'-whole-bounded.json')).write_text(json.dumps({'status':status,'exit':proc.returncode,'seconds':time.monotonic()-t,'peak_rss':peak},indent=2)+'\n');print(status,proc.returncode,peak)

sys.exit(0 if status=="success" else 1)
