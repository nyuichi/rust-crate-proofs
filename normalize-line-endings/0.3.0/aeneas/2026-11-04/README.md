# CR/LF/CRLF normalization for finite character lists

For every finite character list, the verified adaptation of
`normalize-line-endings` 0.3.0 replaces each CRLF pair and each remaining CR with
LF and preserves other characters. Its output contains no CR, never grows, and
is unchanged by another normalization pass. The theorem quantifies over all
finite input lengths; its observation budget must exceed the input length.

The proof directly calls the Aeneas-generated `normalized` constructor and
`Iterator::next` body. The underlying iterator is a total, fused character-list
model. Arbitrary external iterators, including their panics, divergence, or
resumption after `None`, are outside this end-to-end guarantee.

The root `src/lib.rs` is the unchanged published source. The pinned Aeneas
translator cannot handle its character-pattern switches, so extraction uses
`translation-source/src/lib.rs`, which replaces three character patterns with
equality guards. Kani now checks the actual original and adapted `next` bodies
for one-step equivalence: returned value, adapter state, callback count, and
remaining modeled iterator state agree for both prior-state flags and every
possible pair of `Option<char>` replies. Callback updates to a modeled `u64`
state are also arbitrary. This includes non-fused reply sequences.

The two-reply bound covers the maximum calls made by one `next`; it is not a
bound on input string length. This supplementary check is limited to
non-panicking callback traces. It does not prove generic contextual equivalence
for every Rust iterator, its panics or divergence, or an entire execution
history. The end-to-end Aeneas theorem remains a proof of the adapted extraction
under the total fused list iterator model, rather than a direct extraction proof
of the unmodified release.

Existing optimized-MIR options did not remove the original unsupported switch.
An [isolated investigation](evidence/original-mir-options/README.md) also
records a successful external IR-normalization experiment. That new pass is
unverified and is not used by the published proof.

## Evidence

- [Published source and archive provenance](evidence/UPSTREAM.md).
- [Complete adaptation patch](evidence/char-pattern-to-guard.patch) and
  [independent manual review](evidence/ADAPTATION-REVIEW.md).
- [Kani source-equivalence harness, limits, and results](evidence/kani-equivalence/README.md).
- [Generated bodies](proofs/Generated/NormalizeLineEndings.lean) and
  [translation manifest](proofs/Generated/translation.json).
- [Public theorem and body refinement](proofs/Proofs.lean), supported by the
  [independent list specification](proofs/NormalizationSpec.lean).
- [Successful integrated Lean build](evidence/lean-build.log) and
  [transitive axiom listing](evidence/axioms.log). Only Lean's `propext`,
  `Classical.choice`, and `Quot.sound` occur; there is no `sorryAx`.

The failed original translation is archived in
`evidence/original-translation/` and `evidence/aeneas.log`. Its generated `sorry`
is not used by the checked proof.

## Reproduce

The pinned versions are Aeneas
`be86ab0f1989d04944292b345b24c435be3ad10a`, Charon
`08d7c9d50144bd84848212c24701253585e90d1f`, Rust
`nightly-2026-09-17`, and Lean `4.35.0`. The proof package uses the official
Aeneas Git dependency at this exact revision.

Run `./verify-all.bash` from this directory with the pinned tools available.
For a Lean-only replay, run `lake build` from `proofs/`.

For proofs.rs, run from `proofs/` in a clean, published Git checkout:

```sh
cargo proofs run --manifest-path ../Cargo.toml -- lake build
cargo proofs publish --manifest-path ../Cargo.toml --dry-run
cargo proofs publish --manifest-path ../Cargo.toml
```

`cargo-proofs` 0.4.1 uses two compatibility patches:
[environment CA trust](evidence/cargo-proofs-environment-ca.patch), preserving
TLS verification, and [an audit-local variable rename](evidence/cargo-proofs-audit-compat.patch)
from `name` to `theorem_name` to avoid a Lean 4.35 parser conflict. The latter
matches the official upstream audit at commit
[`74acff732c94bc23142913cd03d398b40648fc27`](https://github.com/proofs-rs/proofs-rs/blob/74acff732c94bc23142913cd03d398b40648fc27/cli/src/aeneas.rs).
The theorem-existence checks and the allowed-axiom policy are unchanged.

The trusted compiler/translation/model/kernel boundary is described in
`proofs.toml`. No derived standard-trait implementations occur in this crate;
the handwritten iterator implementation is included in the body proof.
