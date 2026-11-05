from pathlib import Path
import hashlib,json,tarfile,tomllib,re
p=Path(__file__).resolve().parents[1]; sha=lambda b:hashlib.sha256(b).hexdigest()
with tarfile.open(p/'evidence/upstream.crate') as t:
 checks={}
 for f in ['Cargo.toml','src/lib.rs']:
  actual=(p/f).read_bytes();expected=t.extractfile('oorandom-11.1.5/'+f).read();assert actual==expected;checks[f]=sha(actual)
x=json.loads((p/'proofs/Oorandom/translation.json').read_text());opaque=[r for r in x['functions'] if r['is_opaque']];assert not opaque
for name in ['core::num::{u32}::wrapping_neg','core::num::{u64}::wrapping_neg']:
 assert len([r for r in x['functions'] if r['rust_name']==name])==1,name
for kind in ['Rand32','Rand64']:
 assert len([r for r in x['functions'] if r['lean_name']=='oorandom_range_original.'+kind+'.rand_range' and r['rust_name']=='oorandom::{oorandom::'+kind+'}::rand_range'])==1
assert 'External' not in (p/'proofs/Oorandom/Funs.lean').read_text()
std=json.loads((p/'evidence/std-source-guard.json').read_text());assert sha(Path(std['path']).read_bytes())==std['sha256']
assert '(0 as $SelfT).wrapping_sub(self)' in Path(std['path']).read_text()
for kind in ['U32','U64']:
 assert 'ok (Std.'+kind+'.wrapping_sub 0#'+kind.lower()+' self)' in (p/'proofs/Oorandom/Funs.lean').read_text()
for f in p.rglob('*'):
 if not f.is_file():continue
 assert not (re.search(r'\.(olean|ilean|ir)(\.|$)',f.name) or f.read_bytes()[:5]==b'olean'),str(f)
 if f.suffix=='.lean':assert not re.search(r'\b(sorry|admit|axiom)\b',f.read_text()),str(f)
rows=tomllib.loads((p/'proofs.toml').read_text())['tool']['proofs'];assert len(rows)==2
log=(p/'evidence/final-coimport-audit.log').read_text();assert 'sorryAx' not in log and 'error:' not in log
for name in ['EstablishedRange.rand32_contract','EstablishedRange.rand64_contract']:
 assert "'"+name+"' depends on axioms: [propext, Classical.choice, Quot.sound]" in log
out={'source_archive_sha256':sha((p/'evidence/upstream.crate').read_bytes()),'source_matches_archive':checks,'opaque_functions':0,'new_providers':0,'public_method_mappings':2,'complete_general_range_contract':False,'all_valid_inputs_partial_correctness':True,'simultaneous_import_standard3':True,'source_only':True}
(p/'evidence/source-guard.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
