import Aeneas
import Lean
open Lean Meta Elab Command Term

elab "elicit_contract " target:ident " for " original:ident " requires " pre:term " ensures " post:term : command => do
  let info ← getConstInfo original.getId
  let value ← liftTermElabM do
    let f := Lean.mkConst info.name (info.levelParams.map Level.param)
    forallTelescope info.type fun args resultType => do
      unless resultType.isAppOfArity ``Aeneas.Std.Result 1 do
        throwError "expected an Aeneas Result return type, got {resultType}"
      let outputType := resultType.getAppArgs[0]!
      withLocalDeclD `result outputType fun result => do
        let prop := mkSort Level.zero
        let preType ← mkForallFVars args prop
        let preValue ← Term.elabTermEnsuringType pre preType
        let postType ← mkForallFVars (args.push result) prop
        let postValue ← Term.elabTermEnsuringType post postType
        Term.synthesizeSyntheticMVarsNoPostponing
        let equal ← mkEq (mkAppN f args) (← mkAppM ``Aeneas.Std.Result.ok #[result])
        let both ← mkAppM ``And #[equal, mkAppN postValue (args.push result)]
        let existsResult ← mkAppM ``Exists #[← mkLambdaFVars #[result] both]
        let implication ← mkArrow (mkAppN preValue args) existsResult
        instantiateMVars (← mkForallFVars args implication)
  liftCoreM <| addAndCompile <| .defnDecl {
    «name» := target.getId
    levelParams := info.levelParams
    type := mkSort Level.zero
    value := value
    hints := .abbrev
    safety := .safe
  }

namespace ElicitAdapter
open Aeneas Aeneas.Std Result

/-- Reuse a checked execution contract as a standard Aeneas total-correctness spec. -/
theorem contract_to_spec {α : Type} {m : Result α} {p : α → Prop}
    (h : ∃ r, m = ok r ∧ p r) : m ⦃ p ⦄ := by
  obtain ⟨r, heq, hp⟩ := h
  subst m
  exact Std.WP.spec_ok r |>.mpr hp

/-- Export a standard Aeneas total-correctness spec to the ledger contract form. -/
theorem spec_to_contract {α : Type} {m : Result α} {p : α → Prop}
    (h : m ⦃ p ⦄) : ∃ r, m = ok r ∧ p r := by
  cases m using Result.cases <;> simp_all

end ElicitAdapter

/-- Generate and register a caller-facing WP rule from a checked contract theorem.
    Place this after the theorem and before proving callers (no import cycle). -/
elab "elicit_spec " target:ident " from " source:ident : command => do
  let sourceName ← liftCoreM <| realizeGlobalConstNoOverloadWithInfo source
  let info ← getConstInfo sourceName
  let targetName := (← getCurrNamespace) ++ target.getId
  let (type, value) ← liftTermElabM do
    forallTelescopeReducing info.type fun args _ => do
      let proof := mkAppN (mkConst info.name (info.levelParams.map Level.param)) args
      let value ← mkAppM ``ElicitAdapter.contract_to_spec #[proof]
      let type ← inferType value
      return (← instantiateMVars (← mkForallFVars args type), ← instantiateMVars (← mkLambdaFVars args value))
  liftCoreM <| addDecl <| .thmDecl {
    «name» := targetName
    levelParams := info.levelParams
    type := type
    value := value
  }
  elabCommand (← `(attribute [step] $target))
