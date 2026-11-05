# ASCII character classification

Four original AsciiChar methods are proved over all 128 variants: digits are 0–9; blanks are space and tab; whitespace is space, tab, LF, CR and FF (vertical tab is false); controls are codes 0–31 and DEL. Generated bodies are unchanged, and the whitespace-to-blank call is covered.

Original Rust files and Cargo.toml are byte-identical to the archive. A modern compiler deny-by-default lint on upstream macro binding names is disabled with `-A bindings_with_variant_name`; no source adaptation is used. Each finite case is checked by Lean reduction, without native evaluation trust. The axioms audit permits only the usual three Lean axioms.

No claims cover conversions, strings, allocation, case conversion or unsafe paths. `./verify-all.bash` checks source provenance, runs original tests, regenerates and hash-compares translation, builds and audits the four theorems. See proofs.toml for precise applicability and assumptions.

The unchanged upstream `is_digit_strange_radixes` test fails with the pinned compiler: it calls `char::is_digit` at radix 0/1, which now panics. That separate `is_digit` API is outside the proof scope. `verify-all.bash` records the full failure, checks it has exactly this shape, then reruns the remaining original tests with only this test excluded. No source test is modified.
