import ConditionalProofs
import ReflexiveProofs
import FromProofs
import Generated.EndianType
open Aeneas Aeneas.Std Result
open endian_type
namespace Proofs
theorem bigendian_bitand_spec {T U : Type} (inst : core.ops.bit.BitAnd T T U) (x y : T) :
    BigEndian.Insts.CoreOpsBitBitAndBigEndianBigEndian.bitand inst x y = inst.bitand x y := by
  simp [BigEndian.Insts.CoreOpsBitBitAndBigEndianBigEndian.bitand]
theorem bigendian_bitor_spec {T U : Type} (inst : endian_type.core.ops.bit.BitOr T T U) (x y : T) :
    BigEndian.Insts.CoreOpsBitBitOrBigEndianBigEndian.bitor inst x y = inst.bitor x y := by
  simp [BigEndian.Insts.CoreOpsBitBitOrBigEndianBigEndian.bitor]
theorem bigendian_bitxor_spec {T U : Type} (inst : endian_type.core.ops.bit.BitXor T T U) (x y : T) :
    BigEndian.Insts.CoreOpsBitBitXorBigEndianBigEndian.bitxor inst x y = inst.bitxor x y := by
  simp [BigEndian.Insts.CoreOpsBitBitXorBigEndianBigEndian.bitxor]
theorem littleendian_bitand_spec {T U : Type} (inst : core.ops.bit.BitAnd T T U) (x y : T) :
    LittleEndian.Insts.CoreOpsBitBitAndLittleEndianLittleEndian.bitand inst x y = inst.bitand x y := by
  simp [LittleEndian.Insts.CoreOpsBitBitAndLittleEndianLittleEndian.bitand]
theorem littleendian_bitor_spec {T U : Type} (inst : endian_type.core.ops.bit.BitOr T T U) (x y : T) :
    LittleEndian.Insts.CoreOpsBitBitOrLittleEndianLittleEndian.bitor inst x y = inst.bitor x y := by
  simp [LittleEndian.Insts.CoreOpsBitBitOrLittleEndianLittleEndian.bitor]
theorem littleendian_bitxor_spec {T U : Type} (inst : endian_type.core.ops.bit.BitXor T T U) (x y : T) :
    LittleEndian.Insts.CoreOpsBitBitXorLittleEndianLittleEndian.bitxor inst x y = inst.bitxor x y := by
  simp [LittleEndian.Insts.CoreOpsBitBitXorLittleEndianLittleEndian.bitxor]
end Proofs
#print axioms Proofs.bigendian_bitand_spec
#print axioms Proofs.bigendian_bitor_spec
#print axioms Proofs.bigendian_bitxor_spec
#print axioms Proofs.littleendian_bitand_spec
#print axioms Proofs.littleendian_bitor_spec
#print axioms Proofs.littleendian_bitxor_spec
