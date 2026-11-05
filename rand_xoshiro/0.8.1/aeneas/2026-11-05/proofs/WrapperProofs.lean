import XoshiroApiRoots.Funs
open Aeneas Aeneas.Std Result
open xoshiro_api_roots xoshiro_api_roots.rand_xoshiro
namespace WrapperProofs
set_option linter.unusedSimpArgs false
set_option maxRecDepth 4096

def arr4 {T : Type} (a b c d : T) : Aeneas.Std.Array T 4#usize := Array.from [a,b,c,d] (by simp)
def arr8 {T : Type} (a b c d e f g h : T) : Aeneas.Std.Array T 8#usize := Array.from [a,b,c,d,e,f,g,h] (by simp)

def inputXoroshiro128Plus_64 (a b : U64) : xoroshiro128plus.Xoroshiro128Plus := {s0 := a,s1 := b}
def outputXoroshiro128Plus_64 (a b : U64) : U64 := a.wrapping_add b
def nextXoroshiro128Plus_64 (a b : U64) : xoroshiro128plus.Xoroshiro128Plus :=
  let d := b ^^^ a
  {s0 := (a.rotate_left 24#u32) ^^^ d ^^^ (⟨d.bv <<< 16⟩ : U64), s1 := d.rotate_left 37#u32}
theorem Xoroshiro128Plus_64_spec (a b : U64) :
  xoroshiro128plus.Xoroshiro128Plus.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoroshiro128Plus_64 a b) =
    .ok (.Ok (outputXoroshiro128Plus_64 a b), nextXoroshiro128Plus_64 a b) := by
  simp only [xoroshiro128plus.Xoroshiro128Plus.Insts.Rand_coreTryRngNever.try_next_u64, inputXoroshiro128Plus_64, outputXoroshiro128Plus_64, nextXoroshiro128Plus_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoroshiro128PlusPlus_64 (a b : U64) : xoroshiro128plusplus.Xoroshiro128PlusPlus := {s0 := a,s1 := b}
def outputXoroshiro128PlusPlus_64 (a b : U64) : U64 := ((a.wrapping_add b).rotate_left 17#u32).wrapping_add a
def nextXoroshiro128PlusPlus_64 (a b : U64) : xoroshiro128plusplus.Xoroshiro128PlusPlus :=
  let d := b ^^^ a
  {s0 := (a.rotate_left 49#u32) ^^^ d ^^^ (⟨d.bv <<< 21⟩ : U64), s1 := d.rotate_left 28#u32}
theorem Xoroshiro128PlusPlus_64_spec (a b : U64) :
  xoroshiro128plusplus.Xoroshiro128PlusPlus.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoroshiro128PlusPlus_64 a b) =
    .ok (.Ok (outputXoroshiro128PlusPlus_64 a b), nextXoroshiro128PlusPlus_64 a b) := by
  simp only [xoroshiro128plusplus.Xoroshiro128PlusPlus.Insts.Rand_coreTryRngNever.try_next_u64, inputXoroshiro128PlusPlus_64, outputXoroshiro128PlusPlus_64, nextXoroshiro128PlusPlus_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoroshiro128StarStar_64 (a b : U64) : xoroshiro128starstar.Xoroshiro128StarStar := {s0 := a,s1 := b}
def outputXoroshiro128StarStar_64 (a b : U64) : U64 := ((a.wrapping_mul 5#u64).rotate_left 7#u32).wrapping_mul 9#u64
def nextXoroshiro128StarStar_64 (a b : U64) : xoroshiro128starstar.Xoroshiro128StarStar :=
  let d := b ^^^ a
  {s0 := (a.rotate_left 24#u32) ^^^ d ^^^ (⟨d.bv <<< 16⟩ : U64), s1 := d.rotate_left 37#u32}
theorem Xoroshiro128StarStar_64_spec (a b : U64) :
  xoroshiro128starstar.Xoroshiro128StarStar.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoroshiro128StarStar_64 a b) =
    .ok (.Ok (outputXoroshiro128StarStar_64 a b), nextXoroshiro128StarStar_64 a b) := by
  simp only [xoroshiro128starstar.Xoroshiro128StarStar.Insts.Rand_coreTryRngNever.try_next_u64, inputXoroshiro128StarStar_64, outputXoroshiro128StarStar_64, nextXoroshiro128StarStar_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoshiro256Plus_64 (a b c d : U64) : xoshiro256plus.Xoshiro256Plus := {s := arr4 a b c d}
def outputXoshiro256Plus_64 (a b c d : U64) : U64 := a.wrapping_add d
def nextXoshiro256Plus_64 (a b c d : U64) : xoshiro256plus.Xoshiro256Plus :=
  let c1 := c ^^^ a
  let d1 := d ^^^ b
  {s := arr4 (a ^^^ d1) (b ^^^ c1) (c1 ^^^ (⟨b.bv <<< 17⟩ : U64)) (d1.rotate_left 45#u32)}
theorem Xoshiro256Plus_64_spec (a b c d : U64) :
  xoshiro256plus.Xoshiro256Plus.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoshiro256Plus_64 a b c d) =
    .ok (.Ok (outputXoshiro256Plus_64 a b c d), nextXoshiro256Plus_64 a b c d) := by
  simp only [xoshiro256plus.Xoshiro256Plus.Insts.Rand_coreTryRngNever.try_next_u64, inputXoshiro256Plus_64, outputXoshiro256Plus_64, nextXoshiro256Plus_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoshiro256PlusPlus_64 (a b c d : U64) : xoshiro256plusplus.Xoshiro256PlusPlus := {s := arr4 a b c d}
def outputXoshiro256PlusPlus_64 (a b c d : U64) : U64 := ((a.wrapping_add d).rotate_left 23#u32).wrapping_add a
def nextXoshiro256PlusPlus_64 (a b c d : U64) : xoshiro256plusplus.Xoshiro256PlusPlus :=
  let c1 := c ^^^ a
  let d1 := d ^^^ b
  {s := arr4 (a ^^^ d1) (b ^^^ c1) (c1 ^^^ (⟨b.bv <<< 17⟩ : U64)) (d1.rotate_left 45#u32)}
theorem Xoshiro256PlusPlus_64_spec (a b c d : U64) :
  xoshiro256plusplus.Xoshiro256PlusPlus.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoshiro256PlusPlus_64 a b c d) =
    .ok (.Ok (outputXoshiro256PlusPlus_64 a b c d), nextXoshiro256PlusPlus_64 a b c d) := by
  simp only [xoshiro256plusplus.Xoshiro256PlusPlus.Insts.Rand_coreTryRngNever.try_next_u64, inputXoshiro256PlusPlus_64, outputXoshiro256PlusPlus_64, nextXoshiro256PlusPlus_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoshiro256StarStar_64 (a b c d : U64) : xoshiro256starstar.Xoshiro256StarStar := {s := arr4 a b c d}
def outputXoshiro256StarStar_64 (a b c d : U64) : U64 := ((b.wrapping_mul 5#u64).rotate_left 7#u32).wrapping_mul 9#u64
def nextXoshiro256StarStar_64 (a b c d : U64) : xoshiro256starstar.Xoshiro256StarStar :=
  let c1 := c ^^^ a
  let d1 := d ^^^ b
  {s := arr4 (a ^^^ d1) (b ^^^ c1) (c1 ^^^ (⟨b.bv <<< 17⟩ : U64)) (d1.rotate_left 45#u32)}
theorem Xoshiro256StarStar_64_spec (a b c d : U64) :
  xoshiro256starstar.Xoshiro256StarStar.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoshiro256StarStar_64 a b c d) =
    .ok (.Ok (outputXoshiro256StarStar_64 a b c d), nextXoshiro256StarStar_64 a b c d) := by
  simp only [xoshiro256starstar.Xoshiro256StarStar.Insts.Rand_coreTryRngNever.try_next_u64, inputXoshiro256StarStar_64, outputXoshiro256StarStar_64, nextXoshiro256StarStar_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoshiro512Plus_64 (a b c d e f g h : U64) : xoshiro512plus.Xoshiro512Plus := {s := arr8 a b c d e f g h}
def outputXoshiro512Plus_64 (a b c d e f g h : U64) : U64 := a.wrapping_add c
def nextXoshiro512Plus_64 (a b c d e f g h : U64) : xoshiro512plus.Xoshiro512Plus :=
  let c1 := c ^^^ a
  let f1 := f ^^^ b
  let h1 := h ^^^ d
  {s := arr8 (a ^^^ g) (b ^^^ c1) c1 (d ^^^ e) (e ^^^ f1) f1
    (g ^^^ h1 ^^^ (⟨b.bv <<< 11⟩ : U64)) (h1.rotate_left 21#u32)}
theorem Xoshiro512Plus_64_spec (a b c d e f g h : U64) :
  xoshiro512plus.Xoshiro512Plus.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoshiro512Plus_64 a b c d e f g h) =
    .ok (.Ok (outputXoshiro512Plus_64 a b c d e f g h), nextXoshiro512Plus_64 a b c d e f g h) := by
  simp only [xoshiro512plus.Xoshiro512Plus.Insts.Rand_coreTryRngNever.try_next_u64, inputXoshiro512Plus_64, outputXoshiro512Plus_64, nextXoshiro512Plus_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoshiro512PlusPlus_64 (a b c d e f g h : U64) : xoshiro512plusplus.Xoshiro512PlusPlus := {s := arr8 a b c d e f g h}
def outputXoshiro512PlusPlus_64 (a b c d e f g h : U64) : U64 := ((c.wrapping_add a).rotate_left 17#u32).wrapping_add c
def nextXoshiro512PlusPlus_64 (a b c d e f g h : U64) : xoshiro512plusplus.Xoshiro512PlusPlus :=
  let c1 := c ^^^ a
  let f1 := f ^^^ b
  let h1 := h ^^^ d
  {s := arr8 (a ^^^ g) (b ^^^ c1) c1 (d ^^^ e) (e ^^^ f1) f1
    (g ^^^ h1 ^^^ (⟨b.bv <<< 11⟩ : U64)) (h1.rotate_left 21#u32)}
theorem Xoshiro512PlusPlus_64_spec (a b c d e f g h : U64) :
  xoshiro512plusplus.Xoshiro512PlusPlus.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoshiro512PlusPlus_64 a b c d e f g h) =
    .ok (.Ok (outputXoshiro512PlusPlus_64 a b c d e f g h), nextXoshiro512PlusPlus_64 a b c d e f g h) := by
  simp only [xoshiro512plusplus.Xoshiro512PlusPlus.Insts.Rand_coreTryRngNever.try_next_u64, inputXoshiro512PlusPlus_64, outputXoshiro512PlusPlus_64, nextXoshiro512PlusPlus_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]

def inputXoshiro512StarStar_64 (a b c d e f g h : U64) : xoshiro512starstar.Xoshiro512StarStar := {s := arr8 a b c d e f g h}
def outputXoshiro512StarStar_64 (a b c d e f g h : U64) : U64 := ((b.wrapping_mul 5#u64).rotate_left 7#u32).wrapping_mul 9#u64
def nextXoshiro512StarStar_64 (a b c d e f g h : U64) : xoshiro512starstar.Xoshiro512StarStar :=
  let c1 := c ^^^ a
  let f1 := f ^^^ b
  let h1 := h ^^^ d
  {s := arr8 (a ^^^ g) (b ^^^ c1) c1 (d ^^^ e) (e ^^^ f1) f1
    (g ^^^ h1 ^^^ (⟨b.bv <<< 11⟩ : U64)) (h1.rotate_left 21#u32)}
theorem Xoshiro512StarStar_64_spec (a b c d e f g h : U64) :
  xoshiro512starstar.Xoshiro512StarStar.Insts.Rand_coreTryRngNever.try_next_u64 (inputXoshiro512StarStar_64 a b c d e f g h) =
    .ok (.Ok (outputXoshiro512StarStar_64 a b c d e f g h), nextXoshiro512StarStar_64 a b c d e f g h) := by
  simp only [xoshiro512starstar.Xoshiro512StarStar.Insts.Rand_coreTryRngNever.try_next_u64, inputXoshiro512StarStar_64, outputXoshiro512StarStar_64, nextXoshiro512StarStar_64]
  simp [arr4, arr8, Array.index_usize, Array.update, lift,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U64.wrapping_add, core.num.U64.wrapping_mul,
    core.num.U64.rotate_left, U64.wrapping_add, U64.wrapping_mul,
    -BitVec.shiftLeft_eq]
theorem arr4_surjective {T : Type} (xs : Aeneas.Std.Array T 4#usize) :
  ∃ a b c d, arr4 a b c d = xs := by
  have hl0 : xs.val.length = 4 := by simpa using xs.property
  obtain ⟨a,l1,hh0,hl1⟩ := List.length_eq_succ_iff.mp hl0
  obtain ⟨b,l2,hh1,hl2⟩ := List.length_eq_succ_iff.mp hl1
  obtain ⟨c,l3,hh2,hl3⟩ := List.length_eq_succ_iff.mp hl2
  obtain ⟨d,l4,hh3,hl4⟩ := List.length_eq_succ_iff.mp hl3
  have hz : l4 = [] := by simpa using hl4
  refine ⟨a,b,c,d,?_⟩
  apply Aeneas.Std.Array.ext
  simp only [arr4, Array.from_val]
  simp only [←hh0,←hh1,←hh2,←hh3,hz]
theorem arr8_surjective {T : Type} (xs : Aeneas.Std.Array T 8#usize) :
  ∃ a b c d e f g h, arr8 a b c d e f g h = xs := by
  have hl0 : xs.val.length = 8 := by simpa using xs.property
  obtain ⟨a,l1,hh0,hl1⟩ := List.length_eq_succ_iff.mp hl0
  obtain ⟨b,l2,hh1,hl2⟩ := List.length_eq_succ_iff.mp hl1
  obtain ⟨c,l3,hh2,hl3⟩ := List.length_eq_succ_iff.mp hl2
  obtain ⟨d,l4,hh3,hl4⟩ := List.length_eq_succ_iff.mp hl3
  obtain ⟨e,l5,hh4,hl5⟩ := List.length_eq_succ_iff.mp hl4
  obtain ⟨f,l6,hh5,hl6⟩ := List.length_eq_succ_iff.mp hl5
  obtain ⟨g,l7,hh6,hl7⟩ := List.length_eq_succ_iff.mp hl6
  obtain ⟨h,l8,hh7,hl8⟩ := List.length_eq_succ_iff.mp hl7
  have hz : l8 = [] := by simpa using hl8
  refine ⟨a,b,c,d,e,f,g,h,?_⟩
  apply Aeneas.Std.Array.ext
  simp only [arr8, Array.from_val]
  simp only [←hh0,←hh1,←hh2,←hh3,←hh4,←hh5,←hh6,←hh7,hz]
theorem inputXoroshiro128Plus_64_surjective (s : xoroshiro128plus.Xoroshiro128Plus) :
    ∃ a b, inputXoroshiro128Plus_64 a b = s := by
  exact ⟨s.s0,s.s1,by cases s; rfl⟩
theorem inputXoroshiro128PlusPlus_64_surjective (s : xoroshiro128plusplus.Xoroshiro128PlusPlus) :
    ∃ a b, inputXoroshiro128PlusPlus_64 a b = s := by
  exact ⟨s.s0,s.s1,by cases s; rfl⟩
theorem inputXoroshiro128StarStar_64_surjective (s : xoroshiro128starstar.Xoroshiro128StarStar) :
    ∃ a b, inputXoroshiro128StarStar_64 a b = s := by
  exact ⟨s.s0,s.s1,by cases s; rfl⟩
theorem inputXoshiro256Plus_64_surjective (s : xoshiro256plus.Xoshiro256Plus) :
    ∃ a b c d, inputXoshiro256Plus_64 a b c d = s := by
  obtain ⟨a,b,c,d,heq⟩ := arr4_surjective s.s
  refine ⟨a,b,c,d,?_⟩
  cases s
  unfold inputXoshiro256Plus_64
  congr 1
theorem inputXoshiro256PlusPlus_64_surjective (s : xoshiro256plusplus.Xoshiro256PlusPlus) :
    ∃ a b c d, inputXoshiro256PlusPlus_64 a b c d = s := by
  obtain ⟨a,b,c,d,heq⟩ := arr4_surjective s.s
  refine ⟨a,b,c,d,?_⟩
  cases s
  unfold inputXoshiro256PlusPlus_64
  congr 1
theorem inputXoshiro256StarStar_64_surjective (s : xoshiro256starstar.Xoshiro256StarStar) :
    ∃ a b c d, inputXoshiro256StarStar_64 a b c d = s := by
  obtain ⟨a,b,c,d,heq⟩ := arr4_surjective s.s
  refine ⟨a,b,c,d,?_⟩
  cases s
  unfold inputXoshiro256StarStar_64
  congr 1
theorem inputXoshiro512Plus_64_surjective (s : xoshiro512plus.Xoshiro512Plus) :
    ∃ a b c d e f g h, inputXoshiro512Plus_64 a b c d e f g h = s := by
  obtain ⟨a,b,c,d,e,f,g,h,heq⟩ := arr8_surjective s.s
  refine ⟨a,b,c,d,e,f,g,h,?_⟩
  cases s
  unfold inputXoshiro512Plus_64
  congr 1
theorem inputXoshiro512PlusPlus_64_surjective (s : xoshiro512plusplus.Xoshiro512PlusPlus) :
    ∃ a b c d e f g h, inputXoshiro512PlusPlus_64 a b c d e f g h = s := by
  obtain ⟨a,b,c,d,e,f,g,h,heq⟩ := arr8_surjective s.s
  refine ⟨a,b,c,d,e,f,g,h,?_⟩
  cases s
  unfold inputXoshiro512PlusPlus_64
  congr 1
theorem inputXoshiro512StarStar_64_surjective (s : xoshiro512starstar.Xoshiro512StarStar) :
    ∃ a b c d e f g h, inputXoshiro512StarStar_64 a b c d e f g h = s := by
  obtain ⟨a,b,c,d,e,f,g,h,heq⟩ := arr8_surjective s.s
  refine ⟨a,b,c,d,e,f,g,h,?_⟩
  cases s
  unfold inputXoshiro512StarStar_64
  congr 1
theorem Xoroshiro128Plus_32_via_original_api_spec (a b : U64) :
  xoshiro_api_roots.xoroshiro128plus_next32 (inputXoroshiro128Plus_64 a b) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoroshiro128Plus_64 a b).bv >>> 32⟩ : U64)), nextXoroshiro128Plus_64 a b) := by
  simp [xoshiro_api_roots.xoroshiro128plus_next32, Xoroshiro128Plus_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoroshiro128PlusPlus_32_via_original_api_spec (a b : U64) :
  xoshiro_api_roots.xoroshiro128plusplus_next32 (inputXoroshiro128PlusPlus_64 a b) =
    .ok (.Ok (UScalar.cast .U32 (outputXoroshiro128PlusPlus_64 a b)), nextXoroshiro128PlusPlus_64 a b) := by
  simp [xoshiro_api_roots.xoroshiro128plusplus_next32, Xoroshiro128PlusPlus_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoroshiro128StarStar_32_via_original_api_spec (a b : U64) :
  xoshiro_api_roots.xoroshiro128starstar_next32 (inputXoroshiro128StarStar_64 a b) =
    .ok (.Ok (UScalar.cast .U32 (outputXoroshiro128StarStar_64 a b)), nextXoroshiro128StarStar_64 a b) := by
  simp [xoshiro_api_roots.xoroshiro128starstar_next32, Xoroshiro128StarStar_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoshiro256Plus_32_via_original_api_spec (a b c d : U64) :
  xoshiro_api_roots.xoshiro256plus_next32 (inputXoshiro256Plus_64 a b c d) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoshiro256Plus_64 a b c d).bv >>> 32⟩ : U64)), nextXoshiro256Plus_64 a b c d) := by
  simp [xoshiro_api_roots.xoshiro256plus_next32, Xoshiro256Plus_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoshiro256PlusPlus_32_via_original_api_spec (a b c d : U64) :
  xoshiro_api_roots.xoshiro256plusplus_next32 (inputXoshiro256PlusPlus_64 a b c d) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoshiro256PlusPlus_64 a b c d).bv >>> 32⟩ : U64)), nextXoshiro256PlusPlus_64 a b c d) := by
  simp [xoshiro_api_roots.xoshiro256plusplus_next32, Xoshiro256PlusPlus_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoshiro256StarStar_32_via_original_api_spec (a b c d : U64) :
  xoshiro_api_roots.xoshiro256starstar_next32 (inputXoshiro256StarStar_64 a b c d) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoshiro256StarStar_64 a b c d).bv >>> 32⟩ : U64)), nextXoshiro256StarStar_64 a b c d) := by
  simp [xoshiro_api_roots.xoshiro256starstar_next32, Xoshiro256StarStar_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoshiro512Plus_32_via_original_api_spec (a b c d e f g h : U64) :
  xoshiro_api_roots.xoshiro512plus_next32 (inputXoshiro512Plus_64 a b c d e f g h) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoshiro512Plus_64 a b c d e f g h).bv >>> 32⟩ : U64)), nextXoshiro512Plus_64 a b c d e f g h) := by
  simp [xoshiro_api_roots.xoshiro512plus_next32, Xoshiro512Plus_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoshiro512PlusPlus_32_via_original_api_spec (a b c d e f g h : U64) :
  xoshiro_api_roots.xoshiro512plusplus_next32 (inputXoshiro512PlusPlus_64 a b c d e f g h) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoshiro512PlusPlus_64 a b c d e f g h).bv >>> 32⟩ : U64)), nextXoshiro512PlusPlus_64 a b c d e f g h) := by
  simp [xoshiro_api_roots.xoshiro512plusplus_next32, Xoshiro512PlusPlus_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
theorem Xoshiro512StarStar_32_via_original_api_spec (a b c d e f g h : U64) :
  xoshiro_api_roots.xoshiro512starstar_next32 (inputXoshiro512StarStar_64 a b c d e f g h) =
    .ok (.Ok (UScalar.cast .U32 (⟨(outputXoshiro512StarStar_64 a b c d e f g h).bv >>> 32⟩ : U64)), nextXoshiro512StarStar_64 a b c d e f g h) := by
  simp [xoshiro_api_roots.xoshiro512starstar_next32, Xoshiro512StarStar_64_spec, lift,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    IScalar.toNat, -BitVec.ushiftRight_eq]
  all_goals rfl
end WrapperProofs
#print axioms WrapperProofs.Xoroshiro128Plus_64_spec
#print axioms WrapperProofs.Xoroshiro128PlusPlus_64_spec
#print axioms WrapperProofs.Xoroshiro128StarStar_64_spec
#print axioms WrapperProofs.Xoshiro256Plus_64_spec
#print axioms WrapperProofs.Xoshiro256PlusPlus_64_spec
#print axioms WrapperProofs.Xoshiro256StarStar_64_spec
#print axioms WrapperProofs.Xoshiro512Plus_64_spec
#print axioms WrapperProofs.Xoshiro512PlusPlus_64_spec
#print axioms WrapperProofs.Xoshiro512StarStar_64_spec
#print axioms WrapperProofs.arr4_surjective
#print axioms WrapperProofs.arr8_surjective
#print axioms WrapperProofs.inputXoroshiro128Plus_64_surjective
#print axioms WrapperProofs.inputXoroshiro128PlusPlus_64_surjective
#print axioms WrapperProofs.inputXoroshiro128StarStar_64_surjective
#print axioms WrapperProofs.inputXoshiro256Plus_64_surjective
#print axioms WrapperProofs.inputXoshiro256PlusPlus_64_surjective
#print axioms WrapperProofs.inputXoshiro256StarStar_64_surjective
#print axioms WrapperProofs.inputXoshiro512Plus_64_surjective
#print axioms WrapperProofs.inputXoshiro512PlusPlus_64_surjective
#print axioms WrapperProofs.inputXoshiro512StarStar_64_surjective
#print axioms WrapperProofs.Xoroshiro128Plus_32_via_original_api_spec
#print axioms WrapperProofs.Xoroshiro128PlusPlus_32_via_original_api_spec
#print axioms WrapperProofs.Xoroshiro128StarStar_32_via_original_api_spec
#print axioms WrapperProofs.Xoshiro256Plus_32_via_original_api_spec
#print axioms WrapperProofs.Xoshiro256PlusPlus_32_via_original_api_spec
#print axioms WrapperProofs.Xoshiro256StarStar_32_via_original_api_spec
#print axioms WrapperProofs.Xoshiro512Plus_32_via_original_api_spec
#print axioms WrapperProofs.Xoshiro512PlusPlus_32_via_original_api_spec
#print axioms WrapperProofs.Xoshiro512StarStar_32_via_original_api_spec
