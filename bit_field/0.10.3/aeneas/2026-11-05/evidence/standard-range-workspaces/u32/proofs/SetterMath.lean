import GetterNormal
open Aeneas Aeneas.Std
namespace U32RangeSetterMath

def window (start finish : Nat) : BitVec 32 :=
 (((((~~~(0#32)) <<< (32-finish)) >>> (32-finish)) >>> start) <<< start)

theorem fit_low_word (word : BitVec 32) (width : Nat) (bound : width<=32) (fit : word.toNat<2^width) :
 ((word <<< (32-width)) >>> (32-width))=word := by
 apply BitVec.eq_of_getLsbD_eq
 intro i hi
 have h := U32RangeGetter.clipped_bits word 0 width i bound (by omega)
 simp only [Nat.sub_zero,Nat.zero_add,BitVec.ushiftRight_zero] at h
 rw [h]
 have hm : word.toNat%2^width=word.toNat := Nat.mod_eq_of_lt fit
 have hb := congrArg (fun n : Nat => n.testBit i) hm
 simpa only [Nat.testBit_mod_two_pow,BitVec.testBit_toNat] using hb

theorem fit_bit_false (word : BitVec 32) (width j : Nat) (fit : word.toNat<2^width) (hj : width<=j) :
 word.getLsbD j=false := by
 have hm : word.toNat%2^width=word.toNat := Nat.mod_eq_of_lt fit
 have hb := congrArg (fun n : Nat => n.testBit j) hm
 have hn : ¬j<width := by omega
 have hb' : false=word.getLsbD j := by
  simpa only [Nat.testBit_mod_two_pow,hn,decide_false,Bool.false_and,BitVec.testBit_toNat] using hb
 exact hb'.symm

theorem window_bits (start finish i : Nat) (he : finish<=32) (ho : start<=finish) (hi : i<32) :
 (window start finish).getLsbD i=decide (start<=i ∧ i<finish) := by
 unfold window
 rw [BitVec.getLsbD_shiftLeft,U32RangeGetter.clipped_bits _ start finish (i-start) he ho]
 simp only [BitVec.getLsbD_not,BitVec.getLsbD_zero,Bool.not_false,Bool.and_true]
 by_cases lower : start<=i
 · have hn : ¬i<start := by omega
   have hs : start+(i-start)=i := by omega
   have hc : (i-start<finish-start) ↔ i<finish := by omega
   simp [hi,hn,hs,hc,lower]
 · have hl : i<start := by omega
   simp [hi,hl,lower]

theorem update_bits (word replacement : BitVec 32) (start finish i : Nat)
 (he : finish<=32) (ho : start<=finish) (fit : replacement.toNat<2^(finish-start)) (hi : i<32) :
 ((word &&& ~~~window start finish) ||| (replacement <<< start)).getLsbD i =
 if start<=i ∧ i<finish then replacement.getLsbD (i-start) else word.getLsbD i := by
 rw [BitVec.getLsbD_or,BitVec.getLsbD_and,BitVec.getLsbD_not,window_bits start finish i he ho hi,BitVec.getLsbD_shiftLeft]
 by_cases inside : start<=i ∧ i<finish
 · have hn : ¬i<start := by omega
   simp [inside,hi,hn]
 · by_cases lower : start<=i
   · have hn : ¬i<start := by omega
     have hb := fit_bit_false replacement (finish-start) (i-start) fit (by omega)
     simp [inside,hi,hn,hb]
   · have hl : i<start := by omega
     simp [inside,hi,hl]

#print axioms fit_low_word
#print axioms fit_bit_false
#print axioms window_bits
#print axioms update_bits
end U32RangeSetterMath
