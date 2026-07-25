# Unsafe collection verification foundation

## Scope and model

The first reusable vertical slice lives in
`creusot-libs/creusot-std/src/std/unsafe_collection.rs`.

Runtime storage is modeled as `Seq<MaybeUninit<T>>`; each slot's view is an
`Option<T>`. One canonical `Int` length defines the collection position:

- `initialized_prefix(slots, len)` says every slot in `[0, len)` is initialized.
- `owned_prefix(slots, len)` additionally says every slot in `[len, capacity)`
  is unowned (`None`). This is the representation predicate used for growth.
- `prefix_values(slots, len)` is the functional element sequence in order.

The model separates three concerns:

1. Slot ownership: `write_slot`, `take_slot`, and `drop_slot` transfer ownership
   and update `MaybeUninit` state.
2. Collection orchestration: length changes consume contracts expressed in
   `owned_prefix` and `prefix_values`.
3. Storage access: inline arrays, raw pointers, and allocator-backed buffers
   must each provide a small adapter preserving the same slot sequence and
   capacity. They must not introduce a second progress counter.

For raw pointers, adapters should use the existing `Perm<*const [T]>`,
`PtrLive`, and `PtrAddExt` contracts in `creusot_std::std::ptr`. For allocated
storage, allocation/deallocation remains a separate provenance component:
allocation must establish a live range of `capacity` unowned slots, movement
must preserve `prefix_values`, and deallocation requires an empty ownership
range. This pilot intentionally does not axiomatize an allocator just to make
inline storage pass.

## Pilot: arrayvec 0.7.8

`ArrayVec` keeps its existing integer `View` for compatibility and adds:

- `owns_initialized_prefix()` as its representation relation;
- `elements()` as its functional sequence;
- functional contracts for `push`, `try_push`, `push_unchecked`, and `pop`;
- a capacity invariant linking `CAP` to `LenUint::MAX`;
- a consuming-iterator sequence model. Forward production removes a prefix;
  backward production removes a prefix from the reversed sequence.

The public push/pop bodies use the common ownership API instead of raw
`ptr::write`/`ptr::read`. This proves value order, ownership transfer, and the
length transition, not only panic freedom.

## Existing boundary survey

Counts below are source-level `#[trusted]` and `creusot::no_translate`
occurrences at this checkpoint; they are inventory data, not proof metrics.

| Target | Trusted | Excluded | Relevant current boundary |
|---|---:|---:|---|
| arrayvec 0.7.8 | 100 | 3 | Most mutation, slice/raw-pointer access, drop/drain, and many string methods remain trusted. The former `IntoIter` `true` protocol has been replaced by a strong sequence contract, but its representation bridge remains trusted. |
| smallvec 1.15.2 | 73 | 10 | Inline/heap discriminant access, layout, allocation/deallocation, `triple_mut`, push/pop, drop, and slice construction are trusted. Consuming iterator implementations are excluded under Creusot. |
| heapless 0.9.2 | 28 | 16 | The real `vec` module is replaced under `cfg(creusot)` by `vec_creusot.rs`; only storage capacity/borrow scaffolding is verified, while owned-array borrows are trusted. This is not yet a proof of the real unsafe Vec body. |
| slab 0.4.12 | 55 | 0 | It uses `Vec<Entry<T>>`, but its view is only occupied length. Capacity, reserve/shrink, vacant-list repair, insertion/removal, iterators, and entry APIs are largely trusted, so slot occupancy is not functionally modeled. |
| indexmap 2.14.0 | 4 | 0 | `verification.rs` replaces the production hash-table representation with ordered `Vec` models. Only random hasher constructors are trusted there. This proves the verification model, not the production raw-table/allocator implementation. |

The next reuse order is: heapless owned inline storage, arrayvec truncation and
drop, smallvec inline mode, then a raw-pointer/allocator adapter for smallvec's
heap mode. Slab needs an occupied-slot model rather than an initialized prefix;
indexmap should consume the allocator adapter only after its production
representation is translated.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `initialized_prefix` / `owned_prefix` | yes | logical definitions | no | yes |
| `prefix_values` | yes | yes | no | yes |
| `write_slot` / `take_slot` / `drop_slot` | yes | yes | no | yes |
| array-to-sequence lifting (`push_array_slot`, `pop_array_slot`) | yes | no | temporary | yes |
| `ArrayVec::{push, try_push, push_unchecked, pop}` | yes | yes | no | yes |
| `IntoIter` view, laws, `into_iter`, `next`, `next_back` | yes | no | temporary | yes |
| existing `IntoIter::drop` | partial | no | existing | yes |

The array lifting boundary is trusted because the current prover run leaves one
sequence-extensionality VC for array update/snoc and one for update/unsnoc.
Remove it by proving those two lemmas in the common module without changing the
contracts or callers.

The iterator boundary is trusted because the runtime embeds an `ArrayVec` while
creating moved-out holes before `index`, which is not an initialized prefix.
Remove it by giving `IntoIter` a dedicated initialized range `[index, len)` and
proving its view, iterator laws, next operations, and drop from that range.

## Scoped verification commands

```text
cd creusot-libs/creusot-std
cargo creusot --simple-triggers=false prove 'unsafe_collection::*' -- --lib
# Proved (4 files)

cd arrayvec/0.7.8
./verify-all.bash
# no-default-features: Proved (71 files)
# default features:   Proved (71 files)
# all features:       Proved (71 files)
```

No unrelated crate proof suite was run.
