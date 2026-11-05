import Serializers.Funs
open Aeneas Aeneas.Std Result Std.WP
open XoshiroSerializationModel
namespace SerializerProofs
@[simp] theorem bytes_u32 : SerializerRustLayout.bytes (T:=U32)=4#usize := by rfl
@[simp] theorem bytes_u64 : SerializerRustLayout.bytes (T:=U64)=8#usize := by rfl
theorem u32_size : core.mem.size_of U32=.ok 4#usize := by rfl
theorem u64_size : core.mem.size_of U64=.ok 8#usize := by rfl

theorem overwrite_halves {α} (xs ys : List α) (z : α) (n : Nat)
    (hx : xs.length=n) (hy : ys.length=n) :
    ((List.replicate (n+n) z).setSlice! 0 xs).setSlice! n ys=xs++ys := by
  simp_lists [List.setSlice!]
  simp only [List.take_zero,List.nil_append,List.append_nil]
  rw [←hx,List.take_left]

theorem overwrite_prefix {α} (front tail patch : List α)
    (h : patch.length≤tail.length) :
    (front++tail).setSlice! front.length patch =
      (front++patch)++tail.drop patch.length := by
  simp [List.setSlice!,List.drop_append,h,List.append_assoc]

theorem overwrite_four {α} (a b c d : List α) (z : α) (n : Nat)
    (ha : a.length=n) (hb : b.length=n) (hc : c.length=n) (hd : d.length=n) :
    ((((List.replicate (4*n) z).setSlice! 0 a).setSlice! n b).setSlice! (2*n) c).setSlice! (3*n) d = a++b++c++d := by
  have e4 : 4*n-n=3*n := by omega
  have e3 : 3*n-n=2*n := by omega
  have e2 : 2*n-n=n := by omega
  have h0 : (List.replicate (4*n) z).setSlice! 0 a=a++List.replicate (3*n) z := by
    have h := overwrite_prefix ([] : List α) (List.replicate (4*n) z) a (by simp [ha]; omega)
    simpa [ha,e4] using h
  have h1 : (a++List.replicate (3*n) z).setSlice! n b=(a++b)++List.replicate (2*n) z := by
    have h := overwrite_prefix a (List.replicate (3*n) z) b (by simp [hb]; omega)
    simpa [ha,hb,e3] using h
  have h2 : ((a++b)++List.replicate (2*n) z).setSlice! (2*n) c=((a++b)++c)++List.replicate n z := by
    have h := overwrite_prefix (a++b) (List.replicate (2*n) z) c (by simp [hc]; omega)
    simpa [ha,hb,hc,e2,←two_mul] using h
  have h3 : (((a++b)++c)++List.replicate n z).setSlice! (3*n) d=((a++b)++c)++d := by
    have h := overwrite_prefix ((a++b)++c) (List.replicate n z) d (by simp [hd])
    simpa [ha,hb,hc,hd,show n+(n+n)=3*n by omega] using h
  rw [h0,h1,h2,h3]
  all_goals simp only [List.append_assoc]

def arr4 {T : Type} (a b c d : T) : Array T 4#usize := Array.from [a,b,c,d] (by simp)
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
theorem Xoroshiro64Star_state_spec (s : xoroshiro64star.Xoroshiro64Star) :
    s.state ⦃ bytes => bytes.val=
      s.s0.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ++
      s.s1.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ⦄ := by
  unfold xoroshiro64star.Xoroshiro64Star.state
  simp only [xoroshiro64star.Xoroshiro64Star.state.N,core.mem.size_of,bytes_u32]
  step*
  all_goals try scalar_tac
  subst_vars
  rw [s3_post2,s_post2]
  simp only [Array.val_to_slice,Array.repeat_val]
  rw [←a_post,←a1_post]
  exact overwrite_halves a.val a1.val 0#u8 4 a.property a1.property

theorem Xoroshiro64StarStar_state_spec (s : xoroshiro64starstar.Xoroshiro64StarStar) :
    s.state ⦃ bytes => bytes.val=
      s.s0.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ++
      s.s1.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ⦄ := by
  unfold xoroshiro64starstar.Xoroshiro64StarStar.state
  simp only [xoroshiro64starstar.Xoroshiro64StarStar.state.N,core.mem.size_of,bytes_u32]
  step*
  all_goals try scalar_tac
  subst_vars
  rw [s3_post2,s_post2]
  simp only [Array.val_to_slice,Array.repeat_val]
  rw [←a_post,←a1_post]
  exact overwrite_halves a.val a1.val 0#u8 4 a.property a1.property

theorem Xoroshiro128Plus_state_spec (s : xoroshiro128plus.Xoroshiro128Plus) :
    s.state ⦃ bytes => bytes.val=
      s.s0.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ++
      s.s1.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ⦄ := by
  unfold xoroshiro128plus.Xoroshiro128Plus.state
  simp only [xoroshiro128plus.Xoroshiro128Plus.state.N,core.mem.size_of,bytes_u64]
  step*
  all_goals try scalar_tac
  subst_vars
  rw [s3_post2,s_post2]
  simp only [Array.val_to_slice,Array.repeat_val]
  rw [←a_post,←a1_post]
  exact overwrite_halves a.val a1.val 0#u8 8 a.property a1.property

theorem Xoroshiro128PlusPlus_state_spec (s : xoroshiro128plusplus.Xoroshiro128PlusPlus) :
    s.state ⦃ bytes => bytes.val=
      s.s0.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ++
      s.s1.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ⦄ := by
  unfold xoroshiro128plusplus.Xoroshiro128PlusPlus.state
  simp only [xoroshiro128plusplus.Xoroshiro128PlusPlus.state.N,core.mem.size_of,bytes_u64]
  step*
  all_goals try scalar_tac
  subst_vars
  rw [s3_post2,s_post2]
  simp only [Array.val_to_slice,Array.repeat_val]
  rw [←a_post,←a1_post]
  exact overwrite_halves a.val a1.val 0#u8 8 a.property a1.property

theorem Xoroshiro128StarStar_state_spec (s : xoroshiro128starstar.Xoroshiro128StarStar) :
    s.state ⦃ bytes => bytes.val=
      s.s0.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ++
      s.s1.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ⦄ := by
  unfold xoroshiro128starstar.Xoroshiro128StarStar.state
  simp only [xoroshiro128starstar.Xoroshiro128StarStar.state.N,core.mem.size_of,bytes_u64]
  step*
  all_goals try scalar_tac
  subst_vars
  rw [s3_post2,s_post2]
  simp only [Array.val_to_slice,Array.repeat_val]
  rw [←a_post,←a1_post]
  exact overwrite_halves a.val a1.val 0#u8 8 a.property a1.property


theorem Xoshiro128Plus_state_spec (s : xoshiro128plus.Xoshiro128Plus) :
    s.state ⦃ bytes => bytes.val=s.s.val.flatMap
      (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
  cases s with
  | mk xs =>
    obtain ⟨a,b,c,d,heq⟩ := arr4_surjective xs
    subst xs
    unfold xoshiro128plus.Xoshiro128Plus.state
    simp only [xoshiro128plus.Xoshiro128Plus.state.N,core.mem.size_of,bytes_u32,
      arr4,Array.from_val,List.flatMap_cons,List.flatMap_nil,List.append_nil]
    step*
    all_goals try scalar_tac
    subst_vars
    rw [s9_post2,s6_post2,s3_post2,s_post2]
    simp only [Array.val_to_slice,Array.repeat_val]
    rw [i4_post,i7_post]
    simp only [Array.from_val,List.getElem_cons_zero,List.getElem_cons_succ] at a_post a1_post a2_post a3_post
    rw [←a_post,←a1_post,←a2_post,←a3_post]
    have hn : (16#usize).val=4*4 := by scalar_tac
    rw [hn]
    simpa only [List.append_assoc] using overwrite_four a.val a1.val a2.val a3.val 0#u8 4 a.property a1.property a2.property a3.property
theorem Xoshiro128PlusPlus_state_spec (s : xoshiro128plusplus.Xoshiro128PlusPlus) :
    s.state ⦃ bytes => bytes.val=s.s.val.flatMap
      (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
  cases s with
  | mk xs =>
    obtain ⟨a,b,c,d,heq⟩ := arr4_surjective xs
    subst xs
    unfold xoshiro128plusplus.Xoshiro128PlusPlus.state
    simp only [xoshiro128plusplus.Xoshiro128PlusPlus.state.N,core.mem.size_of,bytes_u32,
      arr4,Array.from_val,List.flatMap_cons,List.flatMap_nil,List.append_nil]
    step*
    all_goals try scalar_tac
    subst_vars
    rw [s9_post2,s6_post2,s3_post2,s_post2]
    simp only [Array.val_to_slice,Array.repeat_val]
    rw [i4_post,i7_post]
    simp only [Array.from_val,List.getElem_cons_zero,List.getElem_cons_succ] at a_post a1_post a2_post a3_post
    rw [←a_post,←a1_post,←a2_post,←a3_post]
    have hn : (16#usize).val=4*4 := by scalar_tac
    rw [hn]
    simpa only [List.append_assoc] using overwrite_four a.val a1.val a2.val a3.val 0#u8 4 a.property a1.property a2.property a3.property

theorem Xoshiro128StarStar_state_spec (s : xoshiro128starstar.Xoshiro128StarStar) :
    s.state ⦃ bytes => bytes.val=s.s.val.flatMap
      (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
  cases s with
  | mk xs =>
    obtain ⟨a,b,c,d,heq⟩ := arr4_surjective xs
    subst xs
    unfold xoshiro128starstar.Xoshiro128StarStar.state
    simp only [xoshiro128starstar.Xoshiro128StarStar.state.N,core.mem.size_of,bytes_u32,
      arr4,Array.from_val,List.flatMap_cons,List.flatMap_nil,List.append_nil]
    step*
    all_goals try scalar_tac
    subst_vars
    rw [s9_post2,s6_post2,s3_post2,s_post2]
    simp only [Array.val_to_slice,Array.repeat_val]
    rw [i4_post,i7_post]
    simp only [Array.from_val,List.getElem_cons_zero,List.getElem_cons_succ] at a_post a1_post a2_post a3_post
    rw [←a_post,←a1_post,←a2_post,←a3_post]
    have hn : (16#usize).val=4*4 := by scalar_tac
    rw [hn]
    simpa only [List.append_assoc] using overwrite_four a.val a1.val a2.val a3.val 0#u8 4 a.property a1.property a2.property a3.property

theorem Xoshiro256Plus_state_spec (s : xoshiro256plus.Xoshiro256Plus) :
    s.state ⦃ bytes => bytes.val=s.s.val.flatMap
      (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
  cases s with
  | mk xs =>
    obtain ⟨a,b,c,d,heq⟩ := arr4_surjective xs
    subst xs
    unfold xoshiro256plus.Xoshiro256Plus.state
    simp only [xoshiro256plus.Xoshiro256Plus.state.N,core.mem.size_of,bytes_u64,
      arr4,Array.from_val,List.flatMap_cons,List.flatMap_nil,List.append_nil]
    step*
    all_goals try scalar_tac
    subst_vars
    rw [s9_post2,s6_post2,s3_post2,s_post2]
    simp only [Array.val_to_slice,Array.repeat_val]
    rw [i4_post,i7_post]
    simp only [Array.from_val,List.getElem_cons_zero,List.getElem_cons_succ] at a_post a1_post a2_post a3_post
    rw [←a_post,←a1_post,←a2_post,←a3_post]
    have hn : (32#usize).val=4*8 := by scalar_tac
    rw [hn]
    simpa only [List.append_assoc] using overwrite_four a.val a1.val a2.val a3.val 0#u8 8 a.property a1.property a2.property a3.property

theorem Xoshiro256PlusPlus_state_spec (s : xoshiro256plusplus.Xoshiro256PlusPlus) :
    s.state ⦃ bytes => bytes.val=s.s.val.flatMap
      (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
  cases s with
  | mk xs =>
    obtain ⟨a,b,c,d,heq⟩ := arr4_surjective xs
    subst xs
    unfold xoshiro256plusplus.Xoshiro256PlusPlus.state
    simp only [xoshiro256plusplus.Xoshiro256PlusPlus.state.N,core.mem.size_of,bytes_u64,
      arr4,Array.from_val,List.flatMap_cons,List.flatMap_nil,List.append_nil]
    step*
    all_goals try scalar_tac
    subst_vars
    rw [s9_post2,s6_post2,s3_post2,s_post2]
    simp only [Array.val_to_slice,Array.repeat_val]
    rw [i4_post,i7_post]
    simp only [Array.from_val,List.getElem_cons_zero,List.getElem_cons_succ] at a_post a1_post a2_post a3_post
    rw [←a_post,←a1_post,←a2_post,←a3_post]
    have hn : (32#usize).val=4*8 := by scalar_tac
    rw [hn]
    simpa only [List.append_assoc] using overwrite_four a.val a1.val a2.val a3.val 0#u8 8 a.property a1.property a2.property a3.property

theorem Xoshiro256StarStar_state_spec (s : xoshiro256starstar.Xoshiro256StarStar) :
    s.state ⦃ bytes => bytes.val=s.s.val.flatMap
      (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
  cases s with
  | mk xs =>
    obtain ⟨a,b,c,d,heq⟩ := arr4_surjective xs
    subst xs
    unfold xoshiro256starstar.Xoshiro256StarStar.state
    simp only [xoshiro256starstar.Xoshiro256StarStar.state.N,core.mem.size_of,bytes_u64,
      arr4,Array.from_val,List.flatMap_cons,List.flatMap_nil,List.append_nil]
    step*
    all_goals try scalar_tac
    subst_vars
    rw [s9_post2,s6_post2,s3_post2,s_post2]
    simp only [Array.val_to_slice,Array.repeat_val]
    rw [i4_post,i7_post]
    simp only [Array.from_val,List.getElem_cons_zero,List.getElem_cons_succ] at a_post a1_post a2_post a3_post
    rw [←a_post,←a1_post,←a2_post,←a3_post]
    have hn : (32#usize).val=4*8 := by scalar_tac
    rw [hn]
    simpa only [List.append_assoc] using overwrite_four a.val a1.val a2.val a3.val 0#u8 8 a.property a1.property a2.property a3.property

end SerializerProofs
#print axioms SerializerProofs.Xoroshiro64Star_state_spec
#print axioms SerializerProofs.Xoroshiro64StarStar_state_spec
#print axioms SerializerProofs.Xoroshiro128Plus_state_spec
#print axioms SerializerProofs.Xoroshiro128PlusPlus_state_spec
#print axioms SerializerProofs.Xoroshiro128StarStar_state_spec

#print axioms SerializerProofs.Xoshiro128Plus_state_spec

#print axioms SerializerProofs.Xoshiro128PlusPlus_state_spec
#print axioms SerializerProofs.Xoshiro128StarStar_state_spec
#print axioms SerializerProofs.Xoshiro256Plus_state_spec
#print axioms SerializerProofs.Xoshiro256PlusPlus_state_spec
#print axioms SerializerProofs.Xoshiro256StarStar_state_spec
