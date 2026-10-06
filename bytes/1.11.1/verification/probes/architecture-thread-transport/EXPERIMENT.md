# One architecture transport experiment

Recorded before execution, 2026-10-06. This is a D06 composition prerequisite;
D01–D05 remain closed. No production source or existing probe is changed.

The changed premise is a real stock Creusot scoped-thread boundary. The previous
physical-retirement gate invokes its two affine retirements sequentially. This
experiment transfers its actual PhysicalRegion/Recovery capabilities to two
children, uses each child's Tokens supplied by stock spawn, and returns the last
observer's recovered capabilities through join. Pointer metadata stays in the
parent because BoundPtr contains NonNull and has no Send/Sync implementation.
The parent performs existing B3 explicit deallocation after joining.

Question: can stock thread contracts transport the existing physical ownership
model and shared weak-memory invariant without new trusted transport/protocol
rules? Existing native atomic/physical primitive TCB is unchanged. Its adequacy
is not established by this experiment.

Acceptance: ordinary native concurrent runs cover empty and nonempty allocations,
spare capacity, both split endpoints, and both child launch orders; translation
and body proof accept actual affine payload movement, shared invariant access,
join, and conditional parent cleanup. Native checks exactly one final observer.
Formal eventual-final-observer/exactly-one completion is NOT asserted unless
the existing callable interface establishes it. Existing retire posts presently
specify recovered payloads conditional on the returned last-observer flag.

Stop at the first structural transport/contract obstruction, followed by one
interface review. No unsafe Send/Sync rescue, new trusted primitive, conversion
of Snapshot<SyncView> into a witness, sequential replacement, or timeout increase.
Only if positive transport passes, replay missing-Acquire as a negative of this
same threaded composition. Existing empty-ticket and other negatives remain
prior component evidence, not new admission evidence.

This is not the architecture admission witness: no actual Bytes handle,
Clone/Deref integration, default trait authority, arbitrary registration,
automatic Drop, or new weak-memory adequacy theorem is claimed. Passing permits
considering explicit-context architecture work; it does not authorize replacing
the original public API or silently reducing the complete-verification goal.
