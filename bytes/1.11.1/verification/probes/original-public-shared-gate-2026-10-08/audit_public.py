#!/usr/bin/env python3
"""Audit immutable snapshot membership and the explicitly selected body proofs."""
import argparse,hashlib,json,pathlib,tarfile
p=argparse.ArgumentParser();p.add_argument('label');a=p.parse_args();r=pathlib.Path(__file__).resolve().parent
manifest=json.loads((r/'evidence'/(a.label+'.json')).read_text());archive=r/'evidence'/manifest['archive']
assert hashlib.sha256(archive.read_bytes()).hexdigest()==manifest['sha256']
with tarfile.open(archive) as t:
 for row in manifest['members']:
  assert hashlib.sha256(t.extractfile(row['path']).read()).hexdigest()==row['sha256'],row['path']
 source_summary=json.load(t.extractfile('probe/generated/public-proof-summary.json'))
 source_rows={str(pathlib.Path(row['source']).resolve()):row for row in manifest['members']}
 for relative,expected in source_summary['source_sha256'].items():
  row=source_rows[str((r/relative).resolve())]
  assert row['sha256']==expected,('source-summary mismatch',relative)
 for row in source_summary['targets']:
  coma=t.extractfile('probe/'+row['coma']).read()
  proof=t.extractfile('probe/'+str(pathlib.Path(row['coma']).with_suffix(''))+'/proof.json').read()
  assert hashlib.sha256(coma).hexdigest()==row['coma_sha256']
  assert hashlib.sha256(proof).hexdigest()==row['proof_sha256']
 target=json.load(t.extractfile('probe/generated/public-proof-targets.json'))
 stats={'files':0,'prover':0,'null':0,'structural':0};rows=[]
 def visit(v):
  if v is None:stats['null']+=1
  elif 'children' in v:
   if not v['children']:stats['structural']+=1
   for c in v['children']:visit(c)
  elif 'prover' in v:stats['prover']+=1
  else:raise ValueError(v)
 for name in target['included']:
  proof='probe/'+str(pathlib.Path(name).with_suffix(''))+'/proof.json'
  data=json.load(t.extractfile(proof));stats['files']+=1;before=stats['prover']
  for tree in data['proofs']['Coma'].values():visit(tree)
  rows.append({'target':name,'actual_prover_leaves':stats['prover']-before})
 assert stats['null']==0,stats
 report={'archive_sha256':manifest['sha256'],'members_verified':len(manifest['members']),'scope':'selected body gate, not full From refinement or eventual final cleanup','statistics':stats,'files':rows}
 (r/'evidence'/(a.label+'-audit.json')).write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
