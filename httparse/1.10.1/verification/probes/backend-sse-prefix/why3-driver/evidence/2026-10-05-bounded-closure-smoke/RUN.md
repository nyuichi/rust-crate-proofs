# Bounded Nth-overlay closure smoke

Run date: 2026-10-05. Root granted the exclusive shared solver slot for this
bounded run. Each target is launched separately through `run-overlay-proof.sh`
and `run-proof.bash`; the latter holds `/tmp/itoa-creusot-proof.lock` and enforces
one prover with a 1000 MiB memory limit. Why3 gets a 30-second target timeout.

## Audited inputs

Both repository-root-relative manifests passed: the preserved 33-entry
positive-input snapshot and the refreshed 34-entry snapshot with the 10-second
negative runner. The source hash is
`a126cf9dcadcdd7e5e96ee3e4f0ebc1a8e78f6150ab59a18afd77d06e2bad90c`. The local
Nth-overlay driver hash is
`276ccd4557d0f4b9c5b625e67261e5d3789aa49874557524cfd2382765b8cdc8`; the
positive runner hash is `ea01aa7c026688d619baaf34493e19aeed1cdfa3cd894860594362cca0d710fa`,
and the 10-second negative runner hash is
`702d331b62178ff880f65973c5df2cefed3aaadde051d70dc9a36b2366da0933`. The
current base Why3 config hash is
`e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`. The
no-solver audit registered `Z3-NthOverlay 4.15.3` with generated-config hash
`fc85a35687e1e7e2fcae52f521970b2f67e963c844573052a548b56b078b14b8`.

## Dependency order and result

Run bottom-up. `prefix_len_from_mask` consumes both the complement lemma and the
trailing-zero bridge; do not run it unless both helper targets return `Valid`.
A standalone target result proves only that target body under its contracts. It
does not establish `why3find` crate-level closure or any runtime connection to
the scanner.

| Order | Target | Input Coma SHA-256 | Result |
| ---: | --- | --- | --- |
| 1 | `mask_complement_preserves_low_bits` | `a4e62f8adf018fefa37977e6f714bdba45640b0acb185d0a1379e484f3617bf5` | `Valid`; one VC, 14,646 steps |
| 2 | `trailing_zeros_shift_to_nth` | `188af68006994ac695cf49f601c078f8a2bee56b8a21655db7e4b9b02c48dfdd` | `Valid`; one VC, 488,496 steps |
| 3 | `prefix_len_from_mask` | `68d2f3112c8ac945ae0133d96ad4126cbdb94dc2ee04e67291b7d2cc12ac0142` | `Valid`; two VCs, 17,318 and 20,543 steps |
| 4 | `movemask_narrow_preserves_bits` | `2f0c5f747ff97b0ac190c3096a46e1fb56a0f51521a17ce952721df8afe2795e` | `Valid`; one VC, 269,961 steps |
| 5 | `u16_nth_constant_bit_smoke` | `dc761b0f164aae1e9c9f29e5bd13f1eaabf477416faa681a18250b6ecb5d6e94` | `Valid`; one VC, 10,984 steps |
| 6 | `u16_nth_out_of_bounds_smoke` | `07e0d39ca197d9a279cc748843bc1b9a4e4c03a6f673a1ca8a0825cd521d107b` | `Valid`; one VC, 6,313 steps |
| 7 | `u32_nth_boundary_smoke` | `7524ffa1f916d7a2bc3a56e34b98d8a938e2fd38d3ce95a1bedf71152cdc864d` | `Valid`; one VC, 14,035 steps |
| 8 | `uri_allowed_mask_16_sse` | `c2d42ae05df51be60fb86ba303914a69282d332f8381eeabac003add0bfc90f1` | `Valid`; 8 VCs, all valid |
| 9 | `match_uri_char_16_sse_pure` | `3623aa3413d9304eeeecaa491aa80cd043f0a0814f7eda10bba6b0abdc112a8c` | `Valid`; two VCs, 1,628 and 25,943 steps |

The separately authorized negative diagnostics used a 10-second per-goal limit.
Every direct Why3 JSON result was `Timeout`, with no counterexample model:

| Order | Target | Input Coma SHA-256 | Actual result |
| ---: | --- | --- | --- |
| 10 | `expected_invalid_u16_wrong_high_bit` | `ca04eaf498b6019704b0fa64327c11af2c54f30f8e6df15a3780865bf7f2c1e6` | Timeout, 45,927,811 steps, zero models |
| 11 | `expected_invalid_u16_width_index_wrap` | `ba802807bc29bad98af2618fdd31bdb161cc2fc6daa77c82a3485966a0c17bde` | Timeout, 53,296,579 steps, zero models |
| 12 | `expected_invalid_u32_negative_index_wrap` | `e0d494f58131abc1a779388d224f59448d3d387224db08afd2207d719c625e8a` | Timeout, 38,834,086 steps, zero models |
| 13 | `expected_invalid_u32_two_to_width_wrap` | `be6b6dca1f717aa3130807d432220ec580e2ad643efcbfb13533d0ed0314b3be` | Timeout, 47,032,647 steps, zero models |

No positive target was inconclusive. The four negative timeouts are not
counterexamples and are not reported as `Invalid` or SAT. The direct Why3
CLI stdout/stderr and `--json` outputs are retained. The target-local
`proof.json` and `why3session.xml` predate this run and contain older attempts;
do not treat or archive them as fresh results. `RESULTS.json` and `RESULTS.tsv`
are deterministic summaries extracted from direct CLI logs, not Why3 session
files or `why3find` proof JSON. The raw direct CLI logs are authoritative.
`WHY3-COMMANDS.txt` retains each exact command line; `COMMANDS.sha256` records
command-file hashes; `ARTIFACTS.sha256` records the saved logs, summaries, and
snapshot manifests. Targets outside this exact 13-run batch remain Pending.

## Caller dependency scope

The authorized follow-up runs the narrowing lemma and u16/u32 bit-index
boundary targets before `uri_allowed_mask_16_sse`. The caller's positive result
depends on the narrowing lemma, the existing scalar helper contracts, and the
local `_mm_*` extern specifications over opaque `__m128i` lanes. Those extern
specifications are trusted assumptions in this probe. The caller body check
does not verify the standard-library intrinsic implementations, a runtime
unaligned load, CPU feature detection or dispatch, or the scanner's call path.
`match_uri_char_16_sse_pure` returned `Valid` for both its lane-type invariant
and final ensures VCs. This proves the extracted composition body under the
proved URI-mask and prefix contracts. The trusted intrinsic assumptions and
unverified runtime scanner path remain outside this closure. No other negative
diagnostics were run in this grant.
