# Attempt 7 Map getter direct-root audit

Solver-free printing of the 13 archived attempt-7 COMAs completed with `why3 prove -D why3`, without a prover, split transformation, or preprocessing. This establishes direct VC arity only; the proof batch is pending.

- Direct roots: 30
- Actual body roots: 8
- Trait refinements: 5
- Imported literal-true support roots: 17

| Target | Roots | Direct root names |
|---|---:|---|
| `get` | 2 | `vc_get2_T`, `vc_get_T` |
| `get2` | 4 | `vc_elim_Some`, `vc_find_K`, `vc_get2_T`, `vc_index_Vec_Bucket_T_Global` |
| `get_mut` | 4 | `vc_elim_Some`, `vc_find_K`, `vc_get_mut_T`, `vc_index_mut_Vec_Bucket_T_Global` |
| `sealed_header_name` | 2 | `vc_find_HeaderName`, `vc_find_T` |
| `sealed_header_name_refines` | 1 | `refines` |
| `sealed_ref_header_name` | 2 | `vc_find_T`, `vc_find_ref_HeaderName` |
| `sealed_ref_header_name_refines` | 1 | `refines` |
| `sealed_ref_str` | 6 | `vc_as_bytes`, `vc_closure0`, `vc_find_T`, `vc_find_ref_str`, `vc_from_bytes_closure0`, `vc_unwrap_or_Option_tup2_usize_usize` |
| `sealed_ref_str_refines` | 1 | `refines` |
| `sealed_string` | 3 | `vc_deref_String`, `vc_find_String`, `vc_find_ref_str` |
| `sealed_string_refines` | 1 | `refines` |
| `sealed_ref_string` | 2 | `vc_find_String`, `vc_find_ref_String` |
| `sealed_ref_string_refines` | 1 | `refines` |
