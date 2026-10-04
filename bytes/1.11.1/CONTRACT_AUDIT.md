# Contract audit

## Fixed runtime arithmetic

The original trusted subtraction contract required `a-b` whenever `b` fit in
usize. At `a=0,b=1` it demanded a negative usize value. It is replaced by
`if b <= a { a-b } else { 0 }`, which is `max(a-b,0)` over mathematical integers.
The `min_u64_usize` result contract is mathematically `min(a,b)`.

Both original runtime bodies were moved without algorithm changes to
`src/arithmetic.rs`; neither is trusted. The helper probe includes that exact file
with `#[path]`, and includes the same `src/std_specs.rs` as the runtime crate.
The helper bodies, general-argument callers and boundary examples are proved on
vanilla Creusot 0.13. This is an isolated body result, not runtime crate integration.

`STD-CONVERT-01` specifies `usize::try_from(u64)`: Ok preserves the integer within
usize range; Err means outside that range. This is a standard-library external
contract, not a bytes ownership assumption. Astra reviewed it. The fixed Rust
source `library/core/src/convert/num.rs` uses unbounded conversion on 64-bit and
upper-bound checks on 16/32-bit. Only the 64-bit proof has been executed.

The wrong-postcondition and reachable-`assert(false)` cases translate successfully
and fail on their intended VCs. Syntax/type/setup failures are not counted as
negative proof successes.

## Runtime/model audit

The new runtime entry no longer substitutes `verification.rs` under `cfg(creusot)`.
The old file is retained as historical, model-only material and is not exported.
The current runtime source has no bytes helper `#[trusted]`. The standard primitive
assumptions used by isolated probes are explicitly listed in `TRUSTED_BASE.md`.

The previous PROVENANCE claim that the old arithmetic contract was reviewed is
superseded by this audit. Its past successful proof logs are not soundness evidence
for the new runtime verification goal.
