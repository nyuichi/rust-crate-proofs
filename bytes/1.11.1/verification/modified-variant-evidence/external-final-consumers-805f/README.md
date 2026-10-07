# Final 805f downstream consumers

These two downstream crates were compiled against `/workspace/rust-crate-proofs/bytes/1.11.1` with the consumer-local Cargo patch resolving the private Creusot Std 0.13 source. Each archive embeds all 84 source files of the compiled bytes dependency and all 105 private Std package members. Those sources hash-match the final `805f61e3c55be6cc32dac3cf3708063ab923f3ff` bytes source baseline and the `production-finite-cursor-std-287` Std snapshot. The checkout head at capture was `c11ba06238d0dffbbe0a016bf0a4ed40894cfe04`; the 84 compiled-source hashes match the 805f baseline exactly.

| Consumer | Native | VCs | Null proof leaves | Archive SHA-256 |
|---|---:|---:|---:|---|
| `public-read` | 3 passed | 6 proved | 0 | `0ea97376bd2db8461fb1094353748405e6b4635696d0d1b5f3944e90097cf1e6` |
| `concrete-io` | 2 passed | 3 proved | 0 | `174d907bd5a9be0245bdcd37e253c9a6c6101d187453375266bed3235372a66a` |

Six actual public read-model clients passed. Three actual concrete IO clients passed: Read, Write, and Flush bodies followed by explicit close. The logs distinguish the public-read full command output from the concrete-IO short summary log; the latter does not claim to preserve the streamed warnings.

Each `receipt.json` contains a full member hash/length map that covers every archive member, including embedded `members.json`. `root-audit.json` records the independent member, proof JSON, native result, and dependency source checks. Both are finite downstream examples only; they do not establish full API coverage or generic law proofs.
