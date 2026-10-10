# bytes 1.11.1 runtime verification

The retained current proof covers nonempty Box input, arbitrary finite capped
cursor steps, Clone and lexical peer Drop at each step, exact suffix contents,
and final normal Root Drop/reclamation across Raw and Shared phases. It proves
191 functions / 2,207 prover leaves with no unproved leaves. This is selected
sequential, normal-return implementation evidence, not complete API verification.
Concurrency, escaping owners, unwind and the remaining BytesMut/API paths are open.
Generic physical, pointer, atomic and native/shadow compiler assumptions remain
explicit in the [current TCB](verification/probes/original-root-phase-clone-2026-10-09/TCB.md).

- `./verify-all.bash --check`: check saved proof results and their exact current
  source correspondence, offline, without compiling or calling a solver.
- `./verify-all.bash`: translate/prove the current positive witness once with
  default features, then check the outputs and source correspondence. Requires
  the bytes Creusot 0.13 toolchain (`bash scripts/setup-bytes-toolchain.sh`).
  Why3 execution requires elevated permissions; the wrapper serializes proofs.

[Results](verification/results/README.md) contain one self-contained current
snapshot and one deduplicated component archive. Current checks do not execute
historical checkers, negative suites or audits. A saved proof is reused only for
identical implementation/contract inputs; changed proofs need reviewed source
correspondence before replacing the snapshot. Manifest test/bench metadata is
excluded from the library identity; runtime features and dependency versions are not.

Upstream tests/benches, failed experiments, progress logs and retired workflows
are removed from the working tree. Their history is available at commit
`e5f127fb`; historical component results do not imply current implementation coverage.
Only required Rust dependencies of the current witness remain under `verification/probes`.

Source: crates.io bytes 1.11.1, archive SHA-256
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`,
revision `417dccdeff249e0c011327de7d92e0d6fbe7cc43`. The original API and representation
are retained with the branch's runtime fixes. [MIT license](LICENSE).
