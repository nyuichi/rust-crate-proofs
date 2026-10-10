use core::str;
use core::mem::MaybeUninit;
use creusot_std::std::option::OptionExt;

#[allow(unused_imports)]
use creusot_std::prelude::{
    bitwise_proof, check, ensures, logic, pearlite, proof_assert, requires, snapshot,
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

/// An injective mathematical state view for each buffer slot. `None` maps to
/// -1, while every initialized u8 maps to its value in 0..=255. This is only a
/// ghost view; it does not inspect or initialize memory.
#[logic(open)]
#[ensures(result.len() == slots.len())]
pub(crate) fn logical_slot_states(slots: Seq<MaybeUninit<u8>>) -> Seq<Int> {
    slots.map(|slot: MaybeUninit<u8>| {
        pearlite! {
            if slot@ == None { -1 } else { slot@.unwrap_logic()@ }
        }
    })
}

/// On an initialized range, the byte view and the injective slot-state view
/// agree. The byte view uses zero as a filler for uninitialized slots, while
/// this precondition excludes those slots.
#[logic]
#[requires(forall<i: Int> 0 <= i && i < slots.len() ==> slots[i]@ != None)]
#[ensures(result)]
#[ensures(logical_slot_bytes(slots) == logical_slot_states(slots))]
pub(crate) fn logical_slot_bytes_equal_states_initialized(
    slots: Seq<MaybeUninit<u8>>,
) -> bool {
    proof_assert!(forall<i: Int> 0 <= i && i < slots.len() ==>
        logical_slot_bytes(slots)[i] == logical_slot_states(slots)[i]);
    proof_assert!(logical_slot_bytes(slots) == logical_slot_states(slots));
    true
}

#[logic]
#[requires(0 <= start && start <= end && end <= slots.len())]
#[ensures(logical_slot_states(slots.subsequence(start, end))
    == logical_slot_states(slots).subsequence(start, end))]
pub(crate) fn logical_slot_states_subsequence(
    slots: Seq<MaybeUninit<u8>>,
    start: Int,
    end: Int,
) {
    let mapped_slice = logical_slot_states(slots.subsequence(start, end));
    let sliced_map = logical_slot_states(slots).subsequence(start, end);
    proof_assert!(mapped_slice.len() == end - start);
    proof_assert!(sliced_map.len() == end - start);
    proof_assert!(forall<i: Int> 0 <= i && i < end - start ==>
        mapped_slice[i] == sliced_map[i]);
}

/// Lift an unchanged initialized-state prefix of a suffix view to the outer
/// buffer, preserving the exact MaybeUninit state of each slot.
#[logic]
#[requires(0 <= offset && offset <= old_outer.len() && offset <= new_outer.len())]
#[requires(old_outer.len() == new_outer.len())]
#[requires(old_suffix == old_outer.subsequence(offset, old_outer.len()))]
#[requires(new_suffix == new_outer.subsequence(offset, new_outer.len()))]
#[requires(0 <= width && width <= old_suffix.len() && width <= new_suffix.len())]
#[requires(logical_slot_states(old_suffix).subsequence(0, width)
    == logical_slot_states(new_suffix).subsequence(0, width))]
#[ensures(forall<i: Int> offset <= i && i < offset + width
    ==> old_outer[i]@ == new_outer[i]@)]
pub(crate) fn range_from_prefix_raw_frame(
    old_outer: Seq<MaybeUninit<u8>>,
    new_outer: Seq<MaybeUninit<u8>>,
    old_suffix: Seq<MaybeUninit<u8>>,
    new_suffix: Seq<MaybeUninit<u8>>,
    offset: Int,
    width: Int,
) {
    logical_slot_states_subsequence(old_outer, offset, old_outer.len());
    logical_slot_states_subsequence(new_outer, offset, new_outer.len());
    logical_slot_states_subsequence(old_suffix, 0, width);
    logical_slot_states_subsequence(new_suffix, 0, width);
    proof_assert!(forall<i: Int> offset <= i && i < offset + width ==>
        logical_slot_states(old_outer)[i] == logical_slot_states(new_outer)[i]);
    proof_assert!(forall<i: Int> offset <= i && i < offset + width ==>
        old_outer[i]@ == new_outer[i]@);
}

#[logic]
#[requires(0 <= start && start <= slots.len())]
#[requires(forall<i: Int> start <= i && i < slots.len() ==> slots[i]@ != None)]
#[ensures(forall<i: Int> start <= i && i < slots.len() ==>
    logical_slot_states(slots)[i] != -1)]
pub(crate) fn initialized_slot_suffix_non_sentinel(
    slots: Seq<MaybeUninit<u8>>,
    start: Int,
) {
    proof_assert!(forall<i: Int> start <= i && i < slots.len() ==>
        logical_slot_states(slots)[i] != -1);
}

#[logic]
#[requires(0 <= start && start <= slots.len())]
#[requires(forall<i: Int> start <= i && i < slots.len() ==>
    logical_slot_states(slots)[i] != -1)]
#[ensures(result)]
#[ensures(forall<i: Int> start <= i && i < slots.len() ==> slots[i]@ != None)]
pub(crate) fn logical_slot_states_suffix_initialized(
    slots: Seq<MaybeUninit<u8>>,
    start: Int,
) -> bool {
    pearlite! {
        forall<i: Int> start <= i && i < slots.len() ==> slots[i]@ != None
    }
}

#[logic]
#[requires(0 <= offset && offset <= outer_states.len())]
#[requires(local_states == outer_states.subsequence(offset, outer_states.len()))]
#[requires(0 <= start && start <= local_states.len())]
#[requires(forall<j: Int> start <= j && j < local_states.len() ==>
    local_states[j] != -1)]
#[ensures(forall<i: Int> offset + start <= i && i < outer_states.len() ==>
    outer_states[i] != -1)]
pub(crate) fn range_from_initialized_suffix_projection(
    outer_states: Seq<Int>,
    local_states: Seq<Int>,
    offset: Int,
    start: Int,
) {
    proof_assert!(forall<i: Int>
        offset + start <= i && i < outer_states.len() ==>
            0 <= i - offset && i - offset < local_states.len());
    proof_assert!(forall<i: Int>
        offset + start <= i && i < outer_states.len() ==>
            local_states[i - offset] != -1);
    proof_assert!(forall<i: Int>
        offset + start <= i && i < outer_states.len() ==>
            outer_states[i] == local_states[i - offset]);
    proof_assert!(forall<i: Int>
        offset + start <= i && i < outer_states.len() ==>
            outer_states[i] != -1);
}

#[logic]
#[requires(0 <= start && start <= end && end <= slots.len())]
#[ensures(logical_slot_bytes(slots.subsequence(start, end))
    == logical_slot_bytes(slots).subsequence(start, end))]
pub fn logical_slot_bytes_subsequence(
    slots: Seq<MaybeUninit<u8>>,
    start: Int,
    end: Int,
) {
    let mapped_slice = logical_slot_bytes(slots.subsequence(start, end));
    let sliced_map = logical_slot_bytes(slots).subsequence(start, end);
    proof_assert!(mapped_slice.len() == end - start);
    proof_assert!(sliced_map.len() == end - start);
    proof_assert!(forall<i: Int> 0 <= i && i < end - start ==>
        mapped_slice[i] == sliced_map[i]);
}

/// Convert exact initialized-slot contents into the corresponding byte model.
#[logic]
#[requires(slots.len() == digits.len())]
#[requires(forall<i: Int> 0 <= i && i < slots.len() ==>
    slots[i]@ != None)]
#[requires(forall<i: Int> 0 <= i && i < slots.len() ==>
    slots[i]@.unwrap_logic()@ == digits[i])]
#[ensures(result == logical_slot_bytes(slots))]
#[ensures(result == digits)]
#[ensures(result.len() == slots.len())]
pub(crate) fn logical_slot_bytes_initialized_range(
    slots: Seq<MaybeUninit<u8>>,
    digits: Seq<Int>,
) -> Seq<Int> {
    let bytes = logical_slot_bytes(slots);
    proof_assert!(bytes.len() == digits.len());
    proof_assert!(forall<i: Int> 0 <= i && i < slots.len() ==>
        bytes[i] == digits[i]);
    proof_assert!(bytes == digits);
    bytes
}


#[logic]
#[requires(0 <= start && start <= middle && middle <= end && end <= slots.len())]
#[ensures(logical_slot_bytes(slots.subsequence(start, end))
    == logical_slot_bytes(slots.subsequence(start, middle)).concat(
        logical_slot_bytes(slots.subsequence(middle, end))
    ))]
pub fn logical_slot_bytes_split(
    slots: Seq<MaybeUninit<u8>>,
    start: Int,
    middle: Int,
    end: Int,
) {
    let bytes = logical_slot_bytes(slots);
    let _ = sequence_subsequence_split(bytes, start, middle, end);
    let _ = logical_slot_bytes_subsequence(slots, start, end);
    let _ = logical_slot_bytes_subsequence(slots, start, middle);
    let _ = logical_slot_bytes_subsequence(slots, middle, end);
}

#[logic]
#[requires(0 <= start && start <= middle && middle <= end && end <= seq.len())]
#[ensures(seq.subsequence(start, end)
    == seq.subsequence(start, middle).concat(seq.subsequence(middle, end)))]
pub fn sequence_subsequence_split(
    seq: Seq<Int>,
    start: Int,
    middle: Int,
    end: Int,
) {
    let whole = seq.subsequence(start, end);
    let left = seq.subsequence(start, middle);
    let right = seq.subsequence(middle, end);
    let combined = left.concat(right);
    proof_assert!(whole.len() == end - start);
    proof_assert!(left.len() == middle - start);
    proof_assert!(right.len() == end - middle);
    proof_assert!(combined.len() == end - start);
    proof_assert!(forall<i: Int> 0 <= i && i < middle - start ==>
        whole[i] == combined[i]);
    proof_assert!(forall<i: Int> middle - start <= i && i < end - start ==>
        whole[i] == combined[i]);
}

#[logic]
#[requires(a.len() == 2 && b.len() == 2)]
#[ensures(a.concat(b)[0] == a[0])]
#[ensures(a.concat(b)[1] == a[1])]
#[ensures(a.concat(b)[2] == b[0])]
#[ensures(a.concat(b)[3] == b[1])]
pub fn concat_two_get_digits(a: Seq<Int>, b: Seq<Int>) {
    proof_assert!(a.concat(b).len() == 4);
    proof_assert!(a.concat(b)[0] == a[0]);
    proof_assert!(a.concat(b)[1] == a[1]);
    proof_assert!(a.concat(b)[2] == b[0]);
    proof_assert!(a.concat(b)[3] == b[1]);
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

/// i64 magnitudes use at most 19 digits; its signed buffer has room for 20.
#[logic]
#[requires(i64::MIN@ <= n && n <= i64::MAX@)]
#[ensures(decimal_values(if n < 0 { -n } else { n }).len() <= 19)]
#[ensures(signed_decimal_values(n).len() <= 20)]
pub(crate) fn i64_signed_decimal_capacity(n: Int) {
    let magnitude = if n < 0 { -n } else { n };
    let _ = power_of_ten_16();
    power_of_ten_unfold(3);
    power_of_ten_unfold(2);
    power_of_ten_unfold(1);
    power_of_ten_unfold(0);
    let _ = power_of_ten_add(16, 3);
    proof_assert!(power_of_ten(19) == 10_000_000_000_000_000_000);
    proof_assert!(0 <= magnitude);
    proof_assert!(magnitude <= -i64::MIN@);
    proof_assert!(-i64::MIN@ < power_of_ten(19));
    proof_assert!(magnitude < power_of_ten(19));
    decimal_len_bounded(magnitude, 19);
    proof_assert!(decimal_values(magnitude).len() <= 19);
    proof_assert!(signed_decimal_values(n).len() <= decimal_values(magnitude).len() + 1);
    proof_assert!(signed_decimal_values(n).len() <= 20);
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

/// Every element in the canonical unsigned decimal model is an ASCII digit.
#[logic]
#[requires(n >= 0)]
#[ensures(forall<i: Int> 0 <= i && i < decimal_values(n).len() ==>
    48 <= decimal_values(n)[i] && decimal_values(n)[i] <= 57)]
#[variant(n)]
pub(crate) fn decimal_values_ascii(n: Int) {
    decimal_values_unfold(n);
    if n < 10 {
        proof_assert!(decimal_values(n) == Seq::singleton(48 + n));
        proof_assert!(decimal_values(n).len() == 1);
        proof_assert!(48 <= 48 + n && 48 + n <= 57);
        proof_assert!(forall<i: Int> 0 <= i && i < decimal_values(n).len() ==>
            48 <= decimal_values(n)[i] && decimal_values(n)[i] <= 57);
    } else {
        let prefix = decimal_values(n / 10);
        decimal_values_ascii(n / 10);
        proof_assert!(0 <= n % 10 && n % 10 < 10);
        proof_assert!(decimal_values(n) == prefix.push_back(48 + n % 10));
        proof_assert!(decimal_values(n).len() == prefix.len() + 1);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            decimal_values(n)[i] == prefix[i]);
        proof_assert!(decimal_values(n)[prefix.len()] == 48 + n % 10);
        proof_assert!(48 <= 48 + n % 10 && 48 + n % 10 <= 57);
        proof_assert!(forall<i: Int> 0 <= i && i < decimal_values(n).len() ==>
            48 <= decimal_values(n)[i] && decimal_values(n)[i] <= 57);
    }
}

/// Every element in the canonical signed decimal model is an ASCII minus sign
/// or an ASCII digit.
#[logic]
#[ensures(forall<i: Int> 0 <= i && i < signed_decimal_values(n).len() ==>
    signed_decimal_values(n)[i] == 45
        || (48 <= signed_decimal_values(n)[i] && signed_decimal_values(n)[i] <= 57))]
pub(crate) fn signed_decimal_values_ascii(n: Int) {
    if n < 0 {
        decimal_values_ascii(-n);
        proof_assert!(signed_decimal_values(n).len() == decimal_values(-n).len() + 1);
        proof_assert!(signed_decimal_values(n)[0] == 45);
        proof_assert!(forall<i: Int> 1 <= i && i < signed_decimal_values(n).len() ==>
            signed_decimal_values(n)[i] == decimal_values(-n)[i - 1]);
        proof_assert!(forall<i: Int> 0 <= i && i < signed_decimal_values(n).len() ==>
            signed_decimal_values(n)[i] == 45
                || (48 <= signed_decimal_values(n)[i]
                    && signed_decimal_values(n)[i] <= 57));
    } else {
        decimal_values_ascii(n);
        proof_assert!(signed_decimal_values(n) == decimal_values(n));
    }
}

/// Every byte in the formatter's canonical whole-integer model is ASCII.
#[logic]
#[ensures(forall<i: Int> 0 <= i && i < integer_decimal_values(value).len() ==>
    integer_decimal_values(value)[i] == 45
        || (48 <= integer_decimal_values(value)[i]
            && integer_decimal_values(value)[i] <= 57))]
pub(crate) fn integer_decimal_values_ascii<I: Integer>(value: I) {
    signed_decimal_values_ascii(integer_value(value));
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
        proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
            == integer_decimal_values(i));
        proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@).len()
            == 40 - start@);
        proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@).len()
            == integer_decimal_values(i).len());
        proof_assert! {
            let _ = integer_decimal_values_ascii(i);
            forall<j: Int> start@ <= j && j < 40 ==>
                self.bytes@[j]@ == integer_decimal_values(i)[j - start@]
        };
        proof_assert!(forall<j: Int> start@ <= j && j < 40 ==>
            self.bytes@[j]@ < 128);
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

/// Construct a `str` view from a model buffer suffix after proving ASCII.
#[requires(start@ <= buf@.len())]
#[requires(forall<i: Int> start@ <= i && i < buf@.len() ==> buf@[i]@ < 128)]
#[ensures(result@.to_bytes() == buf@.subsequence(start@, buf@.len()))]
unsafe fn decimal_slice_to_str(buf: &[u8], start: usize) -> &str {
    proof_assert! {
        let _ = crate::ascii::ascii_bytes_are_utf8(
            buf@.subsequence(start@, buf@.len()),
        );
        exists<characters: Seq<char>>
            characters.to_bytes() == buf@.subsequence(start@, buf@.len())
    };
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

// Phase 6: proved canonical 10^16 chunk composition support.
/// The powers-of-ten model is multiplicative across addition of exponents.
#[logic]
#[requires(exponent >= 0)]
#[ensures(power_of_ten(exponent) >= 1)]
#[variant(exponent)]
pub(crate) fn power_of_ten_positive(exponent: Int) {
    power_of_ten_unfold(exponent);
    if exponent == 0 {
        power_of_ten_unfold(0);
        proof_assert!(power_of_ten(0) == 1);
    } else {
        proof_assert!(exponent > 0);
        power_of_ten_positive(exponent - 1);
        proof_assert!(power_of_ten(exponent - 1) >= 1);
        proof_assert!(power_of_ten(exponent) == 10 * power_of_ten(exponent - 1));
        proof_assert!(power_of_ten(exponent) >= 1);
    }
}

#[logic]
#[requires(a >= 0)]
#[requires(b >= 0)]
#[ensures(power_of_ten(a + b) == power_of_ten(a) * power_of_ten(b))]
#[variant(b)]
pub(crate) fn power_of_ten_add(a: Int, b: Int) {
    power_of_ten_unfold(b);
    if b == 0 {
        power_of_ten_unfold(0);
        proof_assert!(power_of_ten(0) == 1);
        proof_assert!(power_of_ten(a + 0) == power_of_ten(a));
    } else {
        proof_assert!(b > 0);
        proof_assert!(a + b >= 0);
        power_of_ten_unfold(a + b);
        power_of_ten_add(a, b - 1);
        power_of_ten_unfold(b - 1);
        proof_assert!(a + b == a + (b - 1) + 1);
        proof_assert!(power_of_ten(a + b) == 10 * power_of_ten(a + b - 1));
        proof_assert!(power_of_ten(b) == 10 * power_of_ten(b - 1));
        proof_assert!(power_of_ten(a + b) == power_of_ten(a) * power_of_ten(b));
    }
}

/// Euclidean quotient and remainder after splitting a radix that is a
/// multiple of ten. This packages the division facts used by decimal models.
#[logic]
#[requires(high >= 0)]
#[requires(base > 0)]
#[requires(rem >= 0)]
#[requires(rem < base)]
#[ensures((high * base + rem) / base == high)]
#[ensures((high * base + rem) % base == rem)]
pub(crate) fn euclidean_mul_add(high: Int, base: Int, rem: Int) {
    proof_assert!(high * base + rem == high * base + rem);
}

#[logic]
#[requires(high >= 0)]
#[requires(base > 0)]
#[requires(low >= 0)]
#[requires(low < 10 * base)]
#[requires(n == high * (10 * base) + low)]
#[ensures(n / 10 == high * base + low / 10)]
#[ensures(n % 10 == low % 10)]
#[ensures((n / 10) / base == high)]
#[ensures((n / 10) % base == low / 10)]
pub(crate) fn decimal_division_split_10(
    n: Int,
    high: Int,
    base: Int,
    low: Int,
) {
    proof_assert!(low == 10 * (low / 10) + low % 10);
    proof_assert!(n == (n / 10) * 10 + n % 10);
    proof_assert!((n / 10)
        == ((n / 10) / base) * base + (n / 10) % base);
    proof_assert!(0 <= low % 10 && low % 10 < 10);
    proof_assert!(low / 10 < base);
    proof_assert!(n == 10 * (high * base + low / 10) + low % 10);
    proof_assert!(high * base + low / 10 >= 0);
    proof_assert!(n / 10 == high * base + low / 10);
    proof_assert!(n % 10 == low % 10);
    euclidean_mul_add(high, base, low / 10);
}

/// Split a fixed-width sequence at any positive low width.
#[logic]
#[requires(n >= 0)]
#[requires(high_width >= 1)]
#[requires(low_width >= 1)]
#[requires(n < power_of_ten(high_width + low_width))]
#[ensures(fixed_width_decimal_values(n, high_width + low_width)
    == fixed_width_decimal_values(n / power_of_ten(low_width), high_width)
        .concat(fixed_width_decimal_values(n % power_of_ten(low_width), low_width)))]
#[variant(low_width)]
pub(crate) fn fixed_width_decimal_values_split_width(
    n: Int,
    high_width: Int,
    low_width: Int,
) {
    if low_width == 1 {
        let _ = fixed_width_decimal_values_unfold(n, high_width + 1);
        let _ = fixed_width_decimal_values_unfold(n % 10, 1);
        let _ = decimal_values_one_digit(n % 10);
        power_of_ten_unfold(1);
        power_of_ten_unfold(0);
        proof_assert!(power_of_ten(1) == 10);
        proof_assert!(power_of_ten(0) == 1);
        proof_assert!(n % power_of_ten(1) == n % 10);
        proof_assert!(n / power_of_ten(1) == n / 10);
        decimal_seq_snoc_singleton(
            fixed_width_decimal_values(n / 10, high_width),
            48 + n % 10,
        );
    } else {
        power_of_ten_positive(low_width);
        power_of_ten_positive(low_width - 1);
        power_of_ten_positive(high_width + low_width);
        power_of_ten_positive(high_width + low_width - 1);
        power_of_ten_unfold(high_width + low_width);
        power_of_ten_unfold(low_width);
        power_of_ten_unfold(low_width - 1);
        proof_assert!(high_width + low_width > 1);
        proof_assert!(power_of_ten(high_width + low_width)
            == 10 * power_of_ten(high_width + low_width - 1));
        proof_assert!(power_of_ten(low_width)
            == 10 * power_of_ten(low_width - 1));
        proof_assert!(n / 10 >= 0);
        proof_assert!(n / 10 < power_of_ten(high_width + low_width - 1));
        fixed_width_decimal_values_split_width(n / 10, high_width, low_width - 1);

        let high = n / power_of_ten(low_width);
        let low = n % power_of_ten(low_width);
        proof_assert!(n == high * power_of_ten(low_width) + low);
        proof_assert!(0 <= low && low < power_of_ten(low_width));
        proof_assert!(power_of_ten(low_width)
            == 10 * power_of_ten(low_width - 1));
        proof_assert!(high * power_of_ten(low_width)
            == high * 10 * power_of_ten(low_width - 1));
        proof_assert!(n == high * 10 * power_of_ten(low_width - 1) + low);
        decimal_division_split_10(
            n,
            high,
            power_of_ten(low_width - 1),
            low,
        );

        let _ = fixed_width_decimal_values_unfold(n, high_width + low_width);
        let _ = fixed_width_decimal_values_unfold(low, low_width);
        decimal_seq_concat_snoc(
            fixed_width_decimal_values(high, high_width),
            fixed_width_decimal_values(low / 10, low_width - 1),
            48 + n % 10,
        );
        proof_assert!(fixed_width_decimal_values(n, high_width + low_width)
            == fixed_width_decimal_values(high, high_width)
                .concat(fixed_width_decimal_values(low, low_width)));
    }
}

/// Split a canonical decimal sequence at any positive number of low digits.
#[logic]
#[requires(width >= 1)]
#[requires(n >= power_of_ten(width - 1))]
#[ensures(decimal_values(n)
    == (if n / power_of_ten(width) == 0 {
        Seq::empty()
    } else {
        decimal_values(n / power_of_ten(width))
    }).concat(fixed_width_decimal_values(n % power_of_ten(width), width)))]
#[variant(width)]
pub(crate) fn decimal_values_split_width(n: Int, width: Int) {
    if width == 1 {
        power_of_ten_unfold(1);
        power_of_ten_unfold(0);
        proof_assert!(power_of_ten(1) == 10);
        proof_assert!(power_of_ten(0) == 1);
        let high = n / 10;
        let low = n % 10;
        proof_assert!(n == high * 10 + low);
        proof_assert!(0 <= low && low < 10);
        let _ = fixed_width_decimal_values_unfold(low, 1);
        let _ = decimal_values_one_digit(low);
        if high == 0 {
            proof_assert!(n < 10);
            proof_assert!(low == n);
            decimal_values_unfold(n);
            proof_assert!(decimal_values(n) == Seq::singleton(48 + n));
            proof_assert!((if high == 0 { Seq::empty() } else {
                decimal_values(high)
            }).concat(fixed_width_decimal_values(low, 1)) == decimal_values(n));
        } else {
            proof_assert!(high > 0);
            proof_assert!(n >= 10);
            decimal_values_unfold(n);
            decimal_seq_snoc_singleton(decimal_values(high), 48 + low);
            proof_assert!((if high == 0 { Seq::empty() } else {
                decimal_values(high)
            }).concat(fixed_width_decimal_values(low, 1)) == decimal_values(n));
        }
    } else {
        power_of_ten_positive(width);
        power_of_ten_positive(width - 1);
        power_of_ten_positive(width - 2);
        power_of_ten_unfold(width);
        power_of_ten_unfold(width - 1);
        proof_assert!(width - 1 >= 1);
        proof_assert!(power_of_ten(width) == 10 * power_of_ten(width - 1));
        proof_assert!(n / 10 >= power_of_ten(width - 2));
        decimal_values_split_width(n / 10, width - 1);

        let high = n / power_of_ten(width);
        let low = n % power_of_ten(width);
        proof_assert!(n == high * power_of_ten(width) + low);
        proof_assert!(0 <= low && low < power_of_ten(width));
        proof_assert!(power_of_ten(width)
            == 10 * power_of_ten(width - 1));
        proof_assert!(high * power_of_ten(width)
            == high * 10 * power_of_ten(width - 1));
        proof_assert!(n == high * 10 * power_of_ten(width - 1) + low);
        decimal_division_split_10(
            n,
            high,
            power_of_ten(width - 1),
            low,
        );

        decimal_values_unfold(n);
        let _ = fixed_width_decimal_values_unfold(low, width);
        decimal_seq_concat_snoc(
            if high == 0 { Seq::empty() } else { decimal_values(high) },
            fixed_width_decimal_values(low / 10, width - 1),
            48 + n % 10,
        );
        proof_assert!(decimal_values(n)
            == (if high == 0 { Seq::empty() } else { decimal_values(high) })
                .concat(fixed_width_decimal_values(low, width)));
    }
}

/// A concrete power used by the u128 formatter's 16-digit radix.
#[logic]
#[ensures(power_of_ten(16) == 10_000_000_000_000_000)]
pub(crate) fn power_of_ten_16() {
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

/// Canonical decimal values split into an optional high prefix and a padded
/// low 16-digit block. Requires at least 16 canonical digits.
#[logic]
#[requires(n >= 1_000_000_000_000_000)]
#[ensures(decimal_values(n)
    == (if n / 10_000_000_000_000_000 == 0 {
        Seq::empty()
    } else {
        decimal_values(n / 10_000_000_000_000_000)
    }).concat(fixed_width_decimal_values(
        n % 10_000_000_000_000_000,
        16,
    )))]
pub(crate) fn decimal_values_split_16(n: Int) {
    power_of_ten_16();
    power_of_ten_unfold(16);
    proof_assert!(power_of_ten(15) == 1_000_000_000_000_000);
    decimal_values_split_width(n, 16);
    proof_assert!(power_of_ten(16) == 10_000_000_000_000_000);
}

/// Canonical fixed-width sequence composed from two 16-digit radix blocks.
#[logic]
#[requires(n >= 0)]
#[requires(n < 100_000_000_000_000_000_000_000_000_000_000)]
#[ensures(fixed_width_decimal_values(n, 32)
    == fixed_width_decimal_values(n / 10_000_000_000_000_000, 16)
        .concat(fixed_width_decimal_values(n % 10_000_000_000_000_000, 16)))]
pub(crate) fn fixed_width_decimal_values_split_16x16(n: Int) {
    power_of_ten_16();
    power_of_ten_add(16, 16);
    proof_assert!(power_of_ten(32)
        == 100_000_000_000_000_000_000_000_000_000_000);
    fixed_width_decimal_values_split_width(n, 16, 16);
    proof_assert!(power_of_ten(16) == 10_000_000_000_000_000);
}

/// Concatenate the canonical high prefix with a zero-padded low radix block.
#[logic]
#[requires(high > 0)]
#[requires(0 <= low && low < 10_000_000_000_000_000)]
#[ensures(decimal_values(high * 10_000_000_000_000_000 + low)
    == decimal_values(high).concat(fixed_width_decimal_values(low, 16)))]
pub(crate) fn decimal_values_compose_16(high: Int, low: Int) {
    power_of_ten_16();
    decimal_values_split_16(high * 10_000_000_000_000_000 + low);
    proof_assert!(
        (high * 10_000_000_000_000_000 + low) / 10_000_000_000_000_000 == high
    );
    proof_assert!(
        (high * 10_000_000_000_000_000 + low) % 10_000_000_000_000_000 == low
    );
}

/// Compose two consecutive 16-digit low blocks after a nonzero high prefix.
#[logic]
#[requires(top > 0)]
#[requires(0 <= middle && middle < 10_000_000_000_000_000)]
#[requires(0 <= low && low < 10_000_000_000_000_000)]
#[ensures(decimal_values(
    (top * 10_000_000_000_000_000 + middle) * 10_000_000_000_000_000 + low
) == decimal_values(top)
    .concat(fixed_width_decimal_values(middle, 16))
    .concat(fixed_width_decimal_values(low, 16)))]
pub(crate) fn decimal_values_compose_16x2(top: Int, middle: Int, low: Int) {
    let high = top * 10_000_000_000_000_000 + middle;
    decimal_values_compose_16(high, low);
    decimal_values_compose_16(top, middle);
    decimal_seq_concat_assoc(
        decimal_values(top),
        fixed_width_decimal_values(middle, 16),
        fixed_width_decimal_values(low, 16),
    );
}


/// Values at least 10^38 have at least 39 canonical decimal digits.
#[logic]
#[requires(n >= power_of_ten(38))]
#[ensures(decimal_values(n).len() >= 39)]
pub(crate) fn decimal_values_len_ge_39(n: Int) {
    let _ = power_of_ten_16();
    power_of_ten_unfold(16);
    power_of_ten_add(16, 16);
    power_of_ten_unfold(6);
    power_of_ten_unfold(5);
    power_of_ten_unfold(4);
    power_of_ten_unfold(3);
    power_of_ten_unfold(2);
    power_of_ten_unfold(1);
    power_of_ten_unfold(0);
    power_of_ten_add(32, 6);
    proof_assert!(power_of_ten(16) == 10_000_000_000_000_000);
    proof_assert!(power_of_ten(32)
        == 100_000_000_000_000_000_000_000_000_000_000);
    proof_assert!(power_of_ten(38)
        == 100_000_000_000_000_000_000_000_000_000_000_000_000);
    proof_assert!(n >= 100_000_000_000_000_000_000_000_000_000_000_000_000);

    let high = n / 10_000_000_000_000_000;
    let low = n % 10_000_000_000_000_000;
    proof_assert!(n == high * 10_000_000_000_000_000 + low);
    proof_assert!(0 <= low && low < 10_000_000_000_000_000);
    proof_assert!(high >= 10_000_000_000_000_000_000_000);

    let top = high / 10_000_000_000_000_000;
    let middle = high % 10_000_000_000_000_000;
    proof_assert!(high == top * 10_000_000_000_000_000 + middle);
    proof_assert!(0 <= middle && middle < 10_000_000_000_000_000);
    proof_assert!(top >= 1_000_000);

    decimal_values_split_16(n);
    decimal_values_split_16(high);
    proof_assert!(n / 10_000_000_000_000_000 > 0);
    proof_assert!(high / 10_000_000_000_000_000 > 0);
    fixed_width_decimal_values_len(low, 16);
    fixed_width_decimal_values_len(middle, 16);
    decimal_values_len_ge_7(top);
    proof_assert!(decimal_values(n).len() == decimal_values(high).len() + 16);
    proof_assert!(decimal_values(high).len() == decimal_values(top).len() + 16);
    proof_assert!(decimal_values(n).len() >= 39);
}

/// A value at least one million has at least seven canonical digits.
#[logic]
#[requires(n >= 1_000_000)]
#[ensures(decimal_values(n).len() >= 7)]
pub(crate) fn decimal_values_len_ge_7(n: Int) {
    decimal_values_unfold(n);
    decimal_values_unfold(n / 10);
    decimal_values_unfold(n / 100);
    proof_assert!(n / 1_000 >= 1_000);
    decimal_values_len_ge_4(n / 1_000);
    proof_assert!(decimal_values(n).len() >= 7);
}

#[logic(open)]
pub(crate) fn i128_min_magnitude_model() -> Int {
    pearlite! { -i128::MIN@ }
}

#[ensures(result@ == i128_min_magnitude_model())]
pub(crate) fn i128_min_unsigned_magnitude() -> u128 {
    let magnitude = i128::MIN.unsigned_abs();
    proof_assert!(i128::MIN@ < 0);
    proof_assert!(magnitude@ == -i128::MIN@);
    magnitude
}

#[ensures(result@ == -i128::MIN@)]
#[ensures(decimal_values(result@).len() == 39)]
#[ensures(decimal_values(result@).len() <= 39)]
pub(crate) fn i128_min_unsigned_decimal_len() -> u128 {
    let magnitude = i128_min_unsigned_magnitude();
    proof_assert! {
        let _ = decimal_values_len_u128(magnitude);
        decimal_values(magnitude@).len() <= 39
    };
    proof_assert! {
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
        -i128::MIN@ >= power_of_ten(38)
    };
    proof_assert! {
        let _ = decimal_values_len_ge_39(-i128::MIN@);
        decimal_values(-i128::MIN@).len() >= 39
    };
    magnitude
}

#[ensures(signed_decimal_values(i128::MIN@).len() == 40)]
#[ensures(signed_decimal_values(i128::MIN@)[0] == 45)]
#[ensures(signed_decimal_values(i128::MIN@).subsequence(1, 40)
    == decimal_values(-i128::MIN@))]
pub(crate) fn i128_min_signed_decimal_model() {
    let _magnitude = i128_min_unsigned_decimal_len();
}
