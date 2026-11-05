import FullRange
import Zero64
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
namespace PublicDomain
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin U32.rMax U64.rMax U128.rMax I32.rMax I32.rMin I64.rMax I64.rMin I128.rMax I128.rMin
theorem from_state32_exact (s i : U64) : Rand32.from_state (s,i) = .ok ⟨s,i⟩ := by rfl
theorem from_state64_exact (s i : U128) : Rand64.from_state (s,i) = .ok ⟨s,i⟩ := by rfl
theorem from_state32_range3_div :
 (do let r ← Rand32.from_state (0#u64,0#u64); Rand32.rand_range r ⟨0#u32,3#u32⟩) = .div := by
 rw [from_state32_exact]
 simpa only [bind_ok,bind_tc_ok,zero32] using FullRange.zero32_public_div
theorem from_state64_range3_div :
 (do let r ← Rand64.from_state (0#u128,0#u128); Rand64.rand_range r ⟨0#u64,3#u64⟩) = .div := by
 rw [from_state64_exact]
 simpa only [bind_ok,bind_tc_ok,Zero64.zero64] using Zero64.public_div
theorem range32_invalid (r : Rand32) (range : core.ops.range.Range U32) (bad : ¬range.start < range.end) :
 Rand32.rand_range r range = .fail .assertionFailure := by
 simp [Rand32.rand_range,massert,bad]
theorem range64_invalid (r : Rand64) (range : core.ops.range.Range U64) (bad : ¬range.start < range.end) :
 Rand64.rand_range r range = .fail .assertionFailure := by
 simp [Rand64.rand_range,massert,bad]
#print axioms from_state32_range3_div
#print axioms from_state64_range3_div
#print axioms range32_invalid
#print axioms range64_invalid
end PublicDomain
