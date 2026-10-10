# Checked UTF-8 encoding lemmas

This isolated probe uses the existing string-model compiler, standard-library
copy, Why3 package, and configuration. It contains finite `#[check(ghost)]`
proofs for `CharExt::to_utf8` at five Unicode scalar boundaries, plus generic
singleton and push-back `Seq<char>::to_bytes` composition wrappers. The five
scalar targets have been proved; the wrappers remain unproved. It adds no
trusted contracts or httparse-specific axioms.

The seven selected target modules are listed in `proof-targets.txt`: five
scalar bodies (`U+00E9`, `U+20AC`, `U+D7FF`, `U+E000`, and `U+10FFFF`) and the
two generic flat-map bridges. Each scalar helper checks its codepoint value,
UTF-8 sequence length, every byte index, and extensional equality with the
expected byte sequence. `typecheck` fails if the generated COMA inventory
contains a missing or extra module.

The runner supports native checking, Creusot translation, and Why3 type-only
checking. Scalar solver results are recorded in
`evidence/direct-why3-five-scalars-20261005/README.md`, with raw per-target
output and frozen inputs in that bundle. Translation and type-only checking
also passed. The next proof split can use the two bridge postconditions to
connect a character sequence built with `singleton`/`push_back` to the
existing flat-map string byte model. It still needs callers that connect
actual literal `Seq::create` models to those finite character sequences, then
the exact string byte goals.
