import BitField.Funs
open Aeneas Aeneas.Std Result Std.WP
open bit_field
namespace Proofs
set_option linter.unusedSimpArgs false
set_option maxRecDepth 2048
theorem u8_get_bit_spec (x : U8) (bit : Usize) (h : bit.val < 8) :
  U8.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold U8.Insts.Bit_fieldBitField.get_bit
  simp only [U8.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU8, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 8)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 8) <<< bit.val) = BitVec.twoPow 8 bit.val := (BitVec.twoPow_eq 8 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 8 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 8) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 8).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def u8_setWord (x : BitVec 8) (bit : Nat) (value : Bool) : BitVec 8 :=
  if value then x ||| ((1 : BitVec 8) <<< bit)
  else x &&& ~~~((1 : BitVec 8) <<< bit)

theorem u8_set_bit_spec (x : U8) (bit : Usize) (value : Bool) (h : bit.val < 8) :
  U8.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = u8_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold U8.Insts.Bit_fieldBitField.set_bit
  simp only [U8.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU8,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, u8_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 8) <<< bit.val)
    change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, u8_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 8) <<< bit.val)
    change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem u16_get_bit_spec (x : U16) (bit : Usize) (h : bit.val < 16) :
  U16.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold U16.Insts.Bit_fieldBitField.get_bit
  simp only [U16.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU16, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 16)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 16) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 16) <<< bit.val) = BitVec.twoPow 16 bit.val := (BitVec.twoPow_eq 16 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 16 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 16) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 16).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def u16_setWord (x : BitVec 16) (bit : Nat) (value : Bool) : BitVec 16 :=
  if value then x ||| ((1 : BitVec 16) <<< bit)
  else x &&& ~~~((1 : BitVec 16) <<< bit)

theorem u16_set_bit_spec (x : U16) (bit : Usize) (value : Bool) (h : bit.val < 16) :
  U16.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = u16_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold U16.Insts.Bit_fieldBitField.set_bit
  simp only [U16.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU16,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, u16_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 16) <<< bit.val)
    change i1.bv = ((1 : BitVec 16) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, u16_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 16) <<< bit.val)
    change i1.bv = ((1 : BitVec 16) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem u32_get_bit_spec (x : U32) (bit : Usize) (h : bit.val < 32) :
  U32.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold U32.Insts.Bit_fieldBitField.get_bit
  simp only [U32.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU32, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 32)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 32) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 32) <<< bit.val) = BitVec.twoPow 32 bit.val := (BitVec.twoPow_eq 32 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 32 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 32) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 32).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def u32_setWord (x : BitVec 32) (bit : Nat) (value : Bool) : BitVec 32 :=
  if value then x ||| ((1 : BitVec 32) <<< bit)
  else x &&& ~~~((1 : BitVec 32) <<< bit)

theorem u32_set_bit_spec (x : U32) (bit : Usize) (value : Bool) (h : bit.val < 32) :
  U32.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = u32_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold U32.Insts.Bit_fieldBitField.set_bit
  simp only [U32.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU32,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, u32_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 32) <<< bit.val)
    change i1.bv = ((1 : BitVec 32) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, u32_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 32) <<< bit.val)
    change i1.bv = ((1 : BitVec 32) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem u64_get_bit_spec (x : U64) (bit : Usize) (h : bit.val < 64) :
  U64.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold U64.Insts.Bit_fieldBitField.get_bit
  simp only [U64.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU64, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 64)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 64) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 64) <<< bit.val) = BitVec.twoPow 64 bit.val := (BitVec.twoPow_eq 64 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 64 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 64) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 64).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def u64_setWord (x : BitVec 64) (bit : Nat) (value : Bool) : BitVec 64 :=
  if value then x ||| ((1 : BitVec 64) <<< bit)
  else x &&& ~~~((1 : BitVec 64) <<< bit)

theorem u64_set_bit_spec (x : U64) (bit : Usize) (value : Bool) (h : bit.val < 64) :
  U64.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = u64_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold U64.Insts.Bit_fieldBitField.set_bit
  simp only [U64.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU64,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, u64_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 64) <<< bit.val)
    change i1.bv = ((1 : BitVec 64) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, u64_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 64) <<< bit.val)
    change i1.bv = ((1 : BitVec 64) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem u128_get_bit_spec (x : U128) (bit : Usize) (h : bit.val < 128) :
  U128.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold U128.Insts.Bit_fieldBitField.get_bit
  simp only [U128.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU128, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 128)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 128) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 128) <<< bit.val) = BitVec.twoPow 128 bit.val := (BitVec.twoPow_eq 128 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 128 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 128) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 128).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def u128_setWord (x : BitVec 128) (bit : Nat) (value : Bool) : BitVec 128 :=
  if value then x ||| ((1 : BitVec 128) <<< bit)
  else x &&& ~~~((1 : BitVec 128) <<< bit)

theorem u128_set_bit_spec (x : U128) (bit : Usize) (value : Bool) (h : bit.val < 128) :
  U128.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = u128_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold U128.Insts.Bit_fieldBitField.set_bit
  simp only [U128.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofU128,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, u128_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 128) <<< bit.val)
    change i1.bv = ((1 : BitVec 128) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, u128_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 128) <<< bit.val)
    change i1.bv = ((1 : BitVec 128) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem usize_get_bit_spec (x : Usize) (bit : Usize) (platform : System.Platform.numBits = 64) (h : bit.val < 64) :
  Usize.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  have hwidth : bit.val < System.Platform.numBits := by omega
  unfold Usize.Insts.Bit_fieldBitField.get_bit
  simp only [Usize.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofUsize, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec System.Platform.numBits)))) = x.bv.getLsbD bit.val
  have hOne : Usize.bv 1#usize = (1 : BitVec System.Platform.numBits) := by
    simp only [Usize.bv, UScalar.ofNatCore_bv]
    rfl
  rw [hOne] at i1_post1
  simp only [Usize.bv] at i1_post1 ⊢
  simp only [i1_post1]
  have hm : ((1 : BitVec System.Platform.numBits) <<< bit.val) = BitVec.twoPow System.Platform.numBits bit.val := (BitVec.twoPow_eq System.Platform.numBits bit.val).symm
  simp only [hm]
  have hand := BitVec.and_twoPow x.bv bit.val
  simp only [Usize.bv] at hand
  simp only [hand]
  have hn : BitVec.twoPow System.Platform.numBits bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec System.Platform.numBits) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, hwidth, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec System.Platform.numBits).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : UScalar.bv x |>.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def usize_setWord (x : BitVec System.Platform.numBits) (bit : Nat) (value : Bool) : BitVec System.Platform.numBits :=
  if value then x ||| ((1 : BitVec System.Platform.numBits) <<< bit)
  else x &&& ~~~((1 : BitVec System.Platform.numBits) <<< bit)

theorem usize_set_bit_spec (x : Usize) (bit : Usize) (value : Bool) (platform : System.Platform.numBits = 64) (h : bit.val < 64) :
  Usize.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = usize_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  have hwidth : bit.val < System.Platform.numBits := by omega
  unfold Usize.Insts.Bit_fieldBitField.set_bit
  simp only [Usize.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofUsize,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, usize_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec System.Platform.numBits) <<< bit.val)
    have hOne : Usize.bv 1#usize = (1 : BitVec System.Platform.numBits) := by
      simp only [Usize.bv, UScalar.ofNatCore_bv]
      rfl
    rw [hOne] at i1_post1
    simp only [Usize.bv] at i1_post1 ⊢
    simp only [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, usize_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec System.Platform.numBits) <<< bit.val)
    have hOne : Usize.bv 1#usize = (1 : BitVec System.Platform.numBits) := by
      simp only [Usize.bv, UScalar.ofNatCore_bv]
      rfl
    rw [hOne] at i1_post1
    simp only [Usize.bv] at i1_post1 ⊢
    simp only [i1_post1]

theorem i8_get_bit_spec (x : I8) (bit : Usize) (h : bit.val < 8) :
  I8.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold I8.Insts.Bit_fieldBitField.get_bit
  simp only [I8.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI8, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, IScalar.bv_and, IScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 8)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 8) <<< bit.val) = BitVec.twoPow 8 bit.val := (BitVec.twoPow_eq 8 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 8 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 8) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 8).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def i8_setWord (x : BitVec 8) (bit : Nat) (value : Bool) : BitVec 8 :=
  if value then x ||| ((1 : BitVec 8) <<< bit)
  else x &&& ~~~((1 : BitVec 8) <<< bit)

theorem i8_set_bit_spec (x : I8) (bit : Usize) (value : Bool) (h : bit.val < 8) :
  I8.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = i8_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold I8.Insts.Bit_fieldBitField.set_bit
  simp only [I8.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI8,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, i8_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 8) <<< bit.val)
    change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, i8_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 8) <<< bit.val)
    change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem i16_get_bit_spec (x : I16) (bit : Usize) (h : bit.val < 16) :
  I16.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold I16.Insts.Bit_fieldBitField.get_bit
  simp only [I16.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI16, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, IScalar.bv_and, IScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 16)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 16) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 16) <<< bit.val) = BitVec.twoPow 16 bit.val := (BitVec.twoPow_eq 16 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 16 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 16) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 16).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def i16_setWord (x : BitVec 16) (bit : Nat) (value : Bool) : BitVec 16 :=
  if value then x ||| ((1 : BitVec 16) <<< bit)
  else x &&& ~~~((1 : BitVec 16) <<< bit)

theorem i16_set_bit_spec (x : I16) (bit : Usize) (value : Bool) (h : bit.val < 16) :
  I16.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = i16_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold I16.Insts.Bit_fieldBitField.set_bit
  simp only [I16.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI16,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, i16_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 16) <<< bit.val)
    change i1.bv = ((1 : BitVec 16) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, i16_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 16) <<< bit.val)
    change i1.bv = ((1 : BitVec 16) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem i32_get_bit_spec (x : I32) (bit : Usize) (h : bit.val < 32) :
  I32.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold I32.Insts.Bit_fieldBitField.get_bit
  simp only [I32.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI32, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, IScalar.bv_and, IScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 32)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 32) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 32) <<< bit.val) = BitVec.twoPow 32 bit.val := (BitVec.twoPow_eq 32 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 32 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 32) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 32).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def i32_setWord (x : BitVec 32) (bit : Nat) (value : Bool) : BitVec 32 :=
  if value then x ||| ((1 : BitVec 32) <<< bit)
  else x &&& ~~~((1 : BitVec 32) <<< bit)

theorem i32_set_bit_spec (x : I32) (bit : Usize) (value : Bool) (h : bit.val < 32) :
  I32.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = i32_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold I32.Insts.Bit_fieldBitField.set_bit
  simp only [I32.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI32,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, i32_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 32) <<< bit.val)
    change i1.bv = ((1 : BitVec 32) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, i32_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 32) <<< bit.val)
    change i1.bv = ((1 : BitVec 32) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem i64_get_bit_spec (x : I64) (bit : Usize) (h : bit.val < 64) :
  I64.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold I64.Insts.Bit_fieldBitField.get_bit
  simp only [I64.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI64, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, IScalar.bv_and, IScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 64)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 64) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 64) <<< bit.val) = BitVec.twoPow 64 bit.val := (BitVec.twoPow_eq 64 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 64 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 64) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 64).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def i64_setWord (x : BitVec 64) (bit : Nat) (value : Bool) : BitVec 64 :=
  if value then x ||| ((1 : BitVec 64) <<< bit)
  else x &&& ~~~((1 : BitVec 64) <<< bit)

theorem i64_set_bit_spec (x : I64) (bit : Usize) (value : Bool) (h : bit.val < 64) :
  I64.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = i64_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold I64.Insts.Bit_fieldBitField.set_bit
  simp only [I64.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI64,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, i64_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 64) <<< bit.val)
    change i1.bv = ((1 : BitVec 64) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, i64_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 64) <<< bit.val)
    change i1.bv = ((1 : BitVec 64) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem i128_get_bit_spec (x : I128) (bit : Usize) (h : bit.val < 128) :
  I128.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold I128.Insts.Bit_fieldBitField.get_bit
  simp only [I128.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI128, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, IScalar.bv_and, IScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 128)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 128) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 128) <<< bit.val) = BitVec.twoPow 128 bit.val := (BitVec.twoPow_eq 128 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 128 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 128) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 128).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def i128_setWord (x : BitVec 128) (bit : Nat) (value : Bool) : BitVec 128 :=
  if value then x ||| ((1 : BitVec 128) <<< bit)
  else x &&& ~~~((1 : BitVec 128) <<< bit)

theorem i128_set_bit_spec (x : I128) (bit : Usize) (value : Bool) (h : bit.val < 128) :
  I128.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = i128_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold I128.Insts.Bit_fieldBitField.set_bit
  simp only [I128.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofI128,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, i128_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 128) <<< bit.val)
    change i1.bv = ((1 : BitVec 128) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, i128_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 128) <<< bit.val)
    change i1.bv = ((1 : BitVec 128) <<< bit.val) at i1_post1
    rw [i1_post1]

theorem isize_get_bit_spec (x : Isize) (bit : Usize) (platform : System.Platform.numBits = 64) (h : bit.val < 64) :
  Isize.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  have hwidth : bit.val < System.Platform.numBits := by omega
  unfold Isize.Insts.Bit_fieldBitField.get_bit
  simp only [Isize.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofIsize, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, IScalar.bv_and, IScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec System.Platform.numBits)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec System.Platform.numBits) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec System.Platform.numBits) <<< bit.val) = BitVec.twoPow System.Platform.numBits bit.val := (BitVec.twoPow_eq System.Platform.numBits bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow System.Platform.numBits bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec System.Platform.numBits) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, hwidth, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec System.Platform.numBits).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def isize_setWord (x : BitVec System.Platform.numBits) (bit : Nat) (value : Bool) : BitVec System.Platform.numBits :=
  if value then x ||| ((1 : BitVec System.Platform.numBits) <<< bit)
  else x &&& ~~~((1 : BitVec System.Platform.numBits) <<< bit)

theorem isize_set_bit_spec (x : Isize) (bit : Usize) (value : Bool) (platform : System.Platform.numBits = 64) (h : bit.val < 64) :
  Isize.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = isize_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  have hwidth : bit.val < System.Platform.numBits := by omega
  unfold Isize.Insts.Bit_fieldBitField.set_bit
  simp only [Isize.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_ofIsize,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, isize_setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec System.Platform.numBits) <<< bit.val)
    change i1.bv = ((1 : BitVec System.Platform.numBits) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, isize_setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec System.Platform.numBits) <<< bit.val)
    change i1.bv = ((1 : BitVec System.Platform.numBits) <<< bit.val) at i1_post1
    rw [i1_post1]

end Proofs
#print axioms Proofs.u8_get_bit_spec
#print axioms Proofs.u8_set_bit_spec
#print axioms Proofs.u16_get_bit_spec
#print axioms Proofs.u16_set_bit_spec
#print axioms Proofs.u32_get_bit_spec
#print axioms Proofs.u32_set_bit_spec
#print axioms Proofs.u64_get_bit_spec
#print axioms Proofs.u64_set_bit_spec
#print axioms Proofs.u128_get_bit_spec
#print axioms Proofs.u128_set_bit_spec
#print axioms Proofs.usize_get_bit_spec
#print axioms Proofs.usize_set_bit_spec
#print axioms Proofs.i8_get_bit_spec
#print axioms Proofs.i8_set_bit_spec
#print axioms Proofs.i16_get_bit_spec
#print axioms Proofs.i16_set_bit_spec
#print axioms Proofs.i32_get_bit_spec
#print axioms Proofs.i32_set_bit_spec
#print axioms Proofs.i64_get_bit_spec
#print axioms Proofs.i64_set_bit_spec
#print axioms Proofs.i128_get_bit_spec
#print axioms Proofs.i128_set_bit_spec
#print axioms Proofs.isize_get_bit_spec
#print axioms Proofs.isize_set_bit_spec

namespace Proofs
theorem u8_length_spec : U8.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 8#usize := by
  unfold U8.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofU8, Std.bind_ok]
  change UScalar.mul (1#usize) (8#usize) = .ok 8#usize
  have hm : (1#usize).val * (8#usize).val = 8 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem u8_get_bit_invalid (x : U8) (bit : Usize) (h : 8 ≤ bit.val) :
  U8.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 8 := by omega
  simp [U8.Insts.Bit_fieldBitField.get_bit, u8_length_spec, hn, massert]
theorem u8_get_bit_contract (x : U8) (bit : Usize) :
  (bit.val < 8 → U8.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (8 ≤ bit.val → U8.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => u8_get_bit_spec x bit h, fun h => u8_get_bit_invalid x bit h⟩
theorem u8_set_bit_invalid (x : U8) (bit : Usize) (value : Bool) (h : 8 ≤ bit.val) :
  U8.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 8 := by omega
  simp [U8.Insts.Bit_fieldBitField.set_bit, u8_length_spec, hn, massert]
theorem u8_set_bit_contract (x : U8) (bit : Usize) (value : Bool) :
  (bit.val < 8 → U8.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = u8_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (8 ≤ bit.val → U8.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => u8_set_bit_spec x bit value h, fun h => u8_set_bit_invalid x bit value h⟩
theorem u16_length_spec : U16.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 16#usize := by
  unfold U16.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofU16, Std.bind_ok]
  change UScalar.mul (2#usize) (8#usize) = .ok 16#usize
  have hm : (2#usize).val * (8#usize).val = 16 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem u16_get_bit_invalid (x : U16) (bit : Usize) (h : 16 ≤ bit.val) :
  U16.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 16 := by omega
  simp [U16.Insts.Bit_fieldBitField.get_bit, u16_length_spec, hn, massert]
theorem u16_get_bit_contract (x : U16) (bit : Usize) :
  (bit.val < 16 → U16.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (16 ≤ bit.val → U16.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => u16_get_bit_spec x bit h, fun h => u16_get_bit_invalid x bit h⟩
theorem u16_set_bit_invalid (x : U16) (bit : Usize) (value : Bool) (h : 16 ≤ bit.val) :
  U16.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 16 := by omega
  simp [U16.Insts.Bit_fieldBitField.set_bit, u16_length_spec, hn, massert]
theorem u16_set_bit_contract (x : U16) (bit : Usize) (value : Bool) :
  (bit.val < 16 → U16.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = u16_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (16 ≤ bit.val → U16.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => u16_set_bit_spec x bit value h, fun h => u16_set_bit_invalid x bit value h⟩
theorem u32_length_spec : U32.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 32#usize := by
  unfold U32.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofU32, Std.bind_ok]
  change UScalar.mul (4#usize) (8#usize) = .ok 32#usize
  have hm : (4#usize).val * (8#usize).val = 32 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem u32_get_bit_invalid (x : U32) (bit : Usize) (h : 32 ≤ bit.val) :
  U32.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 32 := by omega
  simp [U32.Insts.Bit_fieldBitField.get_bit, u32_length_spec, hn, massert]
theorem u32_get_bit_contract (x : U32) (bit : Usize) :
  (bit.val < 32 → U32.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (32 ≤ bit.val → U32.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => u32_get_bit_spec x bit h, fun h => u32_get_bit_invalid x bit h⟩
theorem u32_set_bit_invalid (x : U32) (bit : Usize) (value : Bool) (h : 32 ≤ bit.val) :
  U32.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 32 := by omega
  simp [U32.Insts.Bit_fieldBitField.set_bit, u32_length_spec, hn, massert]
theorem u32_set_bit_contract (x : U32) (bit : Usize) (value : Bool) :
  (bit.val < 32 → U32.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = u32_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (32 ≤ bit.val → U32.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => u32_set_bit_spec x bit value h, fun h => u32_set_bit_invalid x bit value h⟩
theorem u64_length_spec : U64.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 64#usize := by
  unfold U64.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofU64, Std.bind_ok]
  change UScalar.mul (8#usize) (8#usize) = .ok 64#usize
  have hm : (8#usize).val * (8#usize).val = 64 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem u64_get_bit_invalid (x : U64) (bit : Usize) (h : 64 ≤ bit.val) :
  U64.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [U64.Insts.Bit_fieldBitField.get_bit, u64_length_spec, hn, massert]
theorem u64_get_bit_contract (x : U64) (bit : Usize) :
  (bit.val < 64 → U64.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (64 ≤ bit.val → U64.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => u64_get_bit_spec x bit h, fun h => u64_get_bit_invalid x bit h⟩
theorem u64_set_bit_invalid (x : U64) (bit : Usize) (value : Bool) (h : 64 ≤ bit.val) :
  U64.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [U64.Insts.Bit_fieldBitField.set_bit, u64_length_spec, hn, massert]
theorem u64_set_bit_contract (x : U64) (bit : Usize) (value : Bool) :
  (bit.val < 64 → U64.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = u64_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (64 ≤ bit.val → U64.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => u64_set_bit_spec x bit value h, fun h => u64_set_bit_invalid x bit value h⟩
theorem u128_length_spec : U128.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 128#usize := by
  unfold U128.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofU128, Std.bind_ok]
  change UScalar.mul (16#usize) (8#usize) = .ok 128#usize
  have hm : (16#usize).val * (8#usize).val = 128 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem u128_get_bit_invalid (x : U128) (bit : Usize) (h : 128 ≤ bit.val) :
  U128.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 128 := by omega
  simp [U128.Insts.Bit_fieldBitField.get_bit, u128_length_spec, hn, massert]
theorem u128_get_bit_contract (x : U128) (bit : Usize) :
  (bit.val < 128 → U128.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (128 ≤ bit.val → U128.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => u128_get_bit_spec x bit h, fun h => u128_get_bit_invalid x bit h⟩
theorem u128_set_bit_invalid (x : U128) (bit : Usize) (value : Bool) (h : 128 ≤ bit.val) :
  U128.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 128 := by omega
  simp [U128.Insts.Bit_fieldBitField.set_bit, u128_length_spec, hn, massert]
theorem u128_set_bit_contract (x : U128) (bit : Usize) (value : Bool) :
  (bit.val < 128 → U128.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = u128_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (128 ≤ bit.val → U128.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => u128_set_bit_spec x bit value h, fun h => u128_set_bit_invalid x bit value h⟩
theorem usize_length_spec : Usize.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 64#usize := by
  unfold Usize.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofUsize, Std.bind_ok]
  change UScalar.mul (8#usize) (8#usize) = .ok 64#usize
  have hm : (8#usize).val * (8#usize).val = 64 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem usize_get_bit_invalid (x : Usize) (bit : Usize) (h : 64 ≤ bit.val) :
  Usize.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [Usize.Insts.Bit_fieldBitField.get_bit, usize_length_spec, hn, massert]
theorem usize_get_bit_contract (x : Usize) (bit : Usize) (platform : System.Platform.numBits = 64) :
  (bit.val < 64 → Usize.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (64 ≤ bit.val → Usize.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => usize_get_bit_spec x bit platform h, fun h => usize_get_bit_invalid x bit h⟩
theorem usize_set_bit_invalid (x : Usize) (bit : Usize) (value : Bool) (h : 64 ≤ bit.val) :
  Usize.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [Usize.Insts.Bit_fieldBitField.set_bit, usize_length_spec, hn, massert]
theorem usize_set_bit_contract (x : Usize) (bit : Usize) (value : Bool) (platform : System.Platform.numBits = 64) :
  (bit.val < 64 → Usize.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = usize_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (64 ≤ bit.val → Usize.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => usize_set_bit_spec x bit value platform h, fun h => usize_set_bit_invalid x bit value h⟩
theorem i8_length_spec : I8.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 8#usize := by
  unfold I8.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofI8, Std.bind_ok]
  change UScalar.mul (1#usize) (8#usize) = .ok 8#usize
  have hm : (1#usize).val * (8#usize).val = 8 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem i8_get_bit_invalid (x : I8) (bit : Usize) (h : 8 ≤ bit.val) :
  I8.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 8 := by omega
  simp [I8.Insts.Bit_fieldBitField.get_bit, i8_length_spec, hn, massert]
theorem i8_get_bit_contract (x : I8) (bit : Usize) :
  (bit.val < 8 → I8.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (8 ≤ bit.val → I8.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => i8_get_bit_spec x bit h, fun h => i8_get_bit_invalid x bit h⟩
theorem i8_set_bit_invalid (x : I8) (bit : Usize) (value : Bool) (h : 8 ≤ bit.val) :
  I8.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 8 := by omega
  simp [I8.Insts.Bit_fieldBitField.set_bit, i8_length_spec, hn, massert]
theorem i8_set_bit_contract (x : I8) (bit : Usize) (value : Bool) :
  (bit.val < 8 → I8.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = i8_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (8 ≤ bit.val → I8.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => i8_set_bit_spec x bit value h, fun h => i8_set_bit_invalid x bit value h⟩
theorem i16_length_spec : I16.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 16#usize := by
  unfold I16.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofI16, Std.bind_ok]
  change UScalar.mul (2#usize) (8#usize) = .ok 16#usize
  have hm : (2#usize).val * (8#usize).val = 16 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem i16_get_bit_invalid (x : I16) (bit : Usize) (h : 16 ≤ bit.val) :
  I16.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 16 := by omega
  simp [I16.Insts.Bit_fieldBitField.get_bit, i16_length_spec, hn, massert]
theorem i16_get_bit_contract (x : I16) (bit : Usize) :
  (bit.val < 16 → I16.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (16 ≤ bit.val → I16.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => i16_get_bit_spec x bit h, fun h => i16_get_bit_invalid x bit h⟩
theorem i16_set_bit_invalid (x : I16) (bit : Usize) (value : Bool) (h : 16 ≤ bit.val) :
  I16.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 16 := by omega
  simp [I16.Insts.Bit_fieldBitField.set_bit, i16_length_spec, hn, massert]
theorem i16_set_bit_contract (x : I16) (bit : Usize) (value : Bool) :
  (bit.val < 16 → I16.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = i16_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (16 ≤ bit.val → I16.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => i16_set_bit_spec x bit value h, fun h => i16_set_bit_invalid x bit value h⟩
theorem i32_length_spec : I32.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 32#usize := by
  unfold I32.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofI32, Std.bind_ok]
  change UScalar.mul (4#usize) (8#usize) = .ok 32#usize
  have hm : (4#usize).val * (8#usize).val = 32 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem i32_get_bit_invalid (x : I32) (bit : Usize) (h : 32 ≤ bit.val) :
  I32.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 32 := by omega
  simp [I32.Insts.Bit_fieldBitField.get_bit, i32_length_spec, hn, massert]
theorem i32_get_bit_contract (x : I32) (bit : Usize) :
  (bit.val < 32 → I32.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (32 ≤ bit.val → I32.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => i32_get_bit_spec x bit h, fun h => i32_get_bit_invalid x bit h⟩
theorem i32_set_bit_invalid (x : I32) (bit : Usize) (value : Bool) (h : 32 ≤ bit.val) :
  I32.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 32 := by omega
  simp [I32.Insts.Bit_fieldBitField.set_bit, i32_length_spec, hn, massert]
theorem i32_set_bit_contract (x : I32) (bit : Usize) (value : Bool) :
  (bit.val < 32 → I32.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = i32_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (32 ≤ bit.val → I32.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => i32_set_bit_spec x bit value h, fun h => i32_set_bit_invalid x bit value h⟩
theorem i64_length_spec : I64.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 64#usize := by
  unfold I64.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofI64, Std.bind_ok]
  change UScalar.mul (8#usize) (8#usize) = .ok 64#usize
  have hm : (8#usize).val * (8#usize).val = 64 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem i64_get_bit_invalid (x : I64) (bit : Usize) (h : 64 ≤ bit.val) :
  I64.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [I64.Insts.Bit_fieldBitField.get_bit, i64_length_spec, hn, massert]
theorem i64_get_bit_contract (x : I64) (bit : Usize) :
  (bit.val < 64 → I64.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (64 ≤ bit.val → I64.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => i64_get_bit_spec x bit h, fun h => i64_get_bit_invalid x bit h⟩
theorem i64_set_bit_invalid (x : I64) (bit : Usize) (value : Bool) (h : 64 ≤ bit.val) :
  I64.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [I64.Insts.Bit_fieldBitField.set_bit, i64_length_spec, hn, massert]
theorem i64_set_bit_contract (x : I64) (bit : Usize) (value : Bool) :
  (bit.val < 64 → I64.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = i64_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (64 ≤ bit.val → I64.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => i64_set_bit_spec x bit value h, fun h => i64_set_bit_invalid x bit value h⟩
theorem i128_length_spec : I128.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 128#usize := by
  unfold I128.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofI128, Std.bind_ok]
  change UScalar.mul (16#usize) (8#usize) = .ok 128#usize
  have hm : (16#usize).val * (8#usize).val = 128 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem i128_get_bit_invalid (x : I128) (bit : Usize) (h : 128 ≤ bit.val) :
  I128.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 128 := by omega
  simp [I128.Insts.Bit_fieldBitField.get_bit, i128_length_spec, hn, massert]
theorem i128_get_bit_contract (x : I128) (bit : Usize) :
  (bit.val < 128 → I128.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (128 ≤ bit.val → I128.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => i128_get_bit_spec x bit h, fun h => i128_get_bit_invalid x bit h⟩
theorem i128_set_bit_invalid (x : I128) (bit : Usize) (value : Bool) (h : 128 ≤ bit.val) :
  I128.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 128 := by omega
  simp [I128.Insts.Bit_fieldBitField.set_bit, i128_length_spec, hn, massert]
theorem i128_set_bit_contract (x : I128) (bit : Usize) (value : Bool) :
  (bit.val < 128 → I128.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = i128_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (128 ≤ bit.val → I128.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => i128_set_bit_spec x bit value h, fun h => i128_set_bit_invalid x bit value h⟩
theorem isize_length_spec : Isize.Insts.Bit_fieldBitField.BIT_LENGTH = .ok 64#usize := by
  unfold Isize.Insts.Bit_fieldBitField.BIT_LENGTH
  simp only [core.mem.size_ofIsize, Std.bind_ok]
  change UScalar.mul (8#usize) (8#usize) = .ok 64#usize
  have hm : (8#usize).val * (8#usize).val = 64 := by scalar_tac
  simp only [UScalar.mul, hm, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  split
  · rfl
  · scalar_tac
theorem isize_get_bit_invalid (x : Isize) (bit : Usize) (h : 64 ≤ bit.val) :
  Isize.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [Isize.Insts.Bit_fieldBitField.get_bit, isize_length_spec, hn, massert]
theorem isize_get_bit_contract (x : Isize) (bit : Usize) (platform : System.Platform.numBits = 64) :
  (bit.val < 64 → Isize.Insts.Bit_fieldBitField.get_bit x bit ⦃ b => b = x.bv.getLsbD bit.val ⦄) ∧
  (64 ≤ bit.val → Isize.Insts.Bit_fieldBitField.get_bit x bit = .fail .assertionFailure) := by
  exact ⟨fun h => isize_get_bit_spec x bit platform h, fun h => isize_get_bit_invalid x bit h⟩
theorem isize_set_bit_invalid (x : Isize) (bit : Usize) (value : Bool) (h : 64 ≤ bit.val) :
  Isize.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure := by
  have hn : ¬ bit.val < 64 := by omega
  simp [Isize.Insts.Bit_fieldBitField.set_bit, isize_length_spec, hn, massert]
theorem isize_set_bit_contract (x : Isize) (bit : Usize) (value : Bool) (platform : System.Platform.numBits = 64) :
  (bit.val < 64 → Isize.Insts.Bit_fieldBitField.set_bit x bit value ⦃ r => r.1.bv = isize_setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄) ∧
  (64 ≤ bit.val → Isize.Insts.Bit_fieldBitField.set_bit x bit value = .fail .assertionFailure) := by
  exact ⟨fun h => isize_set_bit_spec x bit value platform h, fun h => isize_set_bit_invalid x bit value h⟩
end Proofs
#print axioms Proofs.u8_get_bit_contract
#print axioms Proofs.u8_set_bit_contract
#print axioms Proofs.u16_get_bit_contract
#print axioms Proofs.u16_set_bit_contract
#print axioms Proofs.u32_get_bit_contract
#print axioms Proofs.u32_set_bit_contract
#print axioms Proofs.u64_get_bit_contract
#print axioms Proofs.u64_set_bit_contract
#print axioms Proofs.u128_get_bit_contract
#print axioms Proofs.u128_set_bit_contract
#print axioms Proofs.usize_get_bit_contract
#print axioms Proofs.usize_set_bit_contract
#print axioms Proofs.i8_get_bit_contract
#print axioms Proofs.i8_set_bit_contract
#print axioms Proofs.i16_get_bit_contract
#print axioms Proofs.i16_set_bit_contract
#print axioms Proofs.i32_get_bit_contract
#print axioms Proofs.i32_set_bit_contract
#print axioms Proofs.i64_get_bit_contract
#print axioms Proofs.i64_set_bit_contract
#print axioms Proofs.i128_get_bit_contract
#print axioms Proofs.i128_set_bit_contract
#print axioms Proofs.isize_get_bit_contract
#print axioms Proofs.isize_set_bit_contract
#print axioms Proofs.u8_length_spec
#print axioms Proofs.u16_length_spec
#print axioms Proofs.u32_length_spec
#print axioms Proofs.u64_length_spec
#print axioms Proofs.u128_length_spec
#print axioms Proofs.usize_length_spec
#print axioms Proofs.i8_length_spec
#print axioms Proofs.i16_length_spec
#print axioms Proofs.i32_length_spec
#print axioms Proofs.i64_length_spec
#print axioms Proofs.i128_length_spec
#print axioms Proofs.isize_length_spec
