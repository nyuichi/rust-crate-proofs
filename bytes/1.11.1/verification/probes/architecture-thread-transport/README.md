# Architecture prerequisite T01: physical thread transport

**BLOCKED before VC generation.** Native execution passes two tests: 84
physical-allocation cases using two actual scoped threads, plus the unchanged
primitive's 64-iteration native atomic race test. The pinned Creusot translation
fails with E0277 at both scoped spawns. No Why3 proof was launched.

The shared invariant stores `expected: Snapshot<(RetiredPart, RetiredPart)>`.
`RetiredPart` carries `PhysicalRegion` and `Recovery`, which deliberately contain
`NotObjective`. Stock `Snapshot<T>` inherits the auto-trait restriction through
`PhantomData<T>`. Stock `AtomicInvariant<State<T>>` requires `State<T>: Objective`
to implement `Sync`; the child closure therefore cannot send a shared reference
to this invariant across the real thread boundary.

The former 36-file physical-retirement gate called both retirements sequentially
and did not require this `Sync` bound. Its proved bodies remain useful component
evidence; they were never a proof of physical resource transport through spawn.
Ordinary native builds do not impose the proof-mode Objective bound, explaining
why these native tests pass while proof translation rejects the same caller.

The planned witness leaves sealed `BoundPtr` metadata in the parent: it contains
`NonNull` and has no Send/Sync implementation. Both children own actual affine
physical ghost payloads and tickets, receive fresh Tokens from stock spawn, and
return the last observer's recovered capabilities through stock join. The parent
then performs conditional B3 cleanup. This is last-owner-to-parent handoff, not
deallocation by the last thread. The compiler obstruction occurs before that
composition can be proved. Exactly one final observer is checked natively only;
the existing retirement postcondition does not specify joint eventual completion.

## One interface review; stopped

Keep retired physical capabilities sealed in the existing `AtView<T>` fields.
A possible future changed premise is to replace the full subjective expected
payload snapshot with an explicitly objective metadata projection, and prove
that each submitted/returned capability matches it. Relevant metadata includes
allocation/resource IDs, interval bounds, allocation capacity, recovery ownership,
and slot/value facts required by cleanup. Such a redesign must preserve the
payload-binding guarantee: deleting `expected` without a replacement is not a
repair. It is not implemented or validated here.

No unsafe Send/Sync/Objective implementation, primitive contract, or bytes
protocol assumption was added. Existing physical and weak-memory primitive files
are imported by path unchanged and hash checked. Generic primitive adequacy
remains an explicit assumption. Do not infer original Bytes thread safety from
these native tests, or treat this failure as proof that an objective projection
cannot work.

T01 is a D06 architecture prerequisite, not the admission witness. D05 remains
closed; actual Bytes Clone/Deref authority, arbitrary registration, automatic
Drop, and the original complete-verification target are unchanged. The existing
missing-Acquire/empty-ticket controls remain prior component evidence; no new
negative was run because the positive transport case did not reach VCs.

## Reproduce the recorded result

From this directory, activate `/workspace/bytes-proof-tools/activate.sh`, then:

```sh
cargo test --offline --locked -- --nocapture
cargo creusot --only=coma -- --locked
```

The second command is expected to fail at the two Objective/Sync obligations.
An unchanged rerun is evidence maintenance only, not a reopened architecture
experiment. `EXPERIMENT.md` records the plan written before execution. The
evidence archive contains exact probe, dependency and relevant tool sources,
logs, per-member hashes, and an independently recomputed archive audit.
