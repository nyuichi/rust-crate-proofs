# Body-proof dependency closure

The two frozen target modules are `utf8_byte.coma` and `to_utf8.coma`. Their
direct imports and the exact model files used by the rebuilt profile are:

| Why3 module | Model file in this bundle | Purpose |
|---|---|---|
| `creusot.int.UInt8` | `model/creusot-int.coma` | `u8` mathematical values and range obligations |
| `creusot.prelude.Char` | `model/creusot-prelude.coma` | Rust `char` model and scalar-value bounds |
| `seq.Seq` | `model/why3-seq.mlw` | Sequence length, singleton, snoc, and indexing contracts |
| `int.ComputerDivision`, `int.Int` | `model/why3-int.mlw` | Integer division, remainder, and arithmetic |
| `bv.BV8`, `bv.BV256`, converters | `model/why3-bv.mlw` | Transitive machine-byte and scalar-character representations |

The primary `to_utf8.coma` module includes `utf8_byte_def` and
`utf8_byte_spec` for calls to the helper. The separate `utf8_byte.coma`
module contains the recursive body goal that establishes the numeric-byte
contract for `0 <= value <= 255`; it passed all eight split VCs before the
`to_utf8` body was run. The `to_utf8` proof passed 14 split VCs.

The `Char.to_int` scalar bounds and sequence operation contracts are trusted
models from the pinned Creusot prelude and Why3 standard library. The model
files are copied and hashed in this bundle; their installed package and source
manifests are referenced by `toolchain-provenance.txt`. This proof does not use
the trusted `injective_to_utf8` or `injective_to_bytes` lemmas.

`closure-typecheck/` records a successful type-only check of the two selected
COMA files with only the evidence bundle as the local COMA load path. The raw
body solver commands also list the complete fresh translation output on the
Why3 load path; the selected modules' direct imports are closed by the copied
prelude and standard-library models above.
