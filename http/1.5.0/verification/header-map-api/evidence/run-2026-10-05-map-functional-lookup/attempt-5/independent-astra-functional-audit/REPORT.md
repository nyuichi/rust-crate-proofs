# Independent frozen functional Map audit

Verdict: accept the successful targets as **partial, conditional modular evidence**. Do not publish the full 15-target batch as complete, public `get`/`get_mut` as functionally verified, or string-key integration as closed.

This audit ran no prover, changed no implementation or contract source, and read the attempt-5 source snapshot. `audit.py` independently verifies all 177 archived source hashes, all 15 COMA hashes, and each fresh proof JSON root set against a newly printed complete task stream. The streams and proof JSON are archived here; hashes and commands are in `report.json`.

The source-freeze SHA-256 is `72f462bb6bd7fa732723db175fc231de0a4735d9b169be55de477ca962f1d4c6`. Frozen Map is `76322738120be3b4a616cd27b4e4fa48fa5b76ba3a5f8db8cfed9869da82a3d8`; frozen Name is `966107ba5e8df15aa82c4d34220ab3a7b8e4e7da7b4dfc4a050cff1232dcb064`.

## Independently reconciled result

There are 37 direct roots: 10 actual function-body roots, 5 trait-refinement roots, and 22 imported support roots whose full goal formula is literally `[@coma:solid] true`. Of the actual body roots, nine are complete and the `&str` implementation is incomplete. The 5 refinements and 22 support roots are complete. Thus 36 of 37 direct roots and 14 of 15 selected COMAs are complete. After recorded splitting there are 40 terminal leaves, of which 39 are successful and one is null. Support stubs do not prove the corresponding implementation bodies.

The incomplete root is `vc_find_ref_str` in `header/map/as_header_name/impl_Sealed_for_ref_str/find.coma`, SHA-256 `50b71da7ff99410b110b56da042f9ce1a87579a125033c77f345c709e5611ab9`. Independent OCaml API replay parses the original COMA, selects this original root, applies `split_vc` with arity 3, selects zero-based child 2, and applies `split_vc` only to that child with arity 2. The null leaf is path `[2,1]`. Successful siblings were not retransformed; no printed Why3 file was reparsed. API root bytes equal independent selected CLI task bytes; the nested parent bytes equal the selected initial child bytes.

## Exact missing bridge in the incomplete task

Frozen Name line 1636 declares the implementation of `PartialEqModel<ReprDeepModel<(Seq<u8>, bool)>> for ReprDeepModel<Seq<u8>>` with `#[logic(open(self))]`. Its body delegates exactly to the public, open `header_name_matches_hdr_name` predicate. Map cannot see that delegation.

Consequently frozen `find.coma` line 422 and the independently printed complete task contain only an **uninterpreted declaration** of `eq_model_ReprDeepModel_Seq_u8(NameModel, ParsedModel)`, with no equation linking it to `header_name_matches_hdr_name`. The callback postcondition, obtained from `HeaderMap::find`, gives the former predicate. The outer `&str` postcondition requires the latter predicate inside an existential parsed-model relation. In the isolated null task, all byte-view, parsed-result, and callback relations are present; the equality-predicate bridge is absent.

The failure is therefore explained by missing logical visibility, not by generic type collapse, tuple encoding, reference deep-model mismatches, or merely an insufficient prover budget. A narrow repair is to change this delegating Name method to `#[logic(open)]`, or to prove and expose a Name-local bridge lemma and use it. This introduces no new assumption: the delegation is already the method's actual pure body. Re-emission must confirm the equation is available, and the changed task must then be proved. This audit does not claim that repair has been performed or verified.

## Actual contract strength of the frozen batch

| Target | What its successful body proof establishes |
|---|---|
| `HeaderMap::get2` | If the result is `Some(value)`, an in-range stored entry has that exact generic `T` value and its key satisfies `K::matches_header`. `None` is unconstrained. |
| Public `HeaderMap::get` | Call and type-invariant obligations under the readiness premise. **There is no explicit functional `ensures` on this frozen public method**, even though it calls `get2`. |
| `HeaderMap::get_mut` | Borrow/index and type-invariant obligations under readiness. **There is no explicit functional `ensures`**: no value correspondence, prophecy relation, frame, or explicit readiness-preservation postcondition is exported. |
| `HeaderMap::get_all` | If the exposed index observer returns `Some`, the index is below the key count and the stored key matches. The contract does not identify the returned view's map with the input map, prove iteration behavior, or prove absence when the index is `None`. |
| `HeaderMap::contains_key` | A true result implies existence of an in-range matching stored key. A false result does not imply absence. |
| `Sealed::find` for `HeaderName` and `&HeaderName` | A returned index is in range and the stored key model equals the query key model, conditionally on the imported core `HeaderMap::find` contract. |
| `Sealed::find` for `String` and `&String` | Correct adapter/body obligations relative to the `&str` source contract. The `String` adapter now uses the genuine `Deref` contract and no longer has the former impossible `String::as_str` precondition. These are conditional caller proofs because the `&str` body is still open. |
| Five `find__refines` roots | Each implementation's declared contract refines the trait contract. This is independent of proving each implementation body. |

The string matching observer is the pure relation `exists parsed_model. hdr_parse_success(text_bytes, parsed_model) && header_name_matches_hdr_name(stored_model, parsed_model)`. It does not call prophetic `inv`, executable lookup, or a trusted HTTP function. This is the explicit meaning of the relation; a separate bridge would be needed to expose a simpler public text/case-normalization law.

## Non-vacuity and boundaries

The selected actual body preconditions use ordinary type invariants and `header_map_find_ready`; no selected external call has an impossible precondition. `header_map_find_ready` is satisfiable for empty entries. A nonempty structural witness is mask 1, two index slots, one stored bucket with hash 0, slot 0 referring to that bucket with hash 0, slot 1 marked empty, empty extra values, and `Danger::Green`, with valid key/value invariants. The ready predicate does not require all stored entries to occur in the table or enforce probe order: it also permits nonempty entries with every table slot empty. Absence completeness cannot follow from it.

The key and value projections are definitions over actual stored fields. Generic `T` remains its own type and value correspondence uses `T` itself; no `T: DeepModel` restriction or axiom equating unrelated model types was added. The concrete query observer implementations remain distinct. Ordinary generated type-invariant/specification axioms are present, but there is no added trusted HTTP axiom or false predicate introduced to close these targets. Unreachable `Option` eliminator branches are not global contradictory preconditions.

The proof remains modular: core lookup, hash helper, and Name parsing bodies are imported through their source contracts. The warning-bearing `RandomState::new`, `BuildHasher::build_hasher`, and `Hasher::finish` paths occur in unselected source helpers; the successful caller proofs do not establish those helper bodies. String-key integration also needs the parser proof chain and the open `&str` functional root. Constructors/mutations establishing readiness, termination, absence completeness, mutable frame/prophecy behavior, and crate-integrated verification are outside this accepted result.
