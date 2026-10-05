# Standard slice-to-array conversion probe

This isolated probe checks the actual caller shape used by httparse's fixed
array lookahead: `slice.get(..N)?.try_into().ok()`. `array8_prefix` and
`array4_prefix` specify that a returned array is exactly the corresponding
input prefix, and that `None` occurs only when the input is shorter than the
requested length.

## Translation status

On 2026-10-05, from this directory and after sourcing
`/workspace/proof-tools/activate.sh`,

```sh
CARGO_NET_OFFLINE=true cargo creusot
```

translated both actual caller bodies with no warnings. The fresh CoMa target is
`verif/httparse_memory_array_conversion_probe_rlib/array8_prefix.coma` and
`array4_prefix.coma`. The standard contract source hash is
`5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348`.

Both caller targets passed through `../../../run-proof.bash why3find prove
--no-cache -s -j 1` with the checked profile (`z3@4.15.3`, one prover,
1000 MiB): `array8_prefix` 9/9 VCs and `array4_prefix` 9/9 VCs. These 18 caller
VCs are included in the adjacent `Bytes` manifest with 141 unique VCs across
31 target files in `memory-pointer/REPORT.md`. They prove the actual
`.get(..N)?.try_into().ok()`
caller shapes against the standard conversion contract; they do not prove the
standard library implementation itself or arbitrary generic `TryFrom` values.

The contract lives in
`../../../../../creusot-libs/creusot-std/src/std/convert.rs` and applies to the
standard `TryFrom<&[T]> for [T; N]` implementation for `T: Copy`:

```text
Ok(array) => input.len() == N && forall i in 0..N, array[i] == input[i]
Err(_)   => input.len() != N
```

It adds no input precondition beyond the standard `T: Copy` bound. For `u8`,
the contract proves an exact same-index copy and cannot change byte order. The
contract is an explicit trusted standard-library semantic boundary; this probe
proves the caller against it, not the Rust core implementation itself.

## Audited Rust source

The source used by the active proof toolchain is from rustc
`1.95.0-nightly (6a979b3e32522049d0acb4a47f7ae44b7c8abfd5)`, file
`$(rustc --print sysroot)/lib/rustlib/src/rust/library/core/src/array/mod.rs`
with SHA-256
`67c051d28fd7a68b7ea49088918a5076329483a95c6d27200078963fcb1c374b`.
Lines 249-260 implement `[T; N]: TryFrom<&[T]>` for `T: Copy` by delegating to
`<&Self>::try_from(slice).copied()`. Lines 302-310 implement the reference
conversion with `slice.as_array().ok_or(TryFromSliceError(()))`; the adjacent
docs state conversion succeeds iff `slice.len() == N`. This establishes both
length behavior and the same-index copy without endian conversion.

The general public `Bytes::peek_n<U: TryFrom<&[u8]>>` remains generic and has
only a bounds fact (`Some(_)` implies `n <= remaining_len`); this conversion
contract does not claim semantics for arbitrary user-defined `U`. The parser
fast paths use private typed adapters so their exact standard-array behavior
can be verified without changing the generic API or adding caller
preconditions.
