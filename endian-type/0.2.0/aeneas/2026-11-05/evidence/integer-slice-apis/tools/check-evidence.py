from pathlib import Path
import re,json,tarfile,hashlib
r=Path(__file__).resolve().parents[1]
types=['u16','u32','u64','u128','usize','i16','i32','i64','i128','isize']
code=(r/'src/lib.rs').read_text()
assert all(x not in code for x in ['kani::assume','kani::stub','kani::unwind'])
assert '#[repr(C,align(16))]' in code
assert set(re.findall(r'fn (\w+)_valid_integer_slice_apis',code))==set(types)
expected_reads={(wrapper,t) for wrapper in ['BigEndian','LittleEndian'] for t in types}
assert set(re.findall(r'(BigEndian|LittleEndian)::<(\w+)>::from_byte_slice',code))==expected_reads
assert code.count('unsafe {')==20
with tarfile.open(r/'evidence/upstream.crate') as archive:
 for m in archive.getmembers():
  if m.isfile():assert (r/'source'/Path(*Path(m.name).parts[1:])).read_bytes()==archive.extractfile(m).read(),m.name
text=(r/'verification.log').read_text()
assert set(re.findall(r'Checking harness checks::(\w+)_valid_integer_slice_apis',text))==set(types)
summaries=re.findall(r'\*\* (\d+) of (\d+) failed(?: \((\d+) unreachable\))?',text)
assert len(summaries)==10 and all(f=='0' for f,n,u in summaries)
records=re.findall(r'Check \d+: ([^\n]+)\n\s+- Status: (\w+)\n\s+- Description: ([^\n]+)',text)
owned=[(name,status) for name,status,_ in records if name.startswith('checks::')]
unsupported=[(name,status) for name,status,_ in records if 'unsupported_construct' in name]
assert len(owned)==180 and all(status=='SUCCESS' for name,status in owned)
assert len(unsupported)==20 and all(status=='SUCCESS' for name,status in unsupported)
assert all(status in ['SUCCESS','UNREACHABLE'] for name,status,_ in records)
summary={'verified_harnesses':10,'generic_APIs':6,'integer_specializations':60,'generic_case_unproved':True,'user_spec_assertions_successful':180,'checks':sum(int(n) for f,n,u in summaries),'unreachable_internal_checks':sum(int(u or 0) for f,n,u in summaries),'failed':0,'unsupported_call_reachability_checks_all_successful':20,'harness_sha256':hashlib.sha256(code.encode()).hexdigest(),'archive_source_files_identical':True}
(r/'evidence/verification-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary))
