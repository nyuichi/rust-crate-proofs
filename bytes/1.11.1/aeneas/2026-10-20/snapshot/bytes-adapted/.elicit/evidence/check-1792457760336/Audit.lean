import Proofs
import Lean
open Lean Meta Elab Command Term

elab "elicit_audit " index:num proposition:str proofName:str : command => do
  let saved ← get
  let row ← try
    let auditName := Name.str `ElicitBatchAudit ("root_" ++ toString index.getNat)
    if (← getEnv).contains auditName then throwError "audit name collision"
    let proofName := proofName.getString.toName
    unless (← getEnv).contains proofName do throwError "missing proof"
    let propSyntax ← match Parser.runParserCategory (← getEnv) `term proposition.getString with
      | .ok propSyntax => pure propSyntax
      | .error error => throwError error
    let (type, value, levels) ← liftTermElabM <| withoutErrToSorry do
      let type ← elabType propSyntax
      unless ← isProp type do throwError "expected a proposition"
      let value ← elabTermEnsuringType (mkIdent proofName) type
      unless ← isDefEq (← inferType value) type do throwError "proof type mismatch"
      synthesizeSyntheticMVarsNoPostponing
      let type ← instantiateMVars type
      let type ← levelMVarToParam type
      let value ← instantiateMVars value
      if type.hasMVar || value.hasMVar || type.hasFVar || value.hasFVar then
        throwError "unresolved proof variables"
      pure (type, value, ← getLevelNames)
    liftCoreM <| addDecl <| .thmDecl {
      «name» := auditName
      levelParams := levels
      type := type
      value := value
    }
    let some (.thmInfo info) := (← getEnv).checked.get.find? auditName
      | throwError "kernel theorem missing"
    unless info.type == type do throwError "unexpected theorem type"
    let axioms ← collectAxioms auditName
    let allowed := axioms.all fun ax => ax == `propext || ax == `Classical.choice || ax == `Quot.sound
    pure <| Json.mkObj [("index", toJson index.getNat), ("checked", toJson allowed),
      ("axioms", toJson (axioms.map Name.toString))]
  catch _ =>
    pure <| Json.mkObj [("index", toJson index.getNat), ("checked", toJson false), ("axioms", Json.null)]
  set saved
  logInfo ("ELICIT_AUDIT=" ++ row.compress)

open Aeneas Aeneas.Std Result ElicitGenerated
elicit_audit 0 "ElicitContracts.c_0ab59922312519d0" "BytesElicit.chain_first_mut_replaces_first"
elicit_audit 1 "ElicitContracts.c_0a3770d6fb2dbe56" "BytesElicit.chain_first_ref_returns_first"
elicit_audit 2 "ElicitContracts.c_cdf4d241f0e1fb55" "BytesElicit.chain_into_inner_returns_fields"
elicit_audit 3 "ElicitContracts.c_4be354299d19c21d" "BytesElicit.chain_last_mut_replaces_second"
elicit_audit 4 "ElicitContracts.c_7142f49d9af652c8" "BytesElicit.chain_last_ref_returns_second"
elicit_audit 5 "ElicitContracts.c_76945f5528ea3028" "BytesElicit.chain_new_fields"
elicit_audit 6 "ElicitContracts.c_a0e18b6702bd3796" "BytesElicit.into_iter_get_mut_replaces_inner"
elicit_audit 7 "ElicitContracts.c_de579782e2a34b9b" "BytesElicit.into_iter_get_ref_returns_inner"
elicit_audit 8 "ElicitContracts.c_0348592fe8519fce" "BytesElicit.into_iter_into_inner_returns_inner"
elicit_audit 9 "ElicitContracts.c_e3299c6f74df6cb4" "BytesElicit.into_iter_new_field"
elicit_audit 10 "ElicitContracts.c_c3a35ebbb8331cde" "BytesElicit.limit_get_mut_replaces_inner_only"
elicit_audit 11 "ElicitContracts.c_5df6b3de38349faa" "BytesElicit.limit_get_ref_returns_inner"
elicit_audit 12 "ElicitContracts.c_6f39d01bffb2e600" "BytesElicit.limit_into_inner_returns_inner"
elicit_audit 13 "ElicitContracts.c_a904d034ecb38bd1" "BytesElicit.limit_returns_stored_limit"
elicit_audit 14 "ElicitContracts.c_b9c9fc9a329282f1" "BytesElicit.limit_set_limit_changes_only_limit"
elicit_audit 15 "ElicitContracts.c_3c397f36f32e60b3" "BytesElicit.limit_new_fields"
elicit_audit 16 "ElicitContracts.c_69283305e385fbc7" "BytesElicit.take_get_mut_replaces_inner_only"
elicit_audit 17 "ElicitContracts.c_f1abf57d300de3dd" "BytesElicit.take_get_ref_returns_inner"
elicit_audit 18 "ElicitContracts.c_67ab1aad488d2950" "BytesElicit.take_into_inner_returns_inner"
elicit_audit 19 "ElicitContracts.c_fe100f19c2bba7e6" "BytesElicit.take_limit_returns_stored_limit"
elicit_audit 20 "ElicitContracts.c_6664141dae95d86d" "BytesElicit.take_set_limit_changes_only_limit"
elicit_audit 21 "ElicitContracts.c_03820710b16d0a9a" "BytesElicit.take_new_fields"
elicit_audit 22 "ElicitContracts.c_26b458661b6dffb1" "BytesElicit.bytes_len_returns_stored_length"
elicit_audit 23 "ElicitContracts.c_6aa70a75372247cf" "BytesElicit.bytes_mut_len_returns_stored_length"
