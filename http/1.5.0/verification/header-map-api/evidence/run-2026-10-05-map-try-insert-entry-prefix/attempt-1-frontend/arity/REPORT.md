# `try_insert_entry` direct-root audit

The fresh attempt-1 `try_insert_entry.coma` was printed with `why3 prove -D why3`, without selecting a prover or applying preprocessing/splitting. The single printed theory contains four direct goals: one actual body root (`vc_try_insert_entry_T`) and three imported literal-true support roots (`vc_len_Bucket_T`, `vc_new`, and `vc_push_Bucket_T`). The support roots are not body proofs of those callees.

The output file is the complete Why3 stdout; root headers and classifications are recorded in `roots.json`. No solver has run for this attempt.
