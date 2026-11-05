#!/usr/bin/env python3
"""Restore unchanged recorded SARIF; publish at most five pending catalogues."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,re,shutil,subprocess,urllib.request
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
p=argparse.ArgumentParser();p.add_argument('--publish',action='store_true');p.add_argument('--max-new-imports',type=int,choices=range(1,6),default=5);a=p.parse_args()
ledger_path=HERE/'publication-ledger.json';ledger=json.loads(ledger_path.read_text())
rows=[r for r in ledger['crates'] if r['publication_status']!='published' and r['fresh_record_passed']][:a.max_new_imports]
for r in rows:print(r['crate'],r['version'],r['run_id'])
if not a.publish:
 print('Plan only; use --publish after the quota reset.')
 raise SystemExit(0)
if datetime.datetime.now(datetime.timezone.utc).date()<datetime.date(2026,11,6):raise SystemExit('No requests sent: earliest next quota bucket is 2026-11-06 UTC.')
for r in rows:
 manifest=(ROOT/r['artifact_path']/'Cargo.toml').resolve();proof=manifest.parent/'proofs'
 sarif=HERE/'runs'/r['crate']/'run.sarif.json'
 assert hashlib.sha256(sarif.read_bytes()).hexdigest()==r['archived_sarif_sha256']
 state=Path(os.environ.get('XDG_DATA_HOME',str(Path.home()/'.local/share')))/'cargo-proofs'/hashlib.sha256(str(manifest).encode()).hexdigest()/'runs'/r['run_id'];state.mkdir(parents=True,exist_ok=True)
 dest=state/'run.sarif.json'
 if dest.exists():assert dest.read_bytes()==sarif.read_bytes(), 'Existing run differs; refusing overwrite'
 else:shutil.copy2(sarif,dest)
 logs=HERE/'resume-logs';logs.mkdir(exist_ok=True)
 for mode in ['dry','publish']:
  cmd=['cargo','proofs','publish','--manifest-path',str(manifest),'--run',r['run_id']]
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
