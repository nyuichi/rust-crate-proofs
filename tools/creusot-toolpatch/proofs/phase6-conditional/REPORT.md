# Phase 6 candidate status

The isolated Phase 6 candidate, based on `1c799dd`, adds the actual optimized
`u128::fmt` body and helper contracts. Its patch applies cleanly to current
main `999f8c3`, but it is preserved here as evidence only; no runtime source
changes from the candidate were integrated.

The focused formatter and trait-refinement targets completed at 146/146 and
1/1. The typed `u128` capacity helper completed at 1/1, and the initialized
slot-range helper completed at 4/4. A negative control with a false assertion
in the reachable nonzero-quotient branch failed at that assertion. Logs and
proof JSON are under `evidence/`.

The result is conditional on Phase 4: the formatter consumes arithmetic
contracts whose implementations, especially `mulhi`, remain unproved. The
candidate does not establish full crate integration, raw buffer formatting,
string conversion, or end-to-end `u128` formatting. See `README.md` for replay
steps and the complete proof boundary.
