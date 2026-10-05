# Replay of five archived UTF-8 scalar targets

This directory records a fresh Why3/Z3 replay of five frozen `.coma` targets recovered from the [`cloud recovery capsule`](../../../../../cloud-handoff/README.md). In the recovered work-in-progress bundle, their historical path was `verification/probes/utf8-lemmas/evidence/direct-why3-five-scalars-20261005/inputs/targets/`. The copied targets are byte-identical to those archived inputs; `inputs/ARCHIVE-SHA256SUMS` preserves their source-bundle checksums and `inputs/COMA-SHA256SUMS` records the five target hashes used here. The replay does not regenerate or edit those COMA files.

## Tool profile

The replay used the rebuilt string-model profile documented in [`tool-profile/PROFILE.tsv`](tool-profile/PROFILE.tsv), with the compiler, standard-library seed, Why3/why3find, prelude, configuration, and solver identities captured alongside it. The profile builder recipe is [`build-string-profile.sh`](../../../method-utf8/tools/build-string-profile.sh). To recreate the shell environment:

```sh
source /workspace/proof-tools/activate.sh
source /workspace/httparse-tool-rebuild/string-model/creusot-env.sh
```

The profile's Why3 configuration limits concurrent provers to one. Each proof was run sequentially while holding `/tmp/itoa-creusot-proof.lock`, with this command shape:

```sh
why3 prove -C "$WHY3CONFIG" \
  -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
  -L <this-directory>/inputs/targets <target>.coma \
  -a split_vc -P Z3,4.15.3 -t 30 -m 1000 --json
```

The exact target paths, stdout, stderr, and exit codes are preserved under `targets/`. Five corresponding `--type-only` runs succeeded with exit code 0 under `type-only/`.

## Results

| Archived scalar | Goals | Result |
| --- | ---: | --- |
| U+00E9 | 6 | 6 Valid |
| U+20AC | 7 | 7 Valid |
| U+D7FF | 7 | 7 Valid |
| U+E000 | 7 | 7 Valid |
| U+10FFFF | 8 | 8 Valid |
| **Total** | **35** | **35 Valid, no counterexamples** |

All five corrected runs exited with status 0. The first U+00E9 invocation included an unsupported `-j 1` option and stopped before proof execution; its command and error are retained at `targets/01-u00e9/attempt-01-invalid-cli/`. The corrected run and its six Valid goals are at `targets/01-u00e9/attempt-02/`.

Some stderr entries warn that source locations embedded in the frozen COMA refer to paths from the previous workspace. Those warnings did not prevent type checking or proof, and the raw stderr is retained with each run. This result covers only these five archived scalar targets under the recorded rebuilt profile; it is not a general proof of UTF-8 correctness.
