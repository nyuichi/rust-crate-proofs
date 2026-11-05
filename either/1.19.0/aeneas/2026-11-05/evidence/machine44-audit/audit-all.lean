import Lean
import BothProofs
import CallbackProofs
import DefaultProofs
import FactorProofs
import IntoEitherProofs
import IntoIterProofs
import IterSharedProofs
import IteratorProofs
import MapInspectProofs
import NextBackProofs
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

run_cmd do
  let theorem_name := `BothProofs.either_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: BothProofs.either_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in BothProofs.either_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:BothProofs.either_spec"

run_cmd do
  let theorem_name := `BothProofs.either_with_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: BothProofs.either_with_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in BothProofs.either_with_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:BothProofs.either_with_spec"

run_cmd do
  let theorem_name := `BothProofs.either_into_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: BothProofs.either_into_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in BothProofs.either_into_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:BothProofs.either_into_spec"

run_cmd do
  let theorem_name := `MapInspectProofs.map_either_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: MapInspectProofs.map_either_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in MapInspectProofs.map_either_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:MapInspectProofs.map_either_spec"

run_cmd do
  let theorem_name := `MapInspectProofs.map_either_with_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: MapInspectProofs.map_either_with_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in MapInspectProofs.map_either_with_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:MapInspectProofs.map_either_with_spec"

run_cmd do
  let theorem_name := `MapInspectProofs.inspect_left_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: MapInspectProofs.inspect_left_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in MapInspectProofs.inspect_left_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:MapInspectProofs.inspect_left_spec"

run_cmd do
  let theorem_name := `MapInspectProofs.inspect_right_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: MapInspectProofs.inspect_right_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in MapInspectProofs.inspect_right_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:MapInspectProofs.inspect_right_spec"

run_cmd do
  let theorem_name := `IteratorProofs.next_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: IteratorProofs.next_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in IteratorProofs.next_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:IteratorProofs.next_spec"

run_cmd do
  let theorem_name := `NextBackProofs.next_back_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: NextBackProofs.next_back_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in NextBackProofs.next_back_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:NextBackProofs.next_back_spec"

run_cmd do
  let theorem_name := `FactorProofs.exact_delegation
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: FactorProofs.exact_delegation"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in FactorProofs.exact_delegation: {ax}"
  liftIO <| IO.println "proofs.rs:checked:FactorProofs.exact_delegation"

run_cmd do
  let theorem_name := `IntoIterProofs.exact_delegation
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: IntoIterProofs.exact_delegation"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in IntoIterProofs.exact_delegation: {ax}"
  liftIO <| IO.println "proofs.rs:checked:IntoIterProofs.exact_delegation"

run_cmd do
  let theorem_name := `IterSharedProofs.exact_delegation
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: IterSharedProofs.exact_delegation"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in IterSharedProofs.exact_delegation: {ax}"
  liftIO <| IO.println "proofs.rs:checked:IterSharedProofs.exact_delegation"
