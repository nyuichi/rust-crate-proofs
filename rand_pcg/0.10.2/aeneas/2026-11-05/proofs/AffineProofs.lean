import PcgAffineAdvance.Funs
import Affine
open Aeneas Aeneas.Std Result Std.WP
open PcgAffineAdvanceModel
namespace AffineProofs
set_option maxHeartbeats 2000000
abbrev State128 := U128 × U128 × U128 × U128 × U128

def bodyPost128 (a p c q d : U128) (r : ControlFlow State128 (U128 × U128)) : Prop :=
  match r with
  | .done (a',p') => (a'.bv,p'.bv)=
      AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val)
  | .cont (a',p',c',q',d') =>
      AffineAdvance.mul (a'.bv,p'.bv) (AffineAdvance.power (c'.bv,q'.bv) d'.val)=
      AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val) ∧ d'.val<d.val

theorem body128_spec (a p c q d : U128) :
    pcg128.Lcg128Xsl64.advance_loop.body a p c q d ⦃ bodyPost128 a p c q d ⦄ := by
  unfold pcg128.Lcg128Xsl64.advance_loop.body
  step*
  all_goals try
    have hz : d.val=0 := by scalar_tac
    simp [bodyPost128,hz]
  all_goals (split <;> step*)
  all_goals
    have hlow : i.val=d.val%2 := by
      rw [i_post]
      change (d.bv &&& (1 : BitVec 128)).toNat=d.bv.toNat%2
      simp
    subst_vars
    simp only [bodyPost128,core.num.U128.wrapping_mul_bv_eq,
      core.num.U128.wrapping_add_bv_eq,U128.ofNat_bv,UScalarTy.U128_numBits_eq]
    rw [AffineAdvance.square_pair]
    constructor
    · rw [mdelta1_post]
      first
      | have hp : d.val%2=0 := by scalar_tac
        rw [AffineAdvance.binary_power (c.bv,q.bv) d.val,ite_eq_left hp]
      | have hp : d.val%2≠0 := by scalar_tac
        rw [AffineAdvance.binary_power (c.bv,q.bv) d.val,ite_eq_right hp]
        exact AffineAdvance.mul_assoc (a.bv,p.bv) (c.bv,q.bv)
          (AffineAdvance.power (AffineAdvance.mul (c.bv,q.bv) (c.bv,q.bv)) (d.val/2))
    · have hp : 0<d.val := by scalar_tac
      rw [mdelta1_post]
      exact Nat.div_lt_self hp (by decide)

def loopInv128 (target : AffineAdvance.Affine 128) (x : State128) : Prop :=
  let (a,p,c,q,d) := x
  AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val)=target

theorem loop128_spec (a p c q d : U128) :
    pcg128.Lcg128Xsl64.advance_loop a p c q d
      ⦃ r => (r.1.bv,r.2.bv)=AffineAdvance.mul (a.bv,p.bv)
        (AffineAdvance.power (c.bv,q.bv) d.val) ⦄ := by
  unfold pcg128.Lcg128Xsl64.advance_loop
  apply loop.spec_decr_nat
    (fun x : State128 => x.2.2.2.2.val)
    (loopInv128 (AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val)))
  · intro x hx
    rcases x with ⟨a',p',c',q',d'⟩
    apply WP.spec_mono (body128_spec a' p' c' q' d')
    intro r hr
    cases r with
    | done z => exact hr.trans hx
    | cont x => exact ⟨hr.1.trans hx,hr.2⟩
  · rfl

theorem lcg128_advance_spec (s : pcg128.Lcg128Xsl64) (delta : U128) :
    s.advance delta ⦃ out => out.state.bv=AffineAdvance.iterate
      47026247687942121848144207491837523525#128 s.increment.bv delta.val s.state.bv ∧
      out.increment=s.increment ⦄ := by
  unfold pcg128.Lcg128Xsl64.advance
  apply WP.spec_bind (loop128_spec 1#u128 0#u128 pcg128.MULTIPLIER s.increment delta)
  rintro ⟨acc,plus⟩ ht
  dsimp only at ht ⊢
  have ht' : (acc.bv,plus.bv)=AffineAdvance.power
      (47026247687942121848144207491837523525#128,s.increment.bv) delta.val := by
    simp only [pcg128.MULTIPLIER,U128.ofNat_bv,UScalarTy.U128_numBits_eq] at ht
    change (acc.bv,plus.bv)=AffineAdvance.mul (AffineAdvance.one 128)
      (AffineAdvance.power (47026247687942121848144207491837523525#128,s.increment.bv) delta.val) at ht
    simpa only [AffineAdvance.one_mul] using ht
  step*
  subst_vars
  simp only [core.num.U128.wrapping_add_bv_eq,core.num.U128.wrapping_mul_bv_eq]
  change AffineAdvance.applyAffine (acc.bv,plus.bv) s.state.bv=_
  rw [ht',AffineAdvance.apply_power]
abbrev State64 := U64 × U64 × U64 × U64 × U64

def bodyPost64 (a p c q d : U64) (r : ControlFlow State64 (U64 × U64)) : Prop :=
  match r with
  | .done (a',p') => (a'.bv,p'.bv)=
      AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val)
  | .cont (a',p',c',q',d') =>
      AffineAdvance.mul (a'.bv,p'.bv) (AffineAdvance.power (c'.bv,q'.bv) d'.val)=
      AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val) ∧ d'.val<d.val

theorem body64_spec (a p c q d : U64) :
    pcg64.Lcg64Xsh32.advance_loop.body a p c q d ⦃ bodyPost64 a p c q d ⦄ := by
  unfold pcg64.Lcg64Xsh32.advance_loop.body
  step*
  all_goals try
    have hz : d.val=0 := by scalar_tac
    simp [bodyPost64,hz]
  all_goals (split <;> step*)
  all_goals
    have hlow : i.val=d.val%2 := by
      rw [i_post]
      change (d.bv &&& (1 : BitVec 64)).toNat=d.bv.toNat%2
      simp
    subst_vars
    simp only [bodyPost64,core.num.U64.wrapping_mul_bv_eq,
      core.num.U64.wrapping_add_bv_eq,U64.ofNat_bv,UScalarTy.U64_numBits_eq]
    rw [AffineAdvance.square_pair]
    constructor
    · rw [mdelta1_post]
      first
      | have hp : d.val%2=0 := by scalar_tac
        rw [AffineAdvance.binary_power (c.bv,q.bv) d.val,ite_eq_left hp]
      | have hp : d.val%2≠0 := by scalar_tac
        rw [AffineAdvance.binary_power (c.bv,q.bv) d.val,ite_eq_right hp]
        exact AffineAdvance.mul_assoc (a.bv,p.bv) (c.bv,q.bv)
          (AffineAdvance.power (AffineAdvance.mul (c.bv,q.bv) (c.bv,q.bv)) (d.val/2))
    · have hp : 0<d.val := by scalar_tac
      rw [mdelta1_post]
      exact Nat.div_lt_self hp (by decide)

def loopInv64 (target : AffineAdvance.Affine 64) (x : State64) : Prop :=
  let (a,p,c,q,d) := x
  AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val)=target

theorem loop64_spec (a p c q d : U64) :
    pcg64.Lcg64Xsh32.advance_loop a p c q d
      ⦃ r => (r.1.bv,r.2.bv)=AffineAdvance.mul (a.bv,p.bv)
        (AffineAdvance.power (c.bv,q.bv) d.val) ⦄ := by
  unfold pcg64.Lcg64Xsh32.advance_loop
  apply loop.spec_decr_nat
    (fun x : State64 => x.2.2.2.2.val)
    (loopInv64 (AffineAdvance.mul (a.bv,p.bv) (AffineAdvance.power (c.bv,q.bv) d.val)))
  · intro x hx
    rcases x with ⟨a',p',c',q',d'⟩
    apply WP.spec_mono (body64_spec a' p' c' q' d')
    intro r hr
    cases r with
    | done z => exact hr.trans hx
    | cont x => exact ⟨hr.1.trans hx,hr.2⟩
  · rfl

theorem lcg64_advance_spec (s : pcg64.Lcg64Xsh32) (delta : U64) :
    s.advance delta ⦃ out => out.state.bv=AffineAdvance.iterate
      6364136223846793005#64 s.increment.bv delta.val s.state.bv ∧
      out.increment=s.increment ⦄ := by
  unfold pcg64.Lcg64Xsh32.advance
  apply WP.spec_bind (loop64_spec 1#u64 0#u64 pcg64.MULTIPLIER s.increment delta)
  rintro ⟨acc,plus⟩ ht
  dsimp only at ht ⊢
  have ht' : (acc.bv,plus.bv)=AffineAdvance.power
      (6364136223846793005#64,s.increment.bv) delta.val := by
    simp only [pcg64.MULTIPLIER,U64.ofNat_bv,UScalarTy.U64_numBits_eq] at ht
    change (acc.bv,plus.bv)=AffineAdvance.mul (AffineAdvance.one 64)
      (AffineAdvance.power (6364136223846793005#64,s.increment.bv) delta.val) at ht
    simpa only [AffineAdvance.one_mul] using ht
  step*
  subst_vars
  simp only [core.num.U64.wrapping_add_bv_eq,core.num.U64.wrapping_mul_bv_eq]
  change AffineAdvance.applyAffine (acc.bv,plus.bv) s.state.bv=_
  rw [ht',AffineAdvance.apply_power]

theorem cm_advance_spec (s : pcg128cm.Lcg128CmDxsm64) (delta : U128) :
    s.advance delta ⦃ out => out.state.bv=AffineAdvance.iterate
      15750249268501108917#128 s.increment.bv delta.val s.state.bv ∧
      out.increment=s.increment ⦄ := by
  unfold pcg128cm.Lcg128CmDxsm64.advance
  have hc : (UScalar.cast .U128 pcg128cm.MULTIPLIER : U128)=15750249268501108917#u128 := by
    unfold pcg128cm.MULTIPLIER
    rfl
  rw [hc]
  simp only [lift,bind_ok]
  have hloop : pcg128cm.Lcg128CmDxsm64.advance_loop=pcg128.Lcg128Xsl64.advance_loop := by rfl
  rw [hloop]
  apply WP.spec_bind (loop128_spec 1#u128 0#u128 15750249268501108917#u128 s.increment delta)
  rintro ⟨acc,plus⟩ ht
  dsimp only at ht ⊢
  have ht' : (acc.bv,plus.bv)=AffineAdvance.power
      (15750249268501108917#128,s.increment.bv) delta.val := by
    simp only [U128.ofNat_bv,UScalarTy.U128_numBits_eq] at ht
    change (acc.bv,plus.bv)=AffineAdvance.mul (AffineAdvance.one 128)
      (AffineAdvance.power (15750249268501108917#128,s.increment.bv) delta.val) at ht
    simpa only [AffineAdvance.one_mul] using ht
  step*
  subst_vars
  simp only [core.num.U128.wrapping_add_bv_eq,core.num.U128.wrapping_mul_bv_eq]
  change AffineAdvance.applyAffine (acc.bv,plus.bv) s.state.bv=_
  rw [ht',AffineAdvance.apply_power]
end AffineProofs
#print axioms AffineProofs.body128_spec

#print axioms AffineProofs.loop128_spec
#print axioms AffineProofs.lcg128_advance_spec

#print axioms AffineProofs.body64_spec
#print axioms AffineProofs.loop64_spec
#print axioms AffineProofs.lcg64_advance_spec

#print axioms AffineProofs.cm_advance_spec
