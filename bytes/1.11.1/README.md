# bytes 1.11.1 runtime verification

This branch verifies selected paths of the actual bytes implementation using
body proofs and audited native/proof-source correspondence. It does not establish
complete verification of the original API. The old length/capacity replacement
model is historical and is not counted as runtime verification.

The latest result (AZ) covers a nonempty `Box<[u8]>`, arbitrary finite capped
cursor steps, a Clone and lexical peer Drop after each step, exact suffix output,
and final normal Root Drop with allocation reclamation. Raw and Shared phases
are covered under the recorded generic physical, pointer, atomic, callback and
compiler/source-correspondence assumptions. Arbitrary concurrent/escaping owners,
unwinding, other constructors and the complete BytesMut/API surface remain open.
See the [precise guarantee](verification/probes/original-root-phase-clone-2026-10-09/README.md)
and its [trusted boundaries](verification/probes/original-root-phase-clone-2026-10-09/TCB.md).

The archived positive proof has 191 function targets, 2,207 prover leaves and
zero unproved leaves. Its original diagnostic status and later successful source
correspondence are separate: [origin audit](verification/evidence-closures/az-proof-origin-v1/reports/AZ_ORIGIN_CLOSURE_AUDIT.md),
[reuse audit](verification/evidence-closures/az-admitted-reuse-v1/reports/AZ_ADMISSION_AUDIT.md).
This is retained evidence, not a claim that the cleanup reran the prover.

Work on default `std`, x86_64 and native atomic orderings; no feature matrix or
`sc-drf`. Use [tool setup](TOOLCHAIN_DECISION.md) and the affected probe's
`run-proof.sh`. `./verify-all.bash` targets the whole runtime and remains a
blocked integration diagnostic, not the successful AZ proof entrypoint.
Do not rerun that unchanged blocker or historical negative suites per iteration.

Keep source/contract/proof inputs, counterexamples and immutable evidence intact.
The reviewed production manifest is hash-bound: deleting its tests or benches
requires a separate capture/correspondence update. For new architecture work,
read [decisions](verification/ARCHITECTURE_DECISIONS.md); update this overview only
when its guarantee or scope changes. Detailed assumptions and helper mappings
remain in [TRUSTED_BASE.md](TRUSTED_BASE.md) and
[SOURCE_CORRESPONDENCE.md](SOURCE_CORRESPONDENCE.md).

Source is crates.io bytes 1.11.1, archive SHA-256
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`,
upstream revision `417dccdeff249e0c011327de7d92e0d6fbe7cc43`. Runtime changes retain
the original API and representation, including the Vec reverse-comparison fix.
[MIT license](LICENSE). Older iteration narratives remain in Git history.
