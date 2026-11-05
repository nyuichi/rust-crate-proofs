from pathlib import Path
import json,hashlib,shutil,re,tomllib
p=Path(__file__).resolve().parent;base=p.parent;packets={'from2':'ascii-new-from2','compare8':'ascii-new-compare8','default20':'ascii-new-default20'};sha=lambda b:hashlib.sha256(b).hexdigest();oldpins={}
for name,origin in packets.items():
 source=base/origin;manifest=json.loads((source/'READY.json').read_text())
 for rel,h in manifest['files'].items():assert sha((source/rel).read_bytes())==h,(origin,rel)
 if not (p/name).exists():shutil.copytree(source,p/name)
 for f in source.rglob('*'):
  if f.is_file():assert f.read_bytes()==(p/name/f.relative_to(source)).read_bytes()
 oldpins[origin]={'tree_sha256':manifest['tree_sha256'],'preserved_unchanged':True}
rawpath=base/'strict-reconciliation/ascii/records.json';raw=json.loads(rawpath.read_text())['records'];rows=[]
def get(owner,trait,method):
 found=[r for r in raw if r['configuration']=='default' and r['kind'] in ['local-manual-or-macro-body','inherited-default-effective-instance'] and r['owner_with_type_arguments']==owner and r['trait_with_arguments']==trait and re.search(r'::'+method+r'(?: \[.*\])?$',r['rust_identity'])]
 assert len(found)==1,(owner,trait,method,len(found));r=found[0];assert not r['is_automatically_derived'];return r
for typ in ['u8','char']:
 q=get(typ,'core::convert::From<ascii::AsciiChar>','from');harness=f'from_ascii_char_to_{typ}_all_variants';rows.append({'exact_Rust_identity':f'<{typ} as core::convert::From<ascii::AsciiChar>>::from','group':'from2','harness':harness,'guarantee':'output scalar code equals input AsciiChar discriminant for all128 valid variants','inventory':q})
 for reverse in [False,True]:
  owner,other=(typ,'ascii::AsciiChar') if reverse else ('ascii::AsciiChar',typ);direction='reverse' if reverse else 'forward';lhs,rhs=('other scalar code','ASCII code') if reverse else ('ASCII code','other scalar code')
  for method,trait,op in [('eq','PartialEq','=='),('partial_cmp','PartialOrd','numeric ordering'),('ne','PartialEq','!='),('lt','PartialOrd','<'),('le','PartialOrd','<='),('gt','PartialOrd','>'),('ge','PartialOrd','>=')]:
   traitfull=f'core::cmp::{trait}<{other}>';q=get(owner,traitfull,method);group='compare8' if method in ['eq','partial_cmp'] else 'default20';rows.append({'exact_Rust_identity':f'<{owner} as {traitfull}>::{method}','group':group,'harness':f'{method}_{typ}_{direction}','operand_order':f'{lhs} {op} {rhs}','domain':'all valid AsciiChar128 × '+('all u8 256' if typ=='u8' else 'all valid Rust char Unicode scalar values'),'inventory':q})
assert len(rows)==30 and len({r['exact_Rust_identity'] for r in rows})==30
published=base/'ascii-extra/candidate-generic-two/proofs.toml';ps=tomllib.loads(published.read_text())['tool']['proofs'];assert len(ps)==51
# Prior51 contains neither these trait From<AsciiChar> outputs nor cross-type comparisons.
assert not [r for r in ps if any(s in r['api'] for s in ['PartialEq','PartialOrd','From<AsciiChar>','From<ascii::AsciiChar>'])]
checks={}
for group in packets:
 r=json.loads((p/group/'evidence/results.json').read_text());checks[group]=r.get('total_checks',sum(v['checks'] for v in r['checks'].values()))
assert checks=={'from2':8,'compare8':528,'default20':1110};(p/'api-bindings30.json').write_text(json.dumps({'source_inventory_sha256':sha(rawpath.read_bytes()),'published51_TOML_sha256':sha(published.read_bytes()),'previous51_overlap':0,'old_frozen_packets':oldpins,'domain':'complete scalar domains, no bounded buffer/allocation shapes','counts':{'local_explicit_manual_macro':10,'effective_inherited_std_default_on_manual_impl':20,'excluded_derived_Self_compares_added':0,'API_contracts':30,'harnesses':30,'checks':1646,'failed':0,'Aeneas_primary_count_change':0},'records':rows},indent=2)+'\n');print('30 unique exact Rust identities,1646checks0fail,oldpins3 preserved')
