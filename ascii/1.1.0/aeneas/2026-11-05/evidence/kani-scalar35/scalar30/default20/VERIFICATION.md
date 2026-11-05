# Inherited ASCII scalar comparisons20 — full input domains

The original std-provided ne/lt/le/gt/ge methods inherited by the four manual AsciiChar↔u8/char comparison implementations return exactly the corresponding inequality on numeric scalar codes. All20 fully qualified original calls passed Kani with zero failed checks. Every valid AsciiChar, every u8 and every valid Unicode scalar are covered, in both operand directions. This does not claim all ASCII APIs are completed.

These methods belong to manually implemented cross-type traits. They are distinct from the user's excluded derived AsciiChar↔AsciiChar comparisons and must remain in the strict API inventory. The direct eq/partial_cmp8 bodies are verified separately; these20 calls exercise their actual inherited defaults rather than inferring results from a wrapper-only proof.

The unchanged original source/Cargo are path dependencies. Inputs use a safe original constructor and a checked public code observation; source audit covers all128 enum variants. Unicode exclusions remove exactly invalid char encodings, rather than valid API inputs. Expected answers are integer relations. No source adaptation, new model, check suppression, buffer-length bound or heap assumption is used.

The actual std bodies are from Kani0.68/CBMC6.11/Rust nightly2026-08-21 on default x86_64. The inventory was collected with nightly2026-09-17; std source byte identity across those toolchains is not claimed. Other targets/configurations, const evaluation and invalid representations are outside this contract. Publication/source-card binding and primary claim counts remain separate gates.
