# AQ trusted-boundary ledger

Development ledger; no positive or architecture admission result is claimed.
AQ inherits AP's explicit physical, atomic, field, erased-callback, private Std
and normal terminal compiler assumptions. All inherited body targets must be
reproved after the explicit proof-only enum transformation. Published ancestor
sources remain immutable. Full original architecture remains NOT ADMITTED.

The new generic pointer adapters produce metadata, never a lifetime ticket,
physical permission or recovery capability. `add_live` executes actual ptr.add:
its strong contract requires matching sealed namespace/capacity, an existing
live PhysicalRegion borrow, and an in-range offset. `wrapping_bounded` executes
wrapping_add, requires bounded metadata (or unbound with zero count), and grants
no liveness. `without_provenance` executes null::<u8>().wrapping_add(address),
preserves the actual numeric address and produces unbound non-null metadata.
It does not preserve an allocation namespace for an empty result.

The pinned private Std PtrAddExt::add_live is the generic permission-bearing
analogue: its trusted erased wrapper requires PtrLive.contains_range and models
actual pointer offset. AQ currently uses the existing B3/B4 PhysicalRegion model
rather than silently converting it into PtrLive. Replace this adapter with a
proved bridge to Std PtrLive and sealed BoundPtr arithmetic, or stronger generic
Std pointer contracts. The address/runtime-metadata pointer model does not prove
Rust provenance adequacy; that interpretation remains an explicit physical TCB.

The generic null_pointer reifier preserves the exact native core::ptr::null_mut
word through an opaque null_word symbol. Shipped Std null contracts expose only
is_null_logic (numeric address zero); this narrow exact-value assumption follows
the existing AE static-null analogue and creates no authority. Replace it with
an exact generic Std null-value contract or proved pointer-word representation.

Empty reads require a non-null pointer and zero length, with no allocation
liveness. The native new_empty_with_ptr deliberately removes provenance. Empty
Static Drop must have no owner-map or recovery effect. Shared view Drop must
recover using original full allocation metadata; the view offset and length
never become deallocation arguments. These Bytes properties are body obligations,
not new trusted ownership, lastness or destruction laws.

Exact-item callback/table reification and erased proof channels remain generic
compiler TCB. New registrations must equal the actual body-checked view callback
contracts and preserve native arguments. Replace this boundary with verified
callback-contract reification and lowering. Native source/MIR correspondence
must preserve built-in Range resolution, assertions, add versus wrapping_add,
empty Static dispatch and actual normal Drop/return ordering. Replace selected
normal-edge lowering with a verified compiler interpretation; unwind remains open.

The selected public slicing theorem specializes built-in Range<usize> under
valid bounds. Pinned Std RangeBounds contracts support that implementation;
no purity or contract is assumed for arbitrary downstream RangeBounds bodies.
Invalid ranges, panic/unwind, concurrent or arbitrary escaping ownership and
remaining representations/APIs/configurations are not admitted by this client.
Native smoke cases corroborate execution and are not formal proof results.
