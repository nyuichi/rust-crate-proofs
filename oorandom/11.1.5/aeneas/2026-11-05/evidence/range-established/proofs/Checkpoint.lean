import Oorandom.Funs
open Aeneas Aeneas.Std Result
open oorandom_range_original
namespace RangeProofs
set_option maxRecDepth 4096
set_option maxHeartbeats 3000000
def output32 (s : U64) : U32 :=
  let x : U64 := ⟨((s.bv >>> 18) ^^^ s.bv) >>> 27⟩
  let r : U64 := ⟨s.bv >>> 59⟩
  (UScalar.cast .U32 x).rotate_right (UScalar.cast .U32 r)
def advance32 (s i : U64) : U64 :=
  (s.wrapping_mul 6364136223846793005#u64).wrapping_add i

theorem rand_u32_spec (s : Rand32) :
    Rand32.rand_u32 s =
      .ok (output32 s.state, { state := advance32 s.state s.inc, inc := s.inc }) := by
  simp only [Rand32.rand_u32, Rand32.MULTIPLIER,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    output32, advance32, U64.wrapping_mul, U64.wrapping_add,
    core.num.U64.wrapping_mul, core.num.U64.wrapping_add, core.num.U32.rotate_right]
  simp only [IScalar.toNat]
  dsimp [lift, Bind.bind, HXor.hXor, UScalar.xor,
    UScalar.wrapping_mul, UScalar.wrapping_add]
  simp only [bind_ok]
  rfl

def zero32 : Rand32 := ⟨0#u64,0#u64⟩
theorem zero32_step : Rand32.rand_u32 zero32 = .ok (0#u32,zero32) := by
  rw [rand_u32_spec]
  congr 1 <;> rfl

theorem zero32_body : Rand32.rand_range_loop.body 3#u32 1#u32 zero32 0#u64 0#u32 =
    .ok (.cont (zero32,0#u64,0#u32)) := by
  simp [Rand32.rand_range_loop.body,zero32_step,lift,
    core.convert.num.FromU64U32.from,core.num.U64.wrapping_mul,
    UScalar.cast, HAnd.hAnd, UScalar.and, UScalar.wrapping_mul]
  constructor <;> rfl
section
unseal Result
attribute [local implicit_reducible] Result
theorem loop_self_div {α β : Type} (body : α → Result (ControlFlow α β))
    (x : α) (hbody : body x = .ok (.cont x)) : loop body x = .div := by
  apply Data.Coinductive.ITree.le_div_is_div
  refine loop.fixpoint_induct body
    (fun f => Lean.Order.PartialOrder.rel (f x) (.div : Result β))
    (Lean.Order.admissible_apply (fun _ y => Lean.Order.PartialOrder.rel y (.div : Result β)) x
      (fun _ hc h => Lean.Order.csup_le hc h)) ?_
  intro f hf
  simpa only [hbody, bind_ok] using hf

end
theorem zero32_loop_div : Rand32.rand_range_loop zero32 3#u32 0#u64 0#u32 1#u32 = .div := by
  unfold Rand32.rand_range_loop
  apply loop_self_div
  exact zero32_body
def prefix32 : Result (Rand32 × U32 × U64 × U32) := do
  massert ((0#u32 : U32) < 3#u32)
  let s ← 3#u32 - 0#u32
  let (i,self) ← Rand32.rand_u32 zero32
  let i1 ← lift (core.convert.num.FromU64U32.from i)
  let i2 ← lift (core.convert.num.FromU64U32.from s)
  let m ← i1 * i2
  let i3 ← lift (m &&& 4294967295#u64)
  let leftover ← lift (UScalar.cast .U32 i3)
  ok (self,s,m,leftover)
theorem prefix32_spec : prefix32 ⦃ p => p = (zero32,3#u32,0#u64,0#u32) ⦄ := by
  unfold prefix32
  step
  step as ⟨s,hs⟩
  have he : s = 3#u32 := by scalar_tac
  subst s
  simp only [zero32_step,bind_ok,core.convert.num.FromU64U32.from,lift]
  have h0 : (⟨BitVec.setWidth UScalarTy.U64.numBits (U32.bv 0#u32)⟩ : U64).val = 0 := by decide
  have h3 : (⟨BitVec.setWidth UScalarTy.U64.numBits (U32.bv 3#u32)⟩ : U64).val = 3 := by decide
  step as ⟨m,hm⟩
  rw [h0,h3] at hm
  have hz : m = 0#u64 := by scalar_tac
  subst m
  constructor <;> rfl

theorem spec_eq_ok {α : Type} (m : Result α) (a : α)
    (h : WP.spec m (fun v => v = a)) : m = .ok a := by
  cases m using Result.cases with
  | ret v =>
    have he : v = a := by simpa only [WP.spec_ok] using h
    subst v
    rfl
  | vis e k => exact False.elim (by simpa only [WP.spec_vis] using h)
  | div => exact False.elim (by simpa only [WP.spec_div] using h)

def finish32 (p : Rand32 × U32 × U64 × U32) : Result (U32 × Rand32) := do
  let (self1,s,m,leftover) := p
  if leftover < s then
    let i4 ← core.num.U32.wrapping_neg s
    let threshold ← i4 % s
    let (self2,m1) ← Rand32.rand_range_loop self1 s m leftover threshold
    let i5 ← m1 >>> 32#i32
    let i6 ← lift (UScalar.cast .U32 i5)
    let i7 ← i6 + 0#u32
    ok (i7,self2)
  else
    let i4 ← m >>> 32#i32
    let i5 ← lift (UScalar.cast .U32 i4)
    let i6 ← i5 + 0#u32
    ok (i6,self1)

end RangeProofs
#print axioms RangeProofs.zero32_step
#print axioms RangeProofs.zero32_body

#print axioms RangeProofs.loop_self_div
#print axioms RangeProofs.zero32_loop_div


#print axioms RangeProofs.prefix32_spec


