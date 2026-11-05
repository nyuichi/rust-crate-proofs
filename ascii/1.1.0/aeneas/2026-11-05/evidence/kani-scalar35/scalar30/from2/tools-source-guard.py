from pathlib import Path
import tarfile,json,re,hashlib
p=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();before=json.loads((p/'evidence/source-before.json').read_text());after={str(f.relative_to(p/'original')):sha(f.read_bytes()) for f in (p/'original').rglob('*') if f.is_file()};assert before==after
with tarfile.open(p/'evidence/upstream.crate') as t:
 for f in ['Cargo.toml','src/ascii_char.rs','src/lib.rs']:assert (p/'original'/f).read_bytes()==t.extractfile('ascii-1.1.0/'+f).read()
s=(p/'original/src/ascii_char.rs').read_text();enum=s.split('pub enum AsciiChar {',1)[1].split('\n}',1)[0];variants=re.findall(r'^\s*(\w+)\s*=\s*(\d+),',enum,re.M);assert len(variants)==128 and sorted(int(v) for _,v in variants)==list(range(128))
checks={}
for name in ['from_ascii_char_to_u8_all_variants','from_ascii_char_to_char_all_variants']:
 text=(p/'evidence'/(name+'.log')).read_text();assert 'VERIFICATION:- SUCCESSFUL' in text and '0 of 4 failed' in text and '1 successfully verified harnesses, 0 failures' in text;checks[name]={'checks':4,'failed':0}
x={'source_unchanged':True,'archive_matched':True,'valid_enum_discriminants':128,'domain':'all valid AsciiChar values, no fixed-buffer or length restriction','apis':['<u8 as core::convert::From<ascii::AsciiChar>>::from','<char as core::convert::From<ascii::AsciiChar>>::from'],'manual_macro_not_std_derive':True,'macro_source_lines':[755,789,790],'checks':checks,'new_models':0,'original_source_adaptations':0,'solver':'Kani0.68 CBMC6.11 nightly2026-08-21','Aeneas_macro_root':'known prior failed route not rerun','primary_public_count_change':0};(p/'evidence/results.json').write_text(json.dumps(x,indent=2)+'\n');print('PASS: full128enum domain,8checks0failed,originalarchive+hashstable')
