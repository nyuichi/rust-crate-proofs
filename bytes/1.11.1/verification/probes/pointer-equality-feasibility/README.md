# Pointer equality feasibility probe

This pinned Creusot 0.13 diagnostic isolates the logical distinction between
pointer identity and numeric address equality. It does not establish facts
about physical addresses, memory validity, provenance of concrete allocations,
or dereference permissions.

- The default positive `native_eq_implies_logical_identity` returns native
  `left == right` and contracts that `result` implies the same logical pointer
  equality. Its proof demonstrates the current MIR translation's model of raw
  pointer `Eq`; it is not evidence that Rust pointer equality distinguishes
  provenance.
- The default positive `addr_eq_implies_numeric_address` contracts only that
  successful `core::ptr::addr_eq` implies equal `addr_logic()` values.
- Feature `wrong_addr_eq_identity` enables
  `wrong_addr_eq_implies_logical_identity`. Its postcondition intentionally
  asks numeric address equality to imply full logical pointer equality and
  should remain unproved.

The vanilla std pointer model defines `PtrDeepModel` with a logical address and
opaque runtime metadata. Its `addr_eq` external specification is exactly
`result == (p.addr_logic() == q.addr_logic())`. The probe adds no trusted spec,
allocation, permission, or bridge axiom. A successful/failed VC here is solely
evidence about the compiler model and those std contracts.

Observed under the repository wrapper: the default configuration proved both
positive files; `wrong_addr_eq_identity` left
`Coma.vc_wrong_addr_eq_implies_logical_identity` unproved, with no translation
or setup failure. This is a failed proof obligation, not a generated concrete
counterexample. Logs are in `logs/`. The two positive Coma and proof JSON files
were retained under
`verification/artifacts/evidence/pointer-equality-feasibility/positive/`.
