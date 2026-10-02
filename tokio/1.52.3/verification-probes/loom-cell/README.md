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
move rather than remove the trusted boundary. Production SetOnce now confines
both generic callbacks inside the transparent, operation-specific
`SetOnceValue<T>` wrapper. The integrated verification uses the body-proved
vstd `PCell` counterpart, checks the erased representation, and directly runs
all four production operations under loom. This removes generic callback
reasoning from SetOnce itself, while standard `UnsafeCell` semantics remain the
foundational adapter.

The direct production boundary can be removed when vstd provides:

- a permission-carrying specification for `UnsafeCell::new` and
  `UnsafeCell::get`; and
- raw-pointer permission transfer sufficient to verify the four small
  `SetOnceValue<T>` method bodies (generic callback contracts are no longer
  required by the rest of SetOnce).
