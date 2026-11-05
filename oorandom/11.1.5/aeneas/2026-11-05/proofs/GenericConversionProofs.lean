import Generated.CoreConversions
import Oorandom
open Aeneas Aeneas.Std Result
namespace GenericConversions
/-- Direct theorem on the generated original core blanket body, for arbitrary input/output types. -/
theorem try_from_exact {T U : Type} (d : core.convert.Into U T) (x : U) :
 oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x =
 (do let y ← d.into x; .ok (.Ok y)) := rfl

theorem try_from_ok {T U : Type} (d : core.convert.Into U T) (x : U) (y : T)
 (h : d.into x = .ok y) : oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x = .ok (.Ok y) := by
 simp [oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from, h]
theorem try_from_fail {T U : Type} (d : core.convert.Into U T) (x : U) (e : Error)
 (h : d.into x = .fail e) : oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x = .fail e := by
 simp [oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from, h]
theorem try_from_div {T U : Type} (d : core.convert.Into U T) (x : U)
 (h : d.into x = .div) : oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x = .div := by
 simp [oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from, h]
theorem try_from_normal_no_err {T U : Type} (d : core.convert.Into U T) (x : U)
 (r : core.result.Result T Never) (_h : oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x = .ok r) :
 ∃ y, r = .Ok y := by
 cases r with
 | Ok y => exact ⟨y,rfl⟩
 | Err e => cases e

/-- Existing registered Std model: this is not a newly exported core body. -/
theorem into_model_exact {T U : Type} (d : core.convert.From U T) (x:T) :
 core.convert.IntoFrom.into d x = d.from x := rfl
theorem try_into_model_exact {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) :
 core.convert.TryInto.Blanket.try_into d x = d.try_from x := rfl

theorem rand32_try_from {U : Type} (d : core.convert.Into U oorandom.Rand32) (x:U) :
 oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x = (do let y ← d.into x; .ok (.Ok y)) := rfl
theorem rand64_try_from {U : Type} (d : core.convert.Into U oorandom.Rand64) (x:U) :
 oorandom_generic_convert_roots.core.convert.TryFromTUNever.Blanket.try_from d x = (do let y ← d.into x; .ok (.Ok y)) := rfl

theorem rand32_into_model {U : Type} (d : core.convert.From U oorandom.Rand32) (x:oorandom.Rand32) :
 core.convert.IntoFrom.into d x = d.from x := rfl
theorem rand64_into_model {U : Type} (d : core.convert.From U oorandom.Rand64) (x:oorandom.Rand64) :
 core.convert.IntoFrom.into d x = d.from x := rfl
theorem rand32_try_into_model {U E : Type} (d : core.convert.TryFrom U oorandom.Rand32 E) (x:oorandom.Rand32) :
 core.convert.TryInto.Blanket.try_into d x = d.try_from x := rfl
theorem rand64_try_into_model {U E : Type} (d : core.convert.TryFrom U oorandom.Rand64 E) (x:oorandom.Rand64) :
 core.convert.TryInto.Blanket.try_into d x = d.try_from x := rfl
end GenericConversions
