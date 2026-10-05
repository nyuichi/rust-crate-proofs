# Current-source Method API audit

The current source task stream was reconciled against the accepted Method proof snapshot: every one of the 103 emitted targets has identical full untransformed Why3 stdout, a matching recorded proof JSON, and a complete successful proof tree. The 877 reusable leaves comprise 737 own-goal leaves and 140 callee-contract leaves. Root and nested `split_vc` structure was independently checked without a prover (28 root nodes / 682 children and 7 selected nested nodes / 15 children). See [task-stream-reconciliation.json](task-stream-reconciliation.json), [manifest.json](manifest.json), and [independent-astra-nested/report.json](independent-astra-nested/report.json).

## Runtime API bodies and contracts

| Surface | Verified behavior |
| --- | --- |
| `Method::from_bytes`, `TryFrom<&[u8]>`, `TryFrom<&str>`, `FromStr` | Success iff nonempty input consists only of HTTP `tchar` bytes; successful values preserve exact input bytes and satisfy the canonical representation invariant. |
| Built-in recognition and extension storage | All ten known-method classifiers/materializers, inline and allocated extension constructors, the validation/copy helper, UTF-8 bridge, and canonical text model are proved. |
| `is_safe`, `is_idempotent`, `as_str`, `AsRef<str>` | Exact model classifications and text refinements. |
| Equality and ordering | `Method`, `Inner`, extensions, references, `str`, and `&str` compare against the byte-sequence model; `Ord`/`PartialOrd` match lexicographic byte order. |
| Clone, `From<&Method>`, Default | Preserve the byte model and invariant; default is GET. |
| Debug and Display | Successful formatting emits the exact method text; formatter errors preserve the append-only formatter relation. `InvalidMethod` formatters preserve that relation. |
| Hash | Body preserves the valid hasher invariant. The proof makes no digest-value or general coherence claim; the recorded-hasher regression checks callback protocol compatibility. |
| Crate-private Request builder constructors | Nine fixed-method constructors have exact model and invariant postconditions and are proved. |

All emitted executable Method/ASCII body targets are in the accepted 103-target set. The ten public method constants are direct enum-valued constant initializers and do not emit separate body VCs; the model, classifier, text, and conversion behavior they feed is covered. The `QUERY` constant has no separate crate-private builder constructor (the nine Request shortcuts are intentionally the other methods). This is a constant-initializer coverage boundary, not an unproved runtime function body.

The proof is a leaf harness for `method.rs` and `ascii.rs`, not an integrated proof of every HTTP module. Allocation, standard-library string/sequence operations, formatter contracts, and hasher contracts remain explicit library boundaries.
