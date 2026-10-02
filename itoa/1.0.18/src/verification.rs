use core::str;
use core::mem::MaybeUninit;
use creusot_std::std::option::OptionExt;

#[allow(unused_imports)]
use creusot_std::prelude::{
    bitwise_proof, check, ensures, logic, pearlite, proof_assert, requires, snapshot, trusted,
    variant, Int, Seq, View,
};

/// Decimal ASCII byte values for a nonnegative mathematical integer.
#[logic(open)]
#[requires(n >= 0)]
#[variant(n)]
pub fn decimal_values(n: Int) -> Seq<Int> {
    if n < 10 {
        Seq::singleton(48 + n)
    } else {
        decimal_values(n / 10).push_back(48 + n % 10)
    }
}

/// A total ghost view of buffer slots used only for sequence bookkeeping.
/// Initialized slots map to their byte value; an uninitialized slot maps to
/// zero as a mathematical filler. This does not read or initialize memory.
/// Any use of a value as a written output byte must be accompanied by the
/// formatter's separate initialized-slot invariant or postcondition.
#[logic(open)]
#[ensures(result.len() == slots.len())]
pub fn logical_slot_bytes(slots: Seq<MaybeUninit<u8>>) -> Seq<Int> {
    slots.map(|slot: MaybeUninit<u8>| {
        pearlite! {
            if slot@ == None {
                0
            } else {
                slot@.unwrap_logic()@
            }
        }
    })
}

/// A proof witness for the optimized last-digit mask used by the runtime body.
/// The formatter still evaluates its original `remain as u8 & 15` expression;
/// this checked bitwise lemma connects that expression to its decimal-digit
/// range when the remaining value is already known to be at most nine.
#[cfg(creusot)]
#[check(terminates)]
#[bitwise_proof]
#[requires(d@ <= 9)]
#[ensures((d & 15u8)@ == d@)]
#[ensures(48 + (d & 15u8)@ <= 57)]
#[ensures(result@ == d@)]
#[ensures(48 + result@ <= 57)]
pub(crate) fn masked_decimal_digit(d: u8) -> u8 {
    d & 15
}

#[logic]
#[requires(n >= 0)]
#[ensures(decimal_values(n) == if n < 10 {
    Seq::singleton(48 + n)
} else {
    decimal_values(n / 10).push_back(48 + n % 10)
})]
fn decimal_values_unfold(n: Int) {}

#[logic]
#[requires(0 <= n && n < 10)]
#[ensures(decimal_values(n) == Seq::singleton(48 + n))]
pub fn decimal_values_one_digit(n: Int) {
    decimal_values_unfold(n);
}

#[logic(open)]
#[requires(0 <= exponent)]
#[variant(exponent)]
pub fn power_of_ten(exponent: Int) -> Int {
    if exponent == 0 {
        1
    } else {
        10 * power_of_ten(exponent - 1)
    }
}

#[logic]
#[requires(0 <= exponent)]
#[ensures(power_of_ten(exponent) == if exponent == 0 {
    1
} else {
    10 * power_of_ten(exponent - 1)
})]
fn power_of_ten_unfold(exponent: Int) {}

#[logic]
#[ensures(power_of_ten(39)
    == 10 * 100_000_000_000_000_000_000_000_000_000_000_000_000)]
fn power_of_ten_39() {
    power_of_ten_unfold(39);
    power_of_ten_unfold(38);
    power_of_ten_unfold(37);
    power_of_ten_unfold(36);
    power_of_ten_unfold(35);
    power_of_ten_unfold(34);
    power_of_ten_unfold(33);
    power_of_ten_unfold(32);
    power_of_ten_unfold(31);
    power_of_ten_unfold(30);
    power_of_ten_unfold(29);
    power_of_ten_unfold(28);
    power_of_ten_unfold(27);
    power_of_ten_unfold(26);
    power_of_ten_unfold(25);
    power_of_ten_unfold(24);
    power_of_ten_unfold(23);
    power_of_ten_unfold(22);
    power_of_ten_unfold(21);
    power_of_ten_unfold(20);
    power_of_ten_unfold(19);
    power_of_ten_unfold(18);
    power_of_ten_unfold(17);
    power_of_ten_unfold(16);
    power_of_ten_unfold(15);
    power_of_ten_unfold(14);
    power_of_ten_unfold(13);
    power_of_ten_unfold(12);
    power_of_ten_unfold(11);
    power_of_ten_unfold(10);
    power_of_ten_unfold(9);
    power_of_ten_unfold(8);
    power_of_ten_unfold(7);
    power_of_ten_unfold(6);
    power_of_ten_unfold(5);
    power_of_ten_unfold(4);
    power_of_ten_unfold(3);
    power_of_ten_unfold(2);
    power_of_ten_unfold(1);
    power_of_ten_unfold(0);
}

#[logic]
#[requires(0 <= n)]
#[requires(1 <= digits)]
#[requires(n < power_of_ten(digits))]
#[ensures(decimal_values(n).len() <= digits)]
#[variant(digits)]
fn decimal_len_bounded(n: Int, digits: Int) {
    decimal_values_unfold(n);
    if n >= 10 {
        proof_assert!(1 < digits);
        proof_assert!(n / 10 < power_of_ten(digits - 1));
        decimal_len_bounded(n / 10, digits - 1);
    }
}

/// Decimal ASCII byte values, including a leading minus sign when needed.
#[logic(open)]
pub fn signed_decimal_values(n: Int) -> Seq<Int> {
    if n < 0 {
        pearlite! { Seq::singleton(45).concat(decimal_values(-n)) }
    } else {
        decimal_values(n)
    }
}

/// Exact whole-buffer effect of the recursive digit writer.
#[logic(open)]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[variant(n)]
pub fn write_model(values: Seq<Int>, n: Int, end: Int) -> Seq<Int> {
    if n < 10 {
        values.set(end - 1, 48 + n)
    } else {
        write_model(values, n / 10, end - 1).set(end - 1, 48 + n % 10)
    }
}

#[logic]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[ensures(write_model(values, n, end) == if n < 10 {
    values.set(end - 1, 48 + n)
} else {
    write_model(values, n / 10, end - 1).set(end - 1, 48 + n % 10)
})]
fn write_model_unfold(values: Seq<Int>, n: Int, end: Int) {}

#[logic]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[ensures(write_model(values, n, end).len() == values.len())]
#[variant(n)]
fn write_model_preserves_len(values: Seq<Int>, n: Int, end: Int) {
    if n >= 10 {
        decimal_values_unfold(n);
        write_model_preserves_len(values, n / 10, end - 1);
    }
}

/// Representation lemma connecting the update-oriented model to canonical
/// most-significant-first decimal bytes.
#[logic]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[ensures(write_model(values, n, end)
    .subsequence(end - decimal_values(n).len(), end) == decimal_values(n))]
#[variant(n)]
fn write_model_is_decimal(values: Seq<Int>, n: Int, end: Int) {
    if n >= 10 {
        decimal_values_unfold(n);
        write_model_preserves_len(values, n, end);
        write_model_preserves_len(values, n / 10, end - 1);
        write_model_is_decimal(values, n / 10, end - 1);
        let prefix = decimal_values(n / 10);
        let digit = 48 + n % 10;
        let start = end - prefix.len() - 1;
        let previous = write_model(values, n / 10, end - 1);
        let output = write_model(values, n, end);
        proof_assert!(decimal_values(n) == prefix.push_back(digit));
        proof_assert!(decimal_values(n).len() == prefix.len() + 1);
        proof_assert!(prefix.len() + 1 <= end);
        proof_assert!(0 <= start);
        proof_assert!(output.len() == values.len());
        proof_assert!(previous.len() == values.len());
        proof_assert!(end <= output.len());
        proof_assert!(output == previous.set(end - 1, digit));
        proof_assert!(start + prefix.len() == end - 1);
        proof_assert!(output.subsequence(start, end).len() == prefix.len() + 1);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            start + i < end - 1);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            output[start + i] == previous[start + i]);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            previous.subsequence(start, end - 1)[i] == prefix[i]);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            output.subsequence(start, end)[i] == prefix[i]);
        proof_assert!(output[end - 1] == digit);
        proof_assert!(output.subsequence(start, end)[prefix.len()] == output[start + prefix.len()]);
        proof_assert!(output.subsequence(start, end)[prefix.len()] == digit);
        proof_assert!(output.subsequence(start, end) == prefix.push_back(digit));
    } else {
        decimal_values_unfold(n);
        write_model_preserves_len(values, n, end);
        let output = write_model(values, n, end);
        proof_assert!(n < 10);
        proof_assert!(decimal_values(n) == Seq::singleton(48 + n));
        proof_assert!(decimal_values(n).len() == 1);
        proof_assert!(1 <= end);
        proof_assert!(0 <= end - 1);
        proof_assert!(output.len() == values.len());
        proof_assert!(end <= output.len());
        proof_assert!(output == values.set(end - 1, 48 + n));
        proof_assert!(output[end - 1] == 48 + n);
        proof_assert!(output.subsequence(end - 1, end).len() == 1);
        proof_assert!(output.subsequence(end - 1, end)[0] == output[end - 1]);
        proof_assert!(output.subsequence(end - 1, end)[0] == 48 + n);
        proof_assert!(output.subsequence(end - 1, end) == Seq::singleton(48 + n));
    }
}

mod private {
    use super::*;

    pub trait Sealed: Copy {
        #[logic]
        fn value(self) -> Int;

        #[ensures(result@ == if Self::value(self) < 0 {
            -Self::value(self)
        } else {
            Self::value(self)
        })]
        fn magnitude(self) -> u128;

        #[ensures(result == (Self::value(self) < 0))]
        fn is_negative(self) -> bool;
    }
}

/// An integer that can be written into a [`Buffer`].
///
/// This trait is sealed and has the same implementations as upstream itoa.
pub trait Integer: private::Sealed {
    /// Maximum decimal representation length for this type.
    const MAX_STR_LEN: usize;
}

/// Exact mathematical integer value of an [`Integer`].
#[logic(open)]
pub fn integer_value<I: Integer>(value: I) -> Int {
    private::Sealed::value(value)
}

/// Exact decimal ASCII model used by [`Buffer::format`].
#[logic(open)]
pub fn integer_decimal_values<I: Integer>(value: I) -> Seq<Int> {
    signed_decimal_values(integer_value(value))
}

macro_rules! impl_unsigned {
    ($($ty:ty => $len:expr),* $(,)?) => {$($crate::verification::impl_unsigned!(@one $ty, $len);)*};
    (@one $ty:ty, $len:expr) => {
        impl Integer for $ty {
            const MAX_STR_LEN: usize = $len;
        }

        impl private::Sealed for $ty {
            #[logic(open)]
            fn value(self) -> Int {
                pearlite! { self@ }
            }

            #[ensures(result@ == Self::value(self))]
            fn magnitude(self) -> u128 {
                self as u128
            }

            #[ensures(!result)]
            fn is_negative(self) -> bool {
                false
            }
        }
    };
}

// Keep this macro local while allowing its recursive arm to be addressed.
pub(crate) use impl_unsigned;

impl_unsigned! {
    u8 => 3,
    u16 => 5,
    u32 => 10,
    u64 => 20,
    u128 => 39,
    usize => 20,
}

macro_rules! unsigned_decimal_capacity {
    ($name:ident, $ty:ty, $digits:literal; $($exponent:literal),* $(,)?) => {
        #[logic]
        #[ensures(decimal_values(n@).len() <= $digits)]
        #[ensures(decimal_values(n@).len() <= (<$ty as Integer>::MAX_STR_LEN)@)]
        #[ensures((<$ty as Integer>::MAX_STR_LEN)@ == $digits)]
        pub(crate) fn $name(n: $ty) {
            let mathematical = pearlite! { n@ };
            $(power_of_ten_unfold($exponent);)*
            proof_assert!(mathematical < power_of_ten($digits));
            decimal_len_bounded(mathematical, $digits);
        }
    };
}

unsigned_decimal_capacity!(decimal_values_len_u8, u8, 3; 3, 2, 1, 0);
unsigned_decimal_capacity!(decimal_values_len_u16, u16, 5; 5, 4, 3, 2, 1, 0);
unsigned_decimal_capacity!(decimal_values_len_u32, u32, 10; 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
unsigned_decimal_capacity!(decimal_values_len_u64, u64, 20;
    20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);

#[logic]
#[ensures(decimal_values(n@).len() <= 39)]
#[ensures(decimal_values(n@).len() <= (<u128 as Integer>::MAX_STR_LEN)@)]
#[ensures((<u128 as Integer>::MAX_STR_LEN)@ == 39)]
pub(crate) fn decimal_values_len_u128(n: u128) {
    let mathematical = pearlite! { n@ };
    let _ = power_of_ten_39();
    proof_assert!(mathematical < power_of_ten(39));
    decimal_len_bounded(mathematical, 39);
}

macro_rules! impl_signed_widening {
    ($($ty:ty => $len:expr),* $(,)?) => {$($crate::verification::impl_signed_widening!(@one $ty, $len);)*};
    (@one $ty:ty, $len:expr) => {
        impl Integer for $ty {
            const MAX_STR_LEN: usize = $len;
        }

        impl private::Sealed for $ty {
            #[logic(open)]
            fn value(self) -> Int {
                pearlite! { self@ }
            }

            #[ensures(result@ == if Self::value(self) < 0 {
                -Self::value(self)
            } else {
                Self::value(self)
            })]
            fn magnitude(self) -> u128 {
                let wide = self as i128;
                if wide < 0 {
                    (-wide) as u128
                } else {
                    wide as u128
                }
            }

            #[ensures(result == (Self::value(self) < 0))]
            fn is_negative(self) -> bool {
                self < 0
            }
        }
    };
}

pub(crate) use impl_signed_widening;

impl_signed_widening! {
    i8 => 4,
    i16 => 6,
    i32 => 11,
    i64 => 20,
    isize => 20,
}

impl Integer for i128 {
    const MAX_STR_LEN: usize = 40;
}

impl private::Sealed for i128 {
    #[logic(open)]
    fn value(self) -> Int {
        pearlite! { self@ }
    }

    #[ensures(result@ == if Self::value(self) < 0 {
        -Self::value(self)
    } else {
        Self::value(self)
    })]
    fn magnitude(self) -> u128 {
        if self == i128::MIN {
            (i128::MAX as u128) + 1
        } else if self < 0 {
            (-self) as u128
        } else {
            self as u128
        }
    }

    #[ensures(result == (Self::value(self) < 0))]
    fn is_negative(self) -> bool {
        self < 0
    }
}

/// Verification-facing representation of itoa's fixed stack buffer.
///
/// Runtime builds retain upstream's `MaybeUninit` representation and optimized
/// lookup-table algorithm. This initialized representation makes the byte
/// recurrence independently provable without adding raw-memory facts to it.
pub struct Buffer {
    bytes: [u8; 40],
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer { bytes: [0; 40] }
    }

    /// Print an integer and return its exact signed decimal representation.
    #[ensures(result@.to_bytes().map(|byte: u8| byte@) == integer_decimal_values(i))]
    pub fn format<I: Integer>(&mut self, i: I) -> &str {
        let magnitude = private::Sealed::magnitude(i);
        let negative = private::Sealed::is_negative(i);
        proof_assert!(0 <= magnitude@);
        proof_assert! {
            let _ = power_of_ten_39();
            magnitude@ < power_of_ten(39)
        };
        proof_assert! {
            let _ = decimal_len_bounded(magnitude@, 39);
            decimal_values(magnitude@).len() <= 39
        };
        let mut start = write_unsigned(magnitude, &mut self.bytes, 40);
        let digits_start = start;
        if negative {
            start -= 1;
            self.bytes[start] = b'-';
            proof_assert!(integer_value(i) < 0);
            proof_assert!(magnitude@ == -integer_value(i));
            proof_assert!(self.bytes@.subsequence(digits_start@, 40)
                .map(|byte: u8| byte@) == decimal_values(magnitude@));
            proof_assert!(self.bytes@.subsequence(start@, 40)
                .map(|byte: u8| byte@).len() == decimal_values(magnitude@).len() + 1);
            proof_assert!(self.bytes@.subsequence(start@, 40)
                .map(|byte: u8| byte@)[0] == 45);
            proof_assert!(forall<j: Int> 0 <= j && j < decimal_values(magnitude@).len() ==>
                self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)[j + 1]
                    == decimal_values(magnitude@)[j]);
            proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
                == Seq::singleton(45).concat(decimal_values(magnitude@)));
            proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
                == signed_decimal_values(integer_value(i)));
        } else {
            proof_assert!(0 <= integer_value(i));
            proof_assert!(magnitude@ == integer_value(i));
            proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
                == signed_decimal_values(integer_value(i)));
        }
        // SAFETY: write_unsigned emits only ASCII digits, and the optional byte
        // immediately before them is the ASCII minus sign.
        let output = unsafe { decimal_slice_to_str(&self.bytes, start) };
        proof_assert!(output@.to_bytes().map(|byte: u8| byte@)
            == self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@));
        proof_assert!(output@.to_bytes().map(|byte: u8| byte@)
            == integer_decimal_values(i));
        output
    }
}

impl Default for Buffer {
    fn default() -> Buffer {
        Buffer::new()
    }
}

impl Copy for Buffer {}

#[allow(clippy::non_canonical_clone_impl)]
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Buffer::new()
    }
}

/// One byte update expressed directly in the integer-valued vocabulary used
/// by the decimal writer model.
#[requires(index@ < buf@.len())]
#[ensures((^buf)@.map(|value: u8| value@)
    == buf@.map(|value: u8| value@).set(index@, byte@))]
#[check(terminates)]
fn write_byte(buf: &mut [u8], index: usize, byte: u8) {
    buf[index] = byte;
    proof_assert!((^buf)@.map(|value: u8| value@).len()
        == buf@.map(|value: u8| value@).len());
    proof_assert!(forall<i: Int> 0 <= i && i < buf@.len() ==>
        (^buf)@.map(|value: u8| value@)[i]
            == buf@.map(|value: u8| value@).set(index@, byte@)[i]);
}

/// Write the canonical unsigned decimal representation into the suffix ending
/// at `end`. Recursion has one progress measure: the remaining magnitude.
#[requires(end@ <= buf@.len())]
#[requires(decimal_values(n@).len() <= end@)]
#[ensures((^buf)@.map(|byte: u8| byte@)
    == write_model(buf@.map(|byte: u8| byte@), n@, end@))]
#[ensures(result@ + decimal_values(n@).len() == end@)]
#[ensures((^buf)@.subsequence(result@, end@).map(|byte: u8| byte@) == decimal_values(n@))]
#[check(terminates)]
#[variant(n)]
fn write_unsigned(n: u128, buf: &mut [u8], end: usize) -> usize {
    let before = snapshot!(buf@);
    proof_assert!(0 <= n@);
    let result = if n < 10 {
        proof_assert!(n@ < 10);
        proof_assert! {
            let _ = decimal_values_one_digit(n@);
            decimal_values(n@) == Seq::singleton(48 + n@)
        };
        proof_assert!(decimal_values(n@).len() == 1);
        proof_assert!(1 <= end@);
        let start = end - 1;
        write_byte(buf, start, b'0' + n as u8);
        proof_assert!(start@ + decimal_values(n@).len() == end@);
        start
    } else {
        proof_assert!(10 <= n@);
        proof_assert!(decimal_values(n@)
            == decimal_values(n@ / 10).push_back(48 + n@ % 10));
        proof_assert!(decimal_values(n@).len() == decimal_values(n@ / 10).len() + 1);
        proof_assert!(decimal_values(n@ / 10).len() + 1 <= end@);
        proof_assert!(decimal_values(n@ / 10).len() <= end@ - 1);
        let digit = (n % 10) as u8;
        let start = write_unsigned(n / 10, buf, end - 1);
        write_byte(buf, end - 1, b'0' + digit);
        proof_assert!(start@ + decimal_values(n@).len() == end@);
        start
    };
    proof_assert! {
        let _ = decimal_values_unfold(n@);
        result@ + decimal_values(n@).len() == end@
    };
    proof_assert! {
        let _ = write_model_unfold((*before).map(|byte: u8| byte@), n@, end@);
        (^buf)@.map(|byte: u8| byte@)
            == write_model((*before).map(|byte: u8| byte@), n@, end@)
    };
    proof_assert! {
        let _ = write_model_is_decimal((*before).map(|byte: u8| byte@), n@, end@);
        (^buf)@.map(|byte: u8| byte@).subsequence(result@, end@)
            == decimal_values(n@)
    };
    proof_assert!((^buf)@.subsequence(result@, end@).map(|byte: u8| byte@).len()
        == decimal_values(n@).len());
    proof_assert!(forall<i: Int> 0 <= i && i < decimal_values(n@).len() ==>
        (^buf)@.subsequence(result@, end@).map(|byte: u8| byte@)[i]
            == (^buf)@.map(|byte: u8| byte@).subsequence(result@, end@)[i]);
    proof_assert!((^buf)@.subsequence(result@, end@).map(|byte: u8| byte@)
        == decimal_values(n@));
    result
}

/// UTF-8/reference construction boundary. The caller proves the complete byte
/// suffix; this leaf only exposes the corresponding `str` byte sequence.
// TODO: remove `trusted` once Creusot can derive ASCII UTF-8 validity and model
// the slice-to-str reference conversion without raw representation casts.
#[trusted]
#[requires(start@ <= buf@.len())]
#[ensures(result@.to_bytes() == buf@.subsequence(start@, buf@.len()))]
unsafe fn decimal_slice_to_str(buf: &[u8], start: usize) -> &str {
    unsafe { str::from_utf8_unchecked(&buf[start..]) }
}
#[logic(open)]
#[requires(n >= 0)]
#[requires(width >= 1)]
#[variant(width)]
/// Recursive decimal padding model. Under `0 <= n < 10^width`, it gives exactly
/// `width` digits, as established by `fixed_width_decimal_values_len`.
pub fn fixed_width_decimal_values(n: Int, width: Int) -> Seq<Int> {
    if width == 1 {
        decimal_values(n)
    } else {
        fixed_width_decimal_values(n / 10, width - 1).push_back(48 + n % 10)
    }
}

#[logic]
#[requires(n >= 0)]
#[requires(width >= 1)]
#[ensures(fixed_width_decimal_values(n, width) == if width == 1 {
    decimal_values(n)
} else {
    fixed_width_decimal_values(n / 10, width - 1).push_back(48 + n % 10)
})]
fn fixed_width_decimal_values_unfold(n: Int, width: Int) {}

#[logic]
#[requires(0 <= n && n < 10)]
#[ensures(decimal_values(n).len() == 1)]
fn decimal_values_one_digit_len(n: Int) {
    decimal_values_unfold(n);
}

#[logic]
#[requires(n >= 0)]
#[ensures(decimal_values(n).len() >= 1)]
#[variant(n)]
pub(crate) fn decimal_values_len_at_least_one(n: Int) {
    decimal_values_unfold(n);
    if n >= 10 {
        proof_assert!(n / 10 >= 0);
        decimal_values_len_at_least_one(n / 10);
        proof_assert!(decimal_values(n).len() >= 1);
    }
}

/// Values at least 10 have at least two canonical decimal digits.
#[logic]
#[requires(n >= 10)]
#[ensures(decimal_values(n).len() >= 2)]
pub(crate) fn decimal_values_len_ge_2(n: Int) {
    decimal_values_unfold(n);
    proof_assert!(n / 10 >= 0);
    decimal_values_len_at_least_one(n / 10);
    proof_assert!(decimal_values(n).len() >= 2);
}

#[logic]
#[ensures(s.push_back(value) == s.concat(Seq::singleton(value)))]
fn decimal_seq_snoc_singleton(s: Seq<Int>, value: Int) {}

#[logic]
#[ensures((s.concat(t)).push_back(value) == s.concat(t.push_back(value)))]
fn decimal_seq_concat_snoc(s: Seq<Int>, t: Seq<Int>, value: Int) {}

#[logic]
#[ensures((s.concat(t)).concat(u) == s.concat(t.concat(u)))]
pub(crate) fn decimal_seq_concat_assoc(s: Seq<Int>, t: Seq<Int>, u: Seq<Int>) {}

/// Values at least 1000 have at least four canonical decimal digits.
#[logic]
#[requires(n >= 1_000)]
#[ensures(decimal_values(n).len() >= 4)]
pub(crate) fn decimal_values_len_ge_4(n: Int) {
    proof_assert!(n >= 10);
    proof_assert!(n / 10 >= 10);
    proof_assert!(n / 100 >= 10);
    decimal_values_unfold(n);
    decimal_values_unfold(n / 10);
    decimal_values_len_ge_2(n / 100);
    proof_assert!(decimal_values(n).len() == decimal_values(n / 10).len() + 1);
    proof_assert!(decimal_values(n / 10).len() == decimal_values(n / 100).len() + 1);
    proof_assert!(decimal_values(n).len() >= 4);
}

/// Every value below 10^width has exactly `width` fixed-width decimal bytes.
#[logic]
#[requires(n >= 0)]
#[requires(width >= 1)]
#[requires(n < power_of_ten(width))]
#[ensures(fixed_width_decimal_values(n, width).len() == width)]
#[variant(width)]
pub(crate) fn fixed_width_decimal_values_len(n: Int, width: Int) {
    fixed_width_decimal_values_unfold(n, width);
    if width == 1 {
        power_of_ten_unfold(1);
        power_of_ten_unfold(0);
        proof_assert!(power_of_ten(1) == 10);
        decimal_values_one_digit_len(n);
    } else {
        power_of_ten_unfold(width);
        proof_assert!(n / 10 >= 0);
        proof_assert!(width - 1 >= 1);
        proof_assert!(n / 10 < power_of_ten(width - 1));
        fixed_width_decimal_values_len(n / 10, width - 1);
    }
}

/// A width-2 digit pair has exactly the same sequence meaning as the model.
#[logic]
#[requires(0 <= pair && pair < 100)]
#[ensures(Seq::singleton(48 + pair / 10).push_back(48 + pair % 10)
    == fixed_width_decimal_values(pair, 2))]
#[ensures(fixed_width_decimal_values(pair, 2).len() == 2)]
#[ensures(fixed_width_decimal_values(pair, 2)[0] == 48 + pair / 10)]
#[ensures(fixed_width_decimal_values(pair, 2)[1] == 48 + pair % 10)]
pub(crate) fn fixed_width_decimal_values_pair(pair: Int) {
    power_of_ten_unfold(2);
    power_of_ten_unfold(1);
    proof_assert!(power_of_ten(2) == 100);
    proof_assert!(power_of_ten(1) == 10);
    proof_assert!(pair / 10 >= 0);
    proof_assert!(pair / 10 < power_of_ten(1));
    fixed_width_decimal_values_unfold(pair, 2);
    fixed_width_decimal_values_unfold(pair / 10, 1);
    decimal_values_unfold(pair / 10);
}

/// A two-digit value already has the same unpadded and fixed-width sequence.
#[logic]
#[requires(10 <= n && n < 100)]
#[ensures(fixed_width_decimal_values(n, 2) == decimal_values(n))]
pub(crate) fn fixed_width_decimal_values_2_is_decimal(n: Int) {
    fixed_width_decimal_values_unfold(n, 2);
    fixed_width_decimal_values_unfold(n / 10, 1);
    decimal_values_unfold(n);
    decimal_values_unfold(n / 10);
    proof_assert!(1 <= n / 10 && n / 10 < 10);
    proof_assert!(fixed_width_decimal_values(n / 10, 1) == decimal_values(n / 10));
    proof_assert!(fixed_width_decimal_values(n, 2) == decimal_values(n));
}

/// Splits a three-digit value into its leading digit and a two-digit suffix.
#[logic]
#[requires(100 <= n && n < 1_000)]
#[ensures(decimal_values(n)
    == decimal_values(n / 100).concat(fixed_width_decimal_values(n % 100, 2)))]
pub(crate) fn decimal_values_compose_1x2(n: Int) {
    let high = n / 100;
    let low = n % 100;
    proof_assert!(n == high * 100 + low);
    proof_assert!(1 <= high && high < 10);
    proof_assert!(0 <= low && low < 100);
    proof_assert!(n / 10 == 10 * high + low / 10);
    proof_assert!(n % 10 == low % 10);
    proof_assert!((n / 10) / 10 == high);
    proof_assert!((n / 10) % 10 == low / 10);
    decimal_values_unfold(n);
    decimal_values_unfold(n / 10);
    decimal_values_unfold(high);
    fixed_width_decimal_values_unfold(low, 2);
    fixed_width_decimal_values_unfold(low / 10, 1);
    decimal_values_unfold(low / 10);
    proof_assert!(decimal_values(n)
        == decimal_values(high).concat(fixed_width_decimal_values(low, 2)));
}

/// Relates the tens digit of the low pair to the final two digits of a quad.
#[logic]
#[requires(0 <= quad && quad < 10_000)]
#[ensures(quad / 10 == (quad / 100) * 10 + (quad % 100) / 10)]
#[ensures((quad % 100) / 10 == (quad / 10) % 10)]
pub(crate) fn decimal_quad_split_digits(quad: Int) {
    let q = quad / 100;
    let r = quad % 100;
    proof_assert!(quad == 100 * q + r);
    proof_assert!(0 <= r && r < 100);
    proof_assert!(0 <= q && q < 100);
    proof_assert!(quad / 10 == 10 * q + r / 10);
    proof_assert!(r / 10 < 10);
    proof_assert!((10 * q + r / 10) % 10 == r / 10);
}

/// Splits a four-digit model into two two-digit models.
#[logic]
#[requires(0 <= quad && quad < 10_000)]
#[ensures(fixed_width_decimal_values(quad, 4)
    == fixed_width_decimal_values(quad / 100, 2)
        .concat(fixed_width_decimal_values(quad % 100, 2)))]
pub(crate) fn fixed_width_decimal_values_compose_2x2(quad: Int) {
    power_of_ten_unfold(4);
    power_of_ten_unfold(3);
    power_of_ten_unfold(2);
    power_of_ten_unfold(1);
    proof_assert!(power_of_ten(4) == 10_000);
    proof_assert!(power_of_ten(3) == 1_000);
    proof_assert!(power_of_ten(2) == 100);
    proof_assert!(power_of_ten(1) == 10);

    proof_assert!(quad / 10 >= 0 && quad / 10 < power_of_ten(3));
    proof_assert!(quad / 100 >= 0 && quad / 100 < power_of_ten(2));
    proof_assert!(quad / 1_000 >= 0 && quad / 1_000 < power_of_ten(1));
    proof_assert!(quad % 100 >= 0 && quad % 100 < power_of_ten(2));
    proof_assert!((quad % 100) / 10 >= 0 && (quad % 100) / 10 < power_of_ten(1));

    fixed_width_decimal_values_unfold(quad, 4);
    fixed_width_decimal_values_unfold(quad / 10, 3);
    fixed_width_decimal_values_unfold(quad / 100, 2);
    fixed_width_decimal_values_unfold(quad / 1_000, 1);
    fixed_width_decimal_values_unfold(quad % 100, 2);
    fixed_width_decimal_values_unfold((quad % 100) / 10, 1);
    decimal_values_unfold((quad % 100) / 10);

    decimal_quad_split_digits(quad);
    proof_assert!(quad / 1_000 == (quad / 100) / 10);
    proof_assert!((quad % 100) % 10 == quad % 10);
}

/// Splits a canonical value into a prefix and a fixed-width four-digit suffix.
#[logic]
#[requires(n >= 1_000)]
#[ensures(decimal_values(n) == (if n / 10_000 == 0 {
    Seq::empty()
} else {
    decimal_values(n / 10_000)
}).concat(fixed_width_decimal_values(n % 10_000, 4)))]
pub(crate) fn decimal_values_split_4(n: Int) {
    let high = n / 10_000;
    let low = n % 10_000;
    proof_assert!(n == high * 10_000 + low);
    proof_assert!(0 <= low && low < 10_000);
    proof_assert!(0 <= low / 1_000 && low / 1_000 < 10);
    decimal_values_one_digit(low / 1_000);

    if high == 0 {
        proof_assert!(n < 10_000);
        proof_assert!(low == n);
        fixed_width_decimal_values_unfold(low, 4);
        fixed_width_decimal_values_unfold(low / 10, 3);
        fixed_width_decimal_values_unfold(low / 100, 2);
        fixed_width_decimal_values_unfold(low / 1_000, 1);
        decimal_values_unfold(n);
        decimal_values_unfold(n / 10);
        decimal_values_unfold(n / 100);
        decimal_values_unfold(n / 1_000);
    } else {
        proof_assert!(0 < high);
        fixed_width_decimal_values_unfold(low, 4);
        fixed_width_decimal_values_unfold(low / 10, 3);
        fixed_width_decimal_values_unfold(low / 100, 2);
        fixed_width_decimal_values_unfold(low / 1_000, 1);
        decimal_values_unfold(n);
        decimal_values_unfold(n / 10);
        decimal_values_unfold(n / 100);
        decimal_values_unfold(n / 1_000);
        proof_assert!(n / 1_000 == 10 * high + low / 1_000);
        proof_assert!(n / 100 == 100 * high + low / 100);
        proof_assert!(n / 10 == 1_000 * high + low / 10);
        proof_assert!(n == 10_000 * high + low);
        proof_assert!((n / 1_000) % 10 == low / 1_000);
        proof_assert!((n / 100) % 10 == (low / 100) % 10);
        proof_assert!((n / 10) % 10 == (low / 10) % 10);
        proof_assert!(n % 10 == low % 10);
        proof_assert!((n / 1_000) / 10 == high);
        proof_assert!((n / 100) / 10 == n / 1_000);
        proof_assert!((n / 10) / 10 == n / 100);
        proof_assert!(decimal_values(n) == decimal_values(high)
            .push_back(48 + low / 1_000)
            .push_back(48 + (low / 100) % 10)
            .push_back(48 + (low / 10) % 10)
            .push_back(48 + low % 10));
        proof_assert!(fixed_width_decimal_values(low, 4)
            == Seq::singleton(48 + low / 1_000)
                .push_back(48 + (low / 100) % 10)
                .push_back(48 + (low / 10) % 10)
                .push_back(48 + low % 10));
        decimal_seq_snoc_singleton(decimal_values(high), 48 + low / 1_000);
        decimal_seq_concat_snoc(
            decimal_values(high),
            Seq::singleton(48 + low / 1_000),
            48 + (low / 100) % 10,
        );
    }
}
