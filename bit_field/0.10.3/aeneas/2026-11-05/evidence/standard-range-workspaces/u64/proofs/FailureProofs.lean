-- Kernel verification work module; historical draft retained in stage1-preserved.
import LayoutAudit
open Aeneas Aeneas.Std Result
open bitfield_range_u64_inline
namespace U64RangeFailure
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin Usize.rMax U64.rMax

theorem spec_eq_ok {α : Type} (m : Result α) (a : α)
 (h : WP.spec m (fun v => v=a)) : m=.ok a := by
 cases m using Result.cases with
 | ret v =>
  have hv : v=a := by simpa only [WP.spec_ok] using h
  subst v
  rfl
 | vis e k => exact False.elim (by simpa only [WP.spec_vis] using h)
 | div => exact False.elim (by simpa only [WP.spec_div] using h)

theorem bit_length : bitfield_range_u64_inline.U64.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 64#usize := by
 unfold bitfield_range_u64_inline.U64.Insts.Bit_fieldBitField.BIT_LENGTH
 rw [LayoutAudit.u64_size]
 simp only [bind_ok]
 apply spec_eq_ok
 refine WP.spec_mono (Usize.mul_spec (x:=8#usize) (y:=8#usize) (by scalar_tac)) ?_
 intro result h
 apply UScalar.val_eq_imp
 scalar_tac

theorem get_invalid_start (value : U64) (start finish : Usize) (bad : ¬start<64#usize) :
 original_u64_get_range value start finish=.fail .assertionFailure := by
 have hb : ¬start.val<64 := by scalar_tac
 simp [original_u64_get_range,bit_length,massert,hb]

theorem get_invalid_end (value : U64) (start finish : Usize) (hs : start<64#usize) (bad : ¬finish<=64#usize) :
 original_u64_get_range value start finish=.fail .assertionFailure := by
 have hsn : start.val<64 := by scalar_tac
 have hb : ¬finish.val<=64 := by scalar_tac
 simp [original_u64_get_range,bit_length,massert,hsn,hb]

theorem get_reversed (value : U64) (start finish : Usize) (hs : start<64#usize) (he : finish<=64#usize) (bad : ¬start<=finish) :
 original_u64_get_range value start finish=.fail .assertionFailure := by
 have hsn : start.val<64 := by scalar_tac
 have hen : finish.val<=64 := by scalar_tac
 have hb : ¬start.val<=finish.val := by scalar_tac
 simp [original_u64_get_range,bit_length,massert,hsn,hen,hb]

theorem set_invalid_start (value replacement : U64) (start finish : Usize) (bad : ¬start<64#usize) :
 original_u64_set_range value start finish replacement=.fail .assertionFailure := by
 have hb : ¬start.val<64 := by scalar_tac
 simp [original_u64_set_range,bit_length,massert,hb]

theorem set_invalid_end (value replacement : U64) (start finish : Usize) (hs : start<64#usize) (bad : ¬finish<=64#usize) :
 original_u64_set_range value start finish replacement=.fail .assertionFailure := by
 have hsn : start.val<64 := by scalar_tac
 have hb : ¬finish.val<=64 := by scalar_tac
 simp [original_u64_set_range,bit_length,massert,hsn,hb]

theorem set_reversed (value replacement : U64) (start finish : Usize) (hs : start<64#usize) (he : finish<=64#usize) (bad : ¬start<=finish) :
 original_u64_set_range value start finish replacement=.fail .assertionFailure := by
 have hsn : start.val<64 := by scalar_tac
 have hen : finish.val<=64 := by scalar_tac
 have hb : ¬start.val<=finish.val := by scalar_tac
 simp [original_u64_set_range,bit_length,massert,hsn,hen,hb]

theorem get_empty (value : U64) (start : Usize) (hs : start<64#usize) :
 original_u64_get_range value start start=.ok 0#u64 := by
 have he : start<=64#usize := by scalar_tac
 simp_all [original_u64_get_range,bit_length,massert,hs,he]

theorem set_empty_zero (value : U64) (start : Usize) (hs : start<64#usize) :
 original_u64_set_range value start start 0#u64=.ok (value,fun v=>v) := by
 have he : start<=64#usize := by scalar_tac
 simp_all [original_u64_set_range,bit_length,massert,hs,he]

#print axioms bit_length
#print axioms get_invalid_start
#print axioms get_invalid_end
#print axioms get_reversed
#print axioms set_invalid_start
#print axioms set_invalid_end
#print axioms set_reversed
#print axioms get_empty
#print axioms set_empty_zero
end U64RangeFailure
