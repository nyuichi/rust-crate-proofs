import RandPcg
open Aeneas Aeneas.Std Result
open rand_pcg
namespace Proofs
set_option linter.unusedSimpArgs false
/-- PCG-XSH-RR output permutation, using logical 64-bit shifts, low 32 bits,
    and a 32-bit rotate. -/
def pcgOutput (s : U64) : U32 :=
  let x : U64 := ⟨((s.bv >>> 18) ^^^ s.bv) >>> 27⟩
  let r : U64 := ⟨s.bv >>> 59⟩
  (UScalar.cast .U32 x).rotate_right (UScalar.cast .U32 r)
def pcgState (s i : U64) : U64 :=
  (s.wrapping_mul 6364136223846793005#u64).wrapping_add i

theorem try_next_u32_spec (s : rand_pcg.pcg64.Lcg64Xsh32) :
    rand_pcg.pcg64.Lcg64Xsh32.Insts.Rand_coreTryRngNever.try_next_u32 s =
      .ok (.Ok (pcgOutput s.state),
        { state := pcgState s.state s.increment, increment := s.increment }) := by
  simp only [rand_pcg.pcg64.Lcg64Xsh32.Insts.Rand_coreTryRngNever.try_next_u32,
    rand_pcg.pcg64.Lcg64Xsh32.step, rand_pcg.pcg64.MULTIPLIER,
    rand_pcg.pcg64.TryRngLcg64Xsh32Never.try_next_u32.ROTATE,
    rand_pcg.pcg64.TryRngLcg64Xsh32Never.try_next_u32.XSHIFT,
    rand_pcg.pcg64.TryRngLcg64Xsh32Never.try_next_u32.SPARE,
    HShiftRight.hShiftRight, UScalar.shiftRight_UScalar, UScalar.shiftRight,
    pcgOutput, pcgState, U64.wrapping_mul, U64.wrapping_add,
    core.num.U64.wrapping_mul, core.num.U64.wrapping_add, core.num.U32.rotate_right]
  dsimp [lift, Bind.bind, HXor.hXor, UScalar.xor,
    UScalar.wrapping_mul, UScalar.wrapping_add]
  simp only [bind_ok]
  rfl

def incrementLcg64Xsh32 (stream : U64) : U64 := ⟨(stream.bv <<< 1) ||| (1 : BitVec 64)⟩

theorem state_Lcg64Xsh32_spec (x : pcg64.Lcg64Xsh32) :
  pcg64.Lcg64Xsh32.impl.state x = .ok x.state := by rfl

theorem stream_Lcg64Xsh32_spec (x : pcg64.Lcg64Xsh32) :
  pcg64.Lcg64Xsh32.stream x = .ok (⟨x.increment.bv >>> 1⟩ : U64) := by
  simp only [pcg64.Lcg64Xsh32.stream, HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight]
  simp [IScalar.toNat]

theorem from_state_Lcg64Xsh32_spec (state stream : U64) :
  pcg64.Lcg64Xsh32.from_state state stream = .ok {state := state, increment := incrementLcg64Xsh32 stream} := by
  simp only [pcg64.Lcg64Xsh32.from_state, HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft]
  simp [IScalar.toNat, lift, incrementLcg64Xsh32]
  rfl

def initializedLcg64Xsh32 (state stream : U64) : pcg64.Lcg64Xsh32 :=
  let inc := incrementLcg64Xsh32 stream
  { state := ((state.wrapping_add inc).wrapping_mul 6364136223846793005#u64).wrapping_add inc, increment := inc }

theorem new_Lcg64Xsh32_spec (state stream : U64) :
  pcg64.Lcg64Xsh32.new state stream = .ok (initializedLcg64Xsh32 state stream) := by
  simp only [pcg64.Lcg64Xsh32.new, pcg64.Lcg64Xsh32.from_state_incr, pcg64.Lcg64Xsh32.step,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft,
    pcg64.MULTIPLIER]
  simp [IScalar.toNat, lift, incrementLcg64Xsh32, initializedLcg64Xsh32,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    U64.wrapping_add, U64.wrapping_mul]
  dsimp [HOr.hOr, UScalar.or]
  constructor <;> rfl

def incrementLcg128Xsl64 (stream : U128) : U128 := ⟨(stream.bv <<< 1) ||| (1 : BitVec 128)⟩

theorem state_Lcg128Xsl64_spec (x : pcg128.Lcg128Xsl64) :
  pcg128.Lcg128Xsl64.impl.state x = .ok x.state := by rfl

theorem stream_Lcg128Xsl64_spec (x : pcg128.Lcg128Xsl64) :
  pcg128.Lcg128Xsl64.stream x = .ok (⟨x.increment.bv >>> 1⟩ : U128) := by
  simp only [pcg128.Lcg128Xsl64.stream, HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight]
  simp [IScalar.toNat]

theorem from_state_Lcg128Xsl64_spec (state stream : U128) :
  pcg128.Lcg128Xsl64.from_state state stream = .ok {state := state, increment := incrementLcg128Xsl64 stream} := by
  simp only [pcg128.Lcg128Xsl64.from_state, HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft]
  simp [IScalar.toNat, lift, incrementLcg128Xsl64]
  rfl

def initializedLcg128Xsl64 (state stream : U128) : pcg128.Lcg128Xsl64 :=
  let inc := incrementLcg128Xsl64 stream
  { state := ((state.wrapping_add inc).wrapping_mul 47026247687942121848144207491837523525#u128).wrapping_add inc, increment := inc }

theorem new_Lcg128Xsl64_spec (state stream : U128) :
  pcg128.Lcg128Xsl64.new state stream = .ok (initializedLcg128Xsl64 state stream) := by
  simp only [pcg128.Lcg128Xsl64.new, pcg128.Lcg128Xsl64.from_state_incr, pcg128.Lcg128Xsl64.step,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft,
    pcg128.MULTIPLIER]
  simp [IScalar.toNat, lift, incrementLcg128Xsl64, initializedLcg128Xsl64,
    core.num.U128.wrapping_add, core.num.U128.wrapping_mul,
    U128.wrapping_add, U128.wrapping_mul]
  dsimp [HOr.hOr, UScalar.or]
  constructor <;> rfl

def incrementLcg128CmDxsm64 (stream : U128) : U128 := ⟨(stream.bv <<< 1) ||| (1 : BitVec 128)⟩

theorem state_Lcg128CmDxsm64_spec (x : pcg128cm.Lcg128CmDxsm64) :
  pcg128cm.Lcg128CmDxsm64.impl.state x = .ok x.state := by rfl

theorem stream_Lcg128CmDxsm64_spec (x : pcg128cm.Lcg128CmDxsm64) :
  pcg128cm.Lcg128CmDxsm64.stream x = .ok (⟨x.increment.bv >>> 1⟩ : U128) := by
  simp only [pcg128cm.Lcg128CmDxsm64.stream, HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight]
  simp [IScalar.toNat]

theorem from_state_Lcg128CmDxsm64_spec (state stream : U128) :
  pcg128cm.Lcg128CmDxsm64.from_state state stream = .ok {state := state, increment := incrementLcg128CmDxsm64 stream} := by
  simp only [pcg128cm.Lcg128CmDxsm64.from_state, HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft]
  simp [IScalar.toNat, lift, incrementLcg128CmDxsm64]
  rfl

def initializedLcg128CmDxsm64 (state stream : U128) : pcg128cm.Lcg128CmDxsm64 :=
  let inc := incrementLcg128CmDxsm64 stream
  { state := ((state.wrapping_add inc).wrapping_mul 15750249268501108917#u128).wrapping_add inc, increment := inc }

theorem new_Lcg128CmDxsm64_spec (state stream : U128) :
  pcg128cm.Lcg128CmDxsm64.new state stream = .ok (initializedLcg128CmDxsm64 state stream) := by
  simp only [pcg128cm.Lcg128CmDxsm64.new, pcg128cm.Lcg128CmDxsm64.from_state_incr, pcg128cm.Lcg128CmDxsm64.step,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft,
    pcg128cm.MULTIPLIER]
  simp [IScalar.toNat, lift, incrementLcg128CmDxsm64, initializedLcg128CmDxsm64,
    core.num.U128.wrapping_add, core.num.U128.wrapping_mul,
    U128.wrapping_add, U128.wrapping_mul]
  dsimp [HOr.hOr, UScalar.or]
  constructor <;> rfl

theorem state_Mcg128Xsl64_spec (x : pcg128.Mcg128Xsl64) :
  pcg128.Mcg128Xsl64.impl.state x = .ok x.state := by rfl

theorem new_Mcg128Xsl64_spec (state : U128) :
  pcg128.Mcg128Xsl64.new state = .ok {state := ⟨state.bv ||| (1 : BitVec 128)⟩} := by
  simp [pcg128.Mcg128Xsl64.new, lift]
  rfl
end Proofs
#print axioms Proofs.try_next_u32_spec
#print axioms Proofs.state_Lcg64Xsh32_spec
#print axioms Proofs.stream_Lcg64Xsh32_spec
#print axioms Proofs.from_state_Lcg64Xsh32_spec
#print axioms Proofs.new_Lcg64Xsh32_spec
#print axioms Proofs.state_Lcg128Xsl64_spec
#print axioms Proofs.stream_Lcg128Xsl64_spec
#print axioms Proofs.from_state_Lcg128Xsl64_spec
#print axioms Proofs.new_Lcg128Xsl64_spec
#print axioms Proofs.state_Lcg128CmDxsm64_spec
#print axioms Proofs.stream_Lcg128CmDxsm64_spec
#print axioms Proofs.from_state_Lcg128CmDxsm64_spec
#print axioms Proofs.new_Lcg128CmDxsm64_spec
#print axioms Proofs.state_Mcg128Xsl64_spec
#print axioms Proofs.new_Mcg128Xsl64_spec
