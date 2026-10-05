# Attempt 6 independent direct-task arity audit

Each of the 15 fresh archived COMAs was printed independently with configured Why3 `-D why3`; no prover was selected and no VC-splitting transformation was applied. Every COMA has a unique output directory. The full goal text, names, and SHA-256 values are recorded in `independent-direct-root-audit.json`.

- 15 selected COMAs; all printers returned exit 0.
- 37 direct roots: 10 actual function-body goals, 5 trait-refinement goals, and 22 literal-true imported support stubs.
- This is a task inventory only; proof results are reconciled separately in `full15-proof-summary.json`.

| # | COMA | Body roots | Refinements | Literal-true imported support | Direct roots |
|---:|---|---:|---:|---:|---:|
| 1 | `header/map/impl_HeaderMap_T/get.coma` | 1 | 0 | 1 | 2 |
| 2 | `header/map/impl_HeaderMap_T/get2.coma` | 1 | 0 | 3 | 4 |
| 3 | `header/map/impl_HeaderMap_T/get_mut.coma` | 1 | 0 | 3 | 4 |
| 4 | `header/map/impl_HeaderMap_T/get_all.coma` | 1 | 0 | 3 | 4 |
| 5 | `header/map/impl_HeaderMap_T/contains_key.coma` | 1 | 0 | 2 | 3 |
| 6 | `header/map/as_header_name/impl_Sealed_for_HeaderName/find.coma` | 1 | 0 | 1 | 2 |
| 7 | `header/map/as_header_name/impl_Sealed_for_HeaderName/find__refines.coma` | 0 | 1 | 0 | 1 |
| 8 | `header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find.coma` | 1 | 0 | 1 | 2 |
| 9 | `header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find__refines.coma` | 0 | 1 | 0 | 1 |
| 10 | `header/map/as_header_name/impl_Sealed_for_ref_str/find.coma` | 1 | 0 | 5 | 6 |
| 11 | `header/map/as_header_name/impl_Sealed_for_ref_str/find__refines.coma` | 0 | 1 | 0 | 1 |
| 12 | `header/map/as_header_name/impl_Sealed_for_String/find.coma` | 1 | 0 | 2 | 3 |
| 13 | `header/map/as_header_name/impl_Sealed_for_String/find__refines.coma` | 0 | 1 | 0 | 1 |
| 14 | `header/map/as_header_name/impl_Sealed_for_ref_String/find.coma` | 1 | 0 | 1 | 2 |
| 15 | `header/map/as_header_name/impl_Sealed_for_ref_String/find__refines.coma` | 0 | 1 | 0 | 1 |

The `&str` find task now contains the definition `eq_model_ReprDeepModel_Seq_u8(self, rhs) = header_name_matches_hdr_name(self, rhs)`. That visibility equation was absent from attempt 5 and is present in both the fresh COMA and its independently printed complete Why3 task.
