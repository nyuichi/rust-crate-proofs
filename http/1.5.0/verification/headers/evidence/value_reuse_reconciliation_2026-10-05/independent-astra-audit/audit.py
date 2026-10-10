from pathlib import Path
import json,hashlib,collections,subprocess,gzip,tempfile,shutil,re
repo=Path('/workspace/rust-crate-proofs');evidence=repo/'http/1.5.0/verification/headers/evidence';current=evidence/'value_current_emission_2026-10-05';out=Path(__file__).parent;sha=lambda b:hashlib.sha256(b).hexdigest();h=lambda p:sha(p.read_bytes());manifest=json.loads((current/'manifest.json').read_text()); report={'result':'PASS','solver_invoked':False,'frontend_invoked':False,'source_modified':False,'current_manifest_sha256':h(current/'manifest.json'),'primary_sources':manifest['primary_sources'],'sets':[],'ledgers':[]}
for key,base in [('source_manifest',current),('emitted_coma_manifest',current/'emission')]:
 ledger=current/manifest[key]; lines=ledger.read_text().splitlines(); assert h(ledger)==manifest[key+'_sha256']
 for l in lines:
  digest,name=l.split('  ',1);assert h(base/name)==digest,(key,name)
 report['ledgers'].append({'file':str(ledger.relative_to(repo)),'sha256':h(ledger),'entries':len(lines),'all_match':True})
for path,digest in manifest['primary_sources'].items():assert h(current/'sources'/path)==digest
replay='/tmp/uri-eq-replay-build/replay';report['replay_binary_sha256']=h(Path(replay));src=repo/'http/1.5.0/verification/method/evidence/current-source-reconciliation-20261005/independent-astra-nested/replay.ml';shutil.copyfile(src,out/'replay.ml');report['replay_source_sha256']=h(src)
def keep(path,data):
 path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(gzip.compress(data,mtime=0));return {'file':str(path.relative_to(out)),'bytes':len(data),'sha256':sha(data),'gzip_sha256':h(path)}
def stats(n,root=True,path=[]):
 c=collections.Counter(); nested=[]
 if n is None:c['null']=1
 elif 'prover' in n:c[n['prover']]=1
 elif n.get('children'):
  if not root:c['nested']=1;nested.append(path)
  for i,x in enumerate(n['children']): cc,nn=stats(x,False,path+[i]);c.update(cc);nested.extend(nn)
 else:c['zero']=1
 return c,nested
paths_by_set=[]
for setname,snapshot in [('v7','value_complete_translation_v7'),('5420','value_profile_5420_full_emission_v1')]:
 oldroot=evidence/'archived_snapshots'/snapshot;snap=json.loads((oldroot/'snapshot.json').read_text());targets=[t for b in snap['proof_results'] for t in b['targets']] if setname=='v7' else snap['proof_run']['targets'];group={'set':setname,'snapshot':str((oldroot/'snapshot.json').relative_to(repo)),'snapshot_sha256':h(oldroot/'snapshot.json'),'targets':[]};names=set();total=collections.Counter()
 for t in targets:
  old=repo/t['coma'] if t['coma'].startswith('http/') else oldroot/t['coma'];pf=repo/t['proof_json'] if t['proof_json'].startswith('http/') else oldroot/t['proof_json'];assert h(old)==t['coma_sha256'] and h(pf)==t['proof_json_sha256'];relative=str(old).split('/comas/value/',1)[1];names.add(relative);new=current/'emission/verif/http_headers_proof_rlib/header/value'/relative;tree=json.loads(pf.read_text())['proofs']['Coma'];goal=t['own_vc'];node=tree[goal];ss,nested=stats(node);assert ss['null']==0 and ss['zero']==0;total.update(ss);dest=out/'contexts'/setname/relative.removesuffix('.coma');dest.mkdir(parents=True,exist_ok=True);row={'target':relative,'old_coma':str(old.relative_to(repo)),'old_coma_sha256':h(old),'current_coma':str(new.relative_to(repo)),'current_coma_sha256':h(new),'old_proof':str(pf.relative_to(repo)),'old_proof_sha256':h(pf),'own_root':goal,'stats':dict(ss),'zero_child_support':[k for k,n in tree.items() if k!=goal and n.get('children')==[]],'contexts':{}};streams={};api_contexts={}
  for label,coma in [('old',old),('current',new)]:
   cmd=['/workspace/proof-tools/creusot-data/bin/why3','prove','-C','/workspace/proof-tools/config/creusot/why3.conf','-L','/workspace/proof-tools/creusot-data/share/why3find/packages/creusot','-a','split_vc','-D','why3',str(coma)];r=subprocess.run(cmd,capture_output=True);assert r.returncode==0,r.stderr;streams[label]=r.stdout;context={'full_stdout':keep(dest/(label+'.stdout.gz'),r.stdout)};(dest/(label+'.stderr')).write_bytes(r.stderr)
   with tempfile.TemporaryDirectory(prefix='astra-value-api-') as tmp:
    a=Path(tmp);selected=','.join(map(str,nested[0])) if nested else '-';assert len(nested)<=1;pr=subprocess.run([replay,str(coma),goal,selected,str(a)],capture_output=True);assert pr.returncode==0,pr.stderr;(dest/(label+'.api.stdout')).write_bytes(pr.stdout);(dest/(label+'.api.stderr')).write_bytes(pr.stderr);raws={f.name:f.read_bytes() for f in a.glob('*.why')};initial=len([n for n in raws if n.startswith('depth-0-child-')]);assert initial==len(node['children']);assert len(re.findall(rb'^goal ',r.stdout,re.M))==initial;context['api']={n:keep(dest/label/(n+'.gz'),b) for n,b in sorted(raws.items())};api_contexts[label]=raws
   row['contexts'][label]=context
  assert streams['old']==streams['current'],relative; assert api_contexts['old']==api_contexts['current'],relative;row['full_stdout_byte_identical']=True;row['all_api_contexts_byte_identical']=True;row['initial_arity']=initial;row['nested_paths']=nested
  if nested:
   assert relative=='are_visible_value_bytes.coma' and nested==[[8]];assert len([n for n in api_contexts['old'] if n.startswith('depth-1-child-')])==2;assert api_contexts['old']['parent-1.why']==api_contexts['old']['depth-0-child-8.why'];row['nested_arity']=2
  group['targets'].append(row)
 group['target_count']=len(group['targets']);group['initial_tasks']=sum(t['initial_arity'] for t in group['targets']);group['tree_totals']=dict(total);group['terminal_proved_leaves']=sum(v for k,v in total.items() if '@' in k);group['zero_child_support_occurrences']=sum(len(t['zero_child_support']) for t in group['targets']);report['sets'].append(group);paths_by_set.append(names);print(setname,group['target_count'],group['initial_tasks'],group['terminal_proved_leaves'],flush=True)
report['set_intersection']=sorted(paths_by_set[0]&paths_by_set[1]);report['union_target_count']=len(paths_by_set[0]|paths_by_set[1]);assert not report['set_intersection'] and report['union_target_count']==85;report['counting_note']='Counts remain per evidence set: v7 67 targets/183 initial tasks/184 proved terminal leaves; 5420 18 targets/55 initial tasks/55 proved terminal leaves. No aggregate proof-leaf claim is made.';(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print('report_sha256',h(out/'report.json'))
