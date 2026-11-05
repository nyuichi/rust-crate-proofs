import PcgAdvance.Funs
open Aeneas Aeneas.Std Result Std.WP
open PcgAdvanceModel
namespace AdvanceProofs
set_option maxHeartbeats 2000000

theorem square_pow (x : BitVec w) (n : Nat) : (x*x)^n = x^(n+n) := by
  induction n with
  | zero => simp
  | succ n ih =>
    rw [BitVec.pow_succ, ih]
    have h : n+1+(n+1) = (n+n)+1+1 := by omega
    rw [h, BitVec.pow_succ, BitVec.pow_succ]
    exact (BitVec.mul_assoc _ _ _).symm

theorem binary_pow (x : BitVec w) (n : Nat) :
    x^n = if n%2=0 then (x*x)^(n/2) else x*(x*x)^(n/2) := by
  rw [square_pow]
  split
  next h => congr 1; omega
  next h =>
    have hn : n = (n/2+n/2)+1 := by omega
    calc
      x^n = x^((n/2+n/2)+1) := congrArg (fun k : Nat => x^k) hn
      _ = x^(n/2+n/2)*x := BitVec.pow_succ
      _ = x*x^(n/2+n/2) := BitVec.mul_comm _ _

abbrev LoopState := U128 × U128 × U128 × U128 × U128

def bodyPost (a c d : U128) (r : ControlFlow LoopState (U128 × U128)) : Prop :=
  match r with
  | .done (a',p') => a'.bv = a.bv*c.bv^d.val ∧ p'=0#u128
  | .cont (a',p',c',q',d') => p'=0#u128 ∧ q'=0#u128 ∧
      a'.bv*c'.bv^d'.val=a.bv*c.bv^d.val ∧ d'.val<d.val

theorem body_zero_spec (a c d : U128) :
    pcg128.Mcg128Xsl64.advance_loop.body a 0#u128 c 0#u128 d ⦃ bodyPost a c d ⦄ := by
  unfold pcg128.Mcg128Xsl64.advance_loop.body
  step*
  all_goals try
    have hz : d.val=0 := by scalar_tac
    simp [bodyPost, hz]
  all_goals (split <;> step*)
  all_goals
    have hlow : i.val=d.val%2 := by
      rw [i_post]
      change (d.bv &&& (1 : BitVec 128)).toNat=d.bv.toNat%2
      simp
    subst_vars
    simp only [bodyPost, UScalar.eq_equiv_bv_eq,
      core.num.U128.wrapping_mul_bv_eq, core.num.U128.wrapping_add_bv_eq,
      U128.ofNat_bv, UScalarTy.U128_numBits_eq, BitVec.zero_mul, BitVec.mul_zero, BitVec.add_zero, true_and]
    constructor
    · rw [mdelta1_post]
      first
      | have hp : d.val%2=0 := by scalar_tac
        rw [binary_pow c.bv d.val, ite_eq_left hp]
      | have hp : d.val%2≠0 := by scalar_tac
        rw [binary_pow c.bv d.val, ite_eq_right hp]
        exact BitVec.mul_assoc _ _ _
    · have hp : 0<d.val := by scalar_tac
      rw [mdelta1_post]
      exact Nat.div_lt_self hp (by decide)

def loopInv (target : BitVec 128) (x : LoopState) : Prop :=
  let (a,p,c,q,d) := x
  p=0#u128 ∧ q=0#u128 ∧ a.bv*c.bv^d.val=target

theorem loop_pow_spec (a c d : U128) :
    pcg128.Mcg128Xsl64.advance_loop a 0#u128 c 0#u128 d
      ⦃ r => r.1.bv=a.bv*c.bv^d.val ∧ r.2=0#u128 ⦄ := by
  unfold pcg128.Mcg128Xsl64.advance_loop
  apply loop.spec_decr_nat
    (fun x : LoopState => x.2.2.2.2.val)
    (loopInv (a.bv*c.bv^d.val))
  · intro x hx
    rcases x with ⟨a',p',c',q',d'⟩
    rcases hx with ⟨hp,hq,ht⟩
    subst p'; subst q'
    apply WP.spec_mono (body_zero_spec a' c' d')
    intro r hr
    cases r with
    | done z => exact ⟨hr.1.trans ht,hr.2⟩
    | cont x =>
      rcases x with ⟨a1,p1,c1,q1,d1⟩
      exact ⟨⟨hr.1,hr.2.1,hr.2.2.1.trans ht⟩,hr.2.2.2⟩
  · exact ⟨rfl,rfl,rfl⟩

theorem mcg_advance_spec (s : pcg128.Mcg128Xsl64) (delta : U128) :
    s.advance delta ⦃ out => out.state.bv=
      (47026247687942121848144207491837523525#128)^delta.val*s.state.bv ⦄ := by
  unfold pcg128.Mcg128Xsl64.advance
  apply WP.spec_bind (loop_pow_spec 1#u128 pcg128.MULTIPLIER delta)
  rintro ⟨acc,plus⟩ ⟨ha,hp⟩
  dsimp only at ha hp ⊢
  subst plus
  have hac : acc.bv=(47026247687942121848144207491837523525#128)^delta.val := by
    simpa only [pcg128.MULTIPLIER, U128.ofNat_bv, UScalarTy.U128_numBits_eq,
      BitVec.one_mul] using ha
  step*
  subst_vars
  simp only [core.num.U128.wrapping_add_bv_eq, core.num.U128.wrapping_mul_bv_eq,
    U128.ofNat_bv, UScalarTy.U128_numBits_eq, BitVec.add_zero, hac]
end AdvanceProofs
#print axioms AdvanceProofs.square_pow
#print axioms AdvanceProofs.binary_pow

#print axioms AdvanceProofs.body_zero_spec

#print axioms AdvanceProofs.loop_pow_spec

#print axioms AdvanceProofs.mcg_advance_spec
