import Std.Tactic
/- A finite character-list specification, independent of iterator implementation. -/
set_option linter.unusedSimpArgs false
set_option linter.unusedVariables false

namespace NormalizationSpec

def normalize : List Char → List Char
  | [] => []
  | '\r' :: '\n' :: xs => '\n' :: normalize xs
  | '\r' :: xs => '\n' :: normalize xs
  | c :: xs => c :: normalize xs

def skipLF : List Char → List Char
  | '\n' :: xs => xs
  | xs => xs

def output (prev : Bool) (xs : List Char) : List Char :=
  normalize (if prev then skipLF xs else xs)

def step (prev : Bool) (xs : List Char) : Option Char × Bool × List Char :=
  match xs with
  | [] => (none, false, [])
  | c :: rest =>
    if c = '\n' ∧ prev = true then
      match rest with
      | [] => (none, false, [])
      | d :: tail =>
        if d = '\r' then (some '\n', true, tail)
        else (some d, false, tail)
    else if c = '\r' then (some '\n', true, rest)
    else (some c, false, rest)

 theorem normalize_cr (xs : List Char) :
    normalize ('\r' :: xs) = '\n' :: normalize (skipLF xs) := by
  cases xs with
  | nil => rfl
  | cons c xs =>
    by_cases h : c = '\n'
    · subst c; rfl
    · simp [normalize, skipLF, h]

 theorem normalize_other (c : Char) (xs : List Char) (h : c ≠ '\r') :
    normalize (c :: xs) = c :: normalize xs := by
  simp [normalize, h]

 theorem step_output (prev : Bool) (xs : List Char) :
    output prev xs =
      match step prev xs with
      | (none, _, _) => []
      | (some c, b, rest) => c :: output b rest := by
  cases xs with
  | nil => cases prev <;> rfl
  | cons c xs =>
    by_cases hc : c = '\n'
    · subst c
      cases prev with
      | false => simp [step, output, normalize, skipLF]
      | true =>
        cases xs with
        | nil => rfl
        | cons d xs =>
          by_cases hd : d = '\r'
          · subst d
            simp [step, output, skipLF, normalize_cr]
          · simp [step, output, skipLF, hd, normalize_other d xs hd]
    · by_cases hr : c = '\r'
      · subst c
        cases prev <;> simp [step, output, skipLF, normalize_cr]
      · cases prev <;> simp [step, output, skipLF, hc, hr, normalize_other c xs hr]

 theorem step_progress (prev : Bool) (xs : List Char) (c : Char) (b : Bool)
    (rest : List Char) (h : step prev xs = (some c, b, rest)) :
    rest.length < xs.length := by
  cases xs with
  | nil => simp [step] at h
  | cons d xs =>
    by_cases hd : d = '\n' ∧ prev = true
    · cases xs with
      | nil => simp [step, hd] at h
      | cons e xs =>
        by_cases he : e = '\r'
        · simp [step, hd, he] at h; obtain ⟨rfl, rfl, rfl⟩ := h; simp <;> omega
        · simp [step, hd, he] at h; obtain ⟨rfl, rfl, rfl⟩ := h; simp <;> omega
    · by_cases hr : d = '\r'
      · simp [step, hd, hr] at h; obtain ⟨rfl, rfl, rfl⟩ := h; simp <;> omega
      · simp [step, hd, hr] at h; obtain ⟨rfl, rfl, rfl⟩ := h; simp <;> omega

/-- Run the list adapter until it returns None. The remainder length is a
    strictly decreasing measure, with no bound on the original input. -/
def collect (prev : Bool) (xs : List Char) : List Char :=
  match h : step prev xs with
  | (none, _, _) => []
  | (some c, b, rest) => c :: collect b rest
termination_by xs.length
decreasing_by exact step_progress prev xs _ _ _ h

 theorem collect_output (prev : Bool) (xs : List Char) :
    collect prev xs = output prev xs := by
  rw [collect, step_output]
  split <;> rename_i h
  · simp [h]
  · simp only [h, List.cons.injEq, true_and]
    exact collect_output _ _
termination_by xs.length
decreasing_by exact step_progress prev xs _ _ _ ‹_›

 theorem collect_normalizes (xs : List Char) : collect false xs = normalize xs := by
  simpa [output] using collect_output false xs

 theorem normalize_no_cr (xs : List Char) : '\r' ∉ normalize xs := by
  induction xs using normalize.induct with
  | case1 => simp [normalize]
  | case2 xs ih => simpa [normalize] using ih
  | case3 xs h ih => simpa [normalize] using ih
  | case4 c xs _ h ih =>
    simp only [normalize, List.mem_cons, not_or]
    exact ⟨by simpa [eq_comm] using h, ih⟩

 theorem normalize_of_no_cr (xs : List Char) (h : '\r' ∉ xs) : normalize xs = xs := by
  induction xs with
  | nil => rfl
  | cons c xs ih =>
    simp only [List.mem_cons, not_or] at h
    have hc : c ≠ '\r' := by simpa [eq_comm] using h.1
    rw [normalize_other c xs hc, ih h.2]

 theorem normalize_idempotent (xs : List Char) : normalize (normalize xs) = normalize xs :=
  normalize_of_no_cr _ (normalize_no_cr xs)

 theorem normalize_length_le (xs : List Char) : (normalize xs).length ≤ xs.length := by
  induction xs using normalize.induct with
  | case1 => simp [normalize]
  | case2 xs ih => simp only [normalize, List.length_cons]; omega
  | case3 xs _ ih => simpa [normalize] using ih
  | case4 c xs _ _ ih => simpa [normalize] using ih

 theorem collect_no_cr (xs : List Char) : '\r' ∉ collect false xs := by
  rw [collect_normalizes]; exact normalize_no_cr xs

 theorem collect_idempotent (xs : List Char) :
    collect false (collect false xs) = collect false xs := by
  simp only [collect_normalizes]; exact normalize_idempotent xs

 theorem collect_length_le (xs : List Char) : (collect false xs).length ≤ xs.length := by
  rw [collect_normalizes]; exact normalize_length_le xs

/-- A fuel-indexed observer. The correctness theorem quantifies over all input
    lengths and any fuel strictly larger than that length. -/
def observe : Nat → Bool → List Char → List Char
  | 0, _, _ => []
  | n + 1, b, xs =>
    match step b xs with
    | (none, _, _) => []
    | (some c, b', rest) => c :: observe n b' rest

 theorem observe_output (fuel : Nat) (prev : Bool) (xs : List Char)
    (enough : xs.length < fuel) : observe fuel prev xs = output prev xs := by
  induction fuel generalizing prev xs with
  | zero => omega
  | succ n ih =>
    rw [observe, step_output]
    split <;> rename_i h
    · simp [h]
    · simp only [h, List.cons.injEq, true_and]
      apply ih
      have := step_progress prev xs _ _ _ h
      omega

 theorem observe_normalizes (fuel : Nat) (xs : List Char) (h : xs.length < fuel) :
    observe fuel false xs = normalize xs := by
  simpa [output] using observe_output fuel false xs h

end NormalizationSpec
