# bytes 1.11.1 runtime verification scope

The user requested complete implementation verification. The attached initial
Stage-0/1 request is background guidance, not a limit on the current goal.

Primary configuration: default `std`, no `serde`, no `extra-platforms`, native
atomics and original Ordering, `x86_64-unknown-linux-gnu`. Normal return, documented
panic/abort and unwind/cleanup are separate obligations. Later configurations are
`no_std + alloc` and a fixed 32-bit target; they are not yet verified.

Complete means functional byte correctness, allocation/provenance/initialization
and permission safety, original weak-memory concurrency, abnormal paths, and
source correspondence for the entire runtime/API surface. Model-only proofs,
finite tests, trusted bytes-specific ownership contracts, and excluded runtime
items do not satisfy completion.

Initial repository HEAD: `92fe500df540a2e99f150a9890a05dafb12abde2`, clean tracked
state. Worktree: `/workspace/bytes-runtime-verification`, branch
`bytes-runtime-verification`. The original checkout and itoa configuration remain
unchanged. This task has not pushed or published anything.

Distribution archive SHA-256:
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`.
Upstream revision: `417dccdeff249e0c011327de7d92e0d6fbe7cc43`.
The initial runtime-source difference is only `src/lib.rs`; the legacy proof-only
`src/verification.rs` is additional. See `verification/artifacts/baseline-manifest.json`.

The compiler item inventory includes private runtime helpers, impls and statics
in addition to public APIs. Membership does not imply proof status. The initial
unsafe ledger contains syntactic candidates, including comments; semantic
obligation decomposition and transitive reachability are still required.
