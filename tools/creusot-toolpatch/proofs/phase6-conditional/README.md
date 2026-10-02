# Conditional `u128` formatter evidence

This is an isolated candidate patch, not a runtime source integration. Its
proof source was developed from Phase 5 commit `1c799dd`; the preserved diff
was rebased against current main `999f8c3` and checked with
`git apply --unidiff-zero --check`. Its complete patch is preserved as
`PHASE6_U128_CONDITIONAL.patch`.

The actual shared `u128::fmt` body and its full postconditions are present in
the candidate. The proof has no new concrete precondition: it calls the
body-proved `decimal_values_len_u128(self)` helper to establish the 39-byte
capacity. It proves the canonical output suffix, initialized bytes, returned
offset, and prefix frame, and includes the trait refinement goal.

## Conditional proof results

Using Creusot 0.11.0-dev and the pinned Why3/solver environment from
`../../records/versions.md`:

- `fmt` and `fmt__refines`: completed, 146 and 1 goals.
- `decimal_values_len_u128`: completed, 1 goal.
- initialized slot-range helper: completed, 4 split goals.
- Negative control: a false assertion in the reachable nonzero-quotient
  branch failed at that assertion (`112/113` for `fmt`).

The formatter proof is conditional on the Phase 4 arithmetic contracts,
including the still-unproved `u128_ext::mulhi` body. A green formatter VC does
not prove that callee implementation. This candidate is not a full crate proof,
not the raw `Buffer::format`/string-conversion path, and not an end-to-end
formatter result.

The `evidence/` directory preserves the proof logs and JSON results. To replay
the preserved diff, create a detached worktree at `999f8c3`, apply it with
`git apply --unidiff-zero`, then prove the candidate targets from
`itoa/1.0.18`:

```sh
cargo creusot --simple-triggers=false prove --no-cache \
  verif/itoa_rlib/runtime/impl_Unsigned_for_u128/fmt.coma
cargo creusot --simple-triggers=false prove --no-cache \
  verif/itoa_rlib/runtime/impl_Unsigned_for_u128/fmt__refines.coma
cargo creusot --simple-triggers=false prove --no-cache \
  verif/itoa_rlib/verification/decimal_values_len_u128.coma
```

Use the local pinned compiler and Why3 configuration described in
`../../records/versions.md`; run Why3 outside the filesystem sandbox because it
uses Unix sockets. No Phase 4 or Phase 6 candidate code was applied to the
current-main validation run recorded in the parent runtime verification notes.
