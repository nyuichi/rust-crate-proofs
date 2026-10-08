# Original Shared lifecycle integration plan

User requested physical shared read, native Release/Acquire last-owner recovery,
and original bytes.rs Shared integration on 2026-10-08 UTC. This plan is recorded
before integration changes; it is not proof evidence or an admitted architecture.

## Fixed original-source correspondence

- src/bytes.rs From<Vec<u8>>::from, selected len < capacity arm: actual pointer,
  length, capacity, Box<Shared>, ref_cnt=1, data=shared pointer, SHARED_VTABLE.
- Shared is bytes.rs Shared { buf, cap, ref_cnt }, distinct from bytes_mut Shared.
- shared_clone -> shallow_clone_arc: actual fetch_add(1, Relaxed), original
  old_size > usize::MAX >> 1 abort guard, copied pointer/len/control identity.
- release_shared: actual fetch_sub(1, Release), non-final early return, final
  actual load(Acquire); explicit buffer/control cleanup of matching allocations.
- Actual public vtable dispatch and automatic MIR Drop effects remain separate
  obligations unless the exact caller proof includes them. Do not describe a
  source-gated direct leaf proof as full Bytes::clone or automatic Drop.

## Integration gate

First admit a bounded reclaimable physical-sharing protocol in the adjacent
shared-physical-lifecycle probe. Consume B1 capabilities once into the protocol;
FullBorrow/EndBorrow and lifetime fractions must make all byte read borrows end
before recovery. Each original handle has a different affine ticket; all full
payload/control authority is stored once. The existing exclusive freeze sidecar
cannot be cloned and must not remain an alternative parallel ownership proof.

Attach the generic event boundary to the ACTUAL ref_cnt field. An independent
registry counter with equal initial numeric value is insufficient. Native/model
identity is an explicit generic atomic-adapter assumption, while constructor
field/data/control matching and bytes registration/finalization are body proofs.
The native implementation retains original field layout/orderings.

Require exact input-byte contents for both owners, preservation across non-final
cleanup, native-count/ticket correspondence within the selected bound, all
fractions recovered only after the last ticket returns, required publication
observed by the actual Acquire load, and matching payload/control freed once.
Normal return/non-overflow is a bounded first gate; no wrapping zero reclamation.
No bytes-specific ownership/refcount theorem receives trusted.

Negative controls must reject missing Acquire, premature recovery/live read,
duplicate ticket/finalizer use, wrong actual field binding and retargeted pointer.
Save each exact source/configuration and classify frontend versus VC failures.

Native allocator tests added to the original crate check all 6 three-owner drop
orders for empty/nonempty views and a slice across another owner's release.
Their 2 successful test functions are observations, not formal integration.
Evidence: verification/shared-lifecycle-native-2026-10-08.json.
