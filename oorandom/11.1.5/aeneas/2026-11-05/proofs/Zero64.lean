import Checkpoint
import NegBridge
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
namespace Zero64
set_option maxRecDepth 8192
set_option maxHeartbeats 5000000
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin U32.rMax U64.rMax U128.rMax I32.rMax I32.rMin I64.rMax I64.rMin I128.rMax I128.rMin
def output64 (s : U128) : U64 :=
 let x : U128 := ⟨((s.bv >>> 29) ^^^ s.bv) >>> 58⟩
 let r : U128 := ⟨s.bv >>> 122⟩
 (UScalar.cast .U64 x).rotate_right (UScalar.cast .U32 r)
def advance64 (s i : U128) : U128 :=
 (s.wrapping_mul 47026247687942121848144207491837523525#u128).wrapping_add i
theorem next_spec (s : Rand64) :
 Rand64.rand_u64 s = .ok (output64 s.state,{state := advance64 s.state s.inc,inc := s.inc}) := by
 simp only [Rand64.rand_u64,Rand64.MULTIPLIER,HShiftRight.hShiftRight,
  UScalar.shiftRight_IScalar,UScalar.shiftRight,output64,advance64,
  U128.wrapping_mul,U128.wrapping_add,core.num.U128.wrapping_mul,
  core.num.U128.wrapping_add,core.num.U64.rotate_right]
 simp only [IScalar.toNat]
 dsimp [lift,Bind.bind,HXor.hXor,UScalar.xor,UScalar.wrapping_mul,UScalar.wrapping_add]
 simp only [bind_ok,Std.uncurry]
 rfl
def zero64 : Rand64 := ⟨0#u128,0#u128⟩
theorem zero_step : Rand64.rand_u64 zero64 = .ok (0#u64,zero64) := by
 rw [next_spec]
 congr 1 <;> rfl
theorem mul_zero_three : (0#u128 * 3#u128 : Result U128) = .ok 0#u128 := by
 apply spec_eq_ok
 refine WP.spec_mono (UScalar.mul_spec (x:=0#u128) (y:=3#u128) (by scalar_tac)) ?_
 intro z hz
 scalar_tac
theorem zero_body : Rand64.rand_range_loop.body 3#u64 1#u64 zero64 0#u128 0#u64 =
 .ok (.cont (zero64,0#u128,0#u64)) := by
 simp [Rand64.rand_range_loop.body,zero_step,lift,Std.uncurry,
  core.convert.num.FromU128U64.from,core.num.U128.wrapping_mul,UScalar.cast,
  HAnd.hAnd,UScalar.and,UScalar.wrapping_mul]
 have hzero : ((0#128)#uscalar : U128) = 0#u128 := by apply UScalar.val_eq_imp; decide
 have hthree : ((3#128)#uscalar : U128) = 3#u128 := by apply UScalar.val_eq_imp; decide
 rw [hzero,hthree,mul_zero_three]
 simp only [bind_ok]
 rfl
theorem zero_loop_div : Rand64.rand_range_loop zero64 3#u64 0#u128 0#u64 1#u64 = .div := by
 unfold Rand64.rand_range_loop
 apply loop_self_div
 exact zero_body
def prefix64 : Result (Rand64 × U64 × U128 × U64) := do
 massert ((0#u64 : U64) < 3#u64)
 let s ← 3#u64 - 0#u64
 let (i,self) ← Rand64.rand_u64 zero64
 let i1 ← lift (core.convert.num.FromU128U64.from i)
 let i2 ← lift (core.convert.num.FromU128U64.from s)
 let m ← i1 * i2
 let i3 ← lift (m &&& 18446744073709551615#u128)
 let leftover ← lift (UScalar.cast .U64 i3)
 ok (self,s,m,leftover)
theorem prefix_spec : prefix64 ⦃ p => p = (zero64,3#u64,0#u128,0#u64) ⦄ := by
 unfold prefix64
 step
 step as ⟨s,hs⟩
 have he : s = 3#u64 := by scalar_tac
 subst s
 simp only [zero_step,bind_ok,core.convert.num.FromU128U64.from,lift,Std.uncurry]
 have h0 : (⟨BitVec.setWidth UScalarTy.U128.numBits (U64.bv 0#u64)⟩ : U128).val = 0 := by decide
 have h3 : (⟨BitVec.setWidth UScalarTy.U128.numBits (U64.bv 3#u64)⟩ : U128).val = 3 := by decide
 step as ⟨m,hm⟩
 rw [h0,h3] at hm
 have hz : m = 0#u128 := by scalar_tac
 subst m
 constructor <;> rfl
def finish64 (p : Rand64 × U64 × U128 × U64) : Result (U64 × Rand64) := do
 let (self,s,m,leftover) := p
 if leftover < s then
  let neg ← core.num.U64.wrapping_neg s
  let threshold ← neg % s
  let (self1,m1) ← Rand64.rand_range_loop self s m leftover threshold
  let high ← lift (core.num.U128.wrapping_shr m1 64#u32)
  let out ← lift (UScalar.cast .U64 high)
  let result ← out + 0#u64
  ok (result,self1)
 else
  let high ← lift (core.num.U128.wrapping_shr m 64#u32)
  let out ← lift (UScalar.cast .U64 high)
  let result ← out + 0#u64
  ok (result,self)
theorem decompose : Rand64.rand_range zero64 ⟨0#u64,3#u64⟩ =
 (do let p ← prefix64;finish64 p) := by
 simp only [Rand64.rand_range,prefix64,finish64,Std.bind_assoc,bind_ok,bind_tc_ok,Std.uncurry]
 all_goals rfl
theorem finish_div : finish64 (zero64,3#u64,0#u128,0#u64) = .div := by
 have hn : core.num.U64.wrapping_neg 3#u64 = .ok 18446744073709551613#u64 := by rfl
 have ht : (18446744073709551613#u64 % 3#u64 : Result U64) = .ok 1#u64 := by rfl
 dsimp only [finish64,Std.uncurry]
 simp only [if_pos (show (0#u64 : U64)<3#u64 from by decide),hn,bind_ok,ht,zero_loop_div,bind_div]
theorem public_div : Rand64.rand_range zero64 ⟨0#u64,3#u64⟩ = .div := by
 rw [decompose,spec_eq_ok prefix64 _ prefix_spec]
 simpa only [bind_ok] using finish_div
#print axioms next_spec
#print axioms public_div
end Zero64
