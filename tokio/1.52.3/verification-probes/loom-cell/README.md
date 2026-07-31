# Tokio loom-cell direct Verus probe

The production non-loom wrapper in `src/loom/std/unsafe_cell.rs` has been
presented directly to the pinned Verus toolchain with `#[verifier::verify]` on
the type and implementation.

The direct translation reaches two precise unsupported standard-library
interfaces:

1. `std::cell::UnsafeCell<T>` and its `new`/`get` methods have no vstd
   specification;
2. after temporarily supplying opaque specifications for those three items,
   calls through the generic `FnOnce(*const T)` and `FnOnce(*mut T)` callbacks
   cannot establish the closure precondition.

No opaque specification from this experiment is retained because that would
move rather than remove the trusted boundary. The integrated verification uses
the body-proved vstd `PCell` adapter and directly compiles the production source
in its layout test, checking size, alignment, field address, and both callback
pointer paths.

The direct production boundary can be removed when vstd provides:

- a permission-carrying specification for `UnsafeCell::new` and
  `UnsafeCell::get`; and
- callable `FnOnce` contracts capable of transferring the corresponding raw
  pointer permission into and back out of these callbacks.
