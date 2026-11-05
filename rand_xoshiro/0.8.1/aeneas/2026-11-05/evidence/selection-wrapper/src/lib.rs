#![no_std]
pub fn xoroshiro128plus_next32(s: &mut rand_xoshiro::Xoroshiro128Plus) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoroshiro128Plus as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoroshiro128plusplus_next32(s: &mut rand_xoshiro::Xoroshiro128PlusPlus) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoroshiro128PlusPlus as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoroshiro128starstar_next32(s: &mut rand_xoshiro::Xoroshiro128StarStar) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoroshiro128StarStar as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoshiro256plus_next32(s: &mut rand_xoshiro::Xoshiro256Plus) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoshiro256Plus as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoshiro256plusplus_next32(s: &mut rand_xoshiro::Xoshiro256PlusPlus) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoshiro256PlusPlus as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoshiro256starstar_next32(s: &mut rand_xoshiro::Xoshiro256StarStar) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoshiro256StarStar as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoshiro512plus_next32(s: &mut rand_xoshiro::Xoshiro512Plus) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoshiro512Plus as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoshiro512plusplus_next32(s: &mut rand_xoshiro::Xoshiro512PlusPlus) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoshiro512PlusPlus as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
pub fn xoshiro512starstar_next32(s: &mut rand_xoshiro::Xoshiro512StarStar) -> Result<u32,core::convert::Infallible> { <rand_xoshiro::Xoshiro512StarStar as rand_xoshiro::rand_core::TryRng>::try_next_u32(s) }
