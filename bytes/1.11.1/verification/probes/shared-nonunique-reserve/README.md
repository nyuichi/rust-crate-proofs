# Shared nonunique reserve

This gate extracts actual BytesMut construction, split_to, slice accesses and
explicit release methods, then calls the source shared_copy adapter. The adapter
reads the actual Shared original-capacity metadata through its Perm, allocates a
unique buffer, copies only Known visible source bytes through B4, constructs an
initialized unique handle, and explicitly retires the old registration. Other
registrations retain their existing allocation and control ownership.

The helper and caller proved in the focused two-file run. The complete gate is
being checked separately. Six native allocator scenarios pass, covering live
and empty source views, zero and spare allocation capacity. They check that both
buffer allocations and the Shared cell are each released once without realloc.
The negative_copy_singleton feature deliberately violates the >=2 ticket guard.

This is a sequential explicit-coordinator gate. The proof does not claim
concurrent ARC operations, automatic Drop effects, or public reserve integration.
No bytes ownership-protocol assumption is added; allocation uses the separately
recorded requested-capacity physical Vec bridge and existing B4 boundaries.
