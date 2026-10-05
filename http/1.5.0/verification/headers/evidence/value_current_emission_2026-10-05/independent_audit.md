# Independent audit: HeaderValue error formatters (2026-10-05)

Reviewer: Astra, read-only audit; no source, COMA, proof JSON, or solver-run mutation.

## Result

PASS: eight current-source targets, 33 own terminal leaves. Astra independently ran solver-free Why3 `split_vc` extraction on each archived focused COMA. All 33 fresh tasks and contexts matched the archived `.why` tasks byte-for-byte, including the unsuffixed child-zero file. The proof JSON trees reconcile to 33 successful leaves; there are no null or nested children. All terminal proof results are Z3 4.15.3.

| Target | Own leaves | Independent stdout bytes | SHA-256 |
|---|---:|---:|---|
| `Debug for InvalidHeaderValue` body | 7 | 135603 | `9c1a013f93097b228e802ff3e97fd1771c989f3a98f344937a2723057dada18c` |
| `Display for InvalidHeaderValue` body | 4 | 245492 | `1501dd93e8bf314c0e7bd7dbe4441521cafc37de3385a77827f8d775e9525e93` |
| `Debug for ToStrError` body | 10 | 198786 | `5c8698b1b2b18a6ddde41604d94d5ac6b7de9062ba0edb67d78d488abec69284` |
| `Display for ToStrError` body | 4 | 245440 | `696de5c39fe6157c24957481d910b5f5aa6ea25f5b526d76c03072e785c7360c` |
| `Debug for InvalidHeaderValue` refinement | 2 | 36390 | `68936d28f19d7767d69fadae3753e6772762cf7eabb8e61336d7f00f95275f06` |
| `Display for InvalidHeaderValue` refinement | 2 | 36390 | `68936d28f19d7767d69fadae3753e6772762cf7eabb8e61336d7f00f95275f06` |
| `Debug for ToStrError` refinement | 2 | 36342 | `b3db30f95d35cf308c7c5c46c94302442611cdca930cb44fff80ebe9e3a3e510` |
| `Display for ToStrError` refinement | 2 | 36342 | `b3db30f95d35cf308c7c5c46c94302442611cdca930cb44fff80ebe9e3a3e510` |

The source archive has 181/181 valid hashes; the fresh emission has 333/333 valid COMA hashes; the focused target ledger has 13/13 valid COMA hashes. Eight proof COMAs match their corresponding full-emission COMAs. The native Value source SHA is `197fe1071f054c5f018fe1a8cc21266d679164a77088b9459eeb2ee3dd34330e`; its recorded focused native run passes 38/38 tests. The frozen Name input SHA is `966107ba5e8df15aa82c4d34220ab3a7b8e4e7da7b4dfc4a050cff1232dcb064`.

## Boundary

The proved bodies close their recorded `formatter_extends` contracts; the four refinements close the corresponding current trait contracts. Standard-library formatter behavior remains an external contract boundary: `Formatter::write_str` prefix/partial-write behavior; `debug_struct`'s mutable borrow behavior; `DebugStruct::field` and `finish` invariant behavior; Formatter View/DeepModel, dynamic Debug unit behavior, and borrow resolution. These results do not prove the standard-library implementations or every concrete output string. The indexed `HeaderValue` Debug target and writer/transitivity helpers are outside this 33-leaf audit batch. Separate later batches record 39 helper tasks and 57 Debug body/refinement tasks proved. These results still prove append preservation, not exact rendered text.

The independently generated temporary Why3 output was removed after comparison. The archived task contexts and the independent printer stdout ledger are retained under `arity/` in this evidence directory; see `arity/audit.json`, `arity/proof-batch-task-hashes.sha256`, and `arity/proof-batch-printer-stdout.sha256`.
