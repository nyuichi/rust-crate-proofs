# Independent try_insert_entry prefix audit

Verdict: **the fresh emitted contract has a sound conditional shape and the new prefix clause is present**. This is a source/COMA/direct-arity audit only. No prover was invoked, no source or COMA was changed, and no proof JSON is accepted by this report.

`audit.py` verifies all 177 frozen source hashes and the target COMA hash, independently prints both a full unfiltered task stream and individual task files, and reconciles their root sets. Commands, complete stdout/stderr, individual `.why` files, and hashes are retained in `task-audit.json`.

The frozen source identity is Map `a0543453a9ddcac366e871913dc4b102706e96096a12bcf1c38e3109e1d02191`, Name `271a078034f379ff60a2777ae1b84ac08fdcf3c080f94eeaa6d2b9f8c2921979`, and source-freeze `6d687a13574b5dbcfc96da3e3c3b0acb32f10164c950d472e4f7c99325480e1b`. The fresh COMA is `83b7989409c5a9d8d20c1059e6d281c52cf4c4d54250004bb853d140da1cd976`.

## Independently printed direct roots

| Root | Actual complete formula/classification |
|---|---|
| `vc_try_insert_entry_T` | Nontrivial actual checked function-body VC |
| `vc_len_Bucket_T` | Imported support: `[@coma:solid] true` |
| `vc_push_Bucket_T` | Imported support: `[@coma:solid] true` |
| `vc_new` | Imported support: `true` |

Thus there are exactly four direct roots, one actual body and three literal-true imported support roots. The support roots do not establish Vec::push, Vec::len, or the error constructor implementations. No split or nested transformation was applied.

## Contract and non-vacuity review

The actual function is emitted with a checked `(! bb0 ...)` body. It tests the original entry count, returns Err without writing the map on overflow, or pushes exactly the new bucket and returns Ok. The only caller preconditions are ordinary map/key/value type invariants; no map-wide insertion invariant, false predicate, hash-coherence assumption, or new trusted HTTP axiom was added.

The existing result postcondition remains intact: Ok requires the old length below MAX_SIZE, increases it by one, and gives all four new final-bucket fields at the old length. Err requires the old length at least MAX_SIZE and preserves the length. Independent outer clauses preserve mask, the index-table sequence, the extra-value sequence, and Danger.

The new `ensures #5` is an unconditional, separately scoped universal over `0 <= i < old_entry_count`. It preserves each old bucket's hash, HeaderName deep model, actual generic `T` value, and link variant/offsets. Its `match` and quantifier scopes are correct in the emitted COMA. No `T: DeepModel` or PartialEq constraint, or equality of unrelated generic model types, is introduced.

On success, the genuine Vec::push summary states `post_entries = snoc(pre_entries, new_bucket)`, which supplies both prefix preservation and the appended last element. On error, the map is resolved without mutation, which supplies the same prefix. These interface facts support the contract design but are not a replacement for proving the fresh actual-body VC.

The empty-prefix case is valid when the original map has zero entries: there are no old entries to preserve, and the separate success clauses still constrain the new element and length. The Err branch cannot exploit an empty quantified range, because it requires an original length of at least MAX_SIZE = 32768. Ordinary valid-entry sequences of length zero, one, or MAX_SIZE supply structural witnesses for the success/nonempty-prefix/error cases; readiness and uniqueness are not required by this local helper.

The explicit prefix preserves old HeaderName **models**, not identity of their complete runtime representation. On Err, equal lengths make this prefix cover all old/final entry positions, but the current contract still does not state whole-runtime-key equality, complete entries-sequence equality, or whole-map equality. Those stronger facts are true of the current non-writing error body and can be exported later if a caller needs them. Documentation should retain the current model-level scope.

This helper does not update the index table for the newly added bucket, so no slot coverage, global readiness preservation, insertion reachability, key uniqueness, Robin Hood order, or hash/equality coherence follows. Its eventual successful proof would be a modular local mutation result, not closure of the insertion callers or an integrated HeaderMap proof.
