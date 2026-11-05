import GeneratedDelegations.CoreDelegations
import Oorandom
open Aeneas Aeneas.Std Result
namespace OriginalDelegationProofs
/-- Original core Into body, not a wrapper nor a substitute model. -/
theorem into_exact {T U : Type} (d : core.convert.From U T) (x:T) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x = d.from x := rfl
/-- Original core TryInto body forwards the complete Rust Ok/Err result. -/
theorem try_into_exact {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x = d.try_from x := rfl

theorem into_ok {T U : Type} (d : core.convert.From U T) (x:T) (y:U) (h:d.from x=.ok y) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x=.ok y := by simpa only [into_exact] using h
theorem into_fail {T U : Type} (d : core.convert.From U T) (x:T) (e:Error) (h:d.from x=.fail e) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x=.fail e := by simpa only [into_exact] using h
theorem into_div {T U : Type} (d : core.convert.From U T) (x:T) (h:d.from x=.div) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x=.div := by simpa only [into_exact] using h

theorem try_into_Ok {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) (y:U) (h:d.try_from x=.ok (.Ok y)) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=.ok (.Ok y) := by simpa only [try_into_exact] using h
theorem try_into_Err {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) (e:E) (h:d.try_from x=.ok (.Err e)) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=.ok (.Err e) := by simpa only [try_into_exact] using h
theorem try_into_fail {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) (e:Error) (h:d.try_from x=.fail e) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=.fail e := by simpa only [try_into_exact] using h
theorem try_into_div {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) (h:d.try_from x=.div) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=.div := by simpa only [try_into_exact] using h

theorem into_registered_model_agrees {T U : Type} (d : core.convert.From U T) (x:T) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x=core.convert.IntoFrom.into d x := rfl
theorem try_into_registered_model_agrees {T U E : Type} (d : core.convert.TryFrom U T E) (x:T) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=core.convert.TryInto.Blanket.try_into d x := rfl

theorem rand32_into {U : Type} (d : core.convert.From U oorandom.Rand32) (x:oorandom.Rand32) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x=d.from x := rfl
theorem rand64_into {U : Type} (d : core.convert.From U oorandom.Rand64) (x:oorandom.Rand64) :
 oorandom_core_delegations.core.convert.IntoFrom.into d x=d.from x := rfl
theorem rand32_try_into {U E : Type} (d : core.convert.TryFrom U oorandom.Rand32 E) (x:oorandom.Rand32) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=d.try_from x := rfl
theorem rand64_try_into {U E : Type} (d : core.convert.TryFrom U oorandom.Rand64 E) (x:oorandom.Rand64) :
 oorandom_core_delegations.core.convert.TryInto.Blanket.try_into d x=d.try_from x := rfl
end OriginalDelegationProofs
