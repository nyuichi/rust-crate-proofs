import Generated.NohashHasher

open Aeneas Aeneas.Std Result Error
open nohash_hasher

set_option linter.unusedSimpArgs false

namespace DebugProofs

/-- Construction initializes the stored value to zero and permits a first write. -/
theorem default_spec (T : Type) :
    NoHashHasher.Insts.CoreDefaultDefault.default T = .ok (0#u64, false, ()) := by rfl

/-- The handwritten clone preserves the stored value and previous-write flag. -/
theorem clone_spec {T : Type} (s : NoHashHasher T) :
    NoHashHasher.Insts.CoreCloneClone.clone s = .ok s := by rfl

/-- finish is a read of the stored integer for every state, including no writes. -/
theorem finish_spec {T : Type} (inst : IsEnabled T) (s : NoHashHasher T) :
    NoHashHasher.Insts.CoreHashHasher.finish inst s = .ok s.1 := by rfl

/-- Generic byte writes always panic, regardless of the bytes and prior state. -/
theorem write_spec {T : Type} (inst : IsEnabled T) (s : NoHashHasher T) (bytes : Slice U8) :
    NoHashHasher.Insts.CoreHashHasher.write inst s bytes = .fail .panic := by rfl

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_u8_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U8) :
    NoHashHasher.Insts.CoreHashHasher.write_u8 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_u8, UScalar.cast, IScalar.hcast, core.convert.num.FromU64U8.from]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_u16_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U16) :
    NoHashHasher.Insts.CoreHashHasher.write_u16 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_u16, UScalar.cast, IScalar.hcast, core.convert.num.FromU64U16.from]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_u32_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U32) :
    NoHashHasher.Insts.CoreHashHasher.write_u32 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_u32, UScalar.cast, IScalar.hcast, core.convert.num.FromU64U32.from]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_u64_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U64) :
    NoHashHasher.Insts.CoreHashHasher.write_u64 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_u64, UScalar.cast, IScalar.hcast]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_usize_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : Usize) :
    NoHashHasher.Insts.CoreHashHasher.write_usize inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_usize, UScalar.cast, IScalar.hcast]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_i8_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I8) :
    NoHashHasher.Insts.CoreHashHasher.write_i8 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_i8, UScalar.cast, IScalar.hcast]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_i16_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I16) :
    NoHashHasher.Insts.CoreHashHasher.write_i16 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_i16, UScalar.cast, IScalar.hcast]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_i32_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I32) :
    NoHashHasher.Insts.CoreHashHasher.write_i32 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_i32, UScalar.cast, IScalar.hcast]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_i64_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I64) :
    NoHashHasher.Insts.CoreHashHasher.write_i64 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_i64, UScalar.cast, IScalar.hcast]

/-- Exact debug transition: first write stores the Rust cast; a later write panics. -/
theorem write_isize_spec {T : Type} (inst : IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : Isize) :
    NoHashHasher.Insts.CoreHashHasher.write_isize inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ()) := by
  cases alreadyWritten <;>
    simp (config := { zeta := true }) [bind_tc_ok, bind_tc_fail, massert, lift, NoHashHasher.Insts.CoreHashHasher.write_isize, UScalar.cast, IScalar.hcast]


end DebugProofs
