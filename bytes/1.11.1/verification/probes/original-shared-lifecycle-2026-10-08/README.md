# Original `bytes::Shared` lifecycle adapter

This probe is a source-correspondent leaf for the original `bytes.rs` shared
allocation path. `source_map.py` checks the exact `len < cap` constructor,
`Shared` record and destructor, `shallow_clone_arc`, `release_shared`, and `free_shared` blocks,
as well as the selected `loom.rs` alias that makes `Shared.ref_cnt` a core
`AtomicUsize` under the default non-loom, non-extra-platforms configuration.
It also records hashes for the B1, Box-permission, field-event, and adapter
sources. A mismatch stops the map check until a reviewed refresh.

`source_adapter.rs` keeps the production `Shared` field order and types. Its
constructor consumes the actual Vec through B1, creates `ref_cnt` with the
generic field constructor, moves that returned core atomic directly into
`Shared`, and retains both the B1 recovery/region capabilities and the typed
`Box<Perm<*const Shared>>`. `field_event.rs` takes `&core::sync::atomic::AtomicUsize`
on every event; it does not allocate a parallel counter or wrap the native
field.

The same adapter now constructs the actual four-field native `Bytes` record
shape, including its real `AtomicPtr<()>` `data` field. It initializes that
field with the actual `Shared` pointer, moves the returned core atomic into
`Bytes`, and binds the pointer model to `&bytes.data`. The constructor proof
establishes matching data/control pointer addresses and preserves `ptr`, `len`,
and the passed vtable reference. The vtable reference is an explicit input to
this leaf: identity with `SHARED_VTABLE`, its callback values, and dynamic
dispatch remain open because static-vtable materialization is not part of this
adapter.

The generic synchronization TCB assumes the native atomic constructor maps its
single returned field to one `ModelAtomic`, its initial history and unique
permission, preserving that association across an ordinary move into the
record. Each event is the actual `fetch_add(Relaxed)`, `fetch_sub(Release)`, or
`load(Acquire)` on the borrowed field named by the invariant. It includes the
native atomic operation/order semantics and no bytes ownership, count-to-handle,
last-owner, or reclamation fact. A future typed control-block deallocator may
assume only the exact `Box<Shared>` allocation/layout mapping.

Constructor-stage evidence: native `cargo check` and Creusot translation pass.
The serialized elevated proof run generated 34 proof JSON files containing 114
nested VC leaves; all 114 discharged with zero nulls. The selected
`from_vec_spare_capacity` leaf has 13 VCs, all discharged. Its generic atomic
constructors are trusted; the constructor body proof therefore establishes
the actual `Shared.ref_cnt` field, actual `Bytes.data` field, B1/Box pointer
correspondence, and pointer-address match under those generic native/model
boundaries. The original Shared-only archive remains in
`evidence/constructor-actual-field-2026-10-08/`; the Bytes-record/data-pointer
snapshot is `evidence/bytes-record-data-field-2026-10-08/`.

The B count-1-to-clone ticket protocol, shared byte reads, non-final
preservation, actual Acquire-gated B3 recovery, and exact payload/control
deallocation still need integration. Public vtable dispatch and automatic
`Drop` are outside this leaf and are not claimed here.
