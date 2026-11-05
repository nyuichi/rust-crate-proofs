import json,pathlib,hashlib
ROOT=pathlib.Path('/workspace/full-api/enum')
files=list(ROOT.glob('*/inventory-*.json'))+list(ROOT.glob('*/coverage-gap.json'))
want=set()
for f in files:
 for e in json.load(open(f))['entries']:
  if e['category']=='inherited-default':want.add((e['trait_canonical_path'],e['name']))
resolved={}
std=pathlib.Path('/workspace/proof-tools/rustup/toolchains/nightly-2026-09-17-x86_64-unknown-linux-gnu/share/doc/rust/json')
docs=[std/'core.json',std/'alloc.json',std/'std.json',ROOT/'external-target/doc/serde_core.json',*ROOT.glob('*/all/rustdoc.json')]
for f in docs:
 d=json.load(open(f));idx=d['index'];document_hash=hashlib.sha256(f.read_bytes()).hexdigest()
 for iid,i in idx.items():
  t=i['inner'].get('trait') if isinstance(i['inner'],dict) else None
  if not t:continue
  p=d['paths'].get(iid,{}).get('path');p='::'.join(p) if p else None
  if not any(k[0]==p for k in want):continue
  for mid in t['items']:
   m=idx.get(str(mid));key=(p,m and m['name'])
   if key in want and 'function' in m['inner']:
    resolved[key]={'raw':m['inner']['function'],'dependency_source_span':m['span'],'trait_generics':t['generics'],'provenance_document':str(f),'document_sha256':document_hash}
 del d,idx
for f in files:
 d=json.load(open(f));n=0
 for e in d['entries']:
  if e['category']!='inherited-default':continue
  k=(e['trait_canonical_path'],e['name'])
  if k in resolved:
   e['signature']=resolved[k];e['signature_resolution']='Resolved from exact toolchain or crate dependency rustdoc JSON; trait generic substitution remains visible in owner/trait arguments';n+=1
 f.write_text(json.dumps(d,indent=2)+'\n')
print('Distinct inherited signatures resolved:',len(resolved),'/',len(want))
missing=sorted(want-set(resolved));print('Missing:',missing)
(ROOT/'inherited-signature-resolution.json').write_text(json.dumps({'resolved_distinct':len(resolved),'total_distinct':len(want),'missing':missing},indent=2)+'\n')
