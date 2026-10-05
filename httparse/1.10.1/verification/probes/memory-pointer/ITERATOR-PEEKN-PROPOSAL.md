# Iterator and `peek_n` follow-up proposal

This design note was written before the source freeze was released. The
`IteratorSpec`, explicit `Iterator::next` contract, weak generic `peek_n`
bounds contract, and private `peek_array8`/`peek_array4` adapters are now in
`src/iter.rs` and translate successfully. At current source hash
`2492453eeb1ec520192e3ff8c93f7bd4941ac7439dab0e7b34fc796a43f9a1a0`, the
iterator body, both laws, and their refinement checks pass (24/24 VCs).

## `IteratorSpec` candidate

The existing `CursorModel` uses absolute offsets relative to the original input
allocation, so committing a slice changes `mark` but not the sequence of bytes
that `Iterator::next` will visit. The existing model already defines the exact
consumption relation and exhaustion test:

```text
produces(before, visited, after) = cursor_produces(before, visited, after)
completed(state) = cursor_completed(state)
```

Implemented specification shape, matching the `std::slice::Iter`
implementation in creusot-std:

```rust
impl IteratorSpec for Bytes<'_> {
    #[logic(open, prophetic)]
    fn completed(&mut self) -> bool {
        pearlite! { resolve(self) && cursor_completed(*self@) }
    }

    #[logic(open, prophetic)]
    fn produces(self, visited: Seq<u8>, after: Self) -> bool {
        pearlite! { cursor_produces(self@, visited, after@) }
    }

    #[logic(open, law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) {}

    #[logic(open, law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<u8>, b: Self, bc: Seq<u8>, c: Self) {}
}
```

The actual runtime `Iterator` body is enabled for Creusot and delegates to
`peek` plus `bump`. Its trait postcondition states that the empty branch is
completed and the successful branch produces exactly `Seq::singleton(byte)`
while advancing the logical cursor by one. The body and laws are proved at the
current source hash. A body-checked sequence-head helper supplies the exact
subsequence decomposition, and `bump` exposes the input/mark/end frame from
its call to `advance(1)`.

## Generic `peek_n` boundary

`peek_n` currently does exactly this at runtime: take `self.as_ref().get(..n)`;
if present, convert that prefix using `U: TryFrom<&'a [u8]>`; return the
converted value only on `Ok`. Its model-level contract should state that the
input passed to the conversion is precisely the current suffix's first `n`
bytes and that the cursor/view state is unchanged. It should not claim that
arbitrary `U` contains or preserves those bytes, because Rust's generic
`TryFrom` trait permits type-specific conversion behavior.

The creusot-std conversion file had generic `TryFrom` and `TryInto` extern
specs, with `TryInto` forwarding the selected `TryFrom` precondition and
postcondition, but no contract for the standard slice-to-array conversion. An
audited extern spec for `[T; N]: TryFrom<&[T]>` now lives in
`creusot-libs/creusot-std/src/std/convert.rs`; it records success iff source
length is `N`, and same-index copy for `T: Copy`. Actual `[u8; 8]` and `[u8; 4]`
callers translate against it in `verification/probes/memory-array-conversion`.
The two private helpers and the two actual conversion caller bodies passed
their 38 VCs in the 95-VC dependency batch at source hash
`40cb9e7dae081f7b480ca75ed44d40d209db76de4d2d66e511a414cbd43a9b3b`; rerun
the selected dependency batch before claiming that evidence fresh for the
current source hash.

The array contract must remain limited to the actual standard conversion
implementation and its supported toolchain. It cannot be used to infer byte
identity for an arbitrary `U`. If the generic `peek_n` body cannot be verified
without imposing a new precondition on safe callers, preserve its runtime
behavior and verify the concrete array path through a narrowly scoped helper
or refactor reviewed by the root agent.

## Required next evidence

1. Refresh the pointer/array dependency targets at the current source hash and
   prove remaining actual mutators (`is_empty`, `slice_skip`, `commit`,
   `advance_and_commit`, and `set_cursor`) within the selected proof scope.
2. Keep public generic `peek_n` bounds-only unless a sound generic conversion
   postcondition can be expressed without restricting downstream `TryFrom`
   implementations; its actual body passed at the earlier 95-VC source hash.
3. The parser call sites already use the fixed-array adapters, and the chunk
   owner reported default and no-default native suites passing after that
   change. Re-run only if a later runtime change requires it.
