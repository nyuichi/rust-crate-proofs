from pathlib import Path
import shutil,json,hashlib
p=Path(__file__).resolve().parents[1];w=p.parent/'oorandom-range-work';old=Path('/workspace/full-api/oorandom-range')
for f in ['Cargo.toml','Cargo.lock','src/lib.rs']:
 d=p/f;d.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(old/f,d)
for f in ['lake-manifest.json','lean-toolchain']:
 d=p/'proofs'/f;d.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(old/'proofs'/f,d)
for f in ['Checkpoint.lean','FullRange.lean','Zero64.lean','PublicDomain.lean','NegBridge.lean','Oorandom/Funs.lean','Oorandom/Types.lean','Oorandom/translation.json']:
 d=p/'proofs'/f;d.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(w/'original-route/proofs'/f,d)
for f in ['native-results.json','original-route-reproof.log','stdneg-name-wildcard/result.json','stdneg-name-wildcard/charon.log','stdneg-name-wildcard/aeneas.log','stdneg-name-wildcard/oorandom.llbc']:
 d=p/'evidence'/f;d.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(w/'evidence'/f,d)
(p/'evidence/native').mkdir(exist_ok=True);shutil.copy2(w/'native/main.rs',p/'evidence/native/main.rs')
(p/'proofs/lakefile.lean').write_text('import Lake\nopen Lake DSL\nrequire aeneas from git "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"\npackage proofs\n@[default_target] lean_lib EstablishedRange\nlean_lib Oorandom\n')
(p/'proofs/EstablishedRange.lean').write_text('import PublicDomain\nopen Aeneas Aeneas.Std Result\nopen oorandom_range_original\nnamespace EstablishedRange\ntheorem rand32_contract :\n (∀ (r : Rand32) (q : core.ops.range.Range U32), ¬q.start < q.end → Rand32.rand_range r q = .fail .assertionFailure) ∧\n (do let r ← Rand32.from_state (0#u64,0#u64); Rand32.rand_range r ⟨0#u32,3#u32⟩) = .div :=\n ⟨PublicDomain.range32_invalid, PublicDomain.from_state32_range3_div⟩\ntheorem rand64_contract :\n (∀ (r : Rand64) (q : core.ops.range.Range U64), ¬q.start < q.end → Rand64.rand_range r q = .fail .assertionFailure) ∧\n (do let r ← Rand64.from_state (0#u128,0#u128); Rand64.rand_range r ⟨0#u64,3#u64⟩) = .div :=\n ⟨PublicDomain.range64_invalid, PublicDomain.from_state64_range3_div⟩\n#print axioms rand32_contract\n#print axioms rand64_contract\nend EstablishedRange\n')
