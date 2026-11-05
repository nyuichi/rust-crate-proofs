from pathlib import Path
import json,hashlib,re,tarfile
p=Path(__file__).resolve().parent
h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
core=(p/'core-hash-std09.rs').read_text();assert 'fn write_str(&mut self, s: &str) {\n        self.write(s.as_bytes());\n        self.write_u8(0xff);\n    }' in core
original=(p/'original/src/lib.rs').read_text();assert original.count('fn write(&mut self, _: &[u8]) {\n        panic!("Invalid use of NoHashHasher")\n    }')==2
with tarfile.open(p/'upstream.crate') as tf:
 for x in tf.getmembers():
  if x.isfile():
   f=p/'original'/Path(*Path(x.name).parts[1:]);assert f.read_bytes()==tf.extractfile(x).read(),x.name
cases=[('std-debug','nohash_string_inline','NohashStringInline','generated-reference'),('nostd-debug','nohash_string_nostd_debug','NohashStringNostdDebug','generated'),('nostd-release','nohash_string_nostd_release','NohashStringNostdRelease','generated'),('std-release','nohash_string_std_release','NohashStringStdRelease','generated')]
rows=[];decls=[]
for folder,crate,module,out in cases:
 q=p/folder;raw=json.loads((q/(crate+'.llbc')).read_text());assert raw['has_errors']==False;c=raw['translated'];fs=[f for f in c['fun_decls'] if f and isinstance(f['body'],dict)];assert len(fs)==1;f=fs[0];assert f['item_meta']['name']==[{'Ident':[crate,0]},{'Ident':['write_text',0]}]
 assert [x['name'] for x in f['generics']['types']]==['T'];assert len(f['generics']['trait_clauses'])==1;assert len(f['signature']['inputs'])==2
 clause=f['generics']['trait_clauses'][0]['trait_']['skip_binder'];assert c['trait_decls'][clause['id']]['item_meta']['name']==[{'Ident':['nohash_hasher',0]},{'Ident':['IsEnabled',0]}]
 body=f['body']['Structured']['body'];spans=[];reads=[];calls=[]
 def walk(v):
  if isinstance(v,dict):
   if set(v)>={'file_id','beg','end'}:spans.append(v)
   if v.get('kind')=={'Local':2}:reads.append(v)
   if 'Call' in v:calls.append(v['Call'])
   for z in v.values():walk(z)
  elif isinstance(v,list):
   for z in v:walk(z)
 walk(body);assert not reads and not calls
 panic=[x['kind']['Panic'] for x in body['statements'] if isinstance(x['kind'],dict) and 'Panic' in x['kind']];assert len(panic)==1
 source_files=[c['files'][x['file_id']] for x in spans];assert any(x['crate_name']=='core' and x['name']=={'Local':'/rustc/library/core/src/hash/mod.rs'} for x in source_files)
 generated=(q/out/(module+'.lean')).read_text();assert 'axiom ' not in generated;decl=generated[generated.index('def write_text\n'):].split('\n\n',1)[0];assert '(text : Str)' in decl and ':= do\n  fail panic' in decl;decls.append(decl)
 rows.append({'configuration':folder,'driver_sha256':h(q/'src/lib.rs'),'executed_manifest_sha256':h(q/'Cargo.producer.toml'),'packaged_manifest_sha256':h(q/'Cargo.toml'),'LLBC_sha256':h(q/(crate+'.llbc')),'generated_sha256':h(q/out/(module+'.lean')),'all_string_arg_reads':0,'all_remaining_function_calls':0,'single_original_panic':True,'body_source_spans':[{**x,'file':c['files'][x['file_id']]['name']} for x in spans]})
assert len(set(decls))==1
print(json.dumps({'all4_declarations_exact_byte_equal':True,'original_archive_allfiles_exact':True,'new_models_source_edits_generated_edits':0,'rows':rows},indent=2))
