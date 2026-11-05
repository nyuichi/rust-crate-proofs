import DebugProofs
import ReleaseProofs

open Aeneas Aeneas.Std Result Error

namespace StdProofs

/-- Exact original-source behavior in debug and release builds. -/
theorem default_spec :
    (∀ (T : Type),
    nohash_hasher.NoHashHasher.Insts.CoreDefaultDefault.default T = .ok (0#u64, false, ())) ∧
    (∀ (T : Type),
    nohash_hasher_release.NoHashHasher.Insts.CoreDefaultDefault.default T = .ok (0#u64, ())) :=
  ⟨@DebugProofs.default_spec, @ReleaseProofs.default_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem clone_spec :
    (∀ {T : Type} (s : nohash_hasher.NoHashHasher T),
    nohash_hasher.NoHashHasher.Insts.CoreCloneClone.clone s = .ok s) ∧
    (∀ {T : Type} (s : nohash_hasher_release.NoHashHasher T),
    nohash_hasher_release.NoHashHasher.Insts.CoreCloneClone.clone s = .ok s) :=
  ⟨@DebugProofs.clone_spec, @ReleaseProofs.clone_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem finish_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T) (s : nohash_hasher.NoHashHasher T),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.finish inst s = .ok s.1) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T) (s : nohash_hasher_release.NoHashHasher T),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.finish inst s = .ok s.1) :=
  ⟨@DebugProofs.finish_spec, @ReleaseProofs.finish_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T) (s : nohash_hasher.NoHashHasher T) (bytes : Slice U8),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write inst s bytes = .fail .panic) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T) (s : nohash_hasher_release.NoHashHasher T) (bytes : Slice U8),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write inst s bytes = .fail .panic) :=
  ⟨@DebugProofs.write_spec, @ReleaseProofs.write_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u8_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U8),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_u8 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : U8),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_u8 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_u8_spec, @ReleaseProofs.write_u8_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u16_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U16),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_u16 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : U16),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_u16 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_u16_spec, @ReleaseProofs.write_u16_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u32_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U32),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_u32 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : U32),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_u32 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_u32_spec, @ReleaseProofs.write_u32_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u64_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U64),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_u64 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : U64),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_u64 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_u64_spec, @ReleaseProofs.write_u64_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_usize_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : Usize),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_usize inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : Usize),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_usize inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_usize_spec, @ReleaseProofs.write_usize_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i8_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I8),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_i8 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : I8),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_i8 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_i8_spec, @ReleaseProofs.write_i8_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i16_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I16),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_i16 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : I16),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_i16 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_i16_spec, @ReleaseProofs.write_i16_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i32_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I32),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_i32 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : I32),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_i32 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_i32_spec, @ReleaseProofs.write_i32_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i64_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I64),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_i64 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : I64),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_i64 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_i64_spec, @ReleaseProofs.write_i64_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_isize_spec :
    (∀ {T : Type} (inst : nohash_hasher.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : Isize),
    nohash_hasher.NoHashHasher.Insts.CoreHashHasher.write_isize inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_release.IsEnabled T)
    (old : U64) (n : Isize),
    nohash_hasher_release.NoHashHasher.Insts.CoreHashHasher.write_isize inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@DebugProofs.write_isize_spec, @ReleaseProofs.write_isize_spec⟩

end StdProofs
#print axioms StdProofs.default_spec
#print axioms StdProofs.clone_spec
#print axioms StdProofs.finish_spec
#print axioms StdProofs.write_spec
#print axioms StdProofs.write_u8_spec
#print axioms StdProofs.write_u16_spec
#print axioms StdProofs.write_u32_spec
#print axioms StdProofs.write_u64_spec
#print axioms StdProofs.write_usize_spec
#print axioms StdProofs.write_i8_spec
#print axioms StdProofs.write_i16_spec
#print axioms StdProofs.write_i32_spec
#print axioms StdProofs.write_i64_spec
#print axioms StdProofs.write_isize_spec
