# Current `parse_code` proof evidence

This snapshot contains the current generated Coma, `proof.json`, and Why3
session for each of the 11 selected targets. All recorded leaves passed with
Z3 4.15.3 under the isolated string-model Creusot toolchain and the shared
one-prover, 1000 MiB queue. `TARGETS.tsv` indexes each exact target, goal leaf
count, and artifact hashes; `SHA256SUMS` covers this index and all 33 target
artifacts.

The selected closure has 33 unique `(target path, VC leaf)` entries: 27 for the
actual `Bytes` byte-reading path and 6 for the independent code model plus the
actual `parse_code` body. The earlier interrupted first attempt is recorded
separately in `../first-attempt-before-next-frame/`; it contributes no
`parse_code` result to this snapshot.

These targets prove the helper against the current `Bytes` contracts. The
external memory and pointer specifications supplied by Creusot's standard
library remain part of the verification trusted base; no parser or code-model
body in this selected set is marked trusted.
