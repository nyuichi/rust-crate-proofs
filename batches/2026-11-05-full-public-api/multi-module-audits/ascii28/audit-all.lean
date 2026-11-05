import Lean
import Proofs
import StringProofs
open Lean Elab Command

run_cmd do
  let theorem_name := `Proofs.is_ascii_blank_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_blank_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_blank_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_blank_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_digit_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_digit_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_digit_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_digit_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_whitespace_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_whitespace_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_whitespace_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_whitespace_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_control_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_control_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_control_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_control_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_alphabetic_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_alphabetic_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_alphabetic_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_alphabetic_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_alphanumeric_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_alphanumeric_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_alphanumeric_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_alphanumeric_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_graphic_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_graphic_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_graphic_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_graphic_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_printable_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_printable_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_printable_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_printable_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_lowercase_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_lowercase_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_lowercase_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_lowercase_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_uppercase_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_uppercase_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_uppercase_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_uppercase_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_hexdigit_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_hexdigit_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_hexdigit_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_hexdigit_spec"

run_cmd do
  let theorem_name := `Proofs.as_byte_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.as_byte_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.as_byte_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.as_byte_spec"

run_cmd do
  let theorem_name := `Proofs.eq_ignore_ascii_case_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.eq_ignore_ascii_case_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.eq_ignore_ascii_case_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.eq_ignore_ascii_case_spec"

run_cmd do
  let theorem_name := `Proofs.to_ascii_char_identity_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.to_ascii_char_identity_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.to_ascii_char_identity_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.to_ascii_char_identity_spec"

run_cmd do
  let theorem_name := `Proofs.to_ascii_char_unchecked_identity_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.to_ascii_char_unchecked_identity_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.to_ascii_char_unchecked_identity_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.to_ascii_char_unchecked_identity_spec"

run_cmd do
  let theorem_name := `Proofs.is_alphabetic_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_alphabetic_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_alphabetic_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_alphabetic_spec"

run_cmd do
  let theorem_name := `Proofs.is_alphanumeric_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_alphanumeric_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_alphanumeric_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_alphanumeric_spec"

run_cmd do
  let theorem_name := `Proofs.is_lowercase_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_lowercase_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_lowercase_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_lowercase_spec"

run_cmd do
  let theorem_name := `Proofs.is_uppercase_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_uppercase_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_uppercase_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_uppercase_spec"

run_cmd do
  let theorem_name := `Proofs.is_whitespace_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_whitespace_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_whitespace_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_whitespace_spec"

run_cmd do
  let theorem_name := `Proofs.is_digit_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_digit_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_digit_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_digit_spec"

run_cmd do
  let theorem_name := `Proofs.is_ascii_punctuation_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.is_ascii_punctuation_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.is_ascii_punctuation_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.is_ascii_punctuation_spec"

run_cmd do
  let theorem_name := `Proofs.default_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.default_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.default_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.default_spec"

run_cmd do
  let theorem_name := `Proofs.valid_up_to_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: Proofs.valid_up_to_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in Proofs.valid_up_to_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:Proofs.valid_up_to_spec"

run_cmd do
  let theorem_name := `StringProofs.new_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: StringProofs.new_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in StringProofs.new_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:StringProofs.new_spec"

run_cmd do
  let theorem_name := `StringProofs.len_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: StringProofs.len_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in StringProofs.len_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:StringProofs.len_spec"

run_cmd do
  let theorem_name := `StringProofs.is_empty_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: StringProofs.is_empty_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in StringProofs.is_empty_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:StringProofs.is_empty_spec"

run_cmd do
  let theorem_name := `StringProofs.push_full_spec
  match (← getEnv).find? theorem_name with
  | some (.thmInfo _) => pure ()
  | _ => throwError "Expected a theorem: StringProofs.push_full_spec"
  let axioms ← Lean.collectAxioms theorem_name
  for ax in axioms do
    unless [``propext, ``Classical.choice, ``Quot.sound].contains ax do
      throwError "Unapproved axiom in StringProofs.push_full_spec: {ax}"
  liftIO <| IO.println "proofs.rs:checked:StringProofs.push_full_spec"
