#![feature(prelude_import)]
#![no_std]
extern crate core;
#[prelude_import]
use core::prelude::rust_2021::*;

use core::ops::{BitAnd, BitOr, BitXor};
use core::{mem, slice};

///Type with a specified byte order.
pub trait Endian<T> { }

macro_rules! impl_Endian {
    (for $e:ident) =>
    {
        impl<T> BitAnd for $e<T> where T: BitAnd,
        {
            type Output = $e<<T as BitAnd>::Output>; #[inline] fn
            bitand(self, other: Self) -> Self::Output
            {
                $e(self.0 & other.0)

            }
        } impl<T> BitOr for $e<T> where T: BitOr,
        {
            type Output = $e<<T as BitOr>::Output>; #[inline] fn
            bitor(self, other: Self) -> Self::Output
            {
                $e(self.0 | other.0)


            }
        } impl<T> BitXor for $e<T> where T: BitXor,
        {
            type Output = $e<<T as BitXor>::Output>; #[inline] fn
            bitxor(self, other: Self) -> Self::Output
            {
                $e(self.0 ^ other.0)


            }
        } impl<T> $e<T> where T: Sized + Copy,
        {
            #[inline] pub const unsafe fn from_byte_slice(bytes: &[u8]) ->
            $e<T>
            {
                debug_assert!(bytes.len() >= mem::size_of::<T>());
                $e({
                    *(bytes.as_ptr() as *const T)

                })
            } #[inline] pub fn as_byte_slice(&self) -> &[u8]
            {
                unsafe
                {
                    slice::from_raw_parts(&self.0 as *const T as *const u8,
                    mem::size_of::<T>())

                }
            } #[inline] pub fn as_byte_slice_mut(&mut self) -> &mut [u8]
            {
                unsafe
                {
                    slice::from_raw_parts_mut(&mut self.0 as *mut T as *mut u8,
                    mem::size_of::<T>())

                }
            }
        }
    }
}

///Big endian byte order.
///
///Most significant byte first.
#[repr(transparent)]
pub struct BigEndian<T>(T);
#[automatically_derived]
impl<T: ::core::marker::Copy> ::core::marker::Copy for BigEndian<T> { }
#[automatically_derived]
impl<T: ::core::clone::Clone> ::core::clone::Clone for BigEndian<T> {
    #[inline]
    fn clone(&self) -> BigEndian<T> {
        BigEndian(::core::clone::Clone::clone(&self.0))
    }
}
#[automatically_derived]
impl<T: ::core::fmt::Debug> ::core::fmt::Debug for BigEndian<T> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "BigEndian",
            &&self.0)
    }
}
#[automatically_derived]
impl<T: ::core::cmp::Eq> ::core::cmp::Eq for BigEndian<T> {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) { let _: ::core::cmp::AssertParamIsEq<T>; }
}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq> ::core::marker::StructuralPartialEq for
    BigEndian<T> {
}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq> ::core::cmp::PartialEq for BigEndian<T> {
    #[inline]
    fn eq(&self, other: &BigEndian<T>) -> bool { self.0 == other.0 }
}
#[automatically_derived]
impl<T: ::core::hash::Hash> ::core::hash::Hash for BigEndian<T> {
    #[inline]
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
        ::core::hash::Hash::hash(&self.0, state)
    }
}
#[automatically_derived]
impl<T: ::core::cmp::Ord> ::core::cmp::Ord for BigEndian<T> {
    #[inline]
    fn cmp(&self, other: &BigEndian<T>) -> ::core::cmp::Ordering {
        ::core::cmp::Ord::cmp(&self.0, &other.0)
    }
}
#[automatically_derived]
impl<T: ::core::cmp::PartialOrd> ::core::cmp::PartialOrd for BigEndian<T> {
    #[inline]
    fn partial_cmp(&self, other: &BigEndian<T>)
        -> ::core::option::Option<::core::cmp::Ordering> {
        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
    }
}

impl<T> Endian<T> for BigEndian<T> {}

macro_rules! impl_for_BigEndian {
    ($t:ident) =>
    {
        impl From<BigEndian<$t>> for $t
        {
            #[inline] fn from(data: BigEndian<$t>) -> $t
            { $t::from_be(data.0) }
        } impl From<$t> for BigEndian<$t>
        {
            #[inline] fn from(data: $t) -> Self
            {
                BigEndian(data.to_be())

            }
        } impl From<LittleEndian<$t>> for BigEndian<$t>
        {
            #[inline] fn from(data: LittleEndian<$t>) -> Self
            {
                BigEndian(data.0.swap_bytes())

            }
        } impl
        BigEndian<$t>{
            ///Return the memory representation of this type as a byte array in its endian byte order.
            ///Note: This is just a transmute.
            #[inline] pub const fn to_bytes(self) ->
            [u8; mem::size_of::<BigEndian<$t>>()]
            {
                unsafe
                {
                    mem::transmute(self)

                    //TODO: Move these to Endian when https://github.com/rust-lang/rust/issues/67792 stabilized in a few years
                }
            }
            ///Construct a value from its memory representation as a byte array.
            ///Note: This is just a transmute.
            #[inline] pub const fn
            from_bytes(bytes: [u8; mem::size_of::<BigEndian<$t>>()]) ->
            BigEndian<$t>
            {
                unsafe
                {
                    mem::transmute(bytes)

                }
            }
        }
    };
}
impl<T> BitAnd for BigEndian<T> where T: BitAnd {
    type Output = BigEndian<<T as BitAnd>::Output>;
    #[inline]
    fn bitand(self, other: Self) -> Self::Output {
        BigEndian(self.0 & other.0)
    }
}
impl<T> BitOr for BigEndian<T> where T: BitOr {
    type Output = BigEndian<<T as BitOr>::Output>;
    #[inline]
    fn bitor(self, other: Self) -> Self::Output {
        BigEndian(self.0 | other.0)
    }
}
impl<T> BitXor for BigEndian<T> where T: BitXor {
    type Output = BigEndian<<T as BitXor>::Output>;
    #[inline]
    fn bitxor(self, other: Self) -> Self::Output {
        BigEndian(self.0 ^ other.0)
    }
}
impl<T> BigEndian<T> where T: Sized + Copy {
    #[inline]
    pub const unsafe fn from_byte_slice(bytes: &[u8]) -> BigEndian<T> {


































        if true {
            if !(bytes.len() >= mem::size_of::<T>()) {
                ::core::panicking::panic("assertion failed: bytes.len() >= mem::size_of::<T>()")
            };
        };
        BigEndian({ *(bytes.as_ptr() as *const T) })
    }
    #[inline]
    pub fn as_byte_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(&self.0 as *const T as *const u8,
                mem::size_of::<T>())
        }
    }
    #[inline]
    pub fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(&mut self.0 as *mut T as *mut u8,
                mem::size_of::<T>())
        }
    }
}
impl From<BigEndian<u16>> for u16 {
    #[inline]
    fn from(data: BigEndian<u16>) -> u16 { u16::from_be(data.0) }
}
impl From<u16> for BigEndian<u16> {
    #[inline]
    fn from(data: u16) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<u16>> for BigEndian<u16> {
    #[inline]
    fn from(data: LittleEndian<u16>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<u16> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<u16>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<u16>>()])
        -> BigEndian<u16> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<u32>> for u32 {
    #[inline]
    fn from(data: BigEndian<u32>) -> u32 { u32::from_be(data.0) }
}
impl From<u32> for BigEndian<u32> {
    #[inline]
    fn from(data: u32) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<u32>> for BigEndian<u32> {
    #[inline]
    fn from(data: LittleEndian<u32>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<u32> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<u32>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<u32>>()])
        -> BigEndian<u32> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<u64>> for u64 {
    #[inline]
    fn from(data: BigEndian<u64>) -> u64 { u64::from_be(data.0) }
}
impl From<u64> for BigEndian<u64> {
    #[inline]
    fn from(data: u64) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<u64>> for BigEndian<u64> {
    #[inline]
    fn from(data: LittleEndian<u64>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<u64> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<u64>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<u64>>()])
        -> BigEndian<u64> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<u128>> for u128 {
    #[inline]
    fn from(data: BigEndian<u128>) -> u128 { u128::from_be(data.0) }
}
impl From<u128> for BigEndian<u128> {
    #[inline]
    fn from(data: u128) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<u128>> for BigEndian<u128> {
    #[inline]
    fn from(data: LittleEndian<u128>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<u128> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<u128>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<u128>>()])
        -> BigEndian<u128> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<usize>> for usize {
    #[inline]
    fn from(data: BigEndian<usize>) -> usize { usize::from_be(data.0) }
}
impl From<usize> for BigEndian<usize> {
    #[inline]
    fn from(data: usize) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<usize>> for BigEndian<usize> {
    #[inline]
    fn from(data: LittleEndian<usize>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<usize> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<usize>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<usize>>()])
        -> BigEndian<usize> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<i16>> for i16 {
    #[inline]
    fn from(data: BigEndian<i16>) -> i16 { i16::from_be(data.0) }
}
impl From<i16> for BigEndian<i16> {
    #[inline]
    fn from(data: i16) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<i16>> for BigEndian<i16> {
    #[inline]
    fn from(data: LittleEndian<i16>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<i16> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<i16>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<i16>>()])
        -> BigEndian<i16> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<i32>> for i32 {
    #[inline]
    fn from(data: BigEndian<i32>) -> i32 { i32::from_be(data.0) }
}
impl From<i32> for BigEndian<i32> {
    #[inline]
    fn from(data: i32) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<i32>> for BigEndian<i32> {
    #[inline]
    fn from(data: LittleEndian<i32>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<i32> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<i32>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<i32>>()])
        -> BigEndian<i32> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<i64>> for i64 {
    #[inline]
    fn from(data: BigEndian<i64>) -> i64 { i64::from_be(data.0) }
}
impl From<i64> for BigEndian<i64> {
    #[inline]
    fn from(data: i64) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<i64>> for BigEndian<i64> {
    #[inline]
    fn from(data: LittleEndian<i64>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<i64> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<i64>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<i64>>()])
        -> BigEndian<i64> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<i128>> for i128 {
    #[inline]
    fn from(data: BigEndian<i128>) -> i128 { i128::from_be(data.0) }
}
impl From<i128> for BigEndian<i128> {
    #[inline]
    fn from(data: i128) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<i128>> for BigEndian<i128> {
    #[inline]
    fn from(data: LittleEndian<i128>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<i128> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<i128>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<i128>>()])
        -> BigEndian<i128> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<BigEndian<isize>> for isize {
    #[inline]
    fn from(data: BigEndian<isize>) -> isize { isize::from_be(data.0) }
}
impl From<isize> for BigEndian<isize> {
    #[inline]
    fn from(data: isize) -> Self { BigEndian(data.to_be()) }
}
impl From<LittleEndian<isize>> for BigEndian<isize> {
    #[inline]
    fn from(data: LittleEndian<isize>) -> Self {
        BigEndian(data.0.swap_bytes())
    }
}
impl BigEndian<isize> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<BigEndian<isize>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<BigEndian<isize>>()])
        -> BigEndian<isize> {
        unsafe { mem::transmute(bytes) }
    }
}
///Little endian byte order.
///
///Least significant byte first.
#[repr(transparent)]
pub struct LittleEndian<T>(T);
#[automatically_derived]
impl<T: ::core::marker::Copy> ::core::marker::Copy for LittleEndian<T> { }
#[automatically_derived]
impl<T: ::core::clone::Clone> ::core::clone::Clone for LittleEndian<T> {
    #[inline]
    fn clone(&self) -> LittleEndian<T> {
        LittleEndian(::core::clone::Clone::clone(&self.0))
    }
}
#[automatically_derived]
impl<T: ::core::fmt::Debug> ::core::fmt::Debug for LittleEndian<T> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "LittleEndian",
            &&self.0)
    }
}
#[automatically_derived]
impl<T: ::core::cmp::Eq> ::core::cmp::Eq for LittleEndian<T> {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) { let _: ::core::cmp::AssertParamIsEq<T>; }
}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq> ::core::marker::StructuralPartialEq for
    LittleEndian<T> {
}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq> ::core::cmp::PartialEq for LittleEndian<T> {
    #[inline]
    fn eq(&self, other: &LittleEndian<T>) -> bool { self.0 == other.0 }
}
#[automatically_derived]
impl<T: ::core::hash::Hash> ::core::hash::Hash for LittleEndian<T> {
    #[inline]
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
        ::core::hash::Hash::hash(&self.0, state)
    }
}
#[automatically_derived]
impl<T: ::core::cmp::Ord> ::core::cmp::Ord for LittleEndian<T> {
    #[inline]
    fn cmp(&self, other: &LittleEndian<T>) -> ::core::cmp::Ordering {
        ::core::cmp::Ord::cmp(&self.0, &other.0)
    }
}
#[automatically_derived]
impl<T: ::core::cmp::PartialOrd> ::core::cmp::PartialOrd for LittleEndian<T> {
    #[inline]
    fn partial_cmp(&self, other: &LittleEndian<T>)
        -> ::core::option::Option<::core::cmp::Ordering> {
        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
    }
}
impl<T> Endian<T> for LittleEndian<T> {}
macro_rules! impl_for_LittleEndian {
    ($t:ident) =>
    {
        impl From<LittleEndian<$t>> for $t
        {
            #[inline] fn from(data: LittleEndian<$t>) -> $t
            { $t::from_le(data.0) }
        } impl From<$t> for LittleEndian<$t>
        { #[inline] fn from(data: $t) -> Self { LittleEndian(data.to_le()) } }
        impl From<BigEndian<$t>> for LittleEndian<$t>
        {
            #[inline] fn from(data: BigEndian<$t>) -> Self
            { LittleEndian(data.0.swap_bytes()) }
        } impl
        LittleEndian<$t>{
            ///Return the memory representation of this type as a byte array in its endian byte order.
            ///Note: This is just a transmute.
            #[inline] pub const fn to_bytes(self) ->
            [u8; mem::size_of::<LittleEndian<$t>>()]
            { unsafe { mem::transmute(self) } }
            ///Construct a value from its memory representation as a byte array.
            ///Note: This is just a transmute.
            #[inline] pub const fn
            from_bytes(bytes: [u8; mem::size_of::<LittleEndian<$t>>()]) ->
            LittleEndian<$t> { unsafe { mem::transmute(bytes) } }
        }
    };
}
impl<T> BitAnd for LittleEndian<T> where T: BitAnd {
    type Output = LittleEndian<<T as BitAnd>::Output>;
    #[inline]
    fn bitand(self, other: Self) -> Self::Output {
        LittleEndian(self.0 & other.0)
    }
}
impl<T> BitOr for LittleEndian<T> where T: BitOr {
    type Output = LittleEndian<<T as BitOr>::Output>;
    #[inline]
    fn bitor(self, other: Self) -> Self::Output {
        LittleEndian(self.0 | other.0)
    }
}
impl<T> BitXor for LittleEndian<T> where T: BitXor {
    type Output = LittleEndian<<T as BitXor>::Output>;
    #[inline]
    fn bitxor(self, other: Self) -> Self::Output {
        LittleEndian(self.0 ^ other.0)
    }
}
impl<T> LittleEndian<T> where T: Sized + Copy {
    #[inline]
    pub const unsafe fn from_byte_slice(bytes: &[u8]) -> LittleEndian<T> {
        if true {
            if !(bytes.len() >= mem::size_of::<T>()) {
                ::core::panicking::panic("assertion failed: bytes.len() >= mem::size_of::<T>()")
            };
        };
        LittleEndian({ *(bytes.as_ptr() as *const T) })
    }
    #[inline]
    pub fn as_byte_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(&self.0 as *const T as *const u8,
                mem::size_of::<T>())
        }
    }
    #[inline]
    pub fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(&mut self.0 as *mut T as *mut u8,
                mem::size_of::<T>())
        }
    }
}
impl From<LittleEndian<u16>> for u16 {
    #[inline]
    fn from(data: LittleEndian<u16>) -> u16 { u16::from_le(data.0) }
}
impl From<u16> for LittleEndian<u16> {
    #[inline]
    fn from(data: u16) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<u16>> for LittleEndian<u16> {
    #[inline]
    fn from(data: BigEndian<u16>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<u16> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<LittleEndian<u16>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<u16>>()])
        -> LittleEndian<u16> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<u32>> for u32 {
    #[inline]
    fn from(data: LittleEndian<u32>) -> u32 { u32::from_le(data.0) }
}
impl From<u32> for LittleEndian<u32> {
    #[inline]
    fn from(data: u32) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<u32>> for LittleEndian<u32> {
    #[inline]
    fn from(data: BigEndian<u32>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<u32> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<LittleEndian<u32>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<u32>>()])
        -> LittleEndian<u32> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<u64>> for u64 {
    #[inline]
    fn from(data: LittleEndian<u64>) -> u64 { u64::from_le(data.0) }
}
impl From<u64> for LittleEndian<u64> {
    #[inline]
    fn from(data: u64) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<u64>> for LittleEndian<u64> {
    #[inline]
    fn from(data: BigEndian<u64>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<u64> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<LittleEndian<u64>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<u64>>()])
        -> LittleEndian<u64> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<u128>> for u128 {
    #[inline]
    fn from(data: LittleEndian<u128>) -> u128 { u128::from_le(data.0) }
}
impl From<u128> for LittleEndian<u128> {
    #[inline]
    fn from(data: u128) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<u128>> for LittleEndian<u128> {
    #[inline]
    fn from(data: BigEndian<u128>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<u128> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self)
        -> [u8; mem::size_of::<LittleEndian<u128>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<u128>>()])
        -> LittleEndian<u128> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<usize>> for usize {
    #[inline]
    fn from(data: LittleEndian<usize>) -> usize { usize::from_le(data.0) }
}
impl From<usize> for LittleEndian<usize> {
    #[inline]
    fn from(data: usize) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<usize>> for LittleEndian<usize> {
    #[inline]
    fn from(data: BigEndian<usize>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<usize> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self)
        -> [u8; mem::size_of::<LittleEndian<usize>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes:
            [u8; mem::size_of::<LittleEndian<usize>>()])
        -> LittleEndian<usize> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<i16>> for i16 {
    #[inline]
    fn from(data: LittleEndian<i16>) -> i16 { i16::from_le(data.0) }
}
impl From<i16> for LittleEndian<i16> {
    #[inline]
    fn from(data: i16) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<i16>> for LittleEndian<i16> {
    #[inline]
    fn from(data: BigEndian<i16>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<i16> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<LittleEndian<i16>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<i16>>()])
        -> LittleEndian<i16> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<i32>> for i32 {
    #[inline]
    fn from(data: LittleEndian<i32>) -> i32 { i32::from_le(data.0) }
}
impl From<i32> for LittleEndian<i32> {
    #[inline]
    fn from(data: i32) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<i32>> for LittleEndian<i32> {
    #[inline]
    fn from(data: BigEndian<i32>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<i32> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<LittleEndian<i32>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<i32>>()])
        -> LittleEndian<i32> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<i64>> for i64 {
    #[inline]
    fn from(data: LittleEndian<i64>) -> i64 { i64::from_le(data.0) }
}
impl From<i64> for LittleEndian<i64> {
    #[inline]
    fn from(data: i64) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<i64>> for LittleEndian<i64> {
    #[inline]
    fn from(data: BigEndian<i64>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<i64> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self) -> [u8; mem::size_of::<LittleEndian<i64>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<i64>>()])
        -> LittleEndian<i64> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<i128>> for i128 {
    #[inline]
    fn from(data: LittleEndian<i128>) -> i128 { i128::from_le(data.0) }
}
impl From<i128> for LittleEndian<i128> {
    #[inline]
    fn from(data: i128) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<i128>> for LittleEndian<i128> {
    #[inline]
    fn from(data: BigEndian<i128>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<i128> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self)
        -> [u8; mem::size_of::<LittleEndian<i128>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes: [u8; mem::size_of::<LittleEndian<i128>>()])
        -> LittleEndian<i128> {
        unsafe { mem::transmute(bytes) }
    }
}
impl From<LittleEndian<isize>> for isize {
    #[inline]
    fn from(data: LittleEndian<isize>) -> isize { isize::from_le(data.0) }
}
impl From<isize> for LittleEndian<isize> {
    #[inline]
    fn from(data: isize) -> Self { LittleEndian(data.to_le()) }
}
impl From<BigEndian<isize>> for LittleEndian<isize> {
    #[inline]
    fn from(data: BigEndian<isize>) -> Self {
        LittleEndian(data.0.swap_bytes())
    }
}
impl LittleEndian<isize> {
    ///Return the memory representation of this type as a byte array in its endian byte order.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn to_bytes(self)
        -> [u8; mem::size_of::<LittleEndian<isize>>()] {
        unsafe { mem::transmute(self) }
    }
    ///Construct a value from its memory representation as a byte array.
    ///Note: This is just a transmute.
    #[inline]
    pub const fn from_bytes(bytes:
            [u8; mem::size_of::<LittleEndian<isize>>()])
        -> LittleEndian<isize> {
        unsafe { mem::transmute(bytes) }
    }
}
///Network byte order as defined by IETF RFC1700 <http://tools.ietf.org/html/rfc1700>.
pub type NetworkOrder<T> = BigEndian<T>;
///Type aliases for primitive types.
pub mod types {
    #![allow(non_camel_case_types)]
    use super::*;
    pub type i16_be = BigEndian<i16>;
    pub type i32_be = BigEndian<i32>;
    pub type i64_be = BigEndian<i64>;
    pub type i128_be = BigEndian<i128>;
    pub type isize_be = BigEndian<isize>;
    pub type u16_be = BigEndian<u16>;
    pub type u32_be = BigEndian<u32>;
    pub type u64_be = BigEndian<u64>;
    pub type u128_be = BigEndian<u128>;
    pub type usize_be = BigEndian<usize>;
    pub type i16_le = LittleEndian<i16>;
    pub type i32_le = LittleEndian<i32>;
    pub type i64_le = LittleEndian<i64>;
    pub type i128_le = LittleEndian<i128>;
    pub type isize_le = LittleEndian<isize>;
    pub type u16_le = LittleEndian<u16>;
    pub type u32_le = LittleEndian<u32>;
    pub type u64_le = LittleEndian<u64>;
    pub type u128_le = LittleEndian<u128>;
    pub type usize_le = LittleEndian<usize>;
    pub type i16_net = NetworkOrder<i16>;
    pub type i32_net = NetworkOrder<i32>;
    pub type i128_net = NetworkOrder<i128>;
    pub type isize_net = NetworkOrder<isize>;
    pub type u16_net = NetworkOrder<u16>;
    pub type u32_net = NetworkOrder<u32>;
    pub type u64_net = NetworkOrder<u64>;
    pub type u128_net = NetworkOrder<u128>;
    pub type usize_net = NetworkOrder<usize>;
}
