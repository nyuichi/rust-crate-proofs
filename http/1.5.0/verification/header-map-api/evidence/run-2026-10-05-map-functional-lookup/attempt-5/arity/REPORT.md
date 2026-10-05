# Attempt 5 direct-task arity audit

Each fresh frozen COMA was printed independently into a unique directory using `why3 prove -D why3`; no solver was selected and no preprocessing was applied. This audit records direct task names only. It does not prove any obligation.

- Selected COMAs: 15
- Printed direct tasks: 37
- Own body roots: 10
- Own refinement roots: 5
- Imported literal-true support stubs: 22

Each task’s exact goal name, category, output path, and task SHA256 is in `independent-direct-root-audit.json`. Imported stubs are caller summaries only; they do not establish the callee body. Fresh `proof.json` root matching is pending the bounded solver stage.

| # | COMA | Own roots | Support stubs | Total direct tasks |
|---:|---|---:|---:|---:|
| 1 | `header/map/as_header_name/impl_Sealed_for_HeaderName/find.coma` | 1 | 1 | 2 |
| 2 | `header/map/as_header_name/impl_Sealed_for_HeaderName/find__refines.coma` | 1 | 0 | 1 |
| 3 | `header/map/as_header_name/impl_Sealed_for_String/find.coma` | 1 | 2 | 3 |
| 4 | `header/map/as_header_name/impl_Sealed_for_String/find__refines.coma` | 1 | 0 | 1 |
| 5 | `header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find.coma` | 1 | 1 | 2 |
| 6 | `header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find__refines.coma` | 1 | 0 | 1 |
| 7 | `header/map/as_header_name/impl_Sealed_for_ref_String/find.coma` | 1 | 1 | 2 |
| 8 | `header/map/as_header_name/impl_Sealed_for_ref_String/find__refines.coma` | 1 | 0 | 1 |
| 9 | `header/map/as_header_name/impl_Sealed_for_ref_str/find.coma` | 1 | 5 | 6 |
| 10 | `header/map/as_header_name/impl_Sealed_for_ref_str/find__refines.coma` | 1 | 0 | 1 |
| 11 | `header/map/impl_HeaderMap_T/contains_key.coma` | 1 | 2 | 3 |
| 12 | `header/map/impl_HeaderMap_T/get.coma` | 1 | 1 | 2 |
| 13 | `header/map/impl_HeaderMap_T/get2.coma` | 1 | 3 | 4 |
| 14 | `header/map/impl_HeaderMap_T/get_all.coma` | 1 | 3 | 4 |
| 15 | `header/map/impl_HeaderMap_T/get_mut.coma` | 1 | 3 | 4 |
