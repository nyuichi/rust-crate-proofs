# Name comparator/parser postproof audit

PASS for the frozen Name source `fa9742ee1a16f3967fcf00338c5e6a36cfa98a41c922faaa3738aff004d6ca7d`: 87 selected COMAs, 323 initial tasks, and 323 successful terminal prover leaves. The prover distribution is 306 Z3 and 17 CVC5; there are no null leaves. All 323 saved initial task/context byte streams match independent ordered Why3 API output from the original COMAs, including unsuffixed child 0.

One unary nested transformation must be retained: `impl_HeaderName/from_bytes.coma`, `vc_from_bytes/21`, splits into one terminal task. Its independently regenerated parent is byte-identical to the saved direct task. The nested task has 277,413 bytes and SHA-256 `4f605890d858903ed254b573d78411da6fade1048802d77fb7f07448fe73b318`. Initial and terminal counts coincide because this split is unary; the proof tree has **one nested node**, not zero. The task proves the lowercase-buffer loop invariant after extending the buffer by one mapped byte.

| Group | Selected COMAs | Own terminal leaves |
| --- | ---: | ---: |
| `bytes_equal` | 1 | 15 |
| Numeric comparator wrappers | 81 | 81 |
| Absence helper | 1 | 81 |
| `StandardHeader::from_bytes` | 1 | 83 |
| `HdrName::from_bytes` | 1 | 13 |
| `HeaderName::from_bytes` | 1 | 48 |
| Selected comparator consumer | 1 | 2 |

All 181 source-ledger entries, 306 emitted COMAs, and 323 saved task hashes match their ledgers. All seven raw proof logs and command hashes match the result ledger. Each wrapper compares real bytes through the proved `bytes_equal` body. Its numeric sequence postcondition uses the same literals as its real byte array. All 81 numeric arrays match their byte/string spellings, ranks 0–80 and wrapper identifiers are unique, and the parser routes all 81 cases through those checked wrappers. No trusted HTTP setter or axiom was added.

The 193 zero-child callee/support-marker occurrences are excluded from the 323 leaves. The wrapper and standard-parser local comparator dependencies are proved within this batch. Higher-level parser bodies still use separately scoped local contracts such as `parse_hdr`, existing Creusot contracts, and the user-accepted Bytes boundary. This audit does not claim all 306 emitted COMAs were proved.

`report.json` contains per-input proof-tree counts, root names, ordered task hashes, compressed complete contexts, full unnormalized Why3 stdout hashes, and the exact nested replay. `audit.py` uses the original COMA through the existing Why3 API replay; it never reparses printed `.why` files. No solver or frontend was invoked, and no source file or prior preproof artifact was changed.
The excluded marker classification is exact by occurrence (the same callee in two selected callers counts twice):

| Boundary | Occurrences | Roots |
| --- | ---: | --- |
| `bytes_equal` body proved in this batch | 81 | `vc_bytes_equal` |
| Numeric comparator bodies proved in this batch | 83 | 81 parser calls plus 2 consumer calls |
| Other local HTTP contracts, separately scoped | 7 | `parse_hdr` ×2; ByteStr `from_utf8_unchecked`; Custom/StandardHeader `from_HeaderName` ×2; `header_chars_byte_value`; InvalidHeaderName `new` |
| Bytes model contracts, user-accepted boundary | 4 | `copy_from_slice`, `extend_from_slice`, `freeze`, `with_capacity` |
| Existing std/Creusot contracts | 11 | slice `len` ×2, Result `branch` ×2, identity `From` ×2, `from_residual` ×2, `Into` ×2, slice `from_ref` |
| Translator enum elimination | 6 | `Break` ×2, `Continue` ×2, `Custom`, `Standard` |
| Caller-supplied generic callback contract | 1 | `call_once_F` |

The last five rows account for all 29 non-comparator markers. The local HTTP rows are imported body contracts, not new trusted HTTP axioms and not additional successful body proofs in this batch. The callback remains parametric; its precondition is part of the caller obligation. Zero-child markers are never counted among the 323 successful terminal leaves.
