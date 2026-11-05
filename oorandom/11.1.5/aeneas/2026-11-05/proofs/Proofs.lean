import Oorandom
import EstablishedRange
open Aeneas Aeneas.Std Result
open oorandom
namespace Proofs
set_option linter.unusedSimpArgs false
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

theorem rand_i32_spec (s : Rand32) :
    Rand32.rand_i32 s =
      .ok (UScalar.hcast .I32 (output32 s.state),
        { state := advance32 s.state s.inc, inc := s.inc }) := by
  simp [Rand32.rand_i32, rand_u32_spec, lift]

theorem state32_spec (s : Rand32) :
    Rand32.impl.state s = .ok (s.state,s.inc) := by rfl

theorem from_state32_spec (s i : U64) :
    Rand32.from_state (s,i) = .ok { state := s, inc := i } := by rfl

theorem restore32_spec (s : Rand32) :
    (do let saved ← Rand32.impl.state s; Rand32.from_state saved) = .ok s := by
  cases s
  simp [Rand32.impl.state, Rand32.from_state]

def initialized32 (seed increment : U64) : Rand32 :=
  let inc : U64 := ⟨(increment.bv <<< 1) ||| (1 : BitVec 64)⟩
  { state := advance32 ((advance32 0#u64 inc).wrapping_add seed) inc, inc := inc }

theorem new_inc32_spec (seed increment : U64) :
    Rand32.new_inc seed increment = .ok (initialized32 seed increment) := by
  simp [Rand32.new_inc, rand_u32_spec, lift, initialized32,
    core.num.U64.wrapping_shl, UScalar.wrapping_shl,
    core.num.U64.wrapping_add, U64.wrapping_add]
  dsimp [HOr.hOr, UScalar.or]
  constructor <;> rfl

theorem new32_spec (seed : U64) :
    Rand32.new seed = .ok (initialized32 seed Rand32.DEFAULT_INC) := by
  simp [Rand32.new, new_inc32_spec]
def output64 (s : U128) : U64 :=
  let x : U128 := ⟨((s.bv >>> 29) ^^^ s.bv) >>> 58⟩
  let r : U128 := ⟨s.bv >>> 122⟩
  (UScalar.cast .U64 x).rotate_right (UScalar.cast .U32 r)
def advance64 (s i : U128) : U128 :=
  (s.wrapping_mul 47026247687942121848144207491837523525#u128).wrapping_add i

theorem rand_u64_spec (s : Rand64) :
    Rand64.rand_u64 s =
      .ok (output64 s.state, { state := advance64 s.state s.inc, inc := s.inc }) := by
  simp only [Rand64.rand_u64, Rand64.MULTIPLIER,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    output64, advance64, U128.wrapping_mul, U128.wrapping_add,
    core.num.U128.wrapping_mul, core.num.U128.wrapping_add, core.num.U64.rotate_right]
  simp only [IScalar.toNat]
  dsimp [lift, Bind.bind, HXor.hXor, UScalar.xor,
    UScalar.wrapping_mul, UScalar.wrapping_add]
  simp only [bind_ok]
  rfl

theorem rand_i64_spec (s : Rand64) :
    Rand64.rand_i64 s =
      .ok (UScalar.hcast .I64 (output64 s.state),
        { state := advance64 s.state s.inc, inc := s.inc }) := by
  simp [Rand64.rand_i64, rand_u64_spec, lift]

theorem state64_spec (s : Rand64) :
    Rand64.impl.state s = .ok (s.state,s.inc) := by rfl

theorem from_state64_spec (s i : U128) :
    Rand64.from_state (s,i) = .ok { state := s, inc := i } := by rfl

theorem restore64_spec (s : Rand64) :
    (do let saved ← Rand64.impl.state s; Rand64.from_state saved) = .ok s := by
  cases s
  simp [Rand64.impl.state, Rand64.from_state]

def initialized64 (seed increment : U128) : Rand64 :=
  let inc : U128 := ⟨(increment.bv <<< 1) ||| (1 : BitVec 128)⟩
  { state := advance64 ((advance64 0#u128 inc).wrapping_add seed) inc, inc := inc }

theorem new_inc64_spec (seed increment : U128) :
    Rand64.new_inc seed increment = .ok (initialized64 seed increment) := by
  simp [Rand64.new_inc, rand_u64_spec, lift, initialized64,
    core.num.U128.wrapping_shl, UScalar.wrapping_shl,
    core.num.U128.wrapping_add, U128.wrapping_add]
  dsimp [HOr.hOr, UScalar.or]
  constructor <;> rfl

theorem new64_spec (seed : U128) :
    Rand64.new seed = .ok (initialized64 seed Rand64.DEFAULT_INC) := by
  simp [Rand64.new, new_inc64_spec]
end Proofs
#print axioms Proofs.rand_u32_spec
#print axioms Proofs.rand_i32_spec
#print axioms Proofs.state32_spec
#print axioms Proofs.from_state32_spec
#print axioms Proofs.restore32_spec
#print axioms Proofs.new_inc32_spec
#print axioms Proofs.new32_spec
#print axioms Proofs.rand_u64_spec
#print axioms Proofs.rand_i64_spec
#print axioms Proofs.state64_spec
#print axioms Proofs.from_state64_spec
#print axioms Proofs.restore64_spec
#print axioms Proofs.new_inc64_spec
#print axioms Proofs.new64_spec
