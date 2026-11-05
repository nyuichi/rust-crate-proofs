from pathlib import Path
import re,json,tarfile,hashlib
r=Path(__file__).resolve().parents[1];source=r/'source';code=(r/'src/lib.rs').read_text()
assert '#[kani::unwind(80)]' in code
assert all(x not in code for x in ['kani::assume','kani::stub','no-unwinding','format!(', 'String::','Vec::'])
assert 'for i in 0..64' in code and 'sink.nonempty_attempt_len,expected_len' in code
with tarfile.open(r/'evidence/upstream.crate') as t:
 for m in t.getmembers():
  if m.isfile():assert (source/Path(*Path(m.name).parts[1:])).read_bytes()==t.extractfile(m).read(),m.name
s=(r/'verification.log').read_text();summary=re.findall(r'\*\* (\d+) of (\d+) failed(?: \((\d+) unreachable\))?',s);assert summary==[('0','1910','125')],summary
records=re.findall(r'Check \d+: ([^\n]+)\n\s+- Status: (\w+)\n\s+- Description: ([^\n]+)',s)
assert all(st in ['SUCCESS','UNREACHABLE'] for n,st,d in records)
own=[(n,st) for n,st,d in records if n.startswith('checks::')];unsupported=[(n,st) for n,st,d in records if 'unsupported_construct' in n]
assert len(own)==11 and all(st=='SUCCESS' for n,st in own)
assert len(unsupported)==3 and all(st=='SUCCESS' for n,st in unsupported)
metadata=json.loads((r/'evidence/kani-metadata.json').read_text());h=metadata['proof_harnesses'];assert len(h)==1 and h[0]['attributes']['unwind_value']==80 and not h[0]['attributes']['stubs'] and not h[0]['attributes']['verified_stubs']
result={'status':'limited_default_format_and_sink_domain_passed','harnesses':1,'symbolic_booleans':2,'input_combinations':4,'total_checks':1910,'unreachable_internal_checks':125,'failed_checks':0,'successful_harness_properties':11,'successful_unsupported_reachability_guards':3,'explicit_unwind':80,'unwinding_checks':'Kani default checks retained; no --no-unwinding-checks flag or custom CBMC arguments supplied','original_archive_files_identical':True,'harness_sha256':hashlib.sha256(code.encode()).hexdigest()}
(r/'evidence/verification-summary.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

x=json.loads((r/'results.json').read_text())
assert x['domain']['literal_lengths']=={'InvalidEncoding':len(b'invalid Base16 encoding'),'InvalidLength':len(b'invalid Base16 length')}
assert 'if encoding {23} else {21}' in code
