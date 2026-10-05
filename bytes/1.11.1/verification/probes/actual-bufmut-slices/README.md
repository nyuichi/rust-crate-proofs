# Exact `BufMut` mutable-slice method bodies

This probe checks the selected `remaining_mut`, `chunk_mut`, and `advance_mut`
bodies from the concrete `BufMut` implementations for `&mut [u8]` and
`&mut [MaybeUninit<u8>]` in `src/buf/buf_mut.rs`. The build script extracts all
six method bodies from those implementations and checks that the generated
items retain the source bodies modulo whitespace. It also extracts
`UninitSlice::new`, `UninitSlice::uninit`, and
`slice_mut_ops::advance_slice_mut`, which the selected methods call.

The two probe-only interfaces have one implementation each, for these exact
slice types. Their sequence and cursor specifications prove the six extracted
bodies; they do not state a generic `BufMut` law or establish behavior for any
other implementor. The
`UninitSlice::new` and `UninitSlice::uninit` functions remain the existing
trusted typed-reinterpretation bridges. The `advance_slice_mut` helper body is
included and body-proved. The proof shows bounded cursor advancement and the
returned chunk's current and prophetic typed view. It does not prove the
unsafe caller's promise to initialize bytes before advancing, nor the public
trait's broader unsafe contract.

`panic_advance` is retained as a source dependency with `requires(false)` in
this probe. The selected advance methods' bound precondition must establish
that their out-of-bounds panic branch is unreachable; the panic implementation
and its message are outside this proof.

Run the native tests from this directory with:

```sh
cargo test --locked --offline
```

Run the serialized Creusot proof with:

```sh
./verify.bash
```

The proof archive records the exact source snapshot, generated extraction,
translation output, Why3 tasks, proof reports, and native test log.
