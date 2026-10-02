# BV128 shift trigger lemma proof

This scratch artifact records a derived Why3 lemma intended to help the existing
128-bit logical-right-shift axiom match Z3 terms. It leaves the original generic
`BV_Gen.to_uint_lsr` axiom unchanged and does not modify other bit-vector widths.

The added lemma is in `BV_Gen_Triggered`; `BV128` clones that module. Its formula
is exactly the existing `to_uint_lsr` formula, with an explicit trigger on
`to_uint (lsr_bv v n)`. The new declaration is a `lemma`, not an `axiom`.

The proof keeps only the cloned original axiom in the task context. The `clear_but`
Why3 transformation removes irrelevant context; it adds no assumptions. The
result is `Valid` under Z3 4.15.3. The copied `bv.mlw` and `proof.log` capture the
source and result. `bv128-trigger.patch` is the source diff from the narrow-cast
Why3 overlay.

Reproduce from the tool environment:

```sh
export RUSTUP_HOME=/tmp/rustup-home
export CARGO_HOME=/tmp/cargo-home
export CREUSOT_DATA_HOME=/tmp/creusot-data
export XDG_CONFIG_HOME=/tmp/why3-capture-config
export WHY3CONFIG=/tmp/why3-capture-config/creusot/why3.conf
export WHY3DATA=/tmp/why3-cast-lsr-bv128only
export PATH=/workspace/proof-tools/opamroot/creusot-v0.11/bin:/tmp/creusot-data/bin:/tmp/cargo-home/bin:$PATH
why3 prove -C "$WHY3CONFIG" \
  -L /tmp/why3-cast-lsr-bv128only/stdlib \
  -P 'Z3,4.15.3' -t 10 \
  -a 'clear_but to_uint_lsr' \
  /tmp/why3-cast-lsr-bv128only/stdlib/bv.mlw \
  -T BV_Gen_Triggered -G to_uint_lsr_triggered
```

Why3 reports: `Valid (0.09s, 199023 steps)`.
