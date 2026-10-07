# Owner I/O proof operation note

## Why3find goal filters

Do not pass a Rust function name alone to `why3find prove -g`. In the pinned
Why3find version, `-g <name>` selects only a theory name or the exact
`Theory.goal` pair. The source check is
`List.mem thy gnames || List.mem (thy ^ "." ^ (Session.goal_name goal)) gnames` in
`/workspace/bytes-proof-tools/why3find-source/src/prove.ml`; the README examples
use `Bar` or `Bar.jazz`.

The inspected pinned source file hashes are `prove.ml`
`e2d99042d4c4439a649f1cdc339f5200a28767a9315d0b2c53af6826e2b39bd3`,
`command.ml` `33864a68722e0f2b777ec130883045563240cbf4f210e287b0afc49ace181d9a`,
and `README.md` `8947402a3af8e479a3497612fd5d646cbfd875165a3e2e46459632dad38bbcb8`.

Candidate 08's actual goal is `Coma.vc_scatter_segment_splice`, so the bare
filter `scatter_segment_splice` matched nothing. Why3find printed a file-level
`Proved (...)` line even though no goal had been selected. Treat that output as
an invalid/vacuous local attempt, not a body proof. The captured full wrapper
run selected the real goal and returned 13/16 children, with null children
3, 6, and 9. Astra reviewed those as the three pointwise gaps before
`Seq::ext_eq`.

Candidate 08 also had a separate exact-file run for
`impl_Cursor/read_scatter_prefix.coma`, without `-g`. It can show the body
against the helper's contract, but it does not discharge the helper body and
cannot be promoted to integrated candidate completion. Both local success
messages were withdrawn for candidate-level completion claims.

Use exact `Theory.goal` names for filtered diagnostics and verify the generated
`proof.json` includes the intended goal and all child results. The invalid
filtered helper command emitted `Proved (…scatter_segment_splice.coma) ✔`,
but had no per-goal summary or selected-goal count; compare this with the
full wrapper's explicit `Goal Coma.vc_scatter_segment_splice: ✘ (13/16)` line.
Do not infer success from the enclosing file's status line. A successful full
modified `verified,std` wrapper run is the only candidate-level
proof-completion signal.

## Candidate 08 proof configuration and result

The full wrapper used the private Why3 config with one prover, 1024 MiB memory,
and 5 seconds per prover, plus `why3find.json` time 1 second and depth 6. Its
actual feature configuration was `verified,std`, host target, with `sc-drf`
disabled. The full proof failed only at `Coma.vc_scatter_segment_splice`
(13/16); native tests passed 55/55. The exact source-matched archive and
checksums are listed in `README.md`.

## Candidate 09 accepted result

After adding pointwise sequence facts at the three candidate 08 extensionality
gaps, the full wrapper passed `Proved (308 files) ✔`. The independent audit
counted 308 `proof.json` files, 1,303 result leaves, and zero null JSON results.
The 85-file source manifest was unchanged through the proof. The separately
rehash-verified private standard package matches the reviewed tree
`878a63ae09dde611de547994499803158b6aa6157ce48f4332f645dff3edb155`
(110 files, 711,729 bytes); all nine reviewed affected files match. Native
tests passed 55/55 with the 4-to-2+2 scatter regression. Full run result,
feature graph, generated proof files, source, and logs are in the candidate 09
pass archive named in the evidence README.
