# Original nohash-hasher 0.2.0 integer writer behavior

The checked models establish the exact state transitions of the original
`NoHashHasher` methods in both default-feature debug and release builds.
`Default::default` starts with stored value zero. `Clone::clone` preserves the
state, and `Hasher::finish` returns its stored `u64`. Each of the ten integer
writers stores its argument converted to `u64`: unsigned inputs are zero-extended
and signed inputs sign-extended to 64 bits. Debug builds accept a first integer
write and report assertion failure on another; release builds accept repeated
writes and retain the latest converted value. The generic byte-slice `write`
always reports a panic in both builds.

These are method-body proofs over arbitrary input values and states, with no
finite testing bound. No separate caller-body or HashMap/HashSet integration
proof is claimed. The original Rust and normalized crates.io Cargo manifest are
retained byte-for-byte; there is no verification fork, source adaptation, IR
rewrite, or local translator patch.

## Scope and representation

The target is x86_64-unknown-linux-gnu with default `std` features. The debug
model represents the private fields as `(stored u64, already-written Bool,
PhantomData-as-Unit)`; the release model omits the Bool. `IsEnabled` is an empty
marker in both models. Writer contracts accept an arbitrary marker instance;
they do not establish that a user-defined `Hash` implementation obeys its
one-write requirement.

Fourteen selected method bodies are proved per build: handwritten Default and
Clone, byte write, finish, and write_u8/u16/u32/u64/usize/i8/i16/i32/i64/isize.
`proofs/Proofs.lean` conjoins the debug and release contracts into one theorem per
API. Its leaves are `DebugProofs.lean` and `ReleaseProofs.lean`. Integer results
are specified explicitly as 64-bit zero/sign extensions rather than a generic
claim that the cast is correct. Assertion failures and explicit panic are
separate errors in Aeneas's model.

The handwritten Debug formatter, full `Hasher` trait instance/default methods
(including u128/i128 writers), BuildHasherDefault, collection aliases and
collection integration are outside scope. Whole-instance extraction encounters
an incomplete builtin Hasher model, and formatting encounters a translator
borrow-model limitation. Official Charon `--start-from` selects the fourteen
original methods without altering their bodies. The release extraction uses
Cargo `--release`; official Aeneas `-namespace nohash_hasher_release` separates
its generated names so both configurations can be checked together. `no_std`
and other targets are not covered.

The Rust compiler, official Charon/Aeneas translations, Aeneas standard-library
models and Lean kernel are trusted. No custom assumptions or local axioms are
introduced. Every public theorem's transitive axiom audit permits only propext,
Classical.choice and Quot.sound.

## Evidence and reproduction

Run `./verify-all.bash` after installing the pinned tools:

- Aeneas be86ab0f1989d04944292b345b24c435be3ad10a.
- Charon 08d7c9d50144bd84848212c24701253585e90d1f.
- Rust nightly-2026-09-17; Lean 4.35.0.

The script checks archive/source equality, runs original debug/release tests,
re-extracts both models and verifies their recorded generated-file hashes,
builds the combined proof project and audits all fourteen public theorems.
`AENEAS_HOME` may select a tool installation with an `activate.sh`; otherwise
`charon`, `aeneas`, `cargo` and `lake` must be on PATH.

`evidence/upstream.crate` establishes source provenance.
`source-provenance.log` and `source-generated.sha256` bind source and proof
artifacts. `extraction-command.json` and `aeneas-command.json` (also under
`evidence/release/`) record exact official options. `verify-all.log`,
`verification.log`, `axioms.log` and the original test logs record successful
checks. `proofs/Generated/` and `proofs/ReleaseGenerated/` contain untouched
Aeneas outputs and both translation manifests. No unfinished proof is included.

| Component | Contract reviewed | Body proved | Local trust | Integrated run |
| --- | --- | --- | --- | --- |
| Debug methods (14) | yes | yes | none | passed |
| Release methods (14) | yes | yes | none | passed |
| Combined public theorems (14) | yes | yes | none | passed |
| Formatter / trait integration | outside scope | no | not assumed by claims | not run |

Publication uses cargo-proofs 0.4.1 with [environment CA trust](evidence/cargo-proofs-environment-ca.patch) and the [upstream Lean audit variable rename](evidence/cargo-proofs-audit-compat.patch). The theorem checks and allowed-axiom policy are unchanged.
