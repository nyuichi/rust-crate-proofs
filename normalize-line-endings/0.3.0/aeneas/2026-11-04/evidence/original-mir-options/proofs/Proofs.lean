import Generated.NormalizeLineEndings
import NormalizationSpec

open Aeneas Aeneas.Std Result
open normalize_line_endings

namespace Proofs

/-- The underlying iterator model returns the next character and consumes one
    list cell; once empty it continues to return None. -/
def listIterator : core.iter.traits.iterator.Iterator (List Char) Char where
  next xs := .ok (match xs with
    | [] => (none, [])
    | c :: rest => (some c, rest))

abbrev adapterNext := Normalized.Insts.CoreIterTraitsIteratorIteratorChar.next listIterator

/-- Direct refinement of the generated (adapted Rust) next body. -/
theorem next_model (prev : Bool) (xs : List Char) :
    adapterNext { iter := xs, prev_was_cr := prev } =
      let (o, b, rest) := NormalizationSpec.step prev xs
      .ok (o, { iter := rest, prev_was_cr := b }) := by
  cases xs with
  | nil => simp [adapterNext, Normalized.Insts.CoreIterTraitsIteratorIteratorChar.next,
      listIterator, NormalizationSpec.step]
  | cons c xs =>
    by_cases hc : c = '\n'
    · subst c
      cases prev with
      | false => simp [adapterNext, Normalized.Insts.CoreIterTraitsIteratorIteratorChar.next,
          listIterator, NormalizationSpec.step]
      | true =>
        cases xs with
        | nil => simp [adapterNext, Normalized.Insts.CoreIterTraitsIteratorIteratorChar.next,
            listIterator, NormalizationSpec.step]
        | cons d xs =>
          by_cases hd : d = '\r' <;>
            simp [adapterNext, Normalized.Insts.CoreIterTraitsIteratorIteratorChar.next,
              listIterator, NormalizationSpec.step, hd]
    · by_cases hr : c = '\r' <;> cases prev <;>
        simp [adapterNext, Normalized.Insts.CoreIterTraitsIteratorIteratorChar.next,
          listIterator, NormalizationSpec.step, hc, hr]

/-- Observe repeated calls to the generated next body. Fuel only makes this
    observer executable; the final theorem covers every finite input length. -/
def run : Nat → Normalized (List Char) → Result (List Char)
  | 0, _ => .ok []
  | n + 1, state => do
    let (o, state') ← adapterNext state
    match o with
    | none => .ok []
    | some c =>
      let rest ← run n state'
      .ok (c :: rest)

theorem run_model (fuel : Nat) (prev : Bool) (xs : List Char) :
    run fuel { iter := xs, prev_was_cr := prev } =
      .ok (NormalizationSpec.observe fuel prev xs) := by
  induction fuel generalizing prev xs with
  | zero => rfl
  | succ n ih =>
    simp only [run, next_model]
    generalize h : NormalizationSpec.step prev xs = st
    obtain ⟨o, b, rest⟩ := st
    cases o <;> simp [NormalizationSpec.observe, h, ih]

/-- The public constructor starts in the non-CR state and preserves the
    underlying iterator without calling it. -/
theorem normalized_initial {I : Type} (inst : core.iter.traits.iterator.Iterator I Char)
    (iter : I) :
    normalized inst iter = .ok { iter, prev_was_cr := false } := by
  rfl

/-- For every finite character list and every sufficiently long observation,
    the adapted Rust constructor and next body implement CR/LF/CRLF
    normalization, without failure. Its output has no CR, never grows, and is
    unchanged by another normalization pass. -/
theorem normalized_spec (xs : List Char) (fuel : Nat) (enough : xs.length < fuel) :
    (do let state ← normalized listIterator xs; run fuel state) =
      .ok (NormalizationSpec.normalize xs) ∧
    '\r' ∉ NormalizationSpec.normalize xs ∧
    NormalizationSpec.normalize (NormalizationSpec.normalize xs) =
      NormalizationSpec.normalize xs ∧
    (NormalizationSpec.normalize xs).length ≤ xs.length := by
  refine ⟨?_, NormalizationSpec.normalize_no_cr xs,
    NormalizationSpec.normalize_idempotent xs, NormalizationSpec.normalize_length_le xs⟩
  simp [normalized, bind_tc_ok, run_model,
    NormalizationSpec.observe_normalizes fuel xs enough]

end Proofs

#print axioms Proofs.normalized_spec
