# Conditional `div_rem_1e16` proof

Base commit: `51090b4`. The isolated `PHASE4_DIVREM_CONDITIONAL.patch` makes
the unchanged optimized divider body available to Creusot, with helpers for
the shift by 51 and magic quotient. The proof contracts cover quotient,
remainder, reconstruction, and remainder range. The patch introduces no
trusted local formatter or divider.

Focused proof results, each with a completed exit, are preserved under `logs/`:

- `shift_by_51`: 2/2.
- `magic_quotient`: 10/10.
- `math::magic_shift_floor`: 1/1.
- `div_rem_1e16`: 31/31.
- Negative control replacing a quotient assertion with `quot < 0`: 26/27.

The `div_rem_1e16` result is conditional: it consumes the postcondition of the
actual `u128_ext::mulhi` body, whose latest isolated attempt is 49/51. The
default and all-features `cargo check --offline` candidate builds passed, but
there is no candidate full-suite or native test claim here. Main-source suite
and native test logs are also in `logs/`; those validate only the unchanged
integrated source plus this evidence documentation.

The divider and `mulhi` patches were developed separately against the same
base and both touch `u128_ext.rs`. Do not apply them sequentially. Use the
replay steps in the parent `README.md` to reproduce either focused result.
