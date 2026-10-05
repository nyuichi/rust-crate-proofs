# Advanced unique trait invariant diagnostic

This is a failed diagnostic, not a positive verification gate. The final exact
source attempts actual unique advance followed by core AsRef/AsMut and generic
trait callers, with full original-allocation cleanup. 77 proof files are
retained; advance_unchecked has one unproved current-state BytesMut invariant
obligation (27/28). The other files contain no unproved leaves. Passing caller
postconditions do not establish that the unproved body preserves its invariant.

The earlier exact task is included for diagnosis. Predicate simplification
removed an impossible Shared alternative. Flattening the invariant did not
solve advance and regressed spare_capacity_mut; that form was discarded.
Computing the next data/ptr/len/cap in locals before assigning fields still left
the same advance obligation. No trust or extra assertion chain was added.

The final diagnostic's two native fixture matrices pass; ordinary test_bytes
passes 118 tests and no-default-features passes. These do not substitute for
the missing proof. The attempted source and native refactor were reverted in
the live crate to the last proved checkpoint. Automatic Drop, Shared invariant
integration and Deref/DerefMut remain excluded. Continue scalable explicit
ownership separately; reconsider this invariant only with a structurally new
proof interface or a generic tool improvement.
