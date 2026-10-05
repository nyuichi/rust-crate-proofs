# Exact `BytesMut::with_capacity` probe

`build.rs` copies the legacy `sequential-bytesmut-split` source-extraction
workflow and extracts the exact `BytesMut` declaration, `with_capacity`
method including its contracts, `proof_unique_at_zero_valid` and supporting
predicates, and `proof_release_unique`. The extracted constructor performs the
real typed field initialization from `src/bytes_mut.rs`, including packed
metadata and all four ghost ownership/control fields. Its proof caller checks
the empty, all-Unknown region and consumes the owner through the exact
`owner.proof_release_unique()` path.

The sole new trust is the narrow constructor-plus-B1 boundary in
`src/ownership_proof/vec_capacity.rs`. The remaining handle predicate comes
from the exact source constructor and B1 contract. No automatic `BytesMut`
Drop claim is made. The feature-gated negative asks for capacity against a
strictly larger request.

Run the native exact-source constructor check with:

```sh
cargo test --locked --offline --manifest-path verification/probes/actual-with-capacity/Cargo.toml
```
