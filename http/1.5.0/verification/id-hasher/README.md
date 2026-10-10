# `Extensions` id hasher leaf

This target includes the production `src/extensions/id_hasher.rs` module. It
isolates the scalar `Default`, `Clone`, `Hasher::write_u64`, and
`Hasher::finish` bodies from the `Extensions` map's dynamic value operations.
`Hasher::write` intentionally panics because `TypeId` uses the `write_u64`
override; it is not claimed as a normal-returning hash operation.

The four normal-returning methods were proved (one VC each). The whole target
also emits an unproved `vc_write_IdHasher` obligation for the unconditional
`unreachable!` body. Do not interpret that panic path as a functional hash
result or hide it with a reachability precondition.

Run the accepted goals under the repository's elevated proof runner from this
directory:

```sh
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  'verif/http_id_hasher_proof_rlib/id_hasher/impl_Default_for_IdHasher/default.coma' \
  'verif/http_id_hasher_proof_rlib/id_hasher/impl_Clone_for_IdHasher/clone.coma' \
  'verif/http_id_hasher_proof_rlib/id_hasher/impl_Hasher_for_IdHasher/write_u64.coma' \
  'verif/http_id_hasher_proof_rlib/id_hasher/impl_Hasher_for_IdHasher/finish.coma' \
  -- --locked --offline
```
