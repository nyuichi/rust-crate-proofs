import NoStdDebugProofs
import NoStdReleaseProofs

open Aeneas Aeneas.Std Result Error

namespace NoStdProofs

/-- Exact original-source behavior in debug and release builds. -/
theorem default_spec :
    (∀ (T : Type),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreDefaultDefault.default T = .ok (0#u64, false, ())) ∧
    (∀ (T : Type),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreDefaultDefault.default T = .ok (0#u64, ())) :=
  ⟨@NoStdDebugProofs.default_spec, @NoStdReleaseProofs.default_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem clone_spec :
    (∀ {T : Type} (s : nohash_hasher_no_std.NoHashHasher T),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreCloneClone.clone s = .ok s) ∧
    (∀ {T : Type} (s : nohash_hasher_no_std_release.NoHashHasher T),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreCloneClone.clone s = .ok s) :=
  ⟨@NoStdDebugProofs.clone_spec, @NoStdReleaseProofs.clone_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem finish_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T) (s : nohash_hasher_no_std.NoHashHasher T),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.finish inst s = .ok s.1) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T) (s : nohash_hasher_no_std_release.NoHashHasher T),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.finish inst s = .ok s.1) :=
  ⟨@NoStdDebugProofs.finish_spec, @NoStdReleaseProofs.finish_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T) (s : nohash_hasher_no_std.NoHashHasher T) (bytes : Slice U8),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write inst s bytes = .fail .panic) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T) (s : nohash_hasher_no_std_release.NoHashHasher T) (bytes : Slice U8),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write inst s bytes = .fail .panic) :=
  ⟨@NoStdDebugProofs.write_spec, @NoStdReleaseProofs.write_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u8_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U8),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_u8 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : U8),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_u8 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_u8_spec, @NoStdReleaseProofs.write_u8_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u16_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U16),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_u16 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : U16),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_u16 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_u16_spec, @NoStdReleaseProofs.write_u16_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u32_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U32),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_u32 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : U32),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_u32 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_u32_spec, @NoStdReleaseProofs.write_u32_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_u64_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : U64),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_u64 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : U64),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_u64 inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_u64_spec, @NoStdReleaseProofs.write_u64_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_usize_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : Usize),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_usize inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.zeroExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : Usize),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_usize inst (old, ()) n =
       .ok ((⟨n.bv.zeroExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_usize_spec, @NoStdReleaseProofs.write_usize_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i8_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I8),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_i8 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : I8),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_i8 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_i8_spec, @NoStdReleaseProofs.write_i8_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i16_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I16),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_i16 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : I16),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_i16 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_i16_spec, @NoStdReleaseProofs.write_i16_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i32_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I32),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_i32 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : I32),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_i32 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_i32_spec, @NoStdReleaseProofs.write_i32_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_i64_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : I64),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_i64 inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : I64),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_i64 inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_i64_spec, @NoStdReleaseProofs.write_i64_spec⟩

/-- Exact original-source behavior in debug and release builds. -/
theorem write_isize_spec :
    (∀ {T : Type} (inst : nohash_hasher_no_std.IsEnabled T)
    (old : U64) (alreadyWritten : Bool) (n : Isize),
    nohash_hasher_no_std.NoHashHasher.Insts.CoreHashHasher.write_isize inst (old, alreadyWritten, ()) n =
      if alreadyWritten then .fail .assertionFailure else .ok ((⟨n.bv.signExtend 64⟩ : U64), true, ())) ∧
    (∀ {T : Type} (inst : nohash_hasher_no_std_release.IsEnabled T)
    (old : U64) (n : Isize),
    nohash_hasher_no_std_release.NoHashHasher.Insts.CoreHashHasher.write_isize inst (old, ()) n =
       .ok ((⟨n.bv.signExtend 64⟩ : U64), ())) :=
  ⟨@NoStdDebugProofs.write_isize_spec, @NoStdReleaseProofs.write_isize_spec⟩

end NoStdProofs
#print axioms NoStdProofs.default_spec
#print axioms NoStdProofs.clone_spec
#print axioms NoStdProofs.finish_spec
#print axioms NoStdProofs.write_spec
#print axioms NoStdProofs.write_u8_spec
#print axioms NoStdProofs.write_u16_spec
#print axioms NoStdProofs.write_u32_spec
#print axioms NoStdProofs.write_u64_spec
#print axioms NoStdProofs.write_usize_spec
#print axioms NoStdProofs.write_i8_spec
#print axioms NoStdProofs.write_i16_spec
#print axioms NoStdProofs.write_i32_spec
#print axioms NoStdProofs.write_i64_spec
#print axioms NoStdProofs.write_isize_spec
