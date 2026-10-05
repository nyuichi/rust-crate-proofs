# Adjacent Shared unsplit

The source adjacent_region adapter keeps the left ticket identity, retires only
the right ticket, and joins their adjacent physical regions. Empty intervals are
set-disjoint while their tickets remain real registered ownership obligations.
The source shared_unsplit wrapper performs the actual Release decrement, then
merges two BytesMut views with the same control and adjacent allocation ranges.
The actual caller makes three views, merges the first two, mutates/reads the
result, and explicitly releases the two survivors in either order.

The affine core and native decrement wrapper proved in the focused run. Handle
reconstruction and caller framing are being checked separately. Native allocator
tests cover every first/second boundary, zero/exact/spare capacity, and both
release orders, with exactly one original buffer/control cleanup and no realloc.

This is a sequential same-control adjacent merge gate. It does not infer common
provenance from equal numeric addresses, and does not cover fallback copying,
arbitrary public unsplit dispatch, automatic Drop effects, or concurrency.
No trusted bytes ownership-protocol function is introduced.
