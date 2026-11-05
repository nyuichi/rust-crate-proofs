import ModelProofs
open Aeneas Aeneas.Std Result
open ModelProofs
namespace FromProofs
/-- Original Rust body: endian_type::{impl core::convert::From<u16> for endian_type::BigEndian<u16>}::from. -/
theorem be_from_u16_spec (x : U16) :
    endian_verification_driver.endian_type.BigEndianU16.Insts.CoreConvertFromU16.from x = .ok ((⟨byteReverse x.bv⟩ : U16)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU16.Insts.CoreConvertFromU16.from, u16_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u16> for endian_type::LittleEndian<u16>}::from. -/
theorem le_from_u16_spec (x : U16) :
    endian_verification_driver.endian_type.LittleEndianU16.Insts.CoreConvertFromU16.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU16.Insts.CoreConvertFromU16.from, u16_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u16>> for u16}::from. -/
theorem be_into_u16_spec (x : U16) :
    endian_verification_driver.U16.Insts.CoreConvertFromBigEndianU16.from x = .ok ((⟨byteReverse x.bv⟩ : U16)) := by
  simp only [endian_verification_driver.U16.Insts.CoreConvertFromBigEndianU16.from, u16_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u16>> for u16}::from. -/
theorem le_into_u16_spec (x : U16) :
    endian_verification_driver.U16.Insts.CoreConvertFromLittleEndianU16.from x = .ok (x) := by
  simp only [endian_verification_driver.U16.Insts.CoreConvertFromLittleEndianU16.from, u16_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u16>> for endian_type::BigEndian<u16>}::from. -/
theorem le_to_be_u16_spec (x : U16) :
    endian_verification_driver.endian_type.BigEndianU16.Insts.CoreConvertFromLittleEndianU16.from x = .ok ((⟨byteReverse x.bv⟩ : U16)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU16.Insts.CoreConvertFromLittleEndianU16.from, u16_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u16>> for endian_type::LittleEndian<u16>}::from. -/
theorem be_to_le_u16_spec (x : U16) :
    endian_verification_driver.endian_type.LittleEndianU16.Insts.CoreConvertFromBigEndianU16.from x = .ok ((⟨byteReverse x.bv⟩ : U16)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU16.Insts.CoreConvertFromBigEndianU16.from, u16_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u32> for endian_type::BigEndian<u32>}::from. -/
theorem be_from_u32_spec (x : U32) :
    endian_verification_driver.endian_type.BigEndianU32.Insts.CoreConvertFromU32.from x = .ok ((⟨byteReverse x.bv⟩ : U32)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU32.Insts.CoreConvertFromU32.from, u32_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u32> for endian_type::LittleEndian<u32>}::from. -/
theorem le_from_u32_spec (x : U32) :
    endian_verification_driver.endian_type.LittleEndianU32.Insts.CoreConvertFromU32.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU32.Insts.CoreConvertFromU32.from, u32_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u32>> for u32}::from. -/
theorem be_into_u32_spec (x : U32) :
    endian_verification_driver.U32.Insts.CoreConvertFromBigEndianU32.from x = .ok ((⟨byteReverse x.bv⟩ : U32)) := by
  simp only [endian_verification_driver.U32.Insts.CoreConvertFromBigEndianU32.from, u32_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u32>> for u32}::from. -/
theorem le_into_u32_spec (x : U32) :
    endian_verification_driver.U32.Insts.CoreConvertFromLittleEndianU32.from x = .ok (x) := by
  simp only [endian_verification_driver.U32.Insts.CoreConvertFromLittleEndianU32.from, u32_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u32>> for endian_type::BigEndian<u32>}::from. -/
theorem le_to_be_u32_spec (x : U32) :
    endian_verification_driver.endian_type.BigEndianU32.Insts.CoreConvertFromLittleEndianU32.from x = .ok ((⟨byteReverse x.bv⟩ : U32)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU32.Insts.CoreConvertFromLittleEndianU32.from, u32_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u32>> for endian_type::LittleEndian<u32>}::from. -/
theorem be_to_le_u32_spec (x : U32) :
    endian_verification_driver.endian_type.LittleEndianU32.Insts.CoreConvertFromBigEndianU32.from x = .ok ((⟨byteReverse x.bv⟩ : U32)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU32.Insts.CoreConvertFromBigEndianU32.from, u32_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u64> for endian_type::BigEndian<u64>}::from. -/
theorem be_from_u64_spec (x : U64) :
    endian_verification_driver.endian_type.BigEndianU64.Insts.CoreConvertFromU64.from x = .ok ((⟨byteReverse x.bv⟩ : U64)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU64.Insts.CoreConvertFromU64.from, u64_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u64> for endian_type::LittleEndian<u64>}::from. -/
theorem le_from_u64_spec (x : U64) :
    endian_verification_driver.endian_type.LittleEndianU64.Insts.CoreConvertFromU64.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU64.Insts.CoreConvertFromU64.from, u64_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u64>> for u64}::from. -/
theorem be_into_u64_spec (x : U64) :
    endian_verification_driver.U64.Insts.CoreConvertFromBigEndianU64.from x = .ok ((⟨byteReverse x.bv⟩ : U64)) := by
  simp only [endian_verification_driver.U64.Insts.CoreConvertFromBigEndianU64.from, u64_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u64>> for u64}::from. -/
theorem le_into_u64_spec (x : U64) :
    endian_verification_driver.U64.Insts.CoreConvertFromLittleEndianU64.from x = .ok (x) := by
  simp only [endian_verification_driver.U64.Insts.CoreConvertFromLittleEndianU64.from, u64_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u64>> for endian_type::BigEndian<u64>}::from. -/
theorem le_to_be_u64_spec (x : U64) :
    endian_verification_driver.endian_type.BigEndianU64.Insts.CoreConvertFromLittleEndianU64.from x = .ok ((⟨byteReverse x.bv⟩ : U64)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU64.Insts.CoreConvertFromLittleEndianU64.from, u64_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u64>> for endian_type::LittleEndian<u64>}::from. -/
theorem be_to_le_u64_spec (x : U64) :
    endian_verification_driver.endian_type.LittleEndianU64.Insts.CoreConvertFromBigEndianU64.from x = .ok ((⟨byteReverse x.bv⟩ : U64)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU64.Insts.CoreConvertFromBigEndianU64.from, u64_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u128> for endian_type::BigEndian<u128>}::from. -/
theorem be_from_u128_spec (x : U128) :
    endian_verification_driver.endian_type.BigEndianU128.Insts.CoreConvertFromU128.from x = .ok ((⟨byteReverse x.bv⟩ : U128)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU128.Insts.CoreConvertFromU128.from, u128_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<u128> for endian_type::LittleEndian<u128>}::from. -/
theorem le_from_u128_spec (x : U128) :
    endian_verification_driver.endian_type.LittleEndianU128.Insts.CoreConvertFromU128.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU128.Insts.CoreConvertFromU128.from, u128_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u128>> for u128}::from. -/
theorem be_into_u128_spec (x : U128) :
    endian_verification_driver.U128.Insts.CoreConvertFromBigEndianU128.from x = .ok ((⟨byteReverse x.bv⟩ : U128)) := by
  simp only [endian_verification_driver.U128.Insts.CoreConvertFromBigEndianU128.from, u128_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u128>> for u128}::from. -/
theorem le_into_u128_spec (x : U128) :
    endian_verification_driver.U128.Insts.CoreConvertFromLittleEndianU128.from x = .ok (x) := by
  simp only [endian_verification_driver.U128.Insts.CoreConvertFromLittleEndianU128.from, u128_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<u128>> for endian_type::BigEndian<u128>}::from. -/
theorem le_to_be_u128_spec (x : U128) :
    endian_verification_driver.endian_type.BigEndianU128.Insts.CoreConvertFromLittleEndianU128.from x = .ok ((⟨byteReverse x.bv⟩ : U128)) := by
  simp only [endian_verification_driver.endian_type.BigEndianU128.Insts.CoreConvertFromLittleEndianU128.from, u128_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<u128>> for endian_type::LittleEndian<u128>}::from. -/
theorem be_to_le_u128_spec (x : U128) :
    endian_verification_driver.endian_type.LittleEndianU128.Insts.CoreConvertFromBigEndianU128.from x = .ok ((⟨byteReverse x.bv⟩ : U128)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianU128.Insts.CoreConvertFromBigEndianU128.from, u128_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<usize> for endian_type::BigEndian<usize>}::from. -/
theorem be_from_usize_spec (x : Usize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.BigEndianUsize.Insts.CoreConvertFromUsize.from x = .ok (reverseUsize x) := by
  simp only [endian_verification_driver.endian_type.BigEndianUsize.Insts.CoreConvertFromUsize.from, usize_to_be_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<usize> for endian_type::LittleEndian<usize>}::from. -/
theorem le_from_usize_spec (x : Usize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.LittleEndianUsize.Insts.CoreConvertFromUsize.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianUsize.Insts.CoreConvertFromUsize.from, usize_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<usize>> for usize}::from. -/
theorem be_into_usize_spec (x : Usize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.Usize.Insts.CoreConvertFromBigEndianUsize.from x = .ok (reverseUsize x) := by
  simp only [endian_verification_driver.Usize.Insts.CoreConvertFromBigEndianUsize.from, usize_from_be_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<usize>> for usize}::from. -/
theorem le_into_usize_spec (x : Usize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.Usize.Insts.CoreConvertFromLittleEndianUsize.from x = .ok (x) := by
  simp only [endian_verification_driver.Usize.Insts.CoreConvertFromLittleEndianUsize.from, usize_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<usize>> for endian_type::BigEndian<usize>}::from. -/
theorem le_to_be_usize_spec (x : Usize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.BigEndianUsize.Insts.CoreConvertFromLittleEndianUsize.from x = .ok (reverseUsize x) := by
  simp only [endian_verification_driver.endian_type.BigEndianUsize.Insts.CoreConvertFromLittleEndianUsize.from, usize_swap_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<usize>> for endian_type::LittleEndian<usize>}::from. -/
theorem be_to_le_usize_spec (x : Usize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.LittleEndianUsize.Insts.CoreConvertFromBigEndianUsize.from x = .ok (reverseUsize x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianUsize.Insts.CoreConvertFromBigEndianUsize.from, usize_swap_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i16> for endian_type::BigEndian<i16>}::from. -/
theorem be_from_i16_spec (x : I16) :
    endian_verification_driver.endian_type.BigEndianI16.Insts.CoreConvertFromI16.from x = .ok ((⟨byteReverse x.bv⟩ : I16)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI16.Insts.CoreConvertFromI16.from, i16_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i16> for endian_type::LittleEndian<i16>}::from. -/
theorem le_from_i16_spec (x : I16) :
    endian_verification_driver.endian_type.LittleEndianI16.Insts.CoreConvertFromI16.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI16.Insts.CoreConvertFromI16.from, i16_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i16>> for i16}::from. -/
theorem be_into_i16_spec (x : I16) :
    endian_verification_driver.I16.Insts.CoreConvertFromBigEndianI16.from x = .ok ((⟨byteReverse x.bv⟩ : I16)) := by
  simp only [endian_verification_driver.I16.Insts.CoreConvertFromBigEndianI16.from, i16_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i16>> for i16}::from. -/
theorem le_into_i16_spec (x : I16) :
    endian_verification_driver.I16.Insts.CoreConvertFromLittleEndianI16.from x = .ok (x) := by
  simp only [endian_verification_driver.I16.Insts.CoreConvertFromLittleEndianI16.from, i16_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i16>> for endian_type::BigEndian<i16>}::from. -/
theorem le_to_be_i16_spec (x : I16) :
    endian_verification_driver.endian_type.BigEndianI16.Insts.CoreConvertFromLittleEndianI16.from x = .ok ((⟨byteReverse x.bv⟩ : I16)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI16.Insts.CoreConvertFromLittleEndianI16.from, i16_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i16>> for endian_type::LittleEndian<i16>}::from. -/
theorem be_to_le_i16_spec (x : I16) :
    endian_verification_driver.endian_type.LittleEndianI16.Insts.CoreConvertFromBigEndianI16.from x = .ok ((⟨byteReverse x.bv⟩ : I16)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI16.Insts.CoreConvertFromBigEndianI16.from, i16_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i32> for endian_type::BigEndian<i32>}::from. -/
theorem be_from_i32_spec (x : I32) :
    endian_verification_driver.endian_type.BigEndianI32.Insts.CoreConvertFromI32.from x = .ok ((⟨byteReverse x.bv⟩ : I32)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI32.Insts.CoreConvertFromI32.from, i32_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i32> for endian_type::LittleEndian<i32>}::from. -/
theorem le_from_i32_spec (x : I32) :
    endian_verification_driver.endian_type.LittleEndianI32.Insts.CoreConvertFromI32.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI32.Insts.CoreConvertFromI32.from, i32_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i32>> for i32}::from. -/
theorem be_into_i32_spec (x : I32) :
    endian_verification_driver.I32.Insts.CoreConvertFromBigEndianI32.from x = .ok ((⟨byteReverse x.bv⟩ : I32)) := by
  simp only [endian_verification_driver.I32.Insts.CoreConvertFromBigEndianI32.from, i32_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i32>> for i32}::from. -/
theorem le_into_i32_spec (x : I32) :
    endian_verification_driver.I32.Insts.CoreConvertFromLittleEndianI32.from x = .ok (x) := by
  simp only [endian_verification_driver.I32.Insts.CoreConvertFromLittleEndianI32.from, i32_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i32>> for endian_type::BigEndian<i32>}::from. -/
theorem le_to_be_i32_spec (x : I32) :
    endian_verification_driver.endian_type.BigEndianI32.Insts.CoreConvertFromLittleEndianI32.from x = .ok ((⟨byteReverse x.bv⟩ : I32)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI32.Insts.CoreConvertFromLittleEndianI32.from, i32_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i32>> for endian_type::LittleEndian<i32>}::from. -/
theorem be_to_le_i32_spec (x : I32) :
    endian_verification_driver.endian_type.LittleEndianI32.Insts.CoreConvertFromBigEndianI32.from x = .ok ((⟨byteReverse x.bv⟩ : I32)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI32.Insts.CoreConvertFromBigEndianI32.from, i32_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i64> for endian_type::BigEndian<i64>}::from. -/
theorem be_from_i64_spec (x : I64) :
    endian_verification_driver.endian_type.BigEndianI64.Insts.CoreConvertFromI64.from x = .ok ((⟨byteReverse x.bv⟩ : I64)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI64.Insts.CoreConvertFromI64.from, i64_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i64> for endian_type::LittleEndian<i64>}::from. -/
theorem le_from_i64_spec (x : I64) :
    endian_verification_driver.endian_type.LittleEndianI64.Insts.CoreConvertFromI64.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI64.Insts.CoreConvertFromI64.from, i64_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i64>> for i64}::from. -/
theorem be_into_i64_spec (x : I64) :
    endian_verification_driver.I64.Insts.CoreConvertFromBigEndianI64.from x = .ok ((⟨byteReverse x.bv⟩ : I64)) := by
  simp only [endian_verification_driver.I64.Insts.CoreConvertFromBigEndianI64.from, i64_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i64>> for i64}::from. -/
theorem le_into_i64_spec (x : I64) :
    endian_verification_driver.I64.Insts.CoreConvertFromLittleEndianI64.from x = .ok (x) := by
  simp only [endian_verification_driver.I64.Insts.CoreConvertFromLittleEndianI64.from, i64_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i64>> for endian_type::BigEndian<i64>}::from. -/
theorem le_to_be_i64_spec (x : I64) :
    endian_verification_driver.endian_type.BigEndianI64.Insts.CoreConvertFromLittleEndianI64.from x = .ok ((⟨byteReverse x.bv⟩ : I64)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI64.Insts.CoreConvertFromLittleEndianI64.from, i64_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i64>> for endian_type::LittleEndian<i64>}::from. -/
theorem be_to_le_i64_spec (x : I64) :
    endian_verification_driver.endian_type.LittleEndianI64.Insts.CoreConvertFromBigEndianI64.from x = .ok ((⟨byteReverse x.bv⟩ : I64)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI64.Insts.CoreConvertFromBigEndianI64.from, i64_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i128> for endian_type::BigEndian<i128>}::from. -/
theorem be_from_i128_spec (x : I128) :
    endian_verification_driver.endian_type.BigEndianI128.Insts.CoreConvertFromI128.from x = .ok ((⟨byteReverse x.bv⟩ : I128)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI128.Insts.CoreConvertFromI128.from, i128_to_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<i128> for endian_type::LittleEndian<i128>}::from. -/
theorem le_from_i128_spec (x : I128) :
    endian_verification_driver.endian_type.LittleEndianI128.Insts.CoreConvertFromI128.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI128.Insts.CoreConvertFromI128.from, i128_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i128>> for i128}::from. -/
theorem be_into_i128_spec (x : I128) :
    endian_verification_driver.I128.Insts.CoreConvertFromBigEndianI128.from x = .ok ((⟨byteReverse x.bv⟩ : I128)) := by
  simp only [endian_verification_driver.I128.Insts.CoreConvertFromBigEndianI128.from, i128_from_be_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i128>> for i128}::from. -/
theorem le_into_i128_spec (x : I128) :
    endian_verification_driver.I128.Insts.CoreConvertFromLittleEndianI128.from x = .ok (x) := by
  simp only [endian_verification_driver.I128.Insts.CoreConvertFromLittleEndianI128.from, i128_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<i128>> for endian_type::BigEndian<i128>}::from. -/
theorem le_to_be_i128_spec (x : I128) :
    endian_verification_driver.endian_type.BigEndianI128.Insts.CoreConvertFromLittleEndianI128.from x = .ok ((⟨byteReverse x.bv⟩ : I128)) := by
  simp only [endian_verification_driver.endian_type.BigEndianI128.Insts.CoreConvertFromLittleEndianI128.from, i128_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<i128>> for endian_type::LittleEndian<i128>}::from. -/
theorem be_to_le_i128_spec (x : I128) :
    endian_verification_driver.endian_type.LittleEndianI128.Insts.CoreConvertFromBigEndianI128.from x = .ok ((⟨byteReverse x.bv⟩ : I128)) := by
  simp only [endian_verification_driver.endian_type.LittleEndianI128.Insts.CoreConvertFromBigEndianI128.from, i128_swap_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<isize> for endian_type::BigEndian<isize>}::from. -/
theorem be_from_isize_spec (x : Isize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.BigEndianIsize.Insts.CoreConvertFromIsize.from x = .ok (reverseIsize x) := by
  simp only [endian_verification_driver.endian_type.BigEndianIsize.Insts.CoreConvertFromIsize.from, isize_to_be_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<isize> for endian_type::LittleEndian<isize>}::from. -/
theorem le_from_isize_spec (x : Isize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.LittleEndianIsize.Insts.CoreConvertFromIsize.from x = .ok (x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianIsize.Insts.CoreConvertFromIsize.from, isize_to_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<isize>> for isize}::from. -/
theorem be_into_isize_spec (x : Isize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.Isize.Insts.CoreConvertFromBigEndianIsize.from x = .ok (reverseIsize x) := by
  simp only [endian_verification_driver.Isize.Insts.CoreConvertFromBigEndianIsize.from, isize_from_be_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<isize>> for isize}::from. -/
theorem le_into_isize_spec (x : Isize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.Isize.Insts.CoreConvertFromLittleEndianIsize.from x = .ok (x) := by
  simp only [endian_verification_driver.Isize.Insts.CoreConvertFromLittleEndianIsize.from, isize_from_le_model x, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::LittleEndian<isize>> for endian_type::BigEndian<isize>}::from. -/
theorem le_to_be_isize_spec (x : Isize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.BigEndianIsize.Insts.CoreConvertFromLittleEndianIsize.from x = .ok (reverseIsize x) := by
  simp only [endian_verification_driver.endian_type.BigEndianIsize.Insts.CoreConvertFromLittleEndianIsize.from, isize_swap_model x h64, bind_ok]

/-- Original Rust body: endian_type::{impl core::convert::From<endian_type::BigEndian<isize>> for endian_type::LittleEndian<isize>}::from. -/
theorem be_to_le_isize_spec (x : Isize) (h64 : System.Platform.numBits = 64) :
    endian_verification_driver.endian_type.LittleEndianIsize.Insts.CoreConvertFromBigEndianIsize.from x = .ok (reverseIsize x) := by
  simp only [endian_verification_driver.endian_type.LittleEndianIsize.Insts.CoreConvertFromBigEndianIsize.from, isize_swap_model x h64, bind_ok]

end FromProofs
#print axioms FromProofs.be_from_u16_spec
#print axioms FromProofs.le_from_u16_spec
#print axioms FromProofs.be_into_u16_spec
#print axioms FromProofs.le_into_u16_spec
#print axioms FromProofs.le_to_be_u16_spec
#print axioms FromProofs.be_to_le_u16_spec
#print axioms FromProofs.be_from_u32_spec
#print axioms FromProofs.le_from_u32_spec
#print axioms FromProofs.be_into_u32_spec
#print axioms FromProofs.le_into_u32_spec
#print axioms FromProofs.le_to_be_u32_spec
#print axioms FromProofs.be_to_le_u32_spec
#print axioms FromProofs.be_from_u64_spec
#print axioms FromProofs.le_from_u64_spec
#print axioms FromProofs.be_into_u64_spec
#print axioms FromProofs.le_into_u64_spec
#print axioms FromProofs.le_to_be_u64_spec
#print axioms FromProofs.be_to_le_u64_spec
#print axioms FromProofs.be_from_u128_spec
#print axioms FromProofs.le_from_u128_spec
#print axioms FromProofs.be_into_u128_spec
#print axioms FromProofs.le_into_u128_spec
#print axioms FromProofs.le_to_be_u128_spec
#print axioms FromProofs.be_to_le_u128_spec
#print axioms FromProofs.be_from_usize_spec
#print axioms FromProofs.le_from_usize_spec
#print axioms FromProofs.be_into_usize_spec
#print axioms FromProofs.le_into_usize_spec
#print axioms FromProofs.le_to_be_usize_spec
#print axioms FromProofs.be_to_le_usize_spec
#print axioms FromProofs.be_from_i16_spec
#print axioms FromProofs.le_from_i16_spec
#print axioms FromProofs.be_into_i16_spec
#print axioms FromProofs.le_into_i16_spec
#print axioms FromProofs.le_to_be_i16_spec
#print axioms FromProofs.be_to_le_i16_spec
#print axioms FromProofs.be_from_i32_spec
#print axioms FromProofs.le_from_i32_spec
#print axioms FromProofs.be_into_i32_spec
#print axioms FromProofs.le_into_i32_spec
#print axioms FromProofs.le_to_be_i32_spec
#print axioms FromProofs.be_to_le_i32_spec
#print axioms FromProofs.be_from_i64_spec
#print axioms FromProofs.le_from_i64_spec
#print axioms FromProofs.be_into_i64_spec
#print axioms FromProofs.le_into_i64_spec
#print axioms FromProofs.le_to_be_i64_spec
#print axioms FromProofs.be_to_le_i64_spec
#print axioms FromProofs.be_from_i128_spec
#print axioms FromProofs.le_from_i128_spec
#print axioms FromProofs.be_into_i128_spec
#print axioms FromProofs.le_into_i128_spec
#print axioms FromProofs.le_to_be_i128_spec
#print axioms FromProofs.be_to_le_i128_spec
#print axioms FromProofs.be_from_isize_spec
#print axioms FromProofs.le_from_isize_spec
#print axioms FromProofs.be_into_isize_spec
#print axioms FromProofs.le_into_isize_spec
#print axioms FromProofs.le_to_be_isize_spec
#print axioms FromProofs.be_to_le_isize_spec
