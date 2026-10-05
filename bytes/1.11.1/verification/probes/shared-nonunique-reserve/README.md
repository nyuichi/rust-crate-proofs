# Shared nonunique reserve

This gate extracts actual BytesMut construction, split_to, slice accesses and
explicit release methods, then calls the source shared_copy adapter. The adapter
reads the actual Shared original-capacity metadata through its Perm, allocates a
unique buffer, copies only Known visible source bytes through B4, constructs an
initialized unique handle, and explicitly retires the old registration. Other
registrations retain their existing allocation and control ownership.

The archived complete gate has 98 proof files with no unproved leaves, including
the helper and actual caller. Six native allocator scenarios pass, covering live
and empty source views, zero and spare allocation capacity. They check that both
buffer allocations and the Shared cell are each released once without realloc.
The negative_copy_singleton feature deliberately violates the >=2 ticket guard.

This is a sequential explicit-coordinator gate. The proof does not claim
concurrent ARC operations, automatic Drop effects, or public reserve integration.
No bytes ownership-protocol assumption is added; allocation uses the separately
recorded requested-capacity physical Vec bridge and existing B4 boundaries.

The archive audit was performed on 2026-10-05 against
`verification/cloud-handoff/session-evidence.tar.gz`; the checked manifest pins
all archived files. Counts concern `live-verif/shared-nonunique-reserve/` and are historical
checkpoint evidence, not a fresh replay or whole-crate integrated proof. Current
the same copy helper and public cfg caller are freshly covered by
`actual-shared-reserve/evidence/positive` (105 files, zero unproved). No negative-control
proof result for this gate was present in that archived directory.
