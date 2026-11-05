# Original AsciiChar conversion2 — full input domains

Both manual macro-generated From implementations preserve the exact ASCII code: converting any valid AsciiChar to u8 returns its discriminant; converting to char returns the same Unicode scalar value. Two original-body Kani harnesses succeed, four checks each and zero failures.

The safe original constructor creates every valid input using symbolic code0..127 and its public observation is checked. Source/archive audit confirms that the repr(u8) enum has exactly those128 discriminants. The code<128 assumption describes the entire valid enum domain; it is not a restriction to a fixed buffer or a sample of enum variants. The target conversion calls are fully qualified original trait calls, not replacement helpers. These manually written macro implementations are not standard derive exclusions.

The unchanged original crate and manifest are path dependencies. No source adaptation, standard model or check suppression was introduced. Kani0.68/CBMC6.11 and Rust nightly2026-08-21 compiler/standard semantics are trusted; only default features on x86_64 were run. Const evaluation, all targets and arbitrary invalid enum bit patterns are not claimed. Aeneas's previously failed macro-root route was not retried. Publication/source-card binding and any primary claim-count increase are separate gates.
