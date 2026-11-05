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
 let theorem_name := `LayoutAudit.u32_size
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: LayoutAudit.u32_size"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in LayoutAudit.u32_size: {ax}"
 liftIO <| IO.println "proofs.rs:checked:LayoutAudit.u32_size"
#print axioms LayoutAudit.u32_size

run_cmd do
 let theorem_name := `U32RangeFailure.bit_length
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.bit_length"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.bit_length: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.bit_length"
#print axioms U32RangeFailure.bit_length

run_cmd do
 let theorem_name := `U32RangeFailure.get_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.get_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.get_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.get_invalid_start"
#print axioms U32RangeFailure.get_invalid_start

run_cmd do
 let theorem_name := `U32RangeFailure.get_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.get_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.get_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.get_invalid_end"
#print axioms U32RangeFailure.get_invalid_end

run_cmd do
 let theorem_name := `U32RangeFailure.get_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.get_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.get_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.get_reversed"
#print axioms U32RangeFailure.get_reversed

run_cmd do
 let theorem_name := `U32RangeFailure.set_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.set_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.set_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.set_invalid_start"
#print axioms U32RangeFailure.set_invalid_start

run_cmd do
 let theorem_name := `U32RangeFailure.set_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.set_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.set_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.set_invalid_end"
#print axioms U32RangeFailure.set_invalid_end

run_cmd do
 let theorem_name := `U32RangeFailure.set_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.set_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.set_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.set_reversed"
#print axioms U32RangeFailure.set_reversed

run_cmd do
 let theorem_name := `U32RangeFailure.get_empty
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.get_empty"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.get_empty: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.get_empty"
#print axioms U32RangeFailure.get_empty

run_cmd do
 let theorem_name := `U32RangeFailure.set_empty_zero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFailure.set_empty_zero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFailure.set_empty_zero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFailure.set_empty_zero"
#print axioms U32RangeFailure.set_empty_zero

run_cmd do
 let theorem_name := `U32RangeEmptyNonzero.sub_self
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeEmptyNonzero.sub_self"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeEmptyNonzero.sub_self: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeEmptyNonzero.sub_self"
#print axioms U32RangeEmptyNonzero.sub_self

run_cmd do
 let theorem_name := `U32RangeEmptyNonzero.eight_sub_zero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeEmptyNonzero.eight_sub_zero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeEmptyNonzero.eight_sub_zero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeEmptyNonzero.eight_sub_zero"
#print axioms U32RangeEmptyNonzero.eight_sub_zero

run_cmd do
 let theorem_name := `U32RangeEmptyNonzero.shift_eight_fails
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeEmptyNonzero.shift_eight_fails"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeEmptyNonzero.shift_eight_fails: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeEmptyNonzero.shift_eight_fails"
#print axioms U32RangeEmptyNonzero.shift_eight_fails

run_cmd do
 let theorem_name := `U32RangeEmptyNonzero.set_empty_nonzero
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeEmptyNonzero.set_empty_nonzero"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeEmptyNonzero.set_empty_nonzero: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeEmptyNonzero.set_empty_nonzero"
#print axioms U32RangeEmptyNonzero.set_empty_nonzero

run_cmd do
 let theorem_name := `U32RangeGetter.value_from_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeGetter.value_from_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeGetter.value_from_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeGetter.value_from_bits"
#print axioms U32RangeGetter.value_from_bits

run_cmd do
 let theorem_name := `U32RangeGetter.get_valid_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeGetter.get_valid_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeGetter.get_valid_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeGetter.get_valid_value"
#print axioms U32RangeGetter.get_valid_value

run_cmd do
 let theorem_name := `U32RangeGetter.clipped_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeGetter.clipped_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeGetter.clipped_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeGetter.clipped_bits"
#print axioms U32RangeGetter.clipped_bits

run_cmd do
 let theorem_name := `U32RangeGetter.get_valid_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeGetter.get_valid_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeGetter.get_valid_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeGetter.get_valid_bits"
#print axioms U32RangeGetter.get_valid_bits

run_cmd do
 let theorem_name := `U32RangeSetterMath.fit_low_word
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeSetterMath.fit_low_word"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeSetterMath.fit_low_word: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeSetterMath.fit_low_word"
#print axioms U32RangeSetterMath.fit_low_word

run_cmd do
 let theorem_name := `U32RangeSetterMath.fit_bit_false
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeSetterMath.fit_bit_false"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeSetterMath.fit_bit_false: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeSetterMath.fit_bit_false"
#print axioms U32RangeSetterMath.fit_bit_false

run_cmd do
 let theorem_name := `U32RangeSetterMath.window_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeSetterMath.window_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeSetterMath.window_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeSetterMath.window_bits"
#print axioms U32RangeSetterMath.window_bits

run_cmd do
 let theorem_name := `U32RangeSetterMath.update_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeSetterMath.update_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeSetterMath.update_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeSetterMath.update_bits"
#print axioms U32RangeSetterMath.update_bits

run_cmd do
 let theorem_name := `U32RangeSetter.set_valid
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeSetter.set_valid"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeSetter.set_valid: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeSetter.set_valid"
#print axioms U32RangeSetter.set_valid

run_cmd do
 let theorem_name := `U32RangeSetter.store_spec
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeSetter.store_spec"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeSetter.store_spec: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeSetter.store_spec"
#print axioms U32RangeSetter.store_spec

run_cmd do
 let theorem_name := `U32RangeFitFailure.low_word_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFitFailure.low_word_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFitFailure.low_word_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFitFailure.low_word_value"
#print axioms U32RangeFitFailure.low_word_value

run_cmd do
 let theorem_name := `U32RangeFitFailure.unfit_low_word_ne
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFitFailure.unfit_low_word_ne"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFitFailure.unfit_low_word_ne: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFitFailure.unfit_low_word_ne"
#print axioms U32RangeFitFailure.unfit_low_word_ne

run_cmd do
 let theorem_name := `U32RangeFitFailure.bind_const
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFitFailure.bind_const"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFitFailure.bind_const: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFitFailure.bind_const"
#print axioms U32RangeFitFailure.bind_const

run_cmd do
 let theorem_name := `U32RangeFitFailure.set_nonempty_unfit
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeFitFailure.set_nonempty_unfit"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeFitFailure.set_nonempty_unfit: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeFitFailure.set_nonempty_unfit"
#print axioms U32RangeFitFailure.set_nonempty_unfit

run_cmd do
 let theorem_name := `U32RangeComplete.invalid_get
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeComplete.invalid_get"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeComplete.invalid_get: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeComplete.invalid_get"
#print axioms U32RangeComplete.invalid_get

run_cmd do
 let theorem_name := `U32RangeComplete.invalid_set
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeComplete.invalid_set"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeComplete.invalid_set: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeComplete.invalid_set"
#print axioms U32RangeComplete.invalid_set

run_cmd do
 let theorem_name := `U32RangeComplete.get_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeComplete.get_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeComplete.get_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeComplete.get_all_inputs"
#print axioms U32RangeComplete.get_all_inputs

run_cmd do
 let theorem_name := `U32RangeComplete.set_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: U32RangeComplete.set_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in U32RangeComplete.set_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:U32RangeComplete.set_all_inputs"
#print axioms U32RangeComplete.set_all_inputs

