# Generic default Buf predicate body probe

This experiment mechanically extracts the actual default method bodies for
`Buf::has_remaining` and `BufMut::has_remaining_mut`, plus the actual
`remaining` and `remaining_mut` bodies for `&[u8]` and `&mut [u8]`. It checks a
small local proof interface that is intentionally narrower than either public
bytes trait.

The probe interface gives the abstract metadata getter a postcondition and
makes the selected slice implementations prove that their runtime length
matches the ghost slice length. The two extracted default bodies then prove
that the returned predicate matches that metadata, both in a generic caller
and at the concrete slice call sites. No trait law or contract is added to
`src/buf/buf_impl.rs` or `src/buf/buf_mut.rs`; the probe does not establish a
contract for the original public traits or arbitrary `Buf`/`BufMut` implementors.

The `wrong_predicate` feature reverses the expected result for the `Buf`
default body. The negative run fails exactly three proof files: the extracted
`has_remaining` body, its generic caller, and its concrete `&[u8]` caller. The
`BufMut::has_remaining_mut` body and callers still pass. This demonstrates
that the positive gate checks the predicate result rather than merely
translating it.

Build-time extraction uses the exact source signatures and bodies from the
current bytes crate. The generated `source_fragments.txt` and translated proof
artifacts are emitted under Cargo's `OUT_DIR`; the source extraction and
round-trip check run each build.

Proofs must be launched outside the sandbox and serialized with the repository
lock because Why3 opens a Unix-domain socket. The recorded runs used the
bytes 1.11.1 toolchain, default feature set for the positive case, and
`--features wrong_predicate` for the negative case. They are isolated probe
evidence, not a runtime API proof or whole-crate proof. The positive run proved
10 files with no unproved obligations; the negative run had the three
intentional failures above. The exact extracted methods and source snippets
are in `extraction/`; logs and proof output are in `logs/` and `evidence/`.
