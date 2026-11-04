# Source adaptation review

The pinned Aeneas translator cannot translate the original character-pattern
switch in `Iterator::next`. The failed original translation and its diagnostic
are retained separately; its generated `sorry` is not proof evidence.

The successful extraction uses `translation-source/src/lib.rs`. The complete
change is `char-pattern-to-guard.patch`: three constant-character patterns are
replaced with primitive character equality guards. No branch body changes.

An independent manual source review checked the following:

- Each guard accepts exactly the same character as its original pattern.
- Branch order, state assignments, and returned values are unchanged.
- The underlying iterator is called once, except when its first result is
  `Some('\n')` and the initial `prev_was_cr` is true, when it is called twice.
- Character equality is pure; short-circuiting `&&` reads `prev_was_cr` under
  the same condition as the original guard.
- The correspondence also preserves calls and state for non-fused iterators.

This review itself is manual. The later
[Kani equivalence harness](kani-equivalence/README.md) checks the original and
adapted bodies for one-step equality under a non-panicking two-reply iterator
trace model. It does not establish generic contextual equivalence for arbitrary
Rust iterator implementations. The Lean body proofs still apply to the adapted
extraction; the original character switches remain unsupported by the pinned
translator.

The root `src/lib.rs` remains byte-for-byte equal to the published source.
Its SHA-256 is `1ae29ba0393192db102a26bf8db05c15ed05adc7c4474ad1bde05ef2bc937225`.
The adapted source SHA-256 is
`f7d000418244c44b0ff534cd182d9ad013ba8912e81734148886dfef54853600`.
