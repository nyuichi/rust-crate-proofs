# CLI compatibility scope

The installed publisher binary for the recorded contracts through ASCII21 has SHA-256 `4d95544e3074f243eaf17c480e59be3ff840e72789db174f9acf7bb3981c4751`. It is cargo-proofs 0.4.1 with the earlier batch patches plus the external-self local-trait-argument ownership fix (base16ct artifact evidence) and this directory's local-provided-default-method discovery patch. The patches affect API discovery/configuration compatibility; they do not add proof axioms or alter Lean's kernel, generated proof bodies or original Rust sources. The CA/proxy trust patch is an execution-environment accommodation.

The local-default patch associates only body-providing methods from crate-local traits with the actual source impl, preserving overrides and generic trait arguments. Required methods and foreign defaults are not fabricated. Selection continues to require exactly one source contract for each requested key.

Typed-self scanner work is tracked separately and was not installed for these records. Exact typed registration for the proposed endian66 extension remains a separate pending publication step; nominal catalogue aliases are not evidence for a distinct specialization.
