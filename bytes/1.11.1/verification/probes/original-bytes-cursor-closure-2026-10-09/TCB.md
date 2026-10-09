# AR generic boundaries and remaining admission gaps

AR extends the complete AQ positive source by one private transient `Vacant`
proof variant and new ordinary body-proved methods. It does not admit the full
crate. Published ancestors and production are unchanged. `Vacant` is excluded
from every Bytes validity predicate; it is used only while moving the actual
sidecar through an exclusive ghost borrow.

The new generic `cursor_pointer::add` executes actual `pointer.add(count)`.
Its Live lease requires the actual physical-region authority, matching sealed
namespace/capacity and in-allocation offset bounds. Its Zero lease requires
count zero, an unbound descriptor and an explicitly nonnull actual pointer;
u8 alignment is one. The postcondition preserves exact pointer metadata,
address and allocation interpretation and exports nonnull. Descriptor bounds
never grant access authority. This adapter is trusted native pointer metadata,
analogous to the existing private Std pointer contracts; replace it when that
Std exports an equivalent capability-aware contract. Address equality alone
is not full pointer provenance.

Two generic erased registrations, checked in ghost mode, associate the actual
Shared/Static native callback with its corresponding ordinary body-proved
three-argument contract. Ghost code selects a specification; native code makes
one unchanged vtable invocation. Registration does not assume a Bytes ownership,
last-owner, recovery or destruction effect. Existing generic invoke3 and
compiler/MIR normal-edge destructor interpretation remain inherited TCB.

The probe-local external contract for `[T]::is_empty` states exactly that its
result equals the slice model length being zero, using the same ghost-checking
discipline as the pinned Std's adjacent slice::len contract. It supplies no
memory, ownership or recovery resource. The native assertion call is unchanged.
Remove the local declaration when pinned Std supplies the same contract. The
first failed diagnostic and exact task remain immutable; neither it nor a
frontend failure is positive admission evidence.

Inherited physical projection, allocation/free interpretations, atomic events,
scoped history, pointer observers, private Std and pinned compiler/prover inputs
remain explicit generic boundaries described by AQ and its ancestors. AR calls
borrow_empty only with an explicitly nonnull actual pointer. Its inherited
adapter contract itself does not require nonnull, and an unbound descriptor
invariant alone does not provide it. Strengthening that generic boundary and
reproving all callers remains required for complete admission; AR caller VCs
do not establish universal adequacy of that boundary.

Normal valid-range Shared/Static cursor composition is the selected gate.
Promotable Root mutation, all remaining representations/APIs/traits, arbitrary
escaping and concurrent ownership including CAS losers, unwind, abort/allocator
behavior and configuration coverage remain open. Full original architecture is
NOT ADMITTED.

A subsequent read-only review also identifies the inherited wrapping_bounded
nonnull post for an unbound/count-zero input as needing an explicit nonnull
premise or a body-proved export from BoundPtr's actual NonNull field. This is
an interface-export obligation, not a reachable native null counterexample.
Both generic boundaries and their callers must be rechecked before full
admission; published ancestors and this canonical archive stay immutable.
