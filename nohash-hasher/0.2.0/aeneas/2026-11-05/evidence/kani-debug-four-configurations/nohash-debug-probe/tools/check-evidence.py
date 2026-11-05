from pathlib import Path
import tarfile,sys,re
r=Path(__file__).resolve().parents[1]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for f in ('src/lib.rs','Cargo.toml'):
  assert (r/'source'/f).read_bytes()==t.extractfile('nohash-hasher-0.2.0/'+f).read()
print('Original source/manifest archive identical')
if '--source-only' in sys.argv:sys.exit(0)
for mode in ('debug','release'):
 s=(r/('evidence/'+mode+'-u8-final.log')).read_text()
 assert 'VERIFICATION:- SUCCESSFUL' in s
 assert '1 successfully verified harnesses, 0 failures, 1 total.' in s
 assert not re.search(r'Status: FAILURE',s)
 n=(r/('evidence/native-'+mode+'.log')).read_text()
 assert 'exhaustive_cases=1024; debug_assertions='+str(mode=='debug').lower() in n
s=(r/'src/lib.rs').read_text()
assert 'kani::assume' not in s and 'kani::stub' not in s
assert '#[kani::unwind(80)]' in s
print('Both original-source Kani configurations and native finite enumerations passed')
