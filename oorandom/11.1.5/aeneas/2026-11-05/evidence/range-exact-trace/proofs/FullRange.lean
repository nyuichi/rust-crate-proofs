import Checkpoint
import NegBridge
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
set_option maxRecDepth 8192
set_option maxHeartbeats 5000000
namespace FullRange
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin U32.rMax U64.rMax U128.rMax I32.rMax I32.rMin I64.rMax I64.rMin I128.rMax I128.rMin
theorem zero32_decompose : Rand32.rand_range zero32 ⟨0#u32,3#u32⟩ =
 (do let p ← prefix32; finish32 p) := by
 simp only [Rand32.rand_range,prefix32,finish32,Std.bind_assoc,Std.bind_assoc_eq,bind_ok,bind_tc_ok,Std.uncurry]
 all_goals first | rfl | simp only [Std.bind_assoc,bind_ok,bind_tc_ok,Std.uncurry]
theorem zero32_finish_div : finish32 (zero32,3#u32,0#u64,0#u32) = .div := by
 have hn : core.num.U32.wrapping_neg 3#u32 = .ok 4294967293#u32 := by rfl
 have ht : (4294967293#u32 % 3#u32 : Result U32) = .ok 1#u32 := by rfl
 dsimp only [finish32,Std.uncurry]
 simp only [if_pos (show (0#u32 : U32) < 3#u32 from by decide), hn, bind_ok,ht,zero32_loop_div,bind_div]
theorem zero32_public_div : Rand32.rand_range zero32 ⟨0#u32,3#u32⟩ = .div := by
 rw [zero32_decompose,spec_eq_ok prefix32 _ prefix32_spec]
 simpa only [bind_ok] using zero32_finish_div
#print axioms zero32_public_div
end FullRange
