# Vec requested-capacity B1 probe

This probe imports the exact `src/ownership_proof/vec_capacity.rs` helper and
the existing B1 pointer, raw-Vec, and owned-region modules. The helper creates
`Vec::with_capacity(requested)` and immediately passes that same Vec to
`bound_ptr::detach_bound_vec`. Its result ties the measured B1 capacity to the
request, while retaining the matching recovery token and full `[0, capacity)`
region. Since the input Vec is empty, every slot is `Unknown` (`Some(None)`).

The sole new trusted fact is the standard allocator guarantee that the
detached Vec capacity is at least the requested capacity. The remaining facts
repeat the existing B1 detach contract. The helper does not assert BytesMut
metadata, shared ownership, reference-count, or byte-access properties.

The Creusot positive caller builds the ownership-related fields used by an
offset-zero BytesMut constructor. The negative caller asks those fields to
cover a larger request than the Vec constructor received. Native tests check
request zero and several nonzero requests, then free each allocation using the
matching B1 recovery and region capabilities.

The proof was run with a temporary copy of `scripts/verify-bytes.sh` that adds
this new probe name to its dispatch list. Native checks run with:

```sh
cargo test --locked --offline --manifest-path verification/probes/vec-capacity-guarantee/Cargo.toml
```
