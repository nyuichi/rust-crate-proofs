import EndianConditionalOriginal
open Aeneas Aeneas.Std
open endian_conditional_original
namespace ConditionalProofs
theorem big_into_exact {T U : Type} (d : core.convert.From U (endian_type.BigEndian T)) (x : endian_type.BigEndian T) : big_into d x = d.«from» x := by simp [big_into]
theorem big_try_into_exact {T U E : Type} (d : core.convert.TryFrom U (endian_type.BigEndian T) E) (x : endian_type.BigEndian T) : big_try_into d x = d.try_from x := by simp [big_try_into]
theorem big_try_from_exact {T U : Type} (d : core.convert.Into U (endian_type.BigEndian T)) (x : U) : big_try_from d x = (do let y ← d.into x; pure (core.result.Result.Ok y)) := by rfl
theorem little_into_exact {T U : Type} (d : core.convert.From U (endian_type.LittleEndian T)) (x : endian_type.LittleEndian T) : little_into d x = d.«from» x := by simp [little_into]
theorem little_try_into_exact {T U E : Type} (d : core.convert.TryFrom U (endian_type.LittleEndian T) E) (x : endian_type.LittleEndian T) : little_try_into d x = d.try_from x := by simp [little_try_into]
theorem little_try_from_exact {T U : Type} (d : core.convert.Into U (endian_type.LittleEndian T)) (x : U) : little_try_from d x = (do let y ← d.into x; pure (core.result.Result.Ok y)) := by rfl
#print axioms big_into_exact
#print axioms big_try_into_exact
#print axioms big_try_from_exact
#print axioms little_into_exact
#print axioms little_try_into_exact
#print axioms little_try_from_exact
end ConditionalProofs
