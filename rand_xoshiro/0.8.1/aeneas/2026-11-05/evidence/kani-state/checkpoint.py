from pathlib import Path
import re,json,datetime
b=Path(__file__).resolve().parent;apis=json.loads((b/'api-bindings.json').read_text());out={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Partial per-configuration checkpoints; not a completed batch claim','configurations':{}}
for mode in ['default','serde']:
 p=b/(mode+'-kani.log');text=p.read_text(errors='replace') if p.exists() else '';parts=re.split(r'Checking harness ([\w:]+)\.\.\.',text);rows=[]
 for api in apis:
  row={'api':api['api'],'harness':api['harness'],'status':'not_started'}
  for i in range(1,len(parts),2):
   if parts[i]!=api['harness']:continue
   s=parts[i+1];row['status']='verified' if 'VERIFICATION:- SUCCESSFUL' in s else 'failed' if 'VERIFICATION:- FAILED' in s else 'in_progress_or_stopped'
   check=re.findall(r'0 of (\d+) failed',s)
   if check:row['checks']=int(check[-1])
  rows.append(row)
 out['configurations'][mode]={'verified_count':sum(r['status']=='verified' for r in rows),'total':len(rows),'apis':rows}
(b/'checkpoint.json').write_text(json.dumps(out,indent=2));print({m:x['verified_count'] for m,x in out['configurations'].items()})
if all(v['verified_count']==15 for v in out['configurations'].values()):
 out['scope']='All fifteen original full-state serialization body contracts verified independently under default and serde';(b/'results.json').write_text(json.dumps(out,indent=2))
