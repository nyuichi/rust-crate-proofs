import SerializerProofs
open Aeneas Aeneas.Std Result Std.WP
open XoshiroSerializationModel
namespace SerializerProofs
set_option maxRecDepth 4096
set_option maxHeartbeats 1500000
abbrev SerIter := core.iter.adapters.enumerate.Enumerate (core.slice.iter.Iter U64)
def packed (ws : List U64) := ws.flatMap (fun w => w.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
theorem body_cont (ws : Slice U64) (n : Usize) (out : Array U8 64#usize)
 (hlen : ws.val.length=8) (hn : n.val<8) :
 xoshiro512plus.Xoshiro512Plus.state_loop.body ⟨⟨ws,n.val⟩,n⟩ out
 ⦃ r => ∃ n' out', r=.cont (⟨⟨ws,n.val+1⟩,n'⟩,out') ∧ n'.val=n.val+1 ∧
 out'.val=out.val.setSlice! (n.val*8) (ws.val[n.val].bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)) ⦄ := by
 unfold xoshiro512plus.Xoshiro512Plus.state_loop.body
 simp only [core.iter.adapters.enumerate.IteratorEnumerate.next,
 core.iter.traits.iterator.IteratorSliceIter,
 core.slice.iter.IteratorSliceIter.next]
 have h : n.val<ws.len := by simpa [Slice.len,hlen] using hn
 simp only [h, _root_.dite_true,bind_tc_ok]
 step*
 all_goals try scalar_tac
 all_goals try (subst_vars; simp_all only [Array.length_to_slice, Array.length]; scalar_tac)

theorem take_setSlice {α} (xs patch : List α) (k : Nat)
 (h : k+patch.length≤xs.length) :
 (xs.setSlice! k patch).take (k+patch.length)=xs.take k++patch := by
 simp only [List.setSlice!]
 rw [List.take_append]
 have hk : k≤xs.length := by omega
 have hp : patch.length≤xs.length-k := by omega
 simp only [Nat.min_eq_left hp,List.take_length]
 have hl : (xs.take k++patch).length=k+patch.length := by simp [List.length_take,Nat.min_eq_left hk]
 rw [←hl,List.take_length,Nat.sub_self]
 simp


theorem body_done (ws : Slice U64) (n : Usize) (out : Array U8 64#usize) (hlen : ws.val.length=8) (he : n.val=8) :
 xoshiro512plus.Xoshiro512Plus.state_loop.body ⟨⟨ws,n.val⟩,n⟩ out = .ok (.done out) := by
 unfold xoshiro512plus.Xoshiro512Plus.state_loop.body
 simp only [core.iter.adapters.enumerate.IteratorEnumerate.next,
 core.iter.traits.iterator.IteratorSliceIter,core.slice.iter.IteratorSliceIter.next]
 have h : ¬n.val<ws.len.val := by simp [Slice.len,hlen,he]
 simp [hlen,he,bind_tc_ok]

#print axioms SerializerProofs.body_cont

def serialInv (ws : Slice U64) (st : SerIter × Array U8 64#usize) : Prop :=
 st.1.iter.slice=ws ∧ st.1.iter.i=st.1.count.val ∧ st.1.count.val≤8 ∧
 st.2.val.take (st.1.count.val*8)=packed (ws.val.take st.1.count.val)

theorem loop_spec (ws : Slice U64) (hlen : ws.val.length=8)
 (it : SerIter) (out : Array U8 64#usize) (hi : serialInv ws (it,out)) :
 xoshiro512plus.Xoshiro512Plus.state_loop it out ⦃ bytes => bytes.val=packed ws.val ⦄ := by
 unfold xoshiro512plus.Xoshiro512Plus.state_loop
 apply loop.spec_decr_nat (fun (st : SerIter × Array U8 64#usize) => 8-st.1.count.val) (serialInv ws)
   (fun (bytes : Array U8 64#usize) => bytes.val=packed ws.val)
   (fun st => xoshiro512plus.Xoshiro512Plus.state_loop.body st.1 st.2) (it,out)
 · intro st hs
   rcases st with ⟨⟨⟨slice,i⟩,n⟩,out⟩
   rcases hs with ⟨hs,hi,hn,hout⟩
   dsimp only at hs hi hn hout ⊢
   subst slice i
   by_cases hlt : n.val<8
   · have hb := body_cont ws n out hlen hlt
     apply WP.spec_mono hb
     rintro r ⟨n',out',rfl,hn',ho⟩
     refine ⟨?_,?_⟩
     · refine ⟨rfl,?_,?_,?_⟩
       · exact hn'.symm
       · dsimp only; omega
       · dsimp only
         rw [ho,hn']
         have hp : (ws.val[n.val].bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)).length=8 := by simp
         have hl : out.val.length=64 := by simpa using out.property
         have hk : (n.val+1)*8=n.val*8+(ws.val[n.val].bv.toLEBytes.map (@UScalar.mk UScalarTy.U8)).length := by rw [hp]; omega
         rw [hk,take_setSlice _ _ _ (by rw [hp,hl]; omega),hout]
         unfold packed
         have hw : ws.val.take (n.val+1)=ws.val.take n.val++[ws.val[n.val]] := List.take_succ_eq_append_getElem (by omega)
         rw [hw]
         simp only [List.flatMap_append,List.flatMap_cons,List.flatMap_nil,List.append_nil]

     · dsimp only; omega
   · have he : n.val=8 := by omega
     rw [body_done ws n out hlen he]
     simp only [WP.spec_ok]
     have hl : out.val.length=64 := by simpa using out.property
     rw [he] at hout
     have ht : out.val.take 64=out.val := by exact List.take_of_length_le (by omega)
     have hw : ws.val.take 8=ws.val := by rw [←hlen,List.take_length]
     simpa only [show (8:Nat)*8=64 from rfl,ht,hw] using hout
 · exact hi

theorem Xoshiro512Plus_state_spec (s : xoshiro512plus.Xoshiro512Plus) :
 s.state ⦃ bytes => bytes.val=packed s.s.val ⦄ := by
 unfold xoshiro512plus.Xoshiro512Plus.state
 step*
 subst_vars
 simp only [core.slice.Slice.iter,core.iter.traits.iterator.Iterator.enumerate.trait_default,
 core.iter.traits.iterator.Iterator.enumerate.default,bind_tc_ok]
 step*
 step with loop_spec s.s.to_slice
 · refine ⟨rfl,rfl,?_,?_⟩
   · scalar_tac
   · simp [packed,Array.repeat_val]

 · simpa only [Array.val_to_slice] using out1_post

#print axioms SerializerProofs.loop_spec

theorem Xoshiro512PlusPlus_state_spec (s : xoshiro512plusplus.Xoshiro512PlusPlus) :
 s.state ⦃ bytes => bytes.val=packed s.s.val ⦄ := by
 exact Xoshiro512Plus_state_spec ⟨s.s⟩

theorem Xoshiro512StarStar_state_spec (s : xoshiro512starstar.Xoshiro512StarStar) :
 s.state ⦃ bytes => bytes.val=packed s.s.val ⦄ := by
 exact Xoshiro512Plus_state_spec ⟨s.s⟩
#print axioms SerializerProofs.Xoshiro512Plus_state_spec
#print axioms SerializerProofs.Xoshiro512PlusPlus_state_spec
#print axioms SerializerProofs.Xoshiro512StarStar_state_spec
