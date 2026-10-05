# Exact slice Buf method bodies

The complete gate proves 40 actual concrete method bodies from `impl Buf for
&[u8]`: 19 checked integer readers, their 19 normal-return getters, remaining
and chunk. Fixed-width readers include u8 and both endian orders of signed and
unsigned 16/32/64/128-bit integers. Variable-width unsigned readers use 0..=8
bytes. Successful reads specify the exact value and remaining suffix; failures
specify requested width, original available length and unchanged input.

The build mechanically extracts actual source bodies and the TryGetError field
declaration. Receiver substitution and getter call rebinding are inverse
checked against each original entire method. Observer receivers retain their
concrete slice/reference lifetimes. There is no local substitute Buf trait or
alternate reader algorithm. Full source, exact fragments, generated adapters,
VCs, logs and hashes are retained in evidence.

19 ordinary slice getters now use an equivalent match instead of
Result::unwrap_or_else, whose standard Creusot contract is absent. The same
source executes in native and proof builds. Getter contracts require sufficient
input; variable readers additionally require width<=8. The actual panic helper
bodies are extracted with false preconditions so verified callers cannot enter
those paths. This is not a proof of panic, formatting or unwind behavior. None
of these functions are trusted. No compiler/std contract is added.

The complete gate passes 82 files with zero unproved leaves and three native
matrices. Ordinary test_bytes118 and no-default-features pass after the native
match change. The 83-file wrong_available feature rejects one intended leaf
(1/2), claiming zero available bytes for a one-byte failed u16 read. Earlier
narrow13/14, fixed-width55/56 and match-getter80/81 configurations remain frozen
as separate evidence, not summed proof counts.

This is concrete body evidence, not Buf trait dispatch/refinement or full-crate
verification. Native-endian defaults, signed variable defaults, floating-point
reads, panic/unwind paths and Bytes/BytesMut ownership remain outside this gate.

From the bytes crate directory:

```sh
./scripts/verify-bytes.sh slice-buf-overrides
./scripts/verify-bytes.sh slice-buf-overrides --features wrong_available
cargo test --offline --locked --manifest-path verification/probes/slice-buf-overrides/Cargo.toml --tests
```
