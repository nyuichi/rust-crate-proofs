#!/usr/bin/env python3
"""Explicitly resume selected latest records; stop on any service error."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,re,shutil,subprocess,urllib.request
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
p=argparse.ArgumentParser();p.add_argument('--publish',action='store_true');p.add_argument('--crate',action='append',default=[]);a=p.parse_args()
ledger_path=HERE/'proof-updates.json';ledger=json.loads(ledger_path.read_text())
rows=[r for r in ledger['updates'] if r['crate'] in a.crate and r['publication_status']=='new_catalogue_quota_hold' and r['publisher_fresh_record_status']=='passed']
if len(rows)>5:raise SystemExit('Select no more than five pending crate imports per invocation; service quota remains authoritative.')
for r in rows:print(r['crate'],r['version'],r['publisher_run_id'])
if not a.publish:
 print('Plan only. Select --crate NAME explicitly; --publish makes service requests. Quota reset is not verified against the deployed revision.')
 raise SystemExit(0)
for r in rows:
 manifest=(ROOT/r['artifact_path']/'Cargo.toml').resolve();proof=manifest.parent/'proofs'
 sarif=HERE/r['archived_sarif']
 assert hashlib.sha256(sarif.read_bytes()).hexdigest()==r['archived_sarif_sha256']
 state=Path(os.environ.get('XDG_DATA_HOME',str(Path.home()/'.local/share')))/'cargo-proofs'/hashlib.sha256(str(manifest).encode()).hexdigest()/'runs'/r['publisher_run_id'];state.mkdir(parents=True,exist_ok=True)
 dest=state/'run.sarif.json'
 if dest.exists():assert dest.read_bytes()==sarif.read_bytes(), 'Existing run differs; refusing overwrite'
 else:shutil.copy2(sarif,dest)
 logs=HERE/'resume-logs';logs.mkdir(exist_ok=True)
 for mode in ['dry','publish']:
  cmd=['cargo','proofs','publish','--manifest-path',str(manifest),'--run',r['publisher_run_id']]
  if mode=='dry':cmd+=['--dry-run']
  result=subprocess.run(cmd,cwd=proof,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
  (logs/(r['crate']+'.'+mode+'.log')).write_text(result.stdout)
  if result.returncode:print(result.stdout);raise SystemExit('Stopped without retry; inspect the saved service error.')
 match=re.search(r'Published report #(\d+), revision (\d+): (https://\S+)',result.stdout);assert match
 r.update(publication_status='published',report_id=int(match[1]),revision=int(match[2]),report_url=match[3])
 ledger_path.write_text(json.dumps(ledger,indent=2)+'\n')
 req=urllib.request.Request(match[3],headers={'User-Agent':'cargo-proofs/0.4.1'})
 with urllib.request.urlopen(req) as response:html=response.read()
 (logs/(r['crate']+'.readback.html')).write_bytes(html)
 r['readback_passed']=all(s.encode() in html for s in [r['crate'],r['version'],r['source_commit'][:8]])
 ledger_path.write_text(json.dumps(ledger,indent=2)+'\n');assert r['readback_passed']
 print(match[3])
