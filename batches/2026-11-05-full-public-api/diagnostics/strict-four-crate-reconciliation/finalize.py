from pathlib import Path
import json,hashlib
p=Path(__file__).parent;s=json.loads((p/'summary.json').read_text());shadows=[]
for crate in s:
 for r in json.loads((p/crate/'records.json').read_text())['records']:
  if r['configuration']=='default' and r['kind']=='shadowed-provided-default':shadows.append({'crate':crate,'rust_identity':r['canonical_rust_identity'],'local_id':r['local_id'],'source_span':r['source_span'],'trait':r['trait_with_arguments'],'owner':r['owner_with_type_arguments'],'impl_generics_and_bounds':r['impl_type_binders_and_bounds'],'explicit_impl_function_names':r['explicit_impl_function_names'],'correction':'Preserve provenance; explicit implementation wins; no second inherited body obligation.'})
assert len(shadows)==35
(p/'shadow-corrections.json').write_text(json.dumps(shadows,indent=2)+'\n')
configs={'ascii':json.loads(Path('/workspace/full-api/enum/ascii/configuration-matrix.json').read_text())}
for crate in ['bit_field','byteorder','byteorder-lite']:
 inv=json.loads((Path('/workspace/full-api/bits')/crate/'inventory.json').read_text());configs[crate]={'observed_configuration_names':list(inv['feature_configs']),'membership':'records.json observed_configs; original compiler rustdoc retained and hashed','signature_basis':'legacy merged definitions; no cross-configuration signature/behavior equivalence inferred','published_proofs':'default, x86_64; native-endian claims additionally little/debug as scoped','other_targets_release_unproved':True}
(p/'configuration-evidence.json').write_text(json.dumps(configs,indent=2)+'\n')
text='''# Strict public API reconciliation: four crates

All170 published scoped keys have one effective default-configuration source entry: ascii37, bit_field27, byteorder42 and byteorder-lite64. No published key, Rust/manifest/proof source or frozen proof artifact is changed. Full owner types, trait arguments, generic binders, extra bounds and theorem preconditions are retained. This source review is not a new proof run or a full API closure claim.

| Crate/version | Local manual or inherited-local instance rows | Published Lean | Frozen unpublished | Without those mappings | Shadowed defaults, separate |
|---|---:|---:|---:|---:|---:|
| ascii1.1.0 |273|37|4|232|31|
| bit_field0.10.3 |53|27|0|26|0|
| byteorder1.5.0 |193|42|0|151|2|
| byteorder-lite0.1.0 |180|64|0|116|2|

ASCII232 is a Lean-mapping gap, not a claim that all entries lack other evidence:11 additional entries retain separate full-domain original-body Kani checks beyond published Lean mappings. Their legacy evidence/domain is preserved and not promoted into additional Lean claims. The partial-domain constructor result remains partial. Generic rows retain their binders and do not enumerate all downstream runtime instantiations.

## Provided-default correction

rustdoc provided_trait_methods includes names that an implementation explicitly overrides. The35 corrected default-configuration rows are preserved in shadow-corrections.json, with explicit impl names and owner/bounds. ASCII accounts for31, including AsciiStr::AsAsciiStr::as_ascii_str/get_ascii and iterator size_hint overrides. The two endian types each override ToOwned::clone_into in byteorder and Lite, giving two shadows per crate. BitField has zero. An explicit implementation wins: the shadow is not another inherited executable body. This does not alter local53/193/180 totals or invalidate a published key.

## Separate obligations and provenance

Only automatically-derived standard-trait implementation bodies are excluded. Handwritten/macro standard impls remain obligations. No production nonstandard derive was observed in these source configurations; ASCII serde impls are handwritten and unproved. Defaults inherited through excluded derived-standard impls are distinguished from ordinary inherited defaults.

ASCII default has273 manual/macro callable rows;8 required custom-trait protocols and3 default templates are separate. There are107 external conditional callable projections and332 effective inherited-default rows, plus32 excluded derived-standard callable bodies and59 defaults via excluded derived-standard impls. Its198 structural/data rows are separate. Do not sum these mixed categories and call the result unverified crate-body APIs.

BitField has9 required protocol declarations separately from53 implementation methods. Its12 BIT_LENGTH invariants have known independent theorem references under the guarded12-primitive layout dictionaries and x86_64 pointer assumptions; these are not12 extra callable mappings or arbitrary Rust layout guarantees. The published generic BitArray bit_length/get_bit/set_bit contracts remain callback-conditional composition, not downstream-body closure. All26 range methods remain unproved; signed behavior observations are not replaced by unsigned mask/frame assumptions.

byteorder has54 manual and139 inherited-local instance rows;122 trait protocol/template records are separate. Lite has86 manual and94 inherited-local rows;109 protocol/template records are separate. Each retains20 external conditional callable rows. Generic R/W I/O instances keep Read/Write bounds and method ByteOrder parameters; they are not proofs of arbitrary downstream callbacks.

## Binders, extra bounds and nightly gates

Structured implementation/method parameters, where clauses, associated projections, unsafe headers and default_unstable values are retained. Inherited dependency source signatures expose extra conditions such as Self:Sized and B:FromIterator<Self::Item>; owner substitution and bound satisfaction are never assumed. Runtime unstable, const-only rustc_const_unstable, deprecation and cfg annotations are stored separately. Source annotations do not prove compiler gate availability or public-module reachability.

For ASCII default,322 of332 effective inherited rows have a dependency-source signature view; ten unresolved local-default/dependency views remain conditional. The all-features surface additionally retains unresolved serde dependency details. Missing/macro/ambiguous signatures stay unresolved. Nightly-tag counts are projection occurrences, not distinct stable public crate functions. No full std generic-bound, inherited module-gate or reachability closure is claimed, and signature resolution discharges no behavior proof.

## Real configurations and data operations

ASCII compiler snapshots separately cover default/all/minimal/alloc_only, with1430/1448/1046/1398 mixed records. These are not API proof counts. Original serde-without-std combinations (with/without alloc) failed to compile and remain recorded. BitField default/no-default/all; byteorder default/no-default/all/i128-only; and Lite default/no-default/all observations are explicit per record. See configuration-evidence.json and compiler rustdoc hashes. Legacy merged definitions are not proof of signature equivalence across configurations. Alternate targets/release and all cfg combinations are not inferred from all-features.

Structural rows retain type, field, variant, associated-type and constant definitions as data/invariant obligations. The128 AsciiChar variants are not128 function proofs: consumer theorems quantify over valid values without guaranteeing invalid enum representations, arbitrary unchecked byte construction or memory layout. Private fields are not promoted into public constructors. Vector/string models preserve element sequences and bounds, not physical capacity, allocation identity or OOM behavior.

## Frozen41 is not published41

The four Vec<AsciiChar> conversion results are a separate frozen-unpublished column. Typed Rust source entries are unique, but that does not resolve the CLI/server binding gate. Legacy candidate generation erases Vec arguments and collides with Vec<u8>. The observed complete live catalogue has three desired typed cards; reverse From<AsciiString> for Vec<AsciiChar> is absent. Exact typed source and remote singleton binding must be verified independently before publication. A typed precondition cannot repair a wrong card. This review leaves all four held and changes no frozen proof artifact.

## Files and limits

Each crate has records.json, summary.json and published-source-bindings.json. shadow-corrections.json lists35 corrections. dependency-signature-resolution.json preserves successful and unresolved source views. Legacy inventories are copied byte-for-byte into legacy/, with SHA256 and compiler-rustdoc paths/hashes in legacy-manifest.json; original inventories stay untouched. Raw legacy coverage and separate evidence are retained. No heavy proof or Rust compilation was run for this reconciliation.

PUBLISH-MANIFEST.json freezes the Git documentation handoff. Legacy snapshots stay locally retained and hash-referenced, avoiding duplication in every proof report. This package does not authorize changing claims, lifting ASCII4's publication gate, or declaring full API completion.
'''
(p/'README.md').write_text(text)
for x in json.loads((p/'legacy-manifest.json').read_text()):
 assert hashlib.sha256(Path(x['path']).read_bytes()).hexdigest()==x['sha256']
 assert hashlib.sha256(Path(x['snapshot']).read_bytes()).hexdigest()==x['sha256']
files=[]
for f in sorted(p.rglob('*')):
 if not f.is_file() or f.name=='PUBLISH-MANIFEST.json' or 'legacy' in f.relative_to(p).parts:continue
 files.append({'path':str(f.relative_to(p)),'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'bytes':f.stat().st_size})
tree=hashlib.sha256(''.join(x['path']+'\0'+x['sha256']+'\n' for x in files).encode()).hexdigest()
(p/'PUBLISH-MANIFEST.json').write_text(json.dumps({'scope':'Git documentation only, no claim/mapping changes','created_utc':'2026-11-05','published_source_keys_unique':170,'shadow_corrections':35,'unpublished_ascii4_gate_held':True,'legacy_snapshot_policy':'Local copies retained byte-identical; public references use legacy-manifest hashes and original compiler artifact paths','exclude':['legacy'],'tree_sha256':tree,'files':files},indent=2)+'\n')
print('DONE',tree,'files',len(files),'bytes',sum(x['bytes'] for x in files))
