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
