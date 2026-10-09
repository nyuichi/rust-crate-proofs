# AO trusted-boundary ledger

AO retains the audited AL/AM/AN generic physical allocation, pointer provenance,
weak synchronization, callback erasure and normal terminal compiler boundaries.
Their exact sources/contracts are captured and pinned; body-proof totals and
native tests do not establish these primitives' adequacy for Rust's memory model.

AO adds two closed callback registration instances:
even_detaching_root_registration and odd_detaching_root_registration. Each
returns the actual native promotable table and a three-runtime-argument callback
with pre/postconditions equal to detaching_root_drop_checked. The extra owning
Ghost input carries the complete PromotionScope and output slots and erases
natively. This is the retained generic selected-body registration interpretation,
not an ownership, lastness or recovery law. The checker binds actual table
entries, full callback bodies/contracts, native root Drop dispatch, owned
pointer finish, release helper and output composition. Replace this boundary
with verified native-to-proof callback reification preserving the runtime ABI
and consuming ghost input. The shipped/private Std analogy remains the closed
function callback and erased invocation interface documented by AL/AN; these
source-checked registration instances are not claimed to be a Std theorem.

The root consumes its owned pointer history through get_mut_finish. The actual
word excludes the raw-free branch after first promotion. Existing release_core
executes the guarded Release decrement and State::on_release; the latter seals
the actual ID-zero root recovery into private state on a nonfinal release.
DetachedScope contains only Cursor. No logic getter extracts RootCore, Perm,
State or Recovery. Model/public/map observations and Boolean acceptance preserve
the child-to-cursor relation without reconstructing the destroyed root invariant.

The surviving child reads through its existing physical FullBorrow and actual
token. Its final release uses the retained child callback; the last branch
performs the final Acquire and Pending::recover, then both payload and control
frees. Root and child completion metadata are checked against the actual same
payload. These ownership transitions and finality facts are body-proved under
the generic atomic/physical contracts, not newly trusted bytes-specific laws.

The normal compiler boundary binds the native client's exact original Drop,
subsequent survivor read, saved return and final survivor Drop places and
successors. The checker records every normal and cleanup MIR block, but the
proof scope covers normal completion only. Exact fields/imports and the
field-no-Drop profile rule out hidden independent field glue in this selected
configuration. Replace this boundary with a verified normal-edge destructor
translation preserving these consuming interfaces; unwind remains open.

Successful AO verification is conditional on these explicit assumptions.
Full original-crate admission remains NOT ADMITTED.
