import Lean
import FieldFromProofs
open Lean Elab Command
run_cmd do
 let n := `FieldFromProofs.shared_original_exact
 match (← getEnv).find? n with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Not theorem"
 let axioms ← Lean.collectAxioms n
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom {ax}"
 liftIO <| IO.println s!"checked:{n}"
run_cmd do
 let n := `FieldFromProofs.mutable_original_exact
 match (← getEnv).find? n with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Not theorem"
 let axioms ← Lean.collectAxioms n
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom {ax}"
 liftIO <| IO.println s!"checked:{n}"
run_cmd do
 let n := `FieldFromProofs.mutable_same_length_writeback
 match (← getEnv).find? n with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Not theorem"
 let axioms ← Lean.collectAxioms n
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom {ax}"
 liftIO <| IO.println s!"checked:{n}"
run_cmd do
 let n := `FieldFromProofs.shared_wrapper_route
 match (← getEnv).find? n with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Not theorem"
 let axioms ← Lean.collectAxioms n
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom {ax}"
 liftIO <| IO.println s!"checked:{n}"
run_cmd do
 let n := `FieldFromProofs.mutable_wrapper_route
 match (← getEnv).find? n with
 | some (.thmInfo _) => pure ()
 | _ => throwError "Not theorem"
 let axioms ← Lean.collectAxioms n
 for ax in axioms do
  unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
   throwError "Unapproved axiom {ax}"
 liftIO <| IO.println s!"checked:{n}"
