# Creusot translation blockers for `http` 1.5.0

This records translation blockers reproduced from source-shaped probes. They
fail before Why3 VC generation. No trust annotation, alternate runtime
representation, or public API exclusion has been accepted to work around them.

## `Extensions`: type erasure and downcasts

The implementation stores `HashMap<TypeId, Box<dyn AnyClone + Send + Sync>>`.
Its public operations form and consume erased values through `AnyClone::{
clone_box, as_any, as_any_mut, into_any}`, `Any::{downcast_ref,
downcast_mut}`, `Box<dyn Any>::downcast`, and `TypeId::of::<T>()`.

The crate-wide `if_downcast_into!` macro in `src/convert.rs` uses the same
`TypeId` and mutable `dyn Any` downcast pattern. Production expansions occur in
`src/uri/mod.rs`, `src/uri/path.rs`, `src/uri/authority.rs`, and twice in
`src/header/value.rs`. This boundary therefore affects those conversion bodies
as well as `Extensions`; it is not isolated to the extension container.

The focused probe is in [`verification/probes`](verification/probes). With
Creusot `0.11.0-dev`, the shared-reference, mutable-reference, and downcast
forms using `dyn Any` produce experimental-support warnings, followed by the
translation error:

```text
error: forbidden dyn type: dyn std::any::Any
       (dyn support is currently minimal, please open an issue to improve this feature)
```

The `AnyClone` probe separately fails at the erased clone-object signature:

```text
error: forbidden dyn type: dyn AnyClone + std::marker::Send + std::marker::Sync
       (dyn support is currently minimal, please open an issue to improve this feature)
```

An isolated `TypeId`-only probe also fails before VC generation: `TypeId::of`
has no external contract, and its return value does not implement Creusot's
`DeepModel`:

```text
warning: calling external function `of` with no contract will yield an impossible precondition
error[E0277]: the trait bound `std::any::TypeId: creusot_std::model::DeepModel` is not satisfied
```

The attempt to add an external specification alone cannot remove the
translator's type guard. A sound typed-existential model would need to carry a
type tag, erased payload, borrow permission, and a witness connecting each
valid concrete implementation to its tag and projections. Dynamic methods must
have contracts that are valid for those implementations. Mutable downcasts also
need a frame/prophecy relation for the returned mutable reference. `AnyClone`
needs a concrete clone contract that relates the cloned payload to the source
without assuming that arbitrary user `Clone` implementations preserve logical
equality.

Required acceptance probes for any future tool support:

- shared and mutable `Any` erase/project round trips;
- `AnyClone::clone_box` with concrete clone relation;
- equal `TypeId` for a type and its alias, but distinct IDs for distinct
  newtypes;
- failed shared and mutable downcasts preserve the erased object, while
  successful mutable downcast frames all disjoint state;
- the existing negative `unsound_dyn` suite continues to reject its unsound
  dynamic dispatch examples.

Until that work exists, `Extensions` bodies remain untranslated. A proof of
`TypeId` equality or of a separate `Any` slice is not a proof of the production
container or its object-safe calls.

The same dynamic type boundary appears in `Error`: `get_ref` returns
`&(dyn std::error::Error + 'static)`, `is::<T>()` queries the erased object's
type, and `source()` returns a nested erased error. The `dyn-error` probe
reproduces those operations and fails before VC generation with:

```text
error: forbidden dyn type: dyn std::error::Error
       (dyn support is currently minimal, please open an issue to improve this feature)
```

## `HeaderMap`: raw pointer permissions and yielded references

`HeaderMap`'s `RawLinks` implements indexing by dereferencing a stored raw
`*mut [Bucket<T>]`. `IterMut`, `ValueIterMut`, and `Drain` also hold pointers
into the `entries` and `extra_values` vectors, and form references as they
advance. `IntoIter` moves values out of the extras vector and has a guard in
`Drop` to finish dropping remaining entries.

The minimal raw-slice probe reproduces `RawLinks`' pointer-to-slice access. It
fails during translation with:

```text
error: Dereference of a raw pointer is forbidden in creusot:
       use `creusot_std::ghost::perm::Perm<*const T>` instead
```

The std model exposes a possible permission route through
`SliceExt::as_mut_ptr_perm`, `Perm<*const [T]>::split_at_mut`,
`elements_mut`/`index_mut`, `Perm::as_mut`, and `PtrLive`/`add_live`. This is a
design route only; no production operation or iterator proof has been closed.
An iterator proof must split permission by field so it can keep already-yielded
`&mut T` values alive while later calls inspect the same bucket's key and link
fields. A whole-`Bucket<T>` borrow cannot justify that access pattern. The
contract must also establish that each yielded value slot is unique, that link
indices stay within live allocations, and that `Drop` removes or drops every
remaining value exactly once.

Required next probe: prove one source-shaped raw slice element read using
`Perm`, then extend it to a field split and one mutable-yield step. A trusted
`next_unsafe` boundary is only temporary scaffolding and would not establish
the unsafe iterator implementation.

## Probe command and status

The probes live at [`verification/probes/Cargo.toml`](verification/probes/Cargo.toml)
and exact diagnostics are preserved in
[`verification/probes/logs`](verification/probes/logs). Run one bin at a time
from that directory with the target-scoped wrapper; it sources
`/workspace/proof-tools/activate.sh`, forces offline Cargo, and holds
`/tmp/http-creusot-proof.lock`. Current source-shaped commands are:

```sh
../../scripts/run-proof.sh cargo creusot prove --no-cache -- \
  --locked --offline --manifest-path Cargo.toml --bin dyn-any
../../scripts/run-proof.sh cargo creusot prove --no-cache -- \
  --locked --offline --manifest-path Cargo.toml --bin dyn-any-views
../../scripts/run-proof.sh cargo creusot prove --no-cache -- \
  --locked --offline --manifest-path Cargo.toml --bin raw-slice-deref
../../scripts/run-proof.sh cargo creusot prove --no-cache -- \
  --locked --offline --manifest-path Cargo.toml --bin type-id
../../scripts/run-proof.sh cargo creusot prove --no-cache -- \
  --locked --offline --manifest-path Cargo.toml --bin dyn-error
```

The wrapper version check is enabled. `cargo creusot version` reports
`0.11.0-dev`, matching `creusot-std`'s pinned `0.11.0-dev`; `--no-check-version`
was removed from the final replay commands so this blocker evidence does not
depend on bypassing a tool/stdlib compatibility check. Each blocker command
exits during Rust/Creusot translation with the diagnostic recorded above and
status 101; none reaches VC generation. `HeaderMap`, `Extensions`, and the
dynamic `Error` methods have not been body-proved or integrated into a
successful crate proof. The runtime/model invariant design for these modules
and generic Request/Response bodies is tracked in
[`RUNTIME_MODEL.md`](RUNTIME_MODEL.md).

The first capacity-helper proof attempt ran inside the sandbox and left a
Why3 server unable to connect to its Unix socket (`Operation not permitted`).
That was an environment failure, not a VC failure. After terminating that
orphaned process group, the same target-scoped command was rerun with the
unsandboxed execution required for Why3 and completed successfully; its three
VCs are recorded in the map-capacity verification README. No blocker probe
process is left active.
