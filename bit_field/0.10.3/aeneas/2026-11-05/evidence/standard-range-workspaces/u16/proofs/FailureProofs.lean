-- Kernel verification work module; historical draft retained in stage1-preserved.
import LayoutAudit
open Aeneas Aeneas.Std Result
open bitfield_range_u16_inline
namespace U16RangeFailure
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin Usize.rMax U16.rMax

theorem spec_eq_ok {α : Type} (m : Result α) (a : α)
 (h : WP.spec m (fun v => v=a)) : m=.ok a := by
 cases m using Result.cases with
 | ret v =>
  have hv : v=a := by simpa only [WP.spec_ok] using h
  subst v
  rfl
 | vis e k => exact False.elim (by simpa only [WP.spec_vis] using h)
 | div => exact False.elim (by simpa only [WP.spec_div] using h)

theorem bit_length : bitfield_range_u16_inline.U16.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 16#usize := by
 unfold bitfield_range_u16_inline.U16.Insts.Bit_fieldBitField.BIT_LENGTH
 rw [LayoutAudit.u16_size]
 simp only [bind_ok]
 apply spec_eq_ok
 refine WP.spec_mono (Usize.mul_spec (x:=2#usize) (y:=8#usize) (by scalar_tac)) ?_
 intro result h
 apply UScalar.val_eq_imp
 scalar_tac

theorem get_invalid_start (value : U16) (start finish : Usize) (bad : ¬start<16#usize) :
 original_u16_get_range value start finish=.fail .assertionFailure := by
 have hb : ¬start.val<16 := by scalar_tac
 simp [original_u16_get_range,bit_length,massert,hb]

theorem get_invalid_end (value : U16) (start finish : Usize) (hs : start<16#usize) (bad : ¬finish<=16#usize) :
 original_u16_get_range value start finish=.fail .assertionFailure := by
 have hsn : start.val<16 := by scalar_tac
 have hb : ¬finish.val<=16 := by scalar_tac
 simp [original_u16_get_range,bit_length,massert,hsn,hb]

theorem get_reversed (value : U16) (start finish : Usize) (hs : start<16#usize) (he : finish<=16#usize) (bad : ¬start<=finish) :
 original_u16_get_range value start finish=.fail .assertionFailure := by
 have hsn : start.val<16 := by scalar_tac
 have hen : finish.val<=16 := by scalar_tac
 have hb : ¬start.val<=finish.val := by scalar_tac
 simp [original_u16_get_range,bit_length,massert,hsn,hen,hb]

theorem set_invalid_start (value replacement : U16) (start finish : Usize) (bad : ¬start<16#usize) :
 original_u16_set_range value start finish replacement=.fail .assertionFailure := by
 have hb : ¬start.val<16 := by scalar_tac
 simp [original_u16_set_range,bit_length,massert,hb]

theorem set_invalid_end (value replacement : U16) (start finish : Usize) (hs : start<16#usize) (bad : ¬finish<=16#usize) :
 original_u16_set_range value start finish replacement=.fail .assertionFailure := by
 have hsn : start.val<16 := by scalar_tac
 have hb : ¬finish.val<=16 := by scalar_tac
 simp [original_u16_set_range,bit_length,massert,hsn,hb]

theorem set_reversed (value replacement : U16) (start finish : Usize) (hs : start<16#usize) (he : finish<=16#usize) (bad : ¬start<=finish) :
 original_u16_set_range value start finish replacement=.fail .assertionFailure := by
 have hsn : start.val<16 := by scalar_tac
 have hen : finish.val<=16 := by scalar_tac
 have hb : ¬start.val<=finish.val := by scalar_tac
 simp [original_u16_set_range,bit_length,massert,hsn,hen,hb]

theorem get_empty (value : U16) (start : Usize) (hs : start<16#usize) :
 original_u16_get_range value start start=.ok 0#u16 := by
 have he : start<=16#usize := by scalar_tac
 simp_all [original_u16_get_range,bit_length,massert,hs,he]

theorem set_empty_zero (value : U16) (start : Usize) (hs : start<16#usize) :
 original_u16_set_range value start start 0#u16=.ok (value,fun v=>v) := by
 have he : start<=16#usize := by scalar_tac
 simp_all [original_u16_set_range,bit_length,massert,hs,he]

#print axioms bit_length
#print axioms get_invalid_start
#print axioms get_invalid_end
#print axioms get_reversed
#print axioms set_invalid_start
#print axioms set_invalid_end
#print axioms set_reversed
#print axioms get_empty
#print axioms set_empty_zero
end U16RangeFailure
