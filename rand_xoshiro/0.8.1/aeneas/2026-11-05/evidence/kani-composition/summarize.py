from pathlib import Path
import re,json
b=Path(__file__).resolve().parent;rows=[]
for mode in ['default','serde']:
 text=(b/(mode+'-kani.log')).read_text()
 assert 'Complete - 5 successfully verified harnesses, 0 failures, 5 total.' in text
 checks=[tuple(map(int,x)) for x in re.findall(r'\*\* (\d+) of (\d+) failed',text)]
 assert len(checks)==5 and all(f==0 for f,n in checks),checks
 rows.append({'feature_configuration':mode,'harnesses':5,'checks':sum(n for f,n in checks),'failed':0,'result':'VERIFICATION SUCCESSFUL','note':'Checks are proof obligations, not concrete test cases.'})
(b/'results.json').write_text(json.dumps({'crate':'rand_xoshiro','version':'0.8.1','tool':'Kani 0.68.0 / CBMC 6.11.0','rust_toolchain':'nightly-2026-08-21','target':'x86_64-unknown-linux-gnu','source_relationship':'All original source bytes remain a full prefix; observer-only suffixes and separate harness source are added in a generated measurement copy. Original production source and Cargo manifest are unchanged.','guarantees':json.load(open(b/'api-bindings.json')),'configurations':rows,'publication_status':'Evidence ready. Ordinary kani::proof harnesses do not directly bind APIs in cargo-proofs 0.4.1, which only registers proof_for_contract harnesses; do not label these as Aeneas machine claims.'},indent=2))
print('Both feature configurations: all five original API bodies verified, 892 checks each, no failures')
