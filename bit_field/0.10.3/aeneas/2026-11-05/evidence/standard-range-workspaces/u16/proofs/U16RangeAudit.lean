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
 let theorem_name := `LayoutAudit.u16_size
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: LayoutAudit.u16_size"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in LayoutAudit.u16_size: {ax}"
 liftIO <| IO.println "proofs.rs:checked:LayoutAudit.u16_size"
#print axioms LayoutAudit.u16_size

run_cmd do
 let theorem_name := `U16RangeFailure.bit_length
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.bit_length"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.bit_length: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.bit_length"
#print axioms U16RangeFailure.bit_length

run_cmd do
 let theorem_name := `U16RangeFailure.get_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.get_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.get_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.get_invalid_start"
#print axioms U16RangeFailure.get_invalid_start

run_cmd do
 let theorem_name := `U16RangeFailure.get_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.get_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.get_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.get_invalid_end"
#print axioms U16RangeFailure.get_invalid_end

run_cmd do
 let theorem_name := `U16RangeFailure.get_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.get_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.get_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.get_reversed"
#print axioms U16RangeFailure.get_reversed

run_cmd do
 let theorem_name := `U16RangeFailure.set_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.set_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.set_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.set_invalid_start"
#print axioms U16RangeFailure.set_invalid_start

run_cmd do
 let theorem_name := `U16RangeFailure.set_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.set_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.set_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.set_invalid_end"
#print axioms U16RangeFailure.set_invalid_end

run_cmd do
 let theorem_name := `U16RangeFailure.set_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.set_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.set_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.set_reversed"
#print axioms U16RangeFailure.set_reversed

run_cmd do
 let theorem_name := `U16RangeFailure.get_empty
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.get_empty"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.get_empty: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.get_empty"
#print axioms U16RangeFailure.get_empty

run_cmd do
 let theorem_name := `U16RangeFailure.set_empty_zero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFailure.set_empty_zero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFailure.set_empty_zero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFailure.set_empty_zero"
#print axioms U16RangeFailure.set_empty_zero

run_cmd do
 let theorem_name := `U16RangeEmptyNonzero.sub_self
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeEmptyNonzero.sub_self"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeEmptyNonzero.sub_self: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeEmptyNonzero.sub_self"
#print axioms U16RangeEmptyNonzero.sub_self

run_cmd do
 let theorem_name := `U16RangeEmptyNonzero.eight_sub_zero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeEmptyNonzero.eight_sub_zero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeEmptyNonzero.eight_sub_zero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeEmptyNonzero.eight_sub_zero"
#print axioms U16RangeEmptyNonzero.eight_sub_zero

run_cmd do
 let theorem_name := `U16RangeEmptyNonzero.shift_eight_fails
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeEmptyNonzero.shift_eight_fails"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeEmptyNonzero.shift_eight_fails: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeEmptyNonzero.shift_eight_fails"
#print axioms U16RangeEmptyNonzero.shift_eight_fails

run_cmd do
 let theorem_name := `U16RangeEmptyNonzero.set_empty_nonzero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeEmptyNonzero.set_empty_nonzero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeEmptyNonzero.set_empty_nonzero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeEmptyNonzero.set_empty_nonzero"
#print axioms U16RangeEmptyNonzero.set_empty_nonzero

run_cmd do
 let theorem_name := `U16RangeGetter.value_from_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeGetter.value_from_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeGetter.value_from_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeGetter.value_from_bits"
#print axioms U16RangeGetter.value_from_bits

run_cmd do
 let theorem_name := `U16RangeGetter.get_valid_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeGetter.get_valid_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeGetter.get_valid_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeGetter.get_valid_value"
#print axioms U16RangeGetter.get_valid_value

run_cmd do
 let theorem_name := `U16RangeGetter.clipped_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeGetter.clipped_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeGetter.clipped_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeGetter.clipped_bits"
#print axioms U16RangeGetter.clipped_bits

run_cmd do
 let theorem_name := `U16RangeGetter.get_valid_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeGetter.get_valid_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeGetter.get_valid_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeGetter.get_valid_bits"
#print axioms U16RangeGetter.get_valid_bits

run_cmd do
 let theorem_name := `U16RangeSetterMath.fit_low_word
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeSetterMath.fit_low_word"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeSetterMath.fit_low_word: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeSetterMath.fit_low_word"
#print axioms U16RangeSetterMath.fit_low_word

run_cmd do
 let theorem_name := `U16RangeSetterMath.fit_bit_false
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeSetterMath.fit_bit_false"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeSetterMath.fit_bit_false: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeSetterMath.fit_bit_false"
#print axioms U16RangeSetterMath.fit_bit_false

run_cmd do
 let theorem_name := `U16RangeSetterMath.window_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeSetterMath.window_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeSetterMath.window_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeSetterMath.window_bits"
#print axioms U16RangeSetterMath.window_bits

run_cmd do
 let theorem_name := `U16RangeSetterMath.update_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeSetterMath.update_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeSetterMath.update_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeSetterMath.update_bits"
#print axioms U16RangeSetterMath.update_bits

run_cmd do
 let theorem_name := `U16RangeSetter.set_valid
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeSetter.set_valid"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeSetter.set_valid: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeSetter.set_valid"
#print axioms U16RangeSetter.set_valid

run_cmd do
 let theorem_name := `U16RangeSetter.store_spec
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeSetter.store_spec"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeSetter.store_spec: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeSetter.store_spec"
#print axioms U16RangeSetter.store_spec

run_cmd do
 let theorem_name := `U16RangeFitFailure.low_word_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFitFailure.low_word_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFitFailure.low_word_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFitFailure.low_word_value"
#print axioms U16RangeFitFailure.low_word_value

run_cmd do
 let theorem_name := `U16RangeFitFailure.unfit_low_word_ne
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFitFailure.unfit_low_word_ne"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFitFailure.unfit_low_word_ne: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFitFailure.unfit_low_word_ne"
#print axioms U16RangeFitFailure.unfit_low_word_ne

run_cmd do
 let theorem_name := `U16RangeFitFailure.bind_const
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFitFailure.bind_const"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFitFailure.bind_const: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFitFailure.bind_const"
#print axioms U16RangeFitFailure.bind_const

run_cmd do
 let theorem_name := `U16RangeFitFailure.set_nonempty_unfit
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeFitFailure.set_nonempty_unfit"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeFitFailure.set_nonempty_unfit: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeFitFailure.set_nonempty_unfit"
#print axioms U16RangeFitFailure.set_nonempty_unfit

run_cmd do
 let theorem_name := `U16RangeComplete.invalid_get
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeComplete.invalid_get"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeComplete.invalid_get: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeComplete.invalid_get"
#print axioms U16RangeComplete.invalid_get

run_cmd do
 let theorem_name := `U16RangeComplete.invalid_set
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeComplete.invalid_set"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeComplete.invalid_set: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeComplete.invalid_set"
#print axioms U16RangeComplete.invalid_set

run_cmd do
 let theorem_name := `U16RangeComplete.get_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeComplete.get_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeComplete.get_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeComplete.get_all_inputs"
#print axioms U16RangeComplete.get_all_inputs

run_cmd do
 let theorem_name := `U16RangeComplete.set_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U16RangeComplete.set_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U16RangeComplete.set_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U16RangeComplete.set_all_inputs"
#print axioms U16RangeComplete.set_all_inputs

