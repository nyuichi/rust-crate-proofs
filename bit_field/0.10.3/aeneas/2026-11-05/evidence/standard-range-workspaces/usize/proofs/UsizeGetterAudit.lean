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
 let theorem_name := `LayoutAudit.usize_size
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: LayoutAudit.usize_size"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in LayoutAudit.usize_size: {ax}"
 liftIO <| IO.println "proofs.rs:checked:LayoutAudit.usize_size"
#print axioms LayoutAudit.usize_size

run_cmd do
 let theorem_name := `UsizeRangeFailure.bit_length
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeFailure.bit_length"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeFailure.bit_length: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeFailure.bit_length"
#print axioms UsizeRangeFailure.bit_length

run_cmd do
 let theorem_name := `UsizeRangeFailure.get_invalid_start
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeFailure.get_invalid_start"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeFailure.get_invalid_start: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeFailure.get_invalid_start"
#print axioms UsizeRangeFailure.get_invalid_start

run_cmd do
 let theorem_name := `UsizeRangeFailure.get_invalid_end
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeFailure.get_invalid_end"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeFailure.get_invalid_end: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeFailure.get_invalid_end"
#print axioms UsizeRangeFailure.get_invalid_end

run_cmd do
 let theorem_name := `UsizeRangeFailure.get_reversed
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeFailure.get_reversed"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeFailure.get_reversed: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeFailure.get_reversed"
#print axioms UsizeRangeFailure.get_reversed

run_cmd do
 let theorem_name := `UsizeRangeFailure.get_empty
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeFailure.get_empty"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeFailure.get_empty: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeFailure.get_empty"
#print axioms UsizeRangeFailure.get_empty

run_cmd do
 let theorem_name := `UsizeRangeGetter.value_from_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeGetter.value_from_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeGetter.value_from_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeGetter.value_from_bits"
#print axioms UsizeRangeGetter.value_from_bits

run_cmd do
 let theorem_name := `UsizeRangeGetter.get_valid_value
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeGetter.get_valid_value"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeGetter.get_valid_value: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeGetter.get_valid_value"
#print axioms UsizeRangeGetter.get_valid_value

run_cmd do
 let theorem_name := `UsizeRangeGetter.clipped_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeGetter.clipped_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeGetter.clipped_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeGetter.clipped_bits"
#print axioms UsizeRangeGetter.clipped_bits

run_cmd do
 let theorem_name := `UsizeRangeGetter.get_valid_bits
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeGetter.get_valid_bits"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeGetter.get_valid_bits: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeGetter.get_valid_bits"
#print axioms UsizeRangeGetter.get_valid_bits

run_cmd do
 let theorem_name := `UsizeRangeComplete.invalid_get
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeComplete.invalid_get"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeComplete.invalid_get: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeComplete.invalid_get"
#print axioms UsizeRangeComplete.invalid_get

run_cmd do
 let theorem_name := `UsizeRangeComplete.get_all_inputs
 match (← getEnv).find? theorem_name with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Expected theorem: UsizeRangeComplete.get_all_inputs"
 let axioms ← Lean.collectAxioms theorem_name
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom in UsizeRangeComplete.get_all_inputs: {ax}"
 liftIO <| IO.println "proofs.rs:checked:UsizeRangeComplete.get_all_inputs"
#print axioms UsizeRangeComplete.get_all_inputs
