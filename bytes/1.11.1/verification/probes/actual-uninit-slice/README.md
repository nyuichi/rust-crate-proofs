# Exact `UninitSlice` runtime probe

The build script extracts `UninitSlice`, its view, the two borrowed-slice
`From` implementations, and the exact `new`, `uninit`, `write_byte`,
`copy_from_slice`, and `len` source methods from `src/buf/uninit_slice.rs`.
The two mutable reinterpretation methods are trusted boundaries for the
`repr(transparent)` casts. Their contracts preserve the typed slice model and
the mutable writeback relation. The source file's separate `uninit_ref` cast and
raw-pointer APIs are outside this probe.

The probe checks writeback after `write_byte` and `copy_from_slice` through the
initialized conversion, plus initialization of every copied slot through the
`MaybeUninit` conversion. The write and copy bodies are translated from the
runtime source and body-proved; no `BytesMut` permission or ownership protocol
is trusted. The current positive run proves all eight generated files, and the
native suite passes all three tests. Captured source, Coma, Why3 sessions, and
logs are in `evidence/positive.tar.gz`.

Run the serialized proof wrapper from this directory:

```sh
./verify.bash
```

Translation only, without starting Why3:

```sh
BYTES_TRANSLATE_ONLY=1 ./verify.bash
```

The native aliasing and writeback checks can be run from the crate root with:

```sh
cargo test --locked --offline --manifest-path verification/probes/actual-uninit-slice/Cargo.toml
```

The `wrong_copy_model` feature adds an intentionally false postcondition to
check that the copy model is observed by the proof gate:

```sh
./verify.bash --features wrong_copy_model
```
