#[cfg(not(creusot))]
#[path = "u128_ext.rs"]
mod u128_ext;

use crate::divmod100::divmod100;
use crate::decimal_pairs::DECIMAL_PAIRS;
#[cfg(creusot)]
use crate::verification::{
    decimal_seq_concat_assoc, decimal_values, decimal_values_compose_1x2, logical_slot_bytes,
    concat_two_get_digits, logical_slot_bytes_split,
    decimal_values_len_at_least_one, decimal_values_one_digit,
    decimal_values_len_ge_2, decimal_values_len_ge_4, decimal_values_len_u16,
    decimal_values_len_u32,
    decimal_values_split_4, masked_decimal_digit,
    fixed_width_decimal_values, fixed_width_decimal_values_2_is_decimal,
};
#[cfg(creusot)]
use crate::decimal_pairs::decimal_pair_correct;
#[cfg(creusot)]
use crate::verification::{fixed_width_decimal_values_compose_2x2, fixed_width_decimal_values_pair};
#[cfg(creusot)]
use creusot_std::prelude::{check, ensures, invariant, proof_assert, requires, snapshot, Int, Seq};
#[cfg(creusot)]
use creusot_std::std::option::OptionExt;
#[cfg(not(creusot))]
use core::hint;
use core::mem::{self, MaybeUninit};
#[cfg(not(creusot))]
use core::str;
#[cfg(feature = "no-panic")]
use no_panic::no_panic;

/// A correctly sized stack allocation for the formatted integer to be written
/// into.
///
/// # Example
///
/// ```
/// let mut buffer = itoa::Buffer::new();
/// let printed = buffer.format(1234);
/// assert_eq!(printed, "1234");
/// ```
pub struct Buffer {
    bytes: [MaybeUninit<u8>; i128::MAX_STR_LEN],
}

impl Default for Buffer {
    #[inline]
    fn default() -> Buffer {
        Buffer::new()
    }
}

impl Copy for Buffer {}

#[allow(clippy::non_canonical_clone_impl)]
impl Clone for Buffer {
    #[inline]
    fn clone(&self) -> Self {
        Buffer::new()
    }
}

impl Buffer {
    /// This is a cheap operation; you don't need to worry about reusing buffers
    /// for efficiency.
    #[inline]
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub fn new() -> Buffer {
        let bytes = [MaybeUninit::<u8>::uninit(); i128::MAX_STR_LEN];
        Buffer { bytes }
    }

    /// Print an integer into this buffer and return a reference to its string
    /// representation within the buffer.
    #[cfg_attr(feature = "no-panic", no_panic)]
    #[cfg(not(creusot))]
    pub fn format<I: Integer>(&mut self, i: I) -> &str {
        let buf_ptr = self.bytes.as_mut_ptr().cast::<I::Buffer>();
        let string = i.write(unsafe { &mut *buf_ptr });
        if string.len() > I::MAX_STR_LEN {
            unsafe { hint::unreachable_unchecked() };
        }
        string
    }
}

/// An integer that can be written into an [`itoa::Buffer`][Buffer].
///
/// This trait is sealed and cannot be implemented for types outside of itoa.
pub trait Integer: private::Sealed {
    /// The maximum length of string that formatting an integer of this type can
    /// produce on the current target platform.
    const MAX_STR_LEN: usize;
}

// Seal to prevent downstream implementations of the Integer trait.
mod private {
    #[doc(hidden)]
    pub trait Sealed: Copy {
        #[doc(hidden)]
        type Buffer: 'static;
        #[cfg(not(creusot))]
        fn write(self, buf: &mut Self::Buffer) -> &str;
    }
}

macro_rules! impl_Integer {
    ($Signed:ident, $Unsigned:ident) => {
        const _: () = {
            assert!($Signed::MIN < 0, "need signed");
            assert!($Unsigned::MIN == 0, "need unsigned");
            assert!($Signed::BITS == $Unsigned::BITS, "need counterparts");
        };

        impl Integer for $Unsigned {
            const MAX_STR_LEN: usize = $Unsigned::MAX.ilog10() as usize + 1;
        }

        impl private::Sealed for $Unsigned {
            type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];

            #[inline]
            #[cfg_attr(feature = "no-panic", no_panic)]
            #[cfg(not(creusot))]
            fn write(self, buf: &mut Self::Buffer) -> &str {
                let offset = Unsigned::fmt(self, buf);
                // SAFETY: Starting from `offset`, all elements of the slice have been set.
                unsafe { slice_buffer_to_str(buf, offset) }
            }
        }

        impl Integer for $Signed {
            const MAX_STR_LEN: usize = $Signed::MAX.ilog10() as usize + 2;
        }

        impl private::Sealed for $Signed {
            type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];

            #[inline]
            #[cfg_attr(feature = "no-panic", no_panic)]
            #[cfg(not(creusot))]
            fn write(self, buf: &mut Self::Buffer) -> &str {
                let mut offset = Self::MAX_STR_LEN - $Unsigned::MAX_STR_LEN;
                offset += Unsigned::fmt(
                    self.unsigned_abs(),
                    (&mut buf[offset..]).try_into().unwrap(),
                );
                if self < 0 {
                    offset -= 1;
                    buf[offset].write(b'-');
                }
                // SAFETY: Starting from `offset`, all elements of the slice have been set.
                unsafe { slice_buffer_to_str(buf, offset) }
            }
        }
    };
}

impl_Integer!(i8, u8);
impl_Integer!(i16, u16);
impl_Integer!(i32, u32);
impl_Integer!(i64, u64);
impl_Integer!(i128, u128);

macro_rules! impl_Integer_size {
    ($t:ty as $primitive:ident #[cfg(target_pointer_width = $width:literal)]) => {
        #[cfg(target_pointer_width = $width)]
        impl Integer for $t {
            const MAX_STR_LEN: usize = <$primitive as Integer>::MAX_STR_LEN;
        }

        #[cfg(target_pointer_width = $width)]
        impl private::Sealed for $t {
            type Buffer = <$primitive as private::Sealed>::Buffer;

            #[inline]
            #[cfg_attr(feature = "no-panic", no_panic)]
            #[cfg(not(creusot))]
            fn write(self, buf: &mut Self::Buffer) -> &str {
                (self as $primitive).write(buf)
            }
        }
    };
}

impl_Integer_size!(isize as i16 #[cfg(target_pointer_width = "16")]);
impl_Integer_size!(usize as u16 #[cfg(target_pointer_width = "16")]);
impl_Integer_size!(isize as i32 #[cfg(target_pointer_width = "32")]);
impl_Integer_size!(usize as u32 #[cfg(target_pointer_width = "32")]);
impl_Integer_size!(isize as i64 #[cfg(target_pointer_width = "64")]);
impl_Integer_size!(usize as u64 #[cfg(target_pointer_width = "64")]);


/// This function converts a slice of ascii characters into a `&str` starting
/// from `offset`.
///
/// # Safety
///
/// `buf` content starting from `offset` index MUST BE initialized and MUST BE
/// ascii characters.
#[cfg_attr(feature = "no-panic", no_panic)]
#[cfg(not(creusot))]
unsafe fn slice_buffer_to_str(buf: &[MaybeUninit<u8>], offset: usize) -> &str {
    // SAFETY: `offset` is always included between 0 and `buf`'s length.
    let written = unsafe { buf.get_unchecked(offset..) };
    // SAFETY: (`assume_init_ref`) All buf content since offset is set.
    // SAFETY: (`from_utf8_unchecked`) Writes use ASCII from the lookup table exclusively.
    unsafe { str::from_utf8_unchecked(&*(written as *const [MaybeUninit<u8>] as *const [u8])) }
}

/// Store one already-masked decimal digit and describe exactly how that slot
/// joins the initialized suffix. The runtime operation remains the original
/// `write(b'0' + digit)` used by the formatter; the contracts let callers
/// compose this one-slot update with the previously written suffix.
#[inline]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, requires(index@ < buf@.len()))]
#[cfg_attr(creusot, requires(digit@ <= 9))]
#[cfg_attr(creusot, requires(forall<i: Int>
    index@ + 1 <= i && i < buf@.len() ==> buf@[i]@ != None))]
#[cfg_attr(creusot, ensures((^buf)@[index@]@ == Some(48u8 + digit)))]
#[cfg_attr(creusot, ensures((^buf)@.len() == buf@.len()))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    0 <= i && i < buf@.len() && i != index@ ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    index@ <= i && i < buf@.len() ==> (^buf)@[i]@ != None))]
#[cfg_attr(creusot, ensures(
    logical_slot_bytes((^buf)@.subsequence(index@, buf@.len()))
        == Seq::singleton(48 + digit@).concat(
            logical_slot_bytes(buf@.subsequence(index@ + 1, buf@.len()))
        )
))]
fn write_decimal_digit(buf: &mut [MaybeUninit<u8>], index: usize, digit: u8) {
    buf[index].write(b'0' + digit);
}

/// Write one two-digit pair using the production lookup table.
#[inline]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, requires(pair@ < 100))]
#[cfg_attr(creusot, requires(index@ + 2 <= buf@.len()))]
#[cfg_attr(creusot, requires(forall<i: Int>
    index@ + 2 <= i && i < buf@.len() ==> buf@[i]@ != None))]
#[cfg_attr(creusot, ensures((^buf)@.len() == buf@.len()))]
#[cfg_attr(creusot, ensures((^buf)@[index@]@ != None))]
#[cfg_attr(creusot, ensures((^buf)@[index@]@.unwrap_logic()@ == 48 + pair@ / 10))]
#[cfg_attr(creusot, ensures((^buf)@[index@ + 1]@ != None))]
#[cfg_attr(creusot, ensures((^buf)@[index@ + 1]@.unwrap_logic()@ == 48 + pair@ % 10))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    0 <= i && i < buf@.len() && i != index@ && i != index@ + 1
        ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    index@ <= i && i < buf@.len() ==> (^buf)@[i]@ != None))]
#[cfg_attr(creusot, ensures(
    logical_slot_bytes((^buf)@.subsequence(index@, buf@.len()))
        == fixed_width_decimal_values(pair@, 2).concat(
            logical_slot_bytes(buf@.subsequence(index@ + 2, buf@.len()))
        )
))]
#[cfg_attr(creusot, check(terminates))]
fn write_decimal_pair(buf: &mut [MaybeUninit<u8>], index: usize, pair: u32) {
    #[cfg(creusot)]
    let pair_buf_before = snapshot!(buf@);
    #[cfg(creusot)]
    let pair_old_tail = snapshot!(
        logical_slot_bytes(buf@.subsequence(index@ + 2, buf@.len()))
    );
    let tens_index = pair as usize * 2 + 0;
    let ones_index = pair as usize * 2 + 1;
    #[cfg(creusot)]
    let (tens, ones) = decimal_pair_correct(pair);
    #[cfg(creusot)]
    proof_assert!(tens_index@ < 200 && ones_index@ < 200);
    #[cfg(creusot)]
    let tens_from_runtime_table = unsafe { *DECIMAL_PAIRS.0.get_unchecked(tens_index) };
    #[cfg(creusot)]
    let ones_from_runtime_table = unsafe { *DECIMAL_PAIRS.0.get_unchecked(ones_index) };
    #[cfg(creusot)]
    proof_assert!(tens_from_runtime_table == tens && ones_from_runtime_table == ones);
    #[cfg(creusot)]
    proof_assert! {
        let _ = fixed_width_decimal_values_pair(pair@);
        tens@ == fixed_width_decimal_values(pair@, 2)[0]
            && ones@ == fixed_width_decimal_values(pair@, 2)[1]
    };

    unsafe {
        buf[index + 0].write(*DECIMAL_PAIRS.0.get_unchecked(tens_index));
        buf[index + 1].write(*DECIMAL_PAIRS.0.get_unchecked(ones_index));
    }
    #[cfg(creusot)]
    {
        proof_assert!(forall<i: Int>
            index@ + 2 <= i && i < buf@.len()
                ==> buf@[i]@ == (*pair_buf_before)[i]@);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@ + 2, buf@.len()))
            == *pair_old_tail);
        proof_assert! {
            let _ = logical_slot_bytes_split(buf@, index@, index@ + 2, buf@.len());
            true
        };
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 2))[0]
            == fixed_width_decimal_values(pair@, 2)[0]);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 2))[1]
            == fixed_width_decimal_values(pair@, 2)[1]);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 2))
            == fixed_width_decimal_values(pair@, 2));
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, buf@.len()))
            == fixed_width_decimal_values(pair@, 2).concat(*pair_old_tail));
    }
}

/// Write one four-digit chunk with the same four production table stores.
#[inline]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, requires(pair1@ < 100 && pair2@ < 100))]
#[cfg_attr(creusot, requires(index@ + 4 <= buf@.len()))]
#[cfg_attr(creusot, requires(forall<i: Int>
    index@ + 4 <= i && i < buf@.len() ==> buf@[i]@ != None))]
#[cfg_attr(creusot, ensures((^buf)@.len() == buf@.len()))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    index@ <= i && i < index@ + 4 ==> (^buf)@[i]@ != None))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    index@ <= i && i < index@ + 4 ==>
        (^buf)@[i]@.unwrap_logic()@
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[i - index@]))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    0 <= i && i < buf@.len() && (i < index@ || index@ + 4 <= i)
        ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    index@ <= i && i < buf@.len() ==> (^buf)@[i]@ != None))]
#[cfg_attr(creusot, ensures(
    logical_slot_bytes((^buf)@.subsequence(index@, buf@.len()))
        == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4).concat(
            logical_slot_bytes(buf@.subsequence(index@ + 4, buf@.len()))
        )
))]
#[cfg_attr(creusot, check(terminates))]
fn write_decimal_quad(buf: &mut [MaybeUninit<u8>], index: usize, pair1: u32, pair2: u32) {
    #[cfg(creusot)]
    let quad_buf_before = snapshot!(buf@);
    #[cfg(creusot)]
    let quad_old_tail = snapshot!(
        logical_slot_bytes(buf@.subsequence(index@ + 4, buf@.len()))
    );
    let pair1_tens_index = pair1 as usize * 2 + 0;
    let pair1_ones_index = pair1 as usize * 2 + 1;
    let pair2_tens_index = pair2 as usize * 2 + 0;
    let pair2_ones_index = pair2 as usize * 2 + 1;
    #[cfg(creusot)]
    {
        let (pair1_tens, pair1_ones) = decimal_pair_correct(pair1);
        let (pair2_tens, pair2_ones) = decimal_pair_correct(pair2);
        proof_assert!(pair1_tens_index@ < 200 && pair1_ones_index@ < 200);
        proof_assert!(pair2_tens_index@ < 200 && pair2_ones_index@ < 200);
        let pair1_tens_from_runtime_table = unsafe {
            *DECIMAL_PAIRS.0.get_unchecked(pair1_tens_index)
        };
        let pair1_ones_from_runtime_table = unsafe {
            *DECIMAL_PAIRS.0.get_unchecked(pair1_ones_index)
        };
        let pair2_tens_from_runtime_table = unsafe {
            *DECIMAL_PAIRS.0.get_unchecked(pair2_tens_index)
        };
        let pair2_ones_from_runtime_table = unsafe {
            *DECIMAL_PAIRS.0.get_unchecked(pair2_ones_index)
        };
        proof_assert!(pair1_tens_from_runtime_table == pair1_tens);
        proof_assert!(pair1_ones_from_runtime_table == pair1_ones);
        proof_assert!(pair2_tens_from_runtime_table == pair2_tens);
        proof_assert!(pair2_ones_from_runtime_table == pair2_ones);
        proof_assert! {
            let pair1_digits = fixed_width_decimal_values(pair1@, 2);
            let pair2_digits = fixed_width_decimal_values(pair2@, 2);
            let _ = fixed_width_decimal_values_pair(pair1@);
            let _ = fixed_width_decimal_values_pair(pair2@);
            let n = pair1@ * 100 + pair2@;
            let _ = fixed_width_decimal_values_compose_2x2(n);
            let _ = concat_two_get_digits(pair1_digits, pair2_digits);
            fixed_width_decimal_values(n, 4)
                == pair1_digits.concat(pair2_digits)
                && pair1_tens@ == fixed_width_decimal_values(n, 4)[0]
                && pair1_ones@ == fixed_width_decimal_values(n, 4)[1]
                && pair2_tens@ == fixed_width_decimal_values(n, 4)[2]
                && pair2_ones@ == fixed_width_decimal_values(n, 4)[3]
        };
    }

    unsafe {
        buf[index + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1_tens_index));
        buf[index + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1_ones_index));
        buf[index + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2_tens_index));
        buf[index + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2_ones_index));
    }
    #[cfg(creusot)]
    {
        proof_assert!(forall<i: Int>
            index@ + 4 <= i && i < buf@.len()
                ==> buf@[i]@ == (*quad_buf_before)[i]@);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@ + 4, buf@.len()))
            == *quad_old_tail);
        proof_assert! {
            let _ = logical_slot_bytes_split(buf@, index@, index@ + 4, buf@.len());
            true
        };
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 4))[0]
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[0]);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 4))[1]
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[1]);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 4))[2]
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[2]);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 4))[3]
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[3]);
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, index@ + 4))
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4));
        proof_assert!(logical_slot_bytes(buf@.subsequence(index@, buf@.len()))
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4).concat(*quad_old_tail));
    }
}

trait Unsigned: Integer {
    fn fmt(self, buf: &mut Self::Buffer) -> usize;
}

macro_rules! impl_Unsigned {
    ($Unsigned:ident, $capacity_lemma:ident) => {
        impl Unsigned for $Unsigned {
            #[cfg_attr(creusot, ensures(result@ + decimal_values(self@).len() == buf@.len()))]
            #[cfg_attr(creusot, ensures(forall<i: Int>
                result@ <= i && i < buf@.len() ==> (^buf)@[i]@ != None))]
            #[cfg_attr(creusot, ensures(forall<i: Int>
                result@ <= i && i < buf@.len() ==>
                    (^buf)@[i]@.unwrap_logic()@ == decimal_values(self@)[i - result@]))]
            #[cfg_attr(creusot, ensures(forall<i: Int>
                0 <= i && i < result@ ==> (^buf)@[i]@ == buf@[i]@))]
            #[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
            fn fmt(self, buf: &mut Self::Buffer) -> usize {
                #[cfg(creusot)]
                proof_assert! {
                    let _ = $capacity_lemma(self);
                    decimal_values(self@).len() <= buf@.len()
                };
                // Count the number of bytes in buf that are not initialized.
                let mut offset = buf.len();
                // Consume the least-significant decimals from a working copy.
                let mut remain = self;
                #[cfg(creusot)]
                let original = self;
                #[cfg(creusot)]
                let buf_before = snapshot!(buf@);

                // Format per four digits from the lookup table.
                // Four digits need a 16-bit $Unsigned or wider.
                #[cfg_attr(creusot, invariant(offset@ <= buf@.len()))]
                #[cfg_attr(creusot, invariant(offset@ % 4 == buf@.len() % 4))]
                #[cfg_attr(creusot, invariant(forall<i: Int>
                    offset@ <= i && i < buf@.len() ==> buf@[i]@ != None))]
                #[cfg_attr(creusot, invariant(forall<i: Int>
                    0 <= i && i < offset@ ==> buf@[i]@ == (*buf_before)[i]@))]
                #[cfg_attr(creusot, invariant(original@ == 0 ==>
                    remain@ == 0 && offset@ == buf@.len()))]
                #[cfg_attr(creusot, invariant(original@ == 0 ||
                    (if remain@ == 0 { Seq::empty() } else { decimal_values(remain@) })
                        .concat(logical_slot_bytes(buf@.subsequence(offset@, buf@.len())))
                        == decimal_values(original@)))]
                #[cfg_attr(creusot, invariant(
                    (if remain@ == 0 { 0 } else { decimal_values(remain@).len() })
                        <= offset@))]
                #[cfg_attr(creusot, variant(remain))]
                while mem::size_of::<Self>() > 1
                    && remain
                        > 999
                            .try_into()
                            .expect("branch is not hit for types that cannot fit 999 (u8)")
                {
                    #[cfg(creusot)]
                    let old_remain = remain;
                    #[cfg(creusot)]
                    let quad_old_offset = offset;
                    #[cfg(creusot)]
                    let quad_old_written_suffix = snapshot!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len())));
                    #[cfg(creusot)]
                    proof_assert! {
                        let _ = decimal_values_split_4(old_remain@);
                        (if old_remain@ == 0 { Seq::empty() } else { decimal_values(old_remain@) })
                            .concat(*quad_old_written_suffix)
                            == decimal_values(original@)
                    };
                    #[cfg(creusot)]
                    proof_assert! {
                        let _ = decimal_values_len_ge_4(remain@);
                        4 <= offset@
                    }
                    offset -= 4;

                    // pull two pairs
                    let scale: Self = 1_00_00
                        .try_into()
                        .expect("branch is not hit for types that cannot fit 1E4 (u8)");
                    #[cfg(creusot)]
                    proof_assert!(scale@ == 10_000);
                    let quad = remain % scale;
                    remain /= scale;
                    let (pair1, pair2) = divmod100(quad as u32);
                    write_decimal_quad(buf, offset, pair1, pair2);
                    #[cfg(creusot)]
                    {
                        proof_assert!(logical_slot_bytes(buf@.subsequence(quad_old_offset@, buf@.len()))
                            == *quad_old_written_suffix);
                        proof_assert!(quad@ == old_remain@ % 10_000
                            && remain@ == old_remain@ / 10_000);
                        proof_assert!(quad@ == pair1@ * 100 + pair2@);
                        proof_assert!(fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)
                            == fixed_width_decimal_values(quad@, 4));
                        proof_assert!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len()))
                            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)
                                .concat(*quad_old_written_suffix));
                        proof_assert! {
                        let _ = decimal_values_split_4(old_remain@);
                            let prefix = if remain@ == 0 {
                                Seq::empty()
                            } else {
                                decimal_values(remain@)
                            };
                            let digits = fixed_width_decimal_values(quad@, 4);
                            let _ = decimal_seq_concat_assoc(
                                prefix,
                                digits,
                                *quad_old_written_suffix,
                            );
                            prefix.concat(digits.concat(*quad_old_written_suffix))
                                == decimal_values(original@)
                        };
                    }
                    #[cfg(creusot)]
                    proof_assert! {
                        (if remain@ == 0 {
                            0
                        } else {
                            decimal_values(remain@).len()
                        }) <= offset@
                    };
                }

                #[cfg(creusot)]
                proof_assert!(remain@ <= 999);

                // Format per two digits from the lookup table.
                if remain > 9 {
                    #[cfg(creusot)]
                    let tail_old_offset = offset;
                    #[cfg(creusot)]
                    let old_written_suffix = snapshot!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len())));
                    #[cfg(creusot)]
                    proof_assert!(original@ != 0);
                    #[cfg(creusot)]
                    proof_assert!(decimal_values(remain@).concat(*old_written_suffix)
                        == decimal_values(original@));
                    #[cfg(creusot)]
                    proof_assert! {
                        let _ = decimal_values_len_ge_2(remain@);
                        2 <= offset@
                    }
                    offset -= 2;

                    #[cfg(creusot)]
                    let tail_before = remain;
                    let (last, pair) = divmod100(remain as u32);
                    #[cfg(creusot)]
                    {
                        proof_assert!(tail_before@ >= 10 && tail_before@ <= 999);
                        proof_assert!(last@ == tail_before@ / 100);
                        proof_assert!(pair@ == tail_before@ % 100);
                        proof_assert!(last@ <= 9 && pair@ < 100);
                        proof_assert!(if last@ == 0 {
                            let _ = fixed_width_decimal_values_2_is_decimal(tail_before@);
                            fixed_width_decimal_values(tail_before@, 2)
                                == fixed_width_decimal_values(pair@, 2)
                        } else {
                            let _ = decimal_values_compose_1x2(tail_before@);
                            decimal_values(tail_before@)
                                == decimal_values(last@)
                                    .concat(fixed_width_decimal_values(pair@, 2))
                        });
                    }
                    remain = last as Self;
                    write_decimal_pair(buf, offset, pair);
                    #[cfg(creusot)]
                    {
                        proof_assert!(logical_slot_bytes(buf@.subsequence(tail_old_offset@, buf@.len()))
                            == *old_written_suffix);
                        proof_assert!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len()))
                            == fixed_width_decimal_values(pair@, 2)
                                .concat(*old_written_suffix));
                        proof_assert!(if remain@ == 0 {
                            fixed_width_decimal_values(tail_before@, 2)
                                == fixed_width_decimal_values(pair@, 2)
                        } else {
                                decimal_values(tail_before@)
                                    == decimal_values(remain@)
                                        .concat(fixed_width_decimal_values(pair@, 2))
                        });
                        proof_assert! {
                            let tail = if remain@ == 0 {
                                Seq::empty()
                            } else {
                                decimal_values(remain@)
                            };
                            let pair_values = fixed_width_decimal_values(pair@, 2);
                            let _ = decimal_seq_concat_assoc(tail, pair_values, *old_written_suffix);
                            tail.concat(pair_values.concat(*old_written_suffix))
                                == decimal_values(tail_before@).concat(*old_written_suffix)
                        };
                        proof_assert!(
                            (if remain@ == 0 { Seq::empty() } else { decimal_values(remain@) })
                                .concat(logical_slot_bytes(buf@.subsequence(offset@, buf@.len())))
                                == decimal_values(original@));
                    }
                }

                // Format the last remaining digit, if any.
                if remain != 0 || self == 0 {
                    #[cfg(creusot)]
                    let last_digit_value = remain;
                    #[cfg(creusot)]
                    proof_assert! {
                        if last_digit_value@ == 0 {
                            let _ = decimal_values_len_at_least_one(original@);
                            original@ == 0 && offset@ == buf@.len()
                        } else {
                            let _ = decimal_values_len_at_least_one(last_digit_value@);
                            true
                        }
                    }
                    #[cfg(creusot)]
                    proof_assert!(last_digit_value@ <= 9);
                    #[cfg(creusot)]
                    proof_assert!(1 <= offset@);
                    #[cfg(creusot)]
                    let last_old_offset = offset;
                    #[cfg(creusot)]
                    let last_old_written_suffix = snapshot!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len())));
                    #[cfg(creusot)]
                    proof_assert!(if original@ == 0 {
                        remain@ == 0 && offset@ == buf@.len()
                            && *last_old_written_suffix == Seq::empty()
                    } else {
                        remain@ != 0 && decimal_values(remain@)
                            .concat(*last_old_written_suffix)
                            == decimal_values(original@)
                    });
                    offset -= 1;
                    #[cfg(creusot)]
                    proof_assert!(last_old_offset@ == offset@ + 1);
                    #[cfg(creusot)]
                    proof_assert!(logical_slot_bytes(buf@.subsequence(offset@ + 1, buf@.len()))
                        == *last_old_written_suffix);

                    // Either the compiler sees that remain < 10, or it prevents
                    // a boundary check up next.
                    #[cfg(creusot)]
                    proof_assert!((remain as u8)@ == remain@);
                    #[cfg(creusot)]
                    let last_digit_proof = masked_decimal_digit(remain as u8);
                    let last = remain as u8 & 15;
                    #[cfg(creusot)]
                    proof_assert!(last == last_digit_proof);
                    #[cfg(creusot)]
                    proof_assert!(48 + last_digit_proof@ <= 57);
                    write_decimal_digit(buf, offset, last);
                    #[cfg(creusot)]
                    {
                        let last_digit_value = remain;
                        proof_assert!(last@ == last_digit_value@);
                        proof_assert!(if original@ == 0 {
                            let _ = decimal_values_one_digit(original@);
                            decimal_values(original@) == Seq::singleton(48)
                        } else {
                            decimal_values(last_digit_value@)
                                == Seq::singleton(48 + last_digit_value@)
                        });
                        proof_assert!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len()))
                            == Seq::singleton(48 + last_digit_value@)
                                .concat(*last_old_written_suffix));
                    }
                    // not used: remain = 0;
                }

                #[cfg(creusot)]
                proof_assert!(forall<i: Int>
                    offset@ <= i && i < buf@.len() ==> buf@[i]@ != None);
                #[cfg(creusot)]
                proof_assert!(logical_slot_bytes(buf@.subsequence(offset@, buf@.len()))
                    == decimal_values(original@));
                #[cfg(creusot)]
                proof_assert!(offset@ + decimal_values(original@).len() == buf@.len());
                #[cfg(creusot)]
                proof_assert!(forall<i: Int>
                    0 <= i && i < offset@ ==> buf@[i]@ == (*buf_before)[i]@);

                offset
            }
        }
    };
}

#[cfg(not(creusot))]
impl_Unsigned!(u8, decimal_values_len_u8);
impl_Unsigned!(u16, decimal_values_len_u16);
#[cfg(not(creusot))]
impl_Unsigned!(u32, decimal_values_len_u32);
#[cfg(creusot)]
impl_Unsigned!(u32, decimal_values_len_u32);
#[cfg(not(creusot))]
impl_Unsigned!(u64, decimal_values_len_u64);

/// A small call-site check that consumes the contract of the actual u16 body.
#[cfg(creusot)]
fn check_u16_fmt_call_site(n: u16) {
    proof_assert! {
        let _ = decimal_values_len_u16(n);
        decimal_values(n@).len() <= 5
    };
    let mut buf = [MaybeUninit::<u8>::uninit(); 5];
    let start = <u16 as Unsigned>::fmt(n, &mut buf);
    proof_assert!(start@ + decimal_values(n@).len() == buf@.len());
    proof_assert!(forall<i: Int>
        start@ <= i && i < buf@.len() ==> buf@[i]@ != None);
    proof_assert!(forall<i: Int>
        start@ <= i && i < buf@.len() ==>
            buf@[i]@.unwrap_logic()@ == decimal_values(n@)[i - start@]);
}

#[cfg(not(creusot))]
impl Unsigned for u128 {
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn fmt(self, buf: &mut Self::Buffer) -> usize {
        // Optimize common-case zero, which would also need special treatment due to
        // its "leading" zero.
        if self == 0 {
            let offset = buf.len() - 1;
            buf[offset].write(b'0');
            return offset;
        }
        // Take the 16 least-significant decimals.
        let (quot_1e16, mod_1e16) = div_rem_1e16(self);
        let (mut remain, mut offset) = if quot_1e16 == 0 {
            (mod_1e16, u128::MAX_STR_LEN)
        } else {
            // Write digits at buf[23..39].
            enc_16lsd::<{ u128::MAX_STR_LEN - 16 }>(buf, mod_1e16);

            // Take another 16 decimals.
            let (quot2, mod2) = div_rem_1e16(quot_1e16);
            if quot2 == 0 {
                (mod2, u128::MAX_STR_LEN - 16)
            } else {
                // Write digits at buf[7..23].
                enc_16lsd::<{ u128::MAX_STR_LEN - 32 }>(buf, mod2);
                // Quot2 has at most 7 decimals remaining after two 1e16 divisions.
                (quot2 as u64, u128::MAX_STR_LEN - 32)
            }
        };

        // Format per four digits from the lookup table.
        while remain > 999 {
            offset -= 4;

            // pull two pairs
            let quad = remain % 1_00_00;
            remain /= 1_00_00;
            let (pair1, pair2) = divmod100(quad as u32);
            unsafe {
                buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
                buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
                buf[offset + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
                buf[offset + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
            }
        }

        // Format per two digits from the lookup table.
        if remain > 9 {
            offset -= 2;

            let (last, pair) = divmod100(remain as u32);
            remain = last as u64;
            unsafe {
                buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 0));
                buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 1));
            }
        }

        // Format the last remaining digit, if any.
        if remain != 0 {
            offset -= 1;

            // Either the compiler sees that remain < 10, or it prevents
            // a boundary check up next.
            let last = remain as u8 & 15;
            buf[offset].write(b'0' + last);
            // not used: remain = 0;
        }
        offset
    }
}

// Encodes the 16 least-significant decimals of n into `buf[OFFSET..OFFSET + 16]`.
#[cfg_attr(feature = "no-panic", no_panic)]
#[cfg(not(creusot))]
fn enc_16lsd<const OFFSET: usize>(buf: &mut [MaybeUninit<u8>], n: u64) {
    // Consume the least-significant decimals from a working copy.
    let mut remain = n;

    // Format per four digits from the lookup table.
    for quad_index in (1..4).rev() {
        // pull two pairs
        let quad = remain % 1_00_00;
        remain /= 1_00_00;
        let (pair1, pair2) = divmod100(quad as u32);
        unsafe {
            buf[quad_index * 4 + OFFSET + 0]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
            buf[quad_index * 4 + OFFSET + 1]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
            buf[quad_index * 4 + OFFSET + 2]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
            buf[quad_index * 4 + OFFSET + 3]
                .write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
        }
    }

    // final two pairs
    let (pair1, pair2) = divmod100(remain as u32);
    unsafe {
        buf[OFFSET + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0));
        buf[OFFSET + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1));
        buf[OFFSET + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0));
        buf[OFFSET + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1));
    }
}

// Euclidean division plus remainder with constant 1E16 basically consumes 16
// decimals from n.
//
// The integer division algorithm is based on the following paper:
//
//   T. Granlund and P. Montgomery, “Division by Invariant Integers Using Multiplication”
//   in Proc. of the SIGPLAN94 Conference on Programming Language Design and
//   Implementation, 1994, pp. 61–72
//
#[cfg_attr(feature = "no-panic", no_panic)]
#[cfg(not(creusot))]
fn div_rem_1e16(n: u128) -> (u128, u64) {
    const D: u128 = 1_0000_0000_0000_0000;
    // The check inlines well with the caller flow.
    if n < D {
        return (0, n as u64);
    }

    // These constant values are computed with the CHOOSE_MULTIPLIER procedure
    // from the Granlund & Montgomery paper, using N=128, prec=128 and d=1E16.
    const M_HIGH: u128 = 76624777043294442917917351357515459181;
    const SH_POST: u8 = 51;

    // n.widening_mul(M_HIGH).1 >> SH_POST
    let quot = u128_ext::mulhi(n, M_HIGH) >> SH_POST;
    let rem = n - quot * D;
    (quot, rem as u64)
}
