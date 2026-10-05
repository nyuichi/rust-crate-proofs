use crate::decimal_pairs::DECIMAL_PAIRS;
use crate::divmod100::divmod100;
use core::mem::MaybeUninit;
#[cfg(all(feature = "no-panic", not(creusot)))]
use no_panic::no_panic;

#[cfg(creusot)]
use crate::decimal_pairs::decimal_pair_correct;
#[cfg(creusot)]
use crate::verification::{
    decimal_values, fixed_width_decimal_values, fixed_width_decimal_values_compose_2x2,
    fixed_width_decimal_values_pair, power_of_ten, concat_two_get_digits,
};
#[cfg(creusot)]
use creusot_std::prelude::{check, ensures, invariant, logic, proof_assert, requires, snapshot, Int, Seq};
#[cfg(creusot)]
use creusot_std::std::option::OptionExt;

#[cfg(creusot)]
mod proof_helpers {
use super::*;

#[logic]
#[requires(exponent > 0)]
#[ensures(power_of_ten(exponent) == 10 * power_of_ten(exponent - 1))]
fn power_of_ten_unfold_local(exponent: Int) {}

#[logic]
#[requires(width >= 8)]
#[ensures(power_of_ten(width) == 10_000 * power_of_ten(width - 4))]
fn power_of_ten_split_4(width: Int) {
    power_of_ten_unfold_local(width);
    power_of_ten_unfold_local(width - 1);
    power_of_ten_unfold_local(width - 2);
    power_of_ten_unfold_local(width - 3);
}

#[logic]
#[ensures(power_of_ten(4) == 10_000)]
fn power_of_ten_4() {
    power_of_ten_unfold_local(4);
    power_of_ten_unfold_local(3);
    power_of_ten_unfold_local(2);
    power_of_ten_unfold_local(1);
    proof_assert!(power_of_ten(0) == 1);
    proof_assert!(power_of_ten(4) == 10_000);
}

#[logic]
#[ensures(power_of_ten(16) == 10_000_000_000_000_000)]
fn power_of_ten_16() {
    power_of_ten_4();
    power_of_ten_split_4(8);
    power_of_ten_split_4(12);
    power_of_ten_split_4(16);
    proof_assert!(power_of_ten(16) == 10_000 * 10_000 * 10_000 * 10_000);
    proof_assert!(power_of_ten(16) == 10_000_000_000_000_000);
}

#[logic]
#[requires(n >= 0)]
#[ensures(n % 10_000 >= 0)]
fn remainder_10_000_nonnegative(n: Int) {}

#[logic]
#[requires(n >= 0)]
#[requires(width >= 1)]
#[requires(n < power_of_ten(width))]
#[ensures(fixed_width_decimal_values(n, width) == if width == 1 {
    decimal_values(n)
} else {
    fixed_width_decimal_values(n / 10, width - 1).push_back(48 + n % 10)
})]
fn fixed_width_decimal_values_unfold_local(n: Int, width: Int) {}

#[logic]
#[requires(n >= 0)]
#[ensures(decimal_values(n) == if n < 10 {
    creusot_std::logic::Seq::singleton(48 + n)
} else {
    decimal_values(n / 10).push_back(48 + n % 10)
})]
fn decimal_values_unfold_local(n: Int) {}

#[logic]
#[requires(0 <= n && n < 10)]
#[ensures(decimal_values(n) == creusot_std::logic::Seq::singleton(48 + n))]
fn decimal_values_one_digit_local(n: Int) {
    decimal_values_unfold_local(n);
}

#[logic]
#[requires(0 <= n && n < 10_000)]
#[ensures(fixed_width_decimal_values(n, 4)
    == Seq::singleton(48 + n / 1_000)
        .push_back(48 + n / 100 % 10)
        .push_back(48 + n / 10 % 10)
        .push_back(48 + n % 10))]
fn fixed_width_decimal_values_expand_4(n: Int) {
    power_of_ten_unfold_local(4);
    power_of_ten_unfold_local(3);
    power_of_ten_unfold_local(2);
    power_of_ten_unfold_local(1);
    proof_assert!(power_of_ten(1) == 10);
    proof_assert!(power_of_ten(2) == 100);
    proof_assert!(power_of_ten(3) == 1_000);
    proof_assert!(power_of_ten(4) == 10_000);
    proof_assert!(0 <= n / 10 && n / 10 < power_of_ten(3));
    proof_assert!(0 <= n / 100 && n / 100 < power_of_ten(2));
    proof_assert!(0 <= n / 1_000 && n / 1_000 < power_of_ten(1));
    fixed_width_decimal_values_unfold_local(n, 4);
    fixed_width_decimal_values_unfold_local(n / 10, 3);
    fixed_width_decimal_values_unfold_local(n / 100, 2);
    fixed_width_decimal_values_unfold_local(n / 1_000, 1);
    decimal_values_one_digit_local(n / 1_000);
}

#[logic]
#[ensures(a.concat(Seq::singleton(x)) == a.push_back(x))]
fn concat_singleton_local(a: Seq<Int>, x: Int) {}

#[logic]
#[ensures(a.concat(b.push_back(x)) == a.concat(b).push_back(x))]
fn concat_push_back_local(a: Seq<Int>, b: Seq<Int>, x: Int) {}

/// Connect a runtime quad and its two runtime pairs to the corresponding
/// four digits in the canonical 16-digit model.
#[logic]
#[requires(n >= 0 && n < 10_000_000_000_000_000 && 0 <= quad_index && quad_index < 4)]
#[requires((quad_index == 3 ==> quad == n % 10_000)
    && (quad_index == 2 ==> quad == n / 10_000 % 10_000)
    && (quad_index == 1 ==> quad == n / 100_000_000 % 10_000)
    && (quad_index == 0 ==> quad == n / 1_000_000_000_000 % 10_000))]
#[requires(0 <= quad && quad < 10_000
    && pair1 == quad / 100 && pair2 == quad % 100
    && 0 <= pair1 && pair1 < 100 && 0 <= pair2 && pair2 < 100)]
#[ensures(result == fixed_width_decimal_values(pair1 * 100 + pair2, 4))]
#[ensures(result.len() == 4)]
#[ensures(forall<digit: Int> 0 <= digit && digit < 4 ==>
    fixed_width_decimal_values(n, 16)[4 * quad_index + digit] == result[digit])]
pub(crate) fn fixed_width_decimal_values_runtime_quad_digits(
    n: Int,
    quad_index: Int,
    quad: Int,
    pair1: Int,
    pair2: Int,
) -> Seq<Int> {
    proof_assert!(quad == pair1 * 100 + pair2);
    fixed_width_decimal_values_chunk_16(n, quad_index);
    fixed_width_decimal_values_compose_2x2(quad);
    fixed_width_decimal_values_pair(pair1);
    fixed_width_decimal_values_pair(pair2);
    concat_two_get_digits(
        fixed_width_decimal_values(pair1, 2),
        fixed_width_decimal_values(pair2, 2),
    );

    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index]
        == fixed_width_decimal_values(quad, 4)[0]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index + 1]
        == fixed_width_decimal_values(quad, 4)[1]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index + 2]
        == fixed_width_decimal_values(quad, 4)[2]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index + 3]
        == fixed_width_decimal_values(quad, 4)[3]);
    proof_assert!(fixed_width_decimal_values(quad, 4)[0]
        == fixed_width_decimal_values(pair1, 2)[0]);
    proof_assert!(fixed_width_decimal_values(quad, 4)[1]
        == fixed_width_decimal_values(pair1, 2)[1]);
    proof_assert!(fixed_width_decimal_values(quad, 4)[2]
        == fixed_width_decimal_values(pair2, 2)[0]);
    proof_assert!(fixed_width_decimal_values(quad, 4)[3]
        == fixed_width_decimal_values(pair2, 2)[1]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index]
        == fixed_width_decimal_values(pair1 * 100 + pair2, 4)[0]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index + 1]
        == fixed_width_decimal_values(pair1 * 100 + pair2, 4)[1]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index + 2]
        == fixed_width_decimal_values(pair1 * 100 + pair2, 4)[2]);
    proof_assert!(fixed_width_decimal_values(n, 16)[4 * quad_index + 3]
        == fixed_width_decimal_values(pair1 * 100 + pair2, 4)[3]);
    proof_assert!(forall<digit: Int>
        0 <= digit && digit < 4 ==>
            digit == 0 || digit == 1 || digit == 2 || digit == 3);
    proof_assert!(forall<digit: Int> 0 <= digit && digit < 4 ==>
        fixed_width_decimal_values(n, 16)[4 * quad_index + digit]
            == fixed_width_decimal_values(pair1 * 100 + pair2, 4)[digit]);
    fixed_width_decimal_values(pair1 * 100 + pair2, 4)
}

/// Split a fixed-width sequence at its final four digits.
#[logic]
#[requires(n >= 0)]
#[requires(width >= 8)]
#[requires(n < power_of_ten(width))]
#[ensures(fixed_width_decimal_values(n, width)
    == fixed_width_decimal_values(n / 10_000, width - 4)
        .concat(fixed_width_decimal_values(n % 10_000, 4)))]
fn fixed_width_decimal_values_split_4(n: Int, width: Int) {
    power_of_ten_split_4(width);
    power_of_ten_4();
    power_of_ten_unfold_local(width);
    power_of_ten_unfold_local(width - 1);
    power_of_ten_unfold_local(width - 2);
    power_of_ten_unfold_local(width - 3);
    proof_assert!(0 <= n / 10_000);
    remainder_10_000_nonnegative(n);
    proof_assert!(n % 10_000 < 10_000);
    proof_assert!(n == (n / 10_000) * 10_000 + n % 10_000);
    proof_assert!(n / 1_000 == (n / 10_000) * 10 + (n % 10_000) / 1_000);
    proof_assert!(n / 100 == (n / 10_000) * 100 + (n % 10_000) / 100);
    proof_assert!(n / 10 == (n / 10_000) * 1_000 + (n % 10_000) / 10);
    proof_assert!(n / 1_000 % 10 == ((n / 10_000) * 10 + (n % 10_000) / 1_000) % 10);
    proof_assert!(n / 1_000 % 10 == ((n % 10_000) / 1_000) % 10);
    proof_assert!(n / 1_000 % 10 == (n % 10_000) / 1_000);
    proof_assert!(n / 100 % 10
        == (10 * ((n / 10_000) * 10) + (n % 10_000) / 100) % 10);
    proof_assert!((10 * ((n / 10_000) * 10) + (n % 10_000) / 100) % 10
        == ((n % 10_000) / 100) % 10);
    proof_assert!(n / 100 % 10 == (n % 10_000) / 100 % 10);
    proof_assert!(n / 10 % 10
        == (10 * ((n / 10_000) * 100) + (n % 10_000) / 10) % 10);
    proof_assert!((10 * ((n / 10_000) * 100) + (n % 10_000) / 10) % 10
        == ((n % 10_000) / 10) % 10);
    proof_assert!(n / 10 % 10 == (n % 10_000) / 10 % 10);
    proof_assert!(n % 10 == (10 * ((n / 10_000) * 1_000) + n % 10_000) % 10);
    proof_assert!((10 * ((n / 10_000) * 1_000) + n % 10_000) % 10
        == (n % 10_000) % 10);
    proof_assert!(n % 10 == (n % 10_000) % 10);
    proof_assert!(0 <= n / 10 && n / 10 < power_of_ten(width - 1));
    proof_assert!(0 <= n / 100 && n / 100 < power_of_ten(width - 2));
    proof_assert!(0 <= n / 1_000 && n / 1_000 < power_of_ten(width - 3));
    proof_assert!(0 <= n / 10_000 && n / 10_000 < power_of_ten(width - 4));
    proof_assert!(n % 10_000 < power_of_ten(4));
    proof_assert!(n == (n / 10_000) * 10_000 + n % 10_000);
    proof_assert!(n % 10_000 < 10_000);

    fixed_width_decimal_values_unfold_local(n, width);
    fixed_width_decimal_values_unfold_local(n / 10, width - 1);
    fixed_width_decimal_values_unfold_local(n / 100, width - 2);
    fixed_width_decimal_values_unfold_local(n / 1_000, width - 3);
    fixed_width_decimal_values_unfold_local(n % 10_000, 4);
    fixed_width_decimal_values_unfold_local((n % 10_000) / 10, 3);
    fixed_width_decimal_values_unfold_local((n % 10_000) / 100, 2);
    fixed_width_decimal_values_unfold_local((n % 10_000) / 1_000, 1);
    decimal_values_one_digit_local((n % 10_000) / 1_000);

    proof_assert!(n / 1_000 % 10 == (n % 10_000) / 1_000);
    proof_assert!(n / 100 % 10 == (n % 10_000) / 100 % 10);
    proof_assert!(n / 10 % 10 == (n % 10_000) / 10 % 10);
    proof_assert!(n % 10 == (n % 10_000) % 10);
    fixed_width_decimal_values_expand_4(n % 10_000);
    let high = fixed_width_decimal_values(n / 10_000, width - 4);
    let low = fixed_width_decimal_values(n % 10_000, 4);
    let digit3 = 48 + n / 1_000 % 10;
    let digit2 = 48 + n / 100 % 10;
    let digit1 = 48 + n / 10 % 10;
    let digit0 = 48 + n % 10;
    proof_assert!(fixed_width_decimal_values(n, width)
        == fixed_width_decimal_values(n / 10, width - 1).push_back(digit0));
    proof_assert!(fixed_width_decimal_values(n / 10, width - 1)
        == fixed_width_decimal_values(n / 100, width - 2).push_back(digit1));
    proof_assert!(fixed_width_decimal_values(n / 100, width - 2)
        == fixed_width_decimal_values(n / 1_000, width - 3).push_back(digit2));
    proof_assert!(fixed_width_decimal_values(n / 1_000, width - 3)
        == high.push_back(digit3));
    proof_assert!(fixed_width_decimal_values(n, width)
        == high.push_back(digit3).push_back(digit2).push_back(digit1).push_back(digit0));
    proof_assert!(low
        == Seq::singleton(digit3)
            .push_back(digit2)
            .push_back(digit1)
            .push_back(digit0));
    concat_singleton_local(high, digit3);
    concat_push_back_local(high, Seq::singleton(digit3), digit2);
    concat_push_back_local(high, Seq::singleton(digit3).push_back(digit2), digit1);
    concat_push_back_local(
        high,
        Seq::singleton(digit3).push_back(digit2).push_back(digit1),
        digit0,
    );
    proof_assert!(high.concat(low)
        == high.push_back(digit3).push_back(digit2).push_back(digit1).push_back(digit0));
}

#[logic]
#[requires(n >= 0)]
#[requires(n < 10_000_000_000_000_000)]
#[ensures(fixed_width_decimal_values(n, 16)
    == fixed_width_decimal_values(n / 1_000_000_000_000, 4)
        .concat(fixed_width_decimal_values(n / 100_000_000 % 10_000, 4))
        .concat(fixed_width_decimal_values(n / 10_000 % 10_000, 4))
        .concat(fixed_width_decimal_values(n % 10_000, 4)))]
fn fixed_width_decimal_values_split_16(n: Int) {
    power_of_ten_16();
    proof_assert!(n < power_of_ten(16));
    fixed_width_decimal_values_split_4(n, 16);
    proof_assert!(n / 10_000 < power_of_ten(12));
    fixed_width_decimal_values_split_4(n / 10_000, 12);
    proof_assert!(n / 100_000_000 < power_of_ten(8));
    fixed_width_decimal_values_split_4(n / 100_000_000, 8);
    proof_assert!(n / 100_000_000 / 10_000 == n / 1_000_000_000_000);
    proof_assert!(n / 10_000 / 10_000 == n / 100_000_000);
}

#[logic]
#[requires(0 <= n && n < 10_000)]
#[ensures(fixed_width_decimal_values(n, 4).len() == 4)]
fn fixed_width_decimal_values_len_4_local(n: Int) {
    power_of_ten_unfold_local(4);
    power_of_ten_unfold_local(3);
    power_of_ten_unfold_local(2);
    power_of_ten_unfold_local(1);
    proof_assert!(power_of_ten(1) == 10);
    proof_assert!(power_of_ten(2) == 100);
    proof_assert!(power_of_ten(3) == 1_000);
    proof_assert!(power_of_ten(4) == 10_000);
    proof_assert!(0 <= n / 10 && n / 10 < power_of_ten(3));
    proof_assert!(0 <= n / 100 && n / 100 < power_of_ten(2));
    proof_assert!(0 <= n / 1_000 && n / 1_000 < power_of_ten(1));
    fixed_width_decimal_values_unfold_local(n, 4);
    fixed_width_decimal_values_unfold_local(n / 10, 3);
    fixed_width_decimal_values_unfold_local(n / 100, 2);
    fixed_width_decimal_values_unfold_local(n / 1_000, 1);
    decimal_values_one_digit_local(n / 1_000);
    proof_assert!(decimal_values(n / 1_000).len() == 1);
    proof_assert!(fixed_width_decimal_values(n / 1_000, 1).len() == 1);
    proof_assert!(fixed_width_decimal_values(n / 100, 2).len() == 2);
    proof_assert!(fixed_width_decimal_values(n / 10, 3).len() == 3);
    proof_assert!(fixed_width_decimal_values(n, 4).len() == 4);
}

#[logic]
#[requires(a.len() == 4 && b.len() == 4 && c.len() == 4 && d.len() == 4)]
#[ensures(forall<i: Int> 0 <= i && i < 4 ==>
    a.concat(b).concat(c).concat(d)[4 + i] == b[i])]
fn concat_four_get_second(a: Seq<Int>, b: Seq<Int>, c: Seq<Int>, d: Seq<Int>) {
    let ab = a.concat(b);
    let abc = ab.concat(c);
    proof_assert!(ab.len() == 8);
    proof_assert!(abc.len() == 12);
    proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
        abc.concat(d)[4 + i] == abc[4 + i]);
    proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
        abc[4 + i] == ab[4 + i]);
    proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
        ab[4 + i] == b[i]);
}

#[logic]
#[requires(a.len() == 4 && b.len() == 4 && c.len() == 4 && d.len() == 4)]
#[ensures(forall<i: Int> 0 <= i && i < 4 ==>
    a.concat(b).concat(c).concat(d)[8 + i] == c[i])]
fn concat_four_get_third(a: Seq<Int>, b: Seq<Int>, c: Seq<Int>, d: Seq<Int>) {
    let ab = a.concat(b);
    let abc = ab.concat(c);
    proof_assert!(ab.len() == 8);
    proof_assert!(abc.len() == 12);
    proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
        abc.concat(d)[8 + i] == abc[8 + i]);
    proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
        abc[8 + i] == c[i]);
}

#[logic]
#[requires(n >= 0)]
#[requires(n < 10_000_000_000_000_000)]
#[requires(0 <= quad_index && quad_index < 4)]
#[ensures(forall<i: Int> 0 <= i && i < 4 ==>
    fixed_width_decimal_values(n, 16)[4 * quad_index + i]
        == fixed_width_decimal_values(
            (if quad_index == 3 {
                n
            } else if quad_index == 2 {
                n / 10_000
            } else if quad_index == 1 {
                n / 100_000_000
            } else {
                n / 1_000_000_000_000
            }) % 10_000,
            4,
        )[i])]
pub(crate) fn fixed_width_decimal_values_chunk_16(n: Int, quad_index: Int) {
    fixed_width_decimal_values_split_16(n);
    power_of_ten_4();
    remainder_10_000_nonnegative(n / 100_000_000);
    remainder_10_000_nonnegative(n / 10_000);
    remainder_10_000_nonnegative(n);
    proof_assert!(n / 1_000_000_000_000 < power_of_ten(4));
    proof_assert!(n / 100_000_000 % 10_000 < power_of_ten(4));
    proof_assert!(n / 10_000 % 10_000 < power_of_ten(4));
    proof_assert!(n % 10_000 < power_of_ten(4));
    fixed_width_decimal_values_len_4_local(n / 1_000_000_000_000);
    fixed_width_decimal_values_len_4_local(n / 100_000_000 % 10_000);
    fixed_width_decimal_values_len_4_local(n / 10_000 % 10_000);
    fixed_width_decimal_values_len_4_local(n % 10_000);
    if quad_index == 0 {
        proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
            fixed_width_decimal_values(n, 16)[i]
                == fixed_width_decimal_values(n / 1_000_000_000_000 % 10_000, 4)[i]);
    } else if quad_index == 1 {
        concat_four_get_second(
            fixed_width_decimal_values(n / 1_000_000_000_000, 4),
            fixed_width_decimal_values(n / 100_000_000 % 10_000, 4),
            fixed_width_decimal_values(n / 10_000 % 10_000, 4),
            fixed_width_decimal_values(n % 10_000, 4),
        );
        proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
            fixed_width_decimal_values(n, 16)[4 + i]
                == fixed_width_decimal_values(n / 100_000_000 % 10_000, 4)[i]);
    } else if quad_index == 2 {
        concat_four_get_third(
            fixed_width_decimal_values(n / 1_000_000_000_000, 4),
            fixed_width_decimal_values(n / 100_000_000 % 10_000, 4),
            fixed_width_decimal_values(n / 10_000 % 10_000, 4),
            fixed_width_decimal_values(n % 10_000, 4),
        );
        proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
            fixed_width_decimal_values(n, 16)[8 + i]
                == fixed_width_decimal_values(n / 10_000 % 10_000, 4)[i]);
    } else {
        proof_assert!(forall<i: Int> 0 <= i && i < 4 ==>
            fixed_width_decimal_values(n, 16)[12 + i]
                == fixed_width_decimal_values(n % 10_000, 4)[i]);
    }
}

}

/// Write a four-digit decimal chunk at the position selected by the runtime
/// formatter. This keeps the four table lookups and stores in one body and
/// exposes their exact initialized-byte effect to callers.
#[cfg_attr(creusot, requires(OFFSET@ + 4 * quad_index@ + 4 <= buf@.len()))]
#[cfg_attr(creusot, requires(pair1@ < 100 && pair2@ < 100))]
#[cfg_attr(creusot, ensures(buf@.len() == (^buf)@.len()))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    OFFSET@ + 4 * quad_index@ <= i && i < OFFSET@ + 4 * quad_index@ + 4
        ==> (^buf)@[i]@ != None))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    OFFSET@ + 4 * quad_index@ <= i && i < OFFSET@ + 4 * quad_index@ + 4
        ==> (^buf)@[i]@.unwrap_logic()@
            == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)
                [i - (OFFSET@ + 4 * quad_index@)]))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    0 <= i && i < OFFSET@ + 4 * quad_index@ ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    OFFSET@ + 4 * quad_index@ + 4 <= i && i < buf@.len()
        ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(creusot, check(terminates))]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
fn write_decimal_quad<const OFFSET: usize>(
    buf: &mut [MaybeUninit<u8>],
    quad_index: usize,
    pair1: u32,
    pair2: u32,
) {
    #[cfg(creusot)]
    {
        let (pair1_tens, pair1_ones) = decimal_pair_correct(pair1);
        let (pair2_tens, pair2_ones) = decimal_pair_correct(pair2);
        let pair1_tens_index = pair1 as usize * 2 + 0;
        let pair1_ones_index = pair1 as usize * 2 + 1;
        let pair2_tens_index = pair2 as usize * 2 + 0;
        let pair2_ones_index = pair2 as usize * 2 + 1;
        proof_assert!(pair1@ < 100 && pair2@ < 100);
        proof_assert!(pair1_tens_index@ < 200);
        proof_assert!(pair1_ones_index@ < 200);
        proof_assert!(pair2_tens_index@ < 200);
        proof_assert!(pair2_ones_index@ < 200);
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
            let _ = fixed_width_decimal_values_pair(pair1@);
            let _ = fixed_width_decimal_values_pair(pair2@);
            let _ = fixed_width_decimal_values_compose_2x2(pair1@ * 100 + pair2@);
            fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)
                == fixed_width_decimal_values(pair1@, 2)
                    .concat(fixed_width_decimal_values(pair2@, 2))
        };
        proof_assert!((pair1@ * 100 + pair2@) / 100 == pair1@);
        proof_assert!((pair1@ * 100 + pair2@) % 100 == pair2@);
        proof_assert! {
            let _ = concat_two_get_digits(
                fixed_width_decimal_values(pair1@, 2),
                fixed_width_decimal_values(pair2@, 2),
            );
            true
        };
        proof_assert!(fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[0]
            == pair1_tens@);
        proof_assert!(fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[1]
            == pair1_ones@);
        proof_assert!(fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[2]
            == pair2_tens@);
        proof_assert!(fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[3]
            == pair2_ones@);
    }

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

#[cfg_attr(creusot, requires(OFFSET@ + 16 <= buf@.len()))]
#[cfg_attr(creusot, requires(n@ < 10_000_000_000_000_000))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    OFFSET@ <= i && i < OFFSET@ + 16 ==> (^buf)@[i]@ != None))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    OFFSET@ <= i && i < OFFSET@ + 16 ==>
        (^buf)@[i]@.unwrap_logic()@ == fixed_width_decimal_values(n@, 16)[i - OFFSET@]))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    0 <= i && i < OFFSET@ ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(creusot, ensures(forall<i: Int>
    OFFSET@ + 16 <= i && i < buf@.len() ==> (^buf)@[i]@ == buf@[i]@))]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
pub(crate) fn enc_16lsd<const OFFSET: usize>(buf: &mut [MaybeUninit<u8>], n: u64) {
    // Consume the least-significant decimals from a working copy.
    let mut remain = n;
    #[cfg(creusot)]
    let buf_before = snapshot!(buf@);

    // Format per four digits from the lookup table.
    #[cfg_attr(creusot, invariant(produced.len() <= 3))]
    #[cfg_attr(creusot, invariant(buf@.len() == (*buf_before).len()))]
    #[cfg_attr(creusot, invariant(remain@ == if produced.len() == 0 {
        n@
    } else if produced.len() == 1 {
        n@ / 10_000
    } else if produced.len() == 2 {
        n@ / 100_000_000
    } else {
        n@ / 1_000_000_000_000
    }))]
    #[cfg_attr(creusot, invariant(forall<i: Int>
        0 <= i && i < OFFSET@ ==>
            buf@[i]@ == (*buf_before)[i]@))]
    #[cfg_attr(creusot, invariant(forall<i: Int>
        OFFSET@ + 16 <= i && i < buf@.len() ==>
            buf@[i]@ == (*buf_before)[i]@))]
    #[cfg_attr(creusot, invariant(forall<i: Int>
        OFFSET@ + 16 - 4 * produced.len() <= i && i < OFFSET@ + 16 ==>
            buf@[i]@ != None))]
    #[cfg_attr(creusot, invariant(forall<i: Int>
        OFFSET@ + 16 - 4 * produced.len() <= i && i < OFFSET@ + 16 ==>
            buf@[i]@.unwrap_logic()@ == fixed_width_decimal_values(n@, 16)[i - OFFSET@]))]
    #[cfg_attr(creusot, variant(3 - produced.len()))]
    for quad_index in (1..4).rev() {
        // pull two pairs
        #[cfg(creusot)]
        {
            proof_assert!(quad_index@ + produced.len() == 4);
            proof_assert!(1 <= quad_index@ && quad_index@ <= 3);
        }
        let quad = remain % 1_00_00;
        #[cfg(creusot)]
        #[cfg(creusot)]
        proof_assert! {
            let _ = proof_helpers::fixed_width_decimal_values_chunk_16(n@, quad_index@);
            quad@ == if quad_index@ == 3 {
                n@ % 10_000
            } else if quad_index@ == 2 {
                n@ / 10_000 % 10_000
            } else {
                n@ / 100_000_000 % 10_000
            }
        };
        remain /= 1_00_00;
        let (pair1, pair2) = divmod100(quad as u32);

        #[cfg(creusot)]
        {
            proof_assert!(quad@ < 10_000);
            proof_assert!(pair1@ < 100 && pair2@ < 100);
        }

        #[cfg(creusot)]
        proof_assert! {
            let quad_digits = proof_helpers::fixed_width_decimal_values_runtime_quad_digits(
                n@, quad_index@, quad@, pair1@, pair2@
            );
            forall<digit: Int> 0 <= digit && digit < 4 ==>
                (digit == 0 || digit == 1 || digit == 2 || digit == 3)
                && fixed_width_decimal_values(n@, 16)[4 * quad_index@ + digit]
                    == quad_digits[digit]
                && quad_digits[digit]
                    == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[digit]
        };

        write_decimal_quad::<OFFSET>(buf, quad_index, pair1, pair2);
    }

    // final two pairs
    #[cfg(creusot)]
    {
        proof_assert!(remain@ == n@ / 1_000_000_000_000);
        proof_assert!(remain@ < 10_000);
        proof_assert! {
            let _ = proof_helpers::fixed_width_decimal_values_chunk_16(n@, 0);
            remain@ == n@ / 1_000_000_000_000 % 10_000
        };
    }
    let (pair1, pair2) = divmod100(remain as u32);

    #[cfg(creusot)]
    {
        proof_assert!(remain@ < 10_000);
        proof_assert!(pair1@ < 100 && pair2@ < 100);
    }

    #[cfg(creusot)]
    proof_assert! {
        let quad_digits = proof_helpers::fixed_width_decimal_values_runtime_quad_digits(
            n@, 0, remain@, pair1@, pair2@
        );
        forall<digit: Int> 0 <= digit && digit < 4 ==>
            (digit == 0 || digit == 1 || digit == 2 || digit == 3)
                &&
            fixed_width_decimal_values(n@, 16)[digit]
                == quad_digits[digit]
                && quad_digits[digit]
                    == fixed_width_decimal_values(pair1@ * 100 + pair2@, 4)[digit]
    };

    write_decimal_quad::<OFFSET>(buf, 0, pair1, pair2);
}
