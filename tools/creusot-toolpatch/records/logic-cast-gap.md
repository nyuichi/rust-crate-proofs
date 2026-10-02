# Logic casts and Rust MIR casts

The narrowing-cast fix in `creusot-narrowcast-backend.patch` only changes
`RValue::Cast` lowering in `creusot/src/backend/program.rs`. That is the path
used for a Rust cast in executable MIR. For an unsigned narrowing cast, it
lowers to mathematical remainder modulo `2^target_width`, then converts that
in-range result to the target type.

A cast in a contract or other logic term goes through
`TermKind::Cast` in `creusot/src/backend/term.rs`. That path still lowers a
bitwise cast as `TargetBW.of_BV256(SourceBW.to_BV256 value)` and a normal
integer cast as `Target.of_int(Source.t'int value)`. The `of_BV256` and
`of_int` constructors model checked embeddings; they are not Rust's total,
truncating `as` operation. The narrow-cast program patch does not make those
logic constructors truncating.

For a runtime cast whose semantic result needs to appear in a contract, state
the mathematical value directly and avoid writing another narrowing cast in
the contract. For example, a helper can retain the body
`(n >> 64i32) as u64` and promise `result@ == n@ / 2^64`. Then its body proof
checks the actual MIR cast against that result. This keeps the target
postcondition meaningful and does not add an axiom. The backend patch adds an
explicit `use int.ComputerDivision` dependency at the point where it emits
modulo, even if the contract itself does not mention `mod`. The isolated
`high_half_as_u64` witness passes 8/8 goals with the postcondition
`result@ == n@.div_euclid(64.pow2())`; its source has proved assertions for
nonnegativity, positive divisor, and the bitvector shift's quotient view.

There is a separate shift distinction. In normal integer mode, `n >> k` is
lowered to `UInt128.shr`; in bitwise mode it is `UInt128BW.shr`. The derived
`to_uint_lsr` theorem in the bundle applies to the bitwise BV128 model only.
It cannot justify the normal-mode shift. The actual bitwise shift witness in
`witnesses/ediv-shift/` is proved; Phase 4 `mulhi` and `div_rem_1e16` callers
remain work in progress.

Generated diagnostics confirmed that normal-mode shifts and bitwise-mode
shifts use different models, and that a cast written in a logic precondition
still uses the partial `UInt64BW.of_BV256 (UInt128BW.to_BV256 ...)` path. The
proved program-body witness and its generated `.coma` are retained under
`proofs/high-half/`.
