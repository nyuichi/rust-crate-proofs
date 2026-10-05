# UTF-8 lemma preparation checkpoint

## Scope and toolchain

This isolated probe uses the string-model compiler and matching Creusot
standard-library copy. It adds no httparse-specific trusted annotations or
axioms. Source translation and Why3 type checking passed; a separately
authorized direct Why3 run also proved all five finite scalar targets. The two
generic sequence-composition targets remain open.

Pinned inputs recorded for this translation:

| Input | Path | SHA-256 |
|---|---|---|
| Creusot compiler | `/workspace/proof-tools/targets/httparse-string-model/debug/creusot-rustc` | `a1ea923760d0225f828e90e2b405c4f7d3de4177868aad6f04f158472778f293` |
| Why3 config | `/workspace/scratch/httparse-string-model/why3.conf` | `e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07` |
| installed Creusot prelude | `/workspace/scratch/httparse-string-model/creusot-data/share/why3find/packages/creusot/creusot/prelude.coma` | `cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b` |
| copied `std/char.rs` | `/workspace/scratch/httparse-string-model/creusot-libs/creusot-std/src/std/char.rs` | `5d6108fd738ae1e3e84afcbff384ba7fb9a0cd9cfe0b16f7d8afb16e27d8ddb2` |
| copied `logic/seq.rs` | `/workspace/scratch/httparse-string-model/creusot-libs/creusot-std/src/logic/seq.rs` | `7eb7c51ac84cc7be2ab87a384979dd7aa50063d47fecc5b5cfcc5b434a11c70f` |
| copied `std/string.rs` | `/workspace/scratch/httparse-string-model/creusot-libs/creusot-std/src/std/string.rs` | `e51e9dd373e683c63c424aca31bb4850f0437e1c5c56c325c8590e46670957df` |

## Prepared targets

`src/lib.rs` contains five checked ghost proof candidates for exact scalar
encoding and two generic wrappers over the standard sequence flat-map laws.
The scalar cases assert the codepoint, encoded length, each byte index, and
extensional equality with the expected byte sequence:

| Target | Codepoint | Expected UTF-8 bytes |
|---|---:|---|
| `utf8_u00e9` | 233 | `195, 169` |
| `utf8_u20ac` | 8364 | `226, 130, 172` |
| `utf8_u_d7ff` | 55295 | `237, 159, 191` |
| `utf8_u_e000` | 57344 | `238, 128, 128` |
| `utf8_u10ffff` | 1114111 | `244, 143, 191, 191` |

`utf8_singleton` states that the UTF-8 bytes of a singleton character sequence
equal that character's encoding. `utf8_push_back` states that appending one
character appends its UTF-8 sequence to the existing byte sequence. The exact
seven generated modules are listed in `proof-targets.txt` and hashed in
`evidence/TRANSLATION-SHA256SUMS`.

## Translation, type-check, and scalar proof status

The isolated translation completed successfully and generated exactly the
seven listed COMA modules. Direct Why3 type-only checking completed successfully
on those same seven modules. Logs are in `evidence/translation.log` and
`evidence/typecheck.log`.

The five scalar targets returned 35/35 `Valid` split goals under Z3 4.15.3,
with no counterexamples. Exact per-target counts, goal timing, raw Why3 output,
frozen COMA inputs, and tool hashes are recorded in
`evidence/direct-why3-five-scalars-20261005/README.md`. These are finite
boundary cases only. The run used the imported standard `utf8_byte_spec`
numeric contract and `to_utf8_char_spec` length contract; it did not
independently prove the former.

The singleton and push-back wrapper targets have not been submitted to a
solver in this checkpoint. They remain open along with the literal `Seq::create`
caller obligations.

## Remaining proof obligations

The translated wrapper COMAs import the standard `flat_map_singleton` and
`flat_map_push_back` postconditions as antecedents. This probe does not include
or prove those standard-library helper VCs; the required helper-body evidence
must be added to the selected proof inventory before claiming the bridges are
closed. In particular, the copied singleton helper has an empty logic body and
an explicit postcondition, while the push-back helper has a recursive body.

The probe also has no literal caller yet. Follow-up targets must connect each
`Seq.create` literal model to a singleton/snoc sequence, apply these bridges and
the scalar encoding facts, and establish the exact expected byte sequence.
The five concrete scalar cases are boundary examples, not a universal theorem
for all valid Rust `char` values. The valid-scalar range and surrogate exclusion
remain properties of the isolated Char model; a general encoding proof is not
established here.

The isolated compiler, standard-library copy, Why3 config, and package are
preparation artifacts only. This checkpoint makes no adoption claim for the
active compiler or shared standard library.
