# Independent audit of attempt 6

Verdict: **accept all 15 selected targets as complete conditional modular body/refinement evidence**, with the contract-strength limits below. All 37 direct roots are complete and all 40 terminal leaves are successful. This is not complete HeaderMap functional verification or crate-integrated verification.

The audit ran no prover and edited no implementation or contract source. It used only archived source, COMA, and proof JSON files. `audit.py` independently verifies all 177 archived source hashes, all 15 COMA hashes, and every proof JSON root set against newly printed complete task streams. It also replays the exact nested transformation path through the Why3 OCaml API. `compare_context.py` compares complete stdout bytes against attempt 5 and validates the source-after archive and proof JSON hashes. Commands, full compressed streams, hashes, and copied proof JSON are retained alongside `report.json` and `context-report.json`.

## Frozen identity and exact proof counts

The source-freeze SHA-256 is `a3b1e4fbee1d42af4145680ee345c3a989d43a9eb05050771b56d01e0e82c899`. Frozen Map is `9e2dbdc382acb0375d3d8592784077799386f75dc252205658759b2604610edf`; frozen Name is `bc3be7d995c314b74f5aaca5b7259f9b7afd9dcd846b528264be1c43eb3ad5ca`. The 177 source-after records match these archived source hashes exactly.

The 37 direct roots comprise 10 actual function-body roots, 5 trait-refinement roots, and 22 imported support roots whose full goal formula is literally `[@coma:solid] true`. All are complete. The 22 support roots do not establish their corresponding implementation bodies.

Only the actual `&str` implementation has a split proof tree. Its COMA, `header/map/as_header_name/impl_Sealed_for_ref_str/find.coma`, has SHA-256 `a118b8ad1d4d2393d3ca5766d19352299eead7866d8284a04528932e3b3d75c0`. For its own `vc_find_ref_str` root, independent API replay of the original COMA yields initial `split_vc` arity 3; selecting zero-based child 2 and applying `split_vc` only there yields arity 2. All four terminal children now succeed, including formerly null path `[2,1]`. Together with the other 36 direct roots this gives 40 successful terminal leaves. API root bytes equal a separate selected CLI task print; the nested parent bytes equal the selected initial child. Successful siblings were not retransformed and no printed Why3 text was reparsed.

## The attempt-5 blocker is actually repaired

The complete raw task streams for 14 of 15 COMAs are byte-identical to attempt 5, without normalization or filtering to goal text. Only the actual `&str` COMA's complete stream changes. Full diffs are archived as `context-*.diff.gz`.

In that changed context, the former uninterpreted declaration is now the actual pure definition:

```why3
predicate eq_model_ReprDeepModel_Seq_u8 (self:NameModel) (rhs:ParsedModel) =
  header_name_matches_hdr_name self rhs
```

The printer also moves the already-existing predicate definition and its datatype dependencies earlier and renames a local bound variable. No new trusted assertion or HTTP axiom is introduced. The frozen Name source differs from attempt 5 by exactly one attribute: the delegating `PartialEqModel<ParsedModel>` implementation changes `#[logic(open(self))]` to `#[logic(open)]`. Map's source differences are import/lint hygiene; public getter contracts and runtime bodies are unchanged. A separate URI file also changed within the broad 177-file snapshot; it has no new declaration in these Map task streams. All source deltas are listed in `context-report.json`.

The callback already supplied `eq_model(stored_key_model, parsed_key_model)`, and the parser contract already supplied the parsed model for the input bytes. The now-visible definition connects those facts to the existential text observer required by the `&str` postcondition. The formerly failed conclusion is proved without weakening that conclusion or strengthening its runtime precondition.

## Accepted contract strength

| Target | Scope of its accepted body/refinement evidence |
|---|---|
| Private `HeaderMap::get2` | `Some(value)` implies an in-range entry with that exact generic `T` value and a key satisfying `K::matches_header`. `None` remains unconstrained. |
| Public `HeaderMap::get` | Call/type-invariant obligations under readiness. **Its frozen contract has no explicit functional `ensures`**, even though it calls `get2`. |
| `HeaderMap::get_mut` | Borrow/index and type-invariant obligations under readiness. **Its frozen contract has no explicit functional `ensures`**: value correspondence, prophecy, frame, and explicit readiness preservation are still local work. |
| `HeaderMap::get_all` | A returned `Some` index is below the key count and its stored key matches the query. The contract does not identify the returned view's map with the input, establish iteration behavior, or prove absence for `None`. |
| `HeaderMap::contains_key` | A true result implies existence of an in-range matching stored key. A false result does not imply absence. |
| `Sealed::find` for `HeaderName` and `&HeaderName` | A returned index is bounded and the stored key model equals the query key model, relative to the core lookup contract. |
| `Sealed::find` for `&str`, `String`, and `&String` | A returned index is bounded and satisfies the explicit parsed-model text relation. All three selected adapter bodies now close; the former `String::as_str` impossible-precondition issue is absent because the adapter uses the genuine `Deref` contract. |
| Five `find__refines` roots | The declared implementation contracts refine the trait contract; all corresponding selected implementation bodies also complete in this batch. |

The text relation means `exists parsed_model. hdr_parse_success(text_bytes, parsed_model) && header_name_matches_hdr_name(stored_model, parsed_model)`. It is a pure model relation, not an executable lookup or prophetic invariant. No bridge to a simpler public case-normalization law is claimed here. The key and value projections refer to actual storage, and generic `T` remains an unconstrained type whose actual values are compared; there is no generic-model collapse or added `T: DeepModel` restriction.

## Non-vacuity and remaining closure obligations

The selected preconditions use ordinary type invariants and `header_map_find_ready`; no selected call acquires an impossible external precondition. The readiness predicate admits empty maps and has nonempty structural witnesses: mask 1, two slots, one valid bucket with hash 0, an occupied slot pointing to it with matching hash, another empty slot, empty extra values, and `Danger::Green`. It does not require slot coverage or Robin Hood probe order and even permits nonempty entries with all slots empty. Therefore absence completeness cannot follow from this predicate. These properties are unchanged from attempt 5.

All evidence remains relative to imported source contracts for core lookup, hash handling, and Name parsing, plus the usual genuine standard-library specifications and the session's assumed-complete bytes dependency. The successful string adapter batch no longer depends on an unproved `&str` body, but it still does not prove the imported parser and core lookup implementations within this batch. The warning-bearing RandomState construction/build-hasher/finish bodies are unselected; their implementation closure is not implied by these successful callers.

Constructors and mutations establishing a sufficient table invariant, exact mutable frame/prophecy behavior, all-values iteration, hash/equality coherence, absence completeness, termination, and an integrated actual-Map proof remain outside this accepted claim. The next local work is to export `get2`'s functional postcondition through public `get`, prove `get_mut`'s value/frame/readiness relation and `get_all`'s map identity, then continue the existing actual `append_value`, insertion-prefix, cyclic-shift, and reserve/grow/rebuild work.
