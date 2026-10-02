`shift_by_u128_euclid` is an actual `n >> k` body. The contract states both
`result@ == n@.div_euclid(k@.pow2())` and `result == n >> k`; `requires` keeps
the Rust shift amount in range. The supplied fresh run passed 1/1 VC with
Z3 4.15.3 under the BV128-only overlay, after the derived LSR lemma check.
