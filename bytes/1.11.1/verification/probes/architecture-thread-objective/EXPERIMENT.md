# T02: objective metadata at the thread boundary

Plan recorded before implementation and execution, 2026-10-06.

D07's changed premise is a different invariant representation: replace the
subjective `Snapshot<(RetiredPart, RetiredPart)>` with body-defined objective
metadata. Actual PhysicalRegion/Recovery capabilities remain affine, submitted
by the child and sealed in `AtView<RetiredPart>` until synchronization. Do not
implement unsafe Objective/Send/Sync or add any primitive/protocol trust.

This new directory preserves T01 unchanged. The local concurrent protocol copy
will document its differences from the existing physical-retirement protocol.
Generic native atomic and physical primitives stay imported unchanged by path
and are hash checked. The metadata/wellformed relation must retain the complete
allocation/partition/recovery facts needed by existing B3 deallocation. Removing
the payload relation merely to obtain Sync is not acceptable.

Use the same real scoped threads and 84 native physical cases as T01. Children
receive their Tokens from stock spawn and return recovered capabilities through
stock join; the parent retains BoundPtr and performs conditional cleanup.
Positive criterion: source translation and body proof of this actual thread
caller and modified protocol under the unchanged primitive TCB. Exactly one
eventual final observer remains a native assertion, not a formal claim, unless
separately established by the callable interfaces and body proof.

After a positive proof, attempt the same threaded missing-Acquire negative.
Preserve every substantive attempt and exact proof-time sources. Use elevated,
serialized Why3 with one solver and 1024 MiB per solver. After two equivalent
failures review the interface; a third requires restructuring. Stop if the
projection requires a large redesign or stronger trusted semantics. No wrapper
or timeout escalation, primitive reinterpretation, or API scope reduction.

This is an architecture prerequisite, not admission. Default Bytes Clone,
read-trait authority, automatic Drop, arbitrary registration, and the original
complete-verification goal remain unchanged. The pending API/tool-scope choice
does not authorize changes here.
