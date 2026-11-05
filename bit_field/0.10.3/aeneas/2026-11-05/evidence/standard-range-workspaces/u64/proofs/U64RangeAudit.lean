import Lean
import CompleteSpec
open Lean Elab Command

run_cmd do
 let theorem_name := `LayoutAudit.alias_exact
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: LayoutAudit.alias_exact"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in LayoutAudit.alias_exact: {ax}"
 liftIO <| IO.println "proofs.rs:checked:LayoutAudit.alias_exact"
#print axioms LayoutAudit.alias_exact

run_cmd do
 let theorem_name := `LayoutAudit.u64_size
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: LayoutAudit.u64_size"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in LayoutAudit.u64_size: {ax}"
 liftIO <| IO.println "proofs.rs:checked:LayoutAudit.u64_size"
#print axioms LayoutAudit.u64_size

run_cmd do
 let theorem_name := `U64RangeFailure.bit_length
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.bit_length"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.bit_length: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.bit_length"
#print axioms U64RangeFailure.bit_length

run_cmd do
 let theorem_name := `U64RangeFailure.get_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.get_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.get_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.get_invalid_start"
#print axioms U64RangeFailure.get_invalid_start

run_cmd do
 let theorem_name := `U64RangeFailure.get_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.get_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.get_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.get_invalid_end"
#print axioms U64RangeFailure.get_invalid_end

run_cmd do
 let theorem_name := `U64RangeFailure.get_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.get_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.get_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.get_reversed"
#print axioms U64RangeFailure.get_reversed

run_cmd do
 let theorem_name := `U64RangeFailure.set_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.set_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.set_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.set_invalid_start"
#print axioms U64RangeFailure.set_invalid_start

run_cmd do
 let theorem_name := `U64RangeFailure.set_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.set_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.set_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.set_invalid_end"
#print axioms U64RangeFailure.set_invalid_end

run_cmd do
 let theorem_name := `U64RangeFailure.set_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.set_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.set_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.set_reversed"
#print axioms U64RangeFailure.set_reversed

run_cmd do
 let theorem_name := `U64RangeFailure.get_empty
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.get_empty"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.get_empty: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.get_empty"
#print axioms U64RangeFailure.get_empty

run_cmd do
 let theorem_name := `U64RangeFailure.set_empty_zero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFailure.set_empty_zero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFailure.set_empty_zero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFailure.set_empty_zero"
#print axioms U64RangeFailure.set_empty_zero

run_cmd do
 let theorem_name := `U64RangeEmptyNonzero.sub_self
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeEmptyNonzero.sub_self"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeEmptyNonzero.sub_self: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeEmptyNonzero.sub_self"
#print axioms U64RangeEmptyNonzero.sub_self

run_cmd do
 let theorem_name := `U64RangeEmptyNonzero.eight_sub_zero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeEmptyNonzero.eight_sub_zero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeEmptyNonzero.eight_sub_zero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeEmptyNonzero.eight_sub_zero"
#print axioms U64RangeEmptyNonzero.eight_sub_zero

run_cmd do
 let theorem_name := `U64RangeEmptyNonzero.shift_eight_fails
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeEmptyNonzero.shift_eight_fails"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeEmptyNonzero.shift_eight_fails: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeEmptyNonzero.shift_eight_fails"
#print axioms U64RangeEmptyNonzero.shift_eight_fails

run_cmd do
 let theorem_name := `U64RangeEmptyNonzero.set_empty_nonzero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeEmptyNonzero.set_empty_nonzero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeEmptyNonzero.set_empty_nonzero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeEmptyNonzero.set_empty_nonzero"
#print axioms U64RangeEmptyNonzero.set_empty_nonzero

run_cmd do
 let theorem_name := `U64RangeGetter.value_from_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeGetter.value_from_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeGetter.value_from_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeGetter.value_from_bits"
#print axioms U64RangeGetter.value_from_bits

run_cmd do
 let theorem_name := `U64RangeGetter.get_valid_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeGetter.get_valid_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeGetter.get_valid_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeGetter.get_valid_value"
#print axioms U64RangeGetter.get_valid_value

run_cmd do
 let theorem_name := `U64RangeGetter.clipped_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeGetter.clipped_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeGetter.clipped_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeGetter.clipped_bits"
#print axioms U64RangeGetter.clipped_bits

run_cmd do
 let theorem_name := `U64RangeGetter.get_valid_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeGetter.get_valid_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeGetter.get_valid_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeGetter.get_valid_bits"
#print axioms U64RangeGetter.get_valid_bits

run_cmd do
 let theorem_name := `U64RangeSetterMath.fit_low_word
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeSetterMath.fit_low_word"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeSetterMath.fit_low_word: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeSetterMath.fit_low_word"
#print axioms U64RangeSetterMath.fit_low_word

run_cmd do
 let theorem_name := `U64RangeSetterMath.fit_bit_false
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeSetterMath.fit_bit_false"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeSetterMath.fit_bit_false: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeSetterMath.fit_bit_false"
#print axioms U64RangeSetterMath.fit_bit_false

run_cmd do
 let theorem_name := `U64RangeSetterMath.window_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeSetterMath.window_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeSetterMath.window_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeSetterMath.window_bits"
#print axioms U64RangeSetterMath.window_bits

run_cmd do
 let theorem_name := `U64RangeSetterMath.update_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeSetterMath.update_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeSetterMath.update_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeSetterMath.update_bits"
#print axioms U64RangeSetterMath.update_bits

run_cmd do
 let theorem_name := `U64RangeSetter.set_valid
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeSetter.set_valid"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeSetter.set_valid: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeSetter.set_valid"
#print axioms U64RangeSetter.set_valid

run_cmd do
 let theorem_name := `U64RangeSetter.store_spec
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeSetter.store_spec"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeSetter.store_spec: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeSetter.store_spec"
#print axioms U64RangeSetter.store_spec

run_cmd do
 let theorem_name := `U64RangeFitFailure.low_word_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFitFailure.low_word_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFitFailure.low_word_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFitFailure.low_word_value"
#print axioms U64RangeFitFailure.low_word_value

run_cmd do
 let theorem_name := `U64RangeFitFailure.unfit_low_word_ne
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFitFailure.unfit_low_word_ne"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFitFailure.unfit_low_word_ne: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFitFailure.unfit_low_word_ne"
#print axioms U64RangeFitFailure.unfit_low_word_ne

run_cmd do
 let theorem_name := `U64RangeFitFailure.bind_const
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFitFailure.bind_const"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFitFailure.bind_const: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFitFailure.bind_const"
#print axioms U64RangeFitFailure.bind_const

run_cmd do
 let theorem_name := `U64RangeFitFailure.set_nonempty_unfit
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeFitFailure.set_nonempty_unfit"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeFitFailure.set_nonempty_unfit: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeFitFailure.set_nonempty_unfit"
#print axioms U64RangeFitFailure.set_nonempty_unfit

run_cmd do
 let theorem_name := `U64RangeComplete.invalid_get
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeComplete.invalid_get"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeComplete.invalid_get: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeComplete.invalid_get"
#print axioms U64RangeComplete.invalid_get

run_cmd do
 let theorem_name := `U64RangeComplete.invalid_set
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeComplete.invalid_set"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeComplete.invalid_set: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeComplete.invalid_set"
#print axioms U64RangeComplete.invalid_set

run_cmd do
 let theorem_name := `U64RangeComplete.get_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeComplete.get_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeComplete.get_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeComplete.get_all_inputs"
#print axioms U64RangeComplete.get_all_inputs

run_cmd do
 let theorem_name := `U64RangeComplete.set_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U64RangeComplete.set_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U64RangeComplete.set_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U64RangeComplete.set_all_inputs"
#print axioms U64RangeComplete.set_all_inputs

