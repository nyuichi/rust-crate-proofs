# Exact `UninitSlice` runtime probe

The probe imports `src/buf/uninit_slice.rs` directly. It exercises both actual
`From` conversions, `write_byte`, and `copy_from_slice`. The three trusted
contracts are limited to the `repr(transparent)` reinterpretation of a borrowed
slice and state the matching length, `MaybeUninit` model, and mutable prophecy
writeback. The write and copy bodies use typed `MaybeUninit::new` stores and are
body-proved; no `BytesMut` protocol or permission is trusted.

Run the native alias/writeback checks with:

```sh
cargo test --locked --offline --manifest-path verification/probes/actual-uninit-slice/Cargo.toml
```

The `wrong_copy_model` feature introduces an intentionally false postcondition
to check that the copy model is observed by the proof gate.
