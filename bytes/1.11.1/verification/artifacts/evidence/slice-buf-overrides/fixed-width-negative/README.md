# Exact slice Buf override bodies

This gate mechanically extracts the actual concrete bodies in `impl Buf for
&[u8]` from `src/buf/buf_impl.rs`. It proves `try_get_u8` and both big- and
little-endian `try_get_{u,i}{16,32,64,128}`: 17 checked integer readers. Each
success specifies the exact decoded value and consumed suffix; each failure
specifies requested width, original available length, and unchanged input.

The concrete receiver adapter changes `&mut self` to `input: &mut &[u8]` and
renames the `self` identifier to `input`. The build checks the inverse transform
against the original entire method text. It also extracts the real TryGetError
field declaration. Source fragments, complete source snapshots, generated
adapters, VC files and hashes are retained in evidence. There is no local fake
Buf trait, trait contract or alternate reader implementation.

The fixed-width positive gate proves 55 files with zero unproved leaves; two
native matrices cover short/exact/extra input, unsigned/signed boundary values,
byte order and error metadata. A wrong_available feature deliberately asserts
zero available bytes for a one-byte failed u16 read and must fail one obligation.
The earlier narrow 13-file positive and 14-file one-leaf negative remain frozen
as separate evidence configurations.

This is a concrete method-body proof, not Buf trait dispatch/refinement or a
full bytes crate proof. It excludes native-endian defaults, variable-width
methods, getters' panic paths, floating-point reads and Bytes/BytesMut ownership.
No production source, compiler/std contract, or trusted function is changed.

From the bytes crate directory:

```sh
./scripts/verify-bytes.sh slice-buf-overrides
./scripts/verify-bytes.sh slice-buf-overrides --features wrong_available
cargo test --offline --locked --manifest-path verification/probes/slice-buf-overrides/Cargo.toml --tests
```
