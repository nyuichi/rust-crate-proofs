# Constraints that affect proof validity

Target the real bytes API and representation. Independent models or accumulated
proof counts do not establish implementation correspondence or API completion.
Generic trusted physical/tool primitives need explicit contracts and assumptions;
bytes-specific ownership, count, retirement and destructor laws remain body proofs.

Do not reopen these rejected inferences without a changed premise:
- Ghost mutable access cannot change native results after erasure.
- Weak atomics cannot use SC permission extraction or reentrant invariant opening.
- Tokenless immutable access cannot manufacture reclaimable ownership.
- A modulo refcount with unbounded pending increments does not establish lastness.
- Sparse ticket safety does not establish an exhaustive ownership ledger.
- Normal-return callback/Drop contracts do not establish unwind or termination.
- Unconstrained public trait implementations do not satisfy universal semantic laws.

Keep original native ordering, guarded refcount increments and nonempty/owned-zero
representation distinctions. Source/native correspondence remains an explicit
boundary. The current theorem is sequential and normal-return only.

Past rejected routes and examples are available in Git at `e5f127fb` when needed.
They are not required inputs or mandatory regression runs for current proof work.

## Creusot FnGhost callback purity validation

`tools/creusot-toolpatch/patches/bytes-fn-ghost-purity.patch` targets Creusot
commit `318615be3b8bbc60d1f6d52469ba5c0bdebed4f1` and changes only
`creusot/src/validate/purity.rs` (base SHA256
`5c56221d3f131c6ad1cfc2aebc85363ba84ae548c742a905f3d38100528b1af0`). It
validates an annotated `#[check(ghost)]` callback body in ghost purity context
when created from program code, and preserves the existing guard that keeps
logic contexts logical. The bytes-only setup builds a separate patched
compiler; vanilla Creusot remains available and the itoa tool selection is
unchanged.

Scratch translation diagnostics with the patched compiler accepted a positive
Relaxed load callback that opens `AtomicInvariant` and calls the exact
`Committer::shoot_load`. The callback is an API/purity-boundary check only; it
does not prove the atomic memory-order protocol. Direct nested atomics,
ordinary helpers containing atomics, generic static-trait helper calls, and
nested ordinary closures were rejected as non-ghost calls. The existing ghost
capture validator rejected mutation of an outer scalar, and purity validation
rejected `Cell::set` and raw-pointer `write` calls. Raw pointer dereference was
also rejected as unsupported. A copyable scalar captured by value and changed
inside a `move` closure is closure-local and does not mutate the caller's
scalar. Creusot's existing `tests/should_succeed/purity.rs` also translated
successfully, including its `#[logic(open)]` trait implementation and call.
Explicit `#[trusted(ghost)]` helpers remain within the trusted boundary and
must not conceal native effects.

This compiler correction closes a generic callback purity gap; it adds no
trusted bytes ownership/count/last-owner/destructor law and proves no
memory-order correctness by itself. The selected toolchain identity is bound
by the direct proof runner and receipt.
