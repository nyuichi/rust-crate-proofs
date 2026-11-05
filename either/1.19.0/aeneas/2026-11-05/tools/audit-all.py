from pathlib import Path
import tomllib,subprocess,sys,json,hashlib
pkg=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=True)
tool=tomllib.loads((pkg/'proofs.toml').read_text())['tool'];proofs=tool['proofs'];modules=sorted(set(p['module'] for p in proofs))
s='import Lean\n'+''.join('import '+m+'\n' for m in modules)+'open Lean Elab Command\n'
for p in proofs:
 n=p['theorem'];s+=f'''\nrun_cmd do
  let theorem_name := `{n}
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: {n}"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in {n}: {{ax}}"
  liftIO <| IO.println "proofs.rs:checked:{n}"
'''
(out/'audit-all.lean').write_text(s)
r=subprocess.run(['lake','env','lean','--stdin'],input=s,text=True,cwd=pkg/tool.get('lean_dir','.'),capture_output=True)
(out/'audit-all.log').write_text(r.stdout+r.stderr);assert r.returncode==0,(r.returncode,r.stderr)
markers=[l.removeprefix('proofs.rs:checked:') for l in r.stdout.splitlines() if l.startswith('proofs.rs:checked:')];assert markers==[p['theorem'] for p in proofs]
(out/'result.json').write_text(json.dumps({'simultaneously_imported_modules':modules,'checked_api_theorems':len(markers),'status':'passed','policy':'Exact CLI audit_source pattern: thmInfo and Lean.collectAxioms allow only propext/Classical.choice/Quot.sound','source_sha256':hashlib.sha256(s.encode()).hexdigest()},indent=2)+'\n');print(len(markers),modules,'passed')
