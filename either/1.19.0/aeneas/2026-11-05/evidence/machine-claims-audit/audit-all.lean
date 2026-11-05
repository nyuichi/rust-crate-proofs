import Lean
import CallbackProofs
import DefaultProofs
import IntoEitherProofs
import Proofs
import ResultProofs
open Lean Elab Command

run_cmd do
  let theorem_name := `Proofs.is_left_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_left_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_left_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_left_spec"

run_cmd do
  let theorem_name := `Proofs.is_right_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_right_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_right_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_right_spec"

run_cmd do
  let theorem_name := `Proofs.left_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.left_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.left_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.left_spec"

run_cmd do
  let theorem_name := `Proofs.right_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.right_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.right_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.right_spec"

run_cmd do
  let theorem_name := `Proofs.flip_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.flip_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.flip_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.flip_spec"

run_cmd do
  let theorem_name := `Proofs.left_or_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.left_or_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.left_or_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.left_or_spec"

run_cmd do
  let theorem_name := `Proofs.right_or_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.right_or_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.right_or_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.right_or_spec"

run_cmd do
  let theorem_name := `Proofs.left_and_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.left_and_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.left_and_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.left_and_spec"

run_cmd do
  let theorem_name := `Proofs.right_and_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.right_and_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.right_and_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.right_and_spec"

run_cmd do
  let theorem_name := `Proofs.factor_none_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.factor_none_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.factor_none_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.factor_none_spec"

run_cmd do
  let theorem_name := `Proofs.factor_first_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.factor_first_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.factor_first_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.factor_first_spec"

run_cmd do
  let theorem_name := `Proofs.factor_second_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.factor_second_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.factor_second_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.factor_second_spec"

run_cmd do
  let theorem_name := `Proofs.as_ref_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.as_ref_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.as_ref_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.as_ref_spec"

run_cmd do
  let theorem_name := `Proofs.as_mut_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.as_mut_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.as_mut_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.as_mut_spec"

run_cmd do
  let theorem_name := `CallbackProofs.is_left_and_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.is_left_and_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.is_left_and_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.is_left_and_spec"

run_cmd do
  let theorem_name := `CallbackProofs.is_right_and_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.is_right_and_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.is_right_and_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.is_right_and_spec"

run_cmd do
  let theorem_name := `CallbackProofs.is_left_or_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.is_left_or_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.is_left_or_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.is_left_or_spec"

run_cmd do
  let theorem_name := `CallbackProofs.is_right_or_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.is_right_or_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.is_right_or_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.is_right_or_spec"

run_cmd do
  let theorem_name := `CallbackProofs.map_left_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.map_left_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.map_left_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.map_left_spec"

run_cmd do
  let theorem_name := `CallbackProofs.map_right_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.map_right_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.map_right_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.map_right_spec"

run_cmd do
  let theorem_name := `CallbackProofs.map_left_or_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.map_left_or_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.map_left_or_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.map_left_or_spec"

run_cmd do
  let theorem_name := `CallbackProofs.map_right_or_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.map_right_or_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.map_right_or_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.map_right_or_spec"

run_cmd do
  let theorem_name := `CallbackProofs.left_or_else_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.left_or_else_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.left_or_else_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.left_or_else_spec"

run_cmd do
  let theorem_name := `CallbackProofs.right_or_else_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.right_or_else_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.right_or_else_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.right_or_else_spec"

run_cmd do
  let theorem_name := `CallbackProofs.left_and_then_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.left_and_then_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.left_and_then_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.left_and_then_spec"

run_cmd do
  let theorem_name := `CallbackProofs.right_and_then_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: CallbackProofs.right_and_then_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in CallbackProofs.right_and_then_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:CallbackProofs.right_and_then_spec"

run_cmd do
  let theorem_name := `IntoEitherProofs.into_either_blanket_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: IntoEitherProofs.into_either_blanket_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in IntoEitherProofs.into_either_blanket_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:IntoEitherProofs.into_either_blanket_spec"

run_cmd do
  let theorem_name := `IntoEitherProofs.into_either_with_blanket_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: IntoEitherProofs.into_either_with_blanket_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in IntoEitherProofs.into_either_with_blanket_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:IntoEitherProofs.into_either_with_blanket_spec"

run_cmd do
  let theorem_name := `ResultProofs.from_result_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: ResultProofs.from_result_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in ResultProofs.from_result_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:ResultProofs.from_result_spec"

run_cmd do
  let theorem_name := `ResultProofs.from_either_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: ResultProofs.from_either_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in ResultProofs.from_either_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:ResultProofs.from_either_spec"

run_cmd do
  let theorem_name := `DefaultProofs.left_or_default_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: DefaultProofs.left_or_default_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in DefaultProofs.left_or_default_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:DefaultProofs.left_or_default_spec"

run_cmd do
  let theorem_name := `DefaultProofs.right_or_default_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: DefaultProofs.right_or_default_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in DefaultProofs.right_or_default_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:DefaultProofs.right_or_default_spec"
