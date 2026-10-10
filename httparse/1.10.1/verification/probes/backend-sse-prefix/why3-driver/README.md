# Target-local Why3 bit-index overlay

This directory contains a diagnostic copy of the pinned Z3 driver and its two
bitvector generator files. It restores selected standard Why3 `nth` axioms that
the installed driver removes. It adds no axioms and changes no installed Why3
files. This is a probe of the proof path, not a crate runtime contract or proof
that SSE scanner code is linked to machine instructions.

## Why the copy is needed

The stock Why3 library defines `nth` as a logical predicate over a bitvector
and integer index (`stdlib/bv.mlw`). The installed SMT generator removes
`nth_out_of_bound` and `Nth_bw_*`; the Z3 generator removes
`Nth_bv_is_nth`/`Nth_bv_is_nth2` for the supported bitvector widths. It also
removes shift-to-`nth` properties used by the standard `trailing_zeros_logic`
contract. A stock-driver SMT dump of the complement lemma therefore declared
`nth` as an uninterpreted function, with no relation to `bvnot`.

The local generators comment out only those removals. For bitwise operations,
the restored properties include `Nth_bw_not`; for bounds they include
`nth_out_of_bound`; the Z3-width clones preserve `Nth_bv_is_nth` and
`Nth_bv_is_nth2`; shift-related `Nth_*` properties are retained so the standard
trailing-zero model can be bridged without a local trusted algorithm. The
standard library axioms remain the TCB. The input source and the generated
`.coma` files use `#[bitwise_proof]`, so the affected values are represented as
`UInt16BW`/`UInt32BW`, not the separate abstract numeric model.

## Local files and source identities

The imported stock files are under
`/workspace/proof-tools/creusot-data/_opam/share/why3/`:

| File | SHA-256 |
| --- | --- |
| `drivers/z3_4_12.drv` | `646cefdba2dd94289ab584f65cbe583cd69f4a6ce1e1bfdf2c249fd794d87f47` |
| `drivers/smt-libv2-bv.gen` | `f3f0657f75b3374bd4bd37e47e1160b83cc28eba348c1cc0793b5547b72b16b9` |
| `drivers/z3_bv.gen` | `a3607c94ce344ca28b85925ac3ede6df1970d5eaea8cee6e0987dc0e7409b4d8` |
| `stdlib/bv.mlw` | `d0efb959305e4afed43873c7154b84b9b89f21ec21c5f377607666325e14339d` |

The target-local copies currently have these identities:

| File | SHA-256 |
| --- | --- |
| `z3_nth_overlay.drv` | `276ccd4557d0f4b9c5b625e67261e5d3789aa49874557524cfd2382765b8cdc8` |
| `smt-libv2-bv-nth.gen` | `3ef3905a1ff73abca30cdb3631ff04789703bc897abd637b3a74f266f316ead3` |
| `z3-bv-nth.gen` | `44f999093ce6b5054dd47c2cc71f8405467c44374e148f54d8673c7625e6318e` |
| `run-overlay-proof.sh` | `ea01aa7c026688d619baaf34493e19aeed1cdfa3cd894860594362cca0d710fa` |

The checked tool versions were Why3 `1.8.2+git` and Z3 `4.15.3` (64 bit).

At the earlier checked complement run, the base Why3 configuration was
`/workspace/proof-tools/config/creusot/why3.conf`, SHA-256
`22eee6c9a8d3f63ef6c0e030156997474cd2002d48ee05cc235a30f294e6c7ed`. The
generated local configuration had SHA-256
`0a72aa93eb5feb4c697d1e2a1e6af3b8cecd1527abff21647b8a7c5eb9f31c87`.
At the latest source-preparation check, the base config hash is
`e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`; both
observed base configurations specify `running_provers_max = 1`,
`memlimit = 1000`, and `timelimit = 5`. The runner derives a temporary
configuration from the currently active base, adds exactly one
`Z3-NthOverlay 4.15.3` entry with an absolute path to this local driver, and
invokes Why3 with `-t 30 -m 1000`. The generated local configuration is removed
on exit. `--audit-config` prints its current hash and prover stanza, verifies
that it names this driver, prints the target hash, and exits without launching
a prover. The current no-solver audit for `movemask_narrow_preserves_bits`
reported generated-config SHA-256
`fc85a35687e1e7e2fcae52f521970b2f67e963c844573052a548b56b078b14b8`, base
config SHA-256 `e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`,
and target Coma SHA-256
`2f0c5f747ff97b0ac190c3096a46e1fb56a0f51521a17ce952721df8afe2795e`.

## Tool and package identities

The no-`--no-check-version` translation command succeeds, so the version-check
bypass is unnecessary and is omitted from the reproducible command. The
earlier translation invocation included it by habit. Its documented effect is
to allow a mismatched `creusot-std`; here `cargo-creusot` and the path-dependent
`creusot-std` are both `0.11.0-dev`, as is `creusot-std-proc` in the generated
lockfile. This does not claim a mismatch was accepted.

The translation command, from the standalone probe directory, is:

```sh
source /workspace/proof-tools/activate.sh
CARGO_NET_OFFLINE=true cargo creusot --simple-triggers=false
```

This command only builds and translates; it does not start Why3. Current binary
and source identities are:

| Artifact | SHA-256 / version |
| --- | --- |
| `cargo-creusot` | `254165917300ddfe9f27bf1b40b4bedbef6012b71b58839e7b25c7b80fab31b5`; `0.11.0-dev` |
| compiler `rustc` | `442ef919ace325b6dee949fbce1acdfe26a0f6cb9239c60860dbd722ad1ddd96`; `1.95.0-nightly (6a979b3e3, 2026-02-26)` |
| rustup `rustc` shim | `dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71` |
| `why3` | `cced91edcb7d8edfb3540f060c1f4609cd02f69bc0e48bc22f8e35d58731bdff`; `1.8.2+git` |
| `z3` | `80ee070b8ffc2fa964b0b879250b2110c47f84c1256a1f9ab76fc7cf95e46292`; `4.15.3` |
| probe `Cargo.toml` | `0c566a8d7faa1ef10ba46b3e37ecc5d2a408b9a2047bc41e3e0ac9cce9e92ad5` |
| probe `Cargo.lock` | `b40c00134515c26e8928813e64657690572aceca142c6e802497e8dd6b20894c` |
| `creusot-libs/Cargo.toml` | `af2583aedceb2ab1d57d797645ed8bcef450fceabf5a93e756f7db33588ad249` |
| `creusot-std/Cargo.toml` | `2a1c605ffee3325212b0eaeaa32c462fc5cb8e439ddf4522be78a79c3b58d837` |
| `creusot-std/src` file-set hash | `d785c00bd397d3b6bbd3242de74107173dbbbb41a7d019fed560d26d407ece2b` |
| `creusot-std-proc/src` file-set hash | `58ce496cdcc071c965adc1e2a9695c75cf00aa35e9c894ac7fcade76405dc3f1` |
| current probe `src/lib.rs` | `a126cf9dcadcdd7e5e96ee3e4f0ebc1a8e78f6150ab59a18afd77d06e2bad90c` |

Each `src` file-set hash is the SHA-256 of the output from
`find <directory> -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum`.

## Checked target

The corrected proof invocation was run from
`/workspace/rust-crate-proofs/httparse/1.10.1` through
`run-proof.bash`, which acquired the shared proof lock. Its command was:

```sh
source /workspace/proof-tools/activate.sh
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh \
  mask_complement_preserves_low_bits
```

The runner selected this generated input:

```text
verification/probes/backend-sse-prefix/verif/httparse_backend_sse_prefix_probe_rlib/sse_prefix/mask_complement_preserves_low_bits.coma
SHA-256 a4e62f8adf018fefa37977e6f714bdba45640b0acb185d0a1379e484f3617bf5
```

The generated task manifest was nonempty: one split sub-goal for
`vc_mask_complement_preserves_low_bits`, proving the quantified relation
`(!value).nth_bit(i) == !value.nth_bit(i)` for `0 <= i < 16`. Z3 reported
`Valid (0.01s, 14646 steps)`. This proves the pure bitvector lemma against the
restored standard Why3 theory. It does not prove the body of an SSE scanner or
establish a runtime refinement relation for any intrinsic.

One earlier invocation passed `-o` to `why3 prove`. Why3 documents that mode as
printing selected goals; despite exit 0, it did not launch a prover and is not
counted as proof evidence.

## Overlay verification snapshot

The source is `../src/lib.rs`; generated Coma targets are in
`../verif/httparse_backend_sse_prefix_probe_rlib/sse_prefix/`. The current
source SHA-256 is `a126cf9dcadcdd7e5e96ee3e4f0ebc1a8e78f6150ab59a18afd77d06e2bad90c`.
The caller Coma hashes are `uri_allowed_mask_16_sse.coma`:
`c2d42ae05df51be60fb86ba303914a69282d332f8381eeabac003add0bfc90f1`,
`prefix_len_from_mask.coma`:
`68d2f3112c8ac945ae0133d96ad4126cbdb94dc2ee04e67291b7d2cc12ac0142`, and
`match_uri_char_16_sse_pure.coma`:
`3623aa3413d9304eeeecaa491aa80cd043f0a0814f7eda10bba6b0abdc112a8c`.

[`SNAPSHOT-HASHES.sha256`](SNAPSHOT-HASHES.sha256) now records the current
source, probe manifests, both target-local runners, the overlay files, and all
25 generated Coma files. Check it from the repository root with:

```sh
sha256sum -c httparse/1.10.1/verification/probes/backend-sse-prefix/why3-driver/SNAPSHOT-HASHES.sha256
```

The original 33-entry input manifest used for the 30-second positive runs is
preserved at
[`evidence/2026-10-05-bounded-closure-smoke/SNAPSHOT-POSITIVE-HASHES.sha256`](evidence/2026-10-05-bounded-closure-smoke/SNAPSHOT-POSITIVE-HASHES.sha256).
The refreshed manifest adds the 10-second negative runner. The 30-second
positive runner hash is `ea01aa7c026688d619baaf34493e19aeed1cdfa3cd894860594362cca0d710fa`;
the negative runner hash is
`702d331b62178ff880f65973c5df2cefed3aaadde051d70dc9a36b2366da0933`.

The fresh direct Why3 run record, command files, logs, log-derived JSON and TSV
summaries, and their hashes are under
[`evidence/2026-10-05-bounded-closure-smoke/`](evidence/2026-10-05-bounded-closure-smoke/).
`RESULTS.json` and `RESULTS.tsv` there are deterministic summaries extracted
from direct Why3 CLI logs. They are not Why3 session files or `why3find` proof
JSON; the raw CLI logs remain authoritative. The pre-existing target-local `proof.json` and `why3session.xml` files
contain older attempts and are not fresh evidence for these runs.

All nine positive targets returned `Valid` across 18 Why3 VCs with the local
Nth-overlay driver. The established positive dependency path is:

1. `mask_complement_preserves_low_bits` and `trailing_zeros_shift_to_nth`
   establish the bit-index facts.
2. `prefix_len_from_mask` consumes both helper contracts and proves its bounds.
3. `movemask_narrow_preserves_bits` and the u16/u32 bit-index boundary targets
   establish narrowing and width behavior.
4. `uri_allowed_mask_16_sse` proves the extracted 16-lane URI mask body under
   five local `_mm_*` `extern_spec` contracts over opaque `__m128i` lanes.
5. `match_uri_char_16_sse_pure` composes the URI mask and prefix bodies.

| Target | Result |
| --- | --- |
| `mask_complement_preserves_low_bits` | **Valid**; 1 VC, 14,646 steps |
| `trailing_zeros_shift_to_nth` | **Valid**; 1 VC, 488,496 steps |
| `prefix_len_from_mask` | **Valid**; 2 VCs, 17,318 and 20,543 steps |
| `movemask_narrow_preserves_bits` | **Valid**; 1 VC, 269,961 steps |
| `u16_nth_constant_bit_smoke` | **Valid**; 1 VC, 10,984 steps |
| `u16_nth_out_of_bounds_smoke` | **Valid**; 1 VC, 6,313 steps |
| `u32_nth_boundary_smoke` | **Valid**; 1 VC, 14,035 steps |
| `uri_allowed_mask_16_sse` | **Valid**; 8 VCs, all valid |
| `match_uri_char_16_sse_pure` | **Valid**; 2 VCs, 1,628 and 25,943 steps |

The four authorized negative diagnostics were run separately with direct Why3,
`--json`, and a fixed 10-second timeout. Each timed out without a model. These
results are inconclusive; they are not counterexamples and are not reported as
`Invalid` or SAT.

| False claim target | Actual result | Counterexample models |
| --- | --- | ---: |
| `expected_invalid_u16_wrong_high_bit` | Timeout at 10s, 45,927,811 steps | 0 |
| `expected_invalid_u16_width_index_wrap` | Timeout at 10s, 53,296,579 steps | 0 |
| `expected_invalid_u32_negative_index_wrap` | Timeout at 10s, 38,834,086 steps | 0 |
| `expected_invalid_u32_two_to_width_wrap` | Timeout at 10s, 47,032,647 steps | 0 |

Targets outside the exact 13-run batch recorded in `RUN.md` remain Pending.
The timeout JSON and full command logs are retained per target. Do not treat an
export-only task dump as a proof or a timeout as an actual solver model.

The positive results prove the extracted function bodies against their
translated contracts. The local vector intrinsic `extern_spec`s remain trusted
assumptions; these runs do not prove the standard-library intrinsic
implementations, the unaligned load, CPU feature checks and dispatch, AVX2,
NEON, or the runtime connection to httparse's scanner. These standalone Why3
results also do not establish `why3find` crate-level closure.
