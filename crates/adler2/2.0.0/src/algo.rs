extern crate creusot_std;
use crate::Adler32;
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, snapshot, trusted, variant,
    DeepModel as _, Int, Invariant as _, Seq, Snapshot, View as _,
};
use std::ops::{AddAssign, MulAssign, RemAssign};

#[logic(open)]
fn adler_congruent(left: Int, right: Int) -> bool {
    pearlite! { exists<factor: Int> left == right + factor * 65521 }
}

#[logic]
#[ensures(adler_congruent(value, value))]
fn adler_congruent_refl(value: Int) {}

#[logic(opaque)]
#[requires(adler_congruent(left1, right1)
    && adler_congruent(left2, right2))]
#[ensures(adler_congruent(left1 + left2, right1 + right2))]
fn adler_congruent_add(left1: Int, right1: Int, left2: Int, right2: Int) {}

#[logic]
#[requires(adler_congruent(left1, right1))]
#[requires(adler_congruent(left2, right2))]
#[ensures(result)]
#[ensures(adler_congruent(left1 + left2, right1 + right2))]
fn adler_congruent_add_direct_certificate(
    left1: Int,
    right1: Int,
    left2: Int,
    right2: Int,
) -> bool {
    adler_congruent_add(left1, right1, left2, right2);
    true
}

#[logic(opaque)]
#[requires(adler_congruent(base_left, base_right)
    && adler_congruent(left0, right0)
    && adler_congruent(left1, right1)
    && adler_congruent(left2, right2)
    && adler_congruent(left3, right3))]
#[ensures(adler_congruent(
    base_left + left0 + left1 + left2 + left3,
    base_right + right0 + right1 + right2 + right3,
))]
fn adler_congruent_add_four(
    base_left: Int,
    base_right: Int,
    left0: Int,
    right0: Int,
    left1: Int,
    right1: Int,
    left2: Int,
    right2: Int,
    left3: Int,
    right3: Int,
) {
    adler_congruent_add(base_left, base_right, left0, right0);
    adler_congruent_add(base_left + left0, base_right + right0, left1, right1);
    adler_congruent_add(
        base_left + left0 + left1,
        base_right + right0 + right1,
        left2,
        right2,
    );
    adler_congruent_add(
        base_left + left0 + left1 + left2,
        base_right + right0 + right1 + right2,
        left3,
        right3,
    );
}

#[logic(opaque)]
#[requires(adler_congruent(base_left, base_right))]
#[requires(adler_congruent(left0, right0))]
#[requires(adler_congruent(left1, right1))]
#[requires(adler_congruent(left2, right2))]
#[requires(adler_congruent(left3, right3))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(
    base_left + left0 + left1 + left2 + left3,
    base_right + right0 + right1 + right2 + right3,
))]
fn adler_congruent_add_four_certificate(
    base_left: Int,
    base_right: Int,
    left0: Int,
    right0: Int,
    left1: Int,
    right1: Int,
    left2: Int,
    right2: Int,
    left3: Int,
    right3: Int,
) -> bool {
    adler_congruent_add_four(
        base_left, base_right, left0, right0, left1, right1, left2, right2, left3, right3,
    );
    adler_congruence_facts(
        base_left + left0 + left1 + left2 + left3,
        base_right + right0 + right1 + right2 + right3,
    )
}

#[logic(opaque)]
#[requires(adler_congruent(
    b_entry,
    initial_b + bytes.len() * initial_a,
))]
#[requires(adler_congruent(lane0, 4 * lane_accumulator(bytes, 0)))]
#[requires(adler_congruent(
    lane1,
    4 * lane_accumulator(bytes, 1) - lane_sum(bytes, 1),
))]
#[requires(adler_congruent(
    lane2,
    4 * lane_accumulator(bytes, 2) - 2 * lane_sum(bytes, 2),
))]
#[requires(adler_congruent(
    lane3,
    4 * lane_accumulator(bytes, 3) - 3 * lane_sum(bytes, 3),
))]
#[ensures(adler_congruent(
    b_entry + lane0 + lane1 + lane2 + lane3,
    initial_b + bytes.len() * initial_a
        + 4 * lane_accumulator(bytes, 0)
        + (4 * lane_accumulator(bytes, 1) - lane_sum(bytes, 1))
        + (4 * lane_accumulator(bytes, 2) - 2 * lane_sum(bytes, 2))
        + (4 * lane_accumulator(bytes, 3) - 3 * lane_sum(bytes, 3)),
))]
fn adler_congruent_weighted_lanes(
    b_entry: Int,
    initial_b: Int,
    initial_a: Int,
    bytes: Seq<u8>,
    lane0: Int,
    lane1: Int,
    lane2: Int,
    lane3: Int,
) {
    adler_congruent_add_four(
        b_entry,
        initial_b + bytes.len() * initial_a,
        lane0,
        4 * lane_accumulator(bytes, 0),
        lane1,
        4 * lane_accumulator(bytes, 1) - lane_sum(bytes, 1),
        lane2,
        4 * lane_accumulator(bytes, 2) - 2 * lane_sum(bytes, 2),
        lane3,
        4 * lane_accumulator(bytes, 3) - 3 * lane_sum(bytes, 3),
    );
}

#[logic(opaque)]
#[requires(adler_congruent(
    b_entry,
    initial_b + bytes.len() * initial_a,
))]
#[requires(adler_congruent(lane0, 4 * lane_accumulator(bytes, 0)))]
#[requires(adler_congruent(
    lane1,
    4 * lane_accumulator(bytes, 1) - lane_sum(bytes, 1),
))]
#[requires(adler_congruent(
    lane2,
    4 * lane_accumulator(bytes, 2) - 2 * lane_sum(bytes, 2),
))]
#[requires(adler_congruent(
    lane3,
    4 * lane_accumulator(bytes, 3) - 3 * lane_sum(bytes, 3),
))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(
    b_entry + lane0 + lane1 + lane2 + lane3,
    initial_b + bytes.len() * initial_a
        + 4 * lane_accumulator(bytes, 0)
        + (4 * lane_accumulator(bytes, 1) - lane_sum(bytes, 1))
        + (4 * lane_accumulator(bytes, 2) - 2 * lane_sum(bytes, 2))
        + (4 * lane_accumulator(bytes, 3) - 3 * lane_sum(bytes, 3)),
))]
fn adler_congruent_weighted_lanes_certificate(
    b_entry: Int,
    initial_b: Int,
    initial_a: Int,
    bytes: Seq<u8>,
    lane0: Int,
    lane1: Int,
    lane2: Int,
    lane3: Int,
) -> bool {
    adler_congruent_weighted_lanes(
        b_entry, initial_b, initial_a, bytes, lane0, lane1, lane2, lane3,
    );
    true
}

#[logic(opaque)]
#[requires(adler_congruent(
    remainder_a_entry,
    initial_a + crate::adler32_byte_sum(prefix),
))]
#[requires(a == remainder_a_entry + crate::adler32_byte_sum(remainder))]
#[requires(original == prefix.concat(remainder))]
#[ensures(adler_congruent(
    a,
    initial_a + crate::adler32_byte_sum(original),
))]
fn adler_congruent_finish_a(
    a: Int,
    remainder_a_entry: Int,
    initial_a: Int,
    prefix: Seq<u8>,
    remainder: Seq<u8>,
    original: Seq<u8>,
) {
    adler_congruent_refl(crate::adler32_byte_sum(remainder));
    adler_congruent_add(
        remainder_a_entry,
        initial_a + crate::adler32_byte_sum(prefix),
        crate::adler32_byte_sum(remainder),
        crate::adler32_byte_sum(remainder),
    );
    crate::adler32_byte_sum_concat(prefix, remainder);
    adler_congruent_reindex(
        a,
        initial_a + crate::adler32_byte_sum(prefix)
            + crate::adler32_byte_sum(remainder),
        initial_a + crate::adler32_byte_sum(original),
    );
}

#[logic(open)]
fn adler_congruent_add_same_facts(left: Int, right: Int, same: Int) -> bool {
    pearlite! {
        exists<factor: Int> left + same == right + same + factor * 65521
    }
}

#[logic(opaque)]
#[requires(adler_congruent(left, right))]
#[ensures(result)]
#[ensures(adler_congruent(left + same, right + same))]
fn adler_congruent_add_same_certificate(left: Int, right: Int, same: Int) -> bool {
    adler_congruent_refl(same);
    adler_congruent_add(left, right, same, same);
    true
}

#[logic(open)]
fn adler_congruence_facts(left: Int, right: Int) -> bool {
    pearlite! { exists<factor: Int> left == right + factor * 65521 }
}

#[logic(opaque)]
#[requires(adler_congruent(
    remainder_a_entry,
    initial_a + crate::adler32_byte_sum(prefix),
))]
#[requires(a == remainder_a_entry + crate::adler32_byte_sum(remainder))]
#[requires(original == prefix.concat(remainder))]
#[ensures(result)]
#[ensures(adler_congruent(
    a,
    initial_a + crate::adler32_byte_sum(original),
))]
fn adler_congruent_finish_a_certificate(
    a: Int,
    remainder_a_entry: Int,
    initial_a: Int,
    prefix: Seq<u8>,
    remainder: Seq<u8>,
    original: Seq<u8>,
) -> bool {
    adler_congruent_finish_a(
        a,
        remainder_a_entry,
        initial_a,
        prefix,
        remainder,
        original,
    );
    adler_congruent(a, initial_a + crate::adler32_byte_sum(original))
}

#[logic(opaque)]
#[requires(adler_congruent(
    remainder_a_entry,
    initial_a + crate::adler32_byte_sum(prefix),
))]
#[requires(adler_congruent(
    remainder_b_entry,
    initial_b + prefix.len() * initial_a
        + crate::adler32_weighted_sum(prefix),
))]
#[requires(b == remainder_b_entry + remainder.len() * remainder_a_entry
    + crate::adler32_weighted_sum(remainder))]
#[requires(original == prefix.concat(remainder))]
#[ensures(adler_congruent(
    b,
    initial_b + original.len() * initial_a
        + crate::adler32_weighted_sum(original),
))]
fn adler_congruent_finish_b(
    b: Int,
    remainder_a_entry: Int,
    remainder_b_entry: Int,
    initial_a: Int,
    initial_b: Int,
    prefix: Seq<u8>,
    remainder: Seq<u8>,
    original: Seq<u8>,
) {
    adler_congruent_scale(
        remainder_a_entry,
        initial_a + crate::adler32_byte_sum(prefix),
        remainder.len(),
    );
    adler_congruent_add(
        remainder_b_entry,
        initial_b + prefix.len() * initial_a
            + crate::adler32_weighted_sum(prefix),
        remainder.len() * remainder_a_entry,
        remainder.len() * (initial_a + crate::adler32_byte_sum(prefix)),
    );
    adler_congruent_refl(crate::adler32_weighted_sum(remainder));
    adler_congruent_add(
        remainder_b_entry + remainder.len() * remainder_a_entry,
        initial_b + prefix.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * (initial_a + crate::adler32_byte_sum(prefix)),
        crate::adler32_weighted_sum(remainder),
        crate::adler32_weighted_sum(remainder),
    );
    crate::adler32_weighted_sum_concat(prefix, remainder);
    proof_assert!(original.len() == prefix.len() + remainder.len());
    proof_assert!(crate::adler32_weighted_sum(original)
        == crate::adler32_weighted_sum(prefix)
            + remainder.len() * crate::adler32_byte_sum(prefix)
            + crate::adler32_weighted_sum(remainder));
    proof_assert!(remainder.len() * (initial_a + crate::adler32_byte_sum(prefix))
        == remainder.len() * initial_a
            + remainder.len() * crate::adler32_byte_sum(prefix));
    proof_assert!((prefix.len() + remainder.len()) * initial_a
        == prefix.len() * initial_a + remainder.len() * initial_a);
    proof_assert!(
        initial_b + prefix.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * (initial_a + crate::adler32_byte_sum(prefix))
            + crate::adler32_weighted_sum(remainder)
        == initial_b + prefix.len() * initial_a + remainder.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * crate::adler32_byte_sum(prefix)
            + crate::adler32_weighted_sum(remainder)
    );
    proof_assert!(
        initial_b + prefix.len() * initial_a + remainder.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * crate::adler32_byte_sum(prefix)
            + crate::adler32_weighted_sum(remainder)
        == initial_b + (prefix.len() + remainder.len()) * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * crate::adler32_byte_sum(prefix)
            + crate::adler32_weighted_sum(remainder)
    );
    multiplication_congruence(
        prefix.len() + remainder.len(),
        original.len(),
        initial_a,
    );
    proof_assert!((prefix.len() + remainder.len()) * initial_a
        == original.len() * initial_a);
    proof_assert!(crate::adler32_weighted_sum(prefix)
        + remainder.len() * crate::adler32_byte_sum(prefix)
        + crate::adler32_weighted_sum(remainder)
        == crate::adler32_weighted_sum(original));
    proof_assert!(
        initial_b + (prefix.len() + remainder.len()) * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * crate::adler32_byte_sum(prefix)
            + crate::adler32_weighted_sum(remainder)
        == initial_b + original.len() * initial_a
            + (crate::adler32_weighted_sum(prefix)
                + remainder.len() * crate::adler32_byte_sum(prefix)
                + crate::adler32_weighted_sum(remainder))
    );
    proof_assert!(
        initial_b + original.len() * initial_a
            + (crate::adler32_weighted_sum(prefix)
                + remainder.len() * crate::adler32_byte_sum(prefix)
                + crate::adler32_weighted_sum(remainder))
        == initial_b + original.len() * initial_a
            + crate::adler32_weighted_sum(original)
    );
    proof_assert!(
        initial_b + prefix.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * (initial_a + crate::adler32_byte_sum(prefix))
            + crate::adler32_weighted_sum(remainder)
        == initial_b + original.len() * initial_a
            + crate::adler32_weighted_sum(original)
    );
    proof_assert!(adler_congruent(
        b,
        initial_b + prefix.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * (initial_a + crate::adler32_byte_sum(prefix))
            + crate::adler32_weighted_sum(remainder),
    ));
    adler_congruent_reindex(
        b,
        initial_b + prefix.len() * initial_a
            + crate::adler32_weighted_sum(prefix)
            + remainder.len() * (initial_a + crate::adler32_byte_sum(prefix))
            + crate::adler32_weighted_sum(remainder),
        initial_b + original.len() * initial_a
            + crate::adler32_weighted_sum(original),
    );
}

#[logic(opaque)]
#[requires(adler_congruent(
    remainder_a_entry,
    initial_a + crate::adler32_byte_sum(prefix),
))]
#[requires(adler_congruent(
    remainder_b_entry,
    initial_b + prefix.len() * initial_a
        + crate::adler32_weighted_sum(prefix),
))]
#[requires(b == remainder_b_entry + remainder.len() * remainder_a_entry
    + crate::adler32_weighted_sum(remainder))]
#[requires(original == prefix.concat(remainder))]
#[ensures(result)]
#[ensures(adler_congruent(
    b,
    initial_b + original.len() * initial_a
        + crate::adler32_weighted_sum(original),
))]
fn adler_congruent_finish_b_certificate(
    b: Int,
    remainder_a_entry: Int,
    remainder_b_entry: Int,
    initial_a: Int,
    initial_b: Int,
    prefix: Seq<u8>,
    remainder: Seq<u8>,
    original: Seq<u8>,
) -> bool {
    adler_congruent_finish_b(
        b,
        remainder_a_entry,
        remainder_b_entry,
        initial_a,
        initial_b,
        prefix,
        remainder,
        original,
    );
    adler_congruent(
        b,
        initial_b + original.len() * initial_a
            + crate::adler32_weighted_sum(original),
    )
}

#[logic]
#[requires(adler_congruence_facts(left, right))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(left, right))]
#[ensures(adler_congruent(left, right))]
fn adler_congruence_to_predicate_certificate(left: Int, right: Int) -> bool {
    adler_congruence_facts(left, right)
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(factor * left, factor * right))]
#[ensures(adler_congruent(factor * left, factor * right))]
fn adler_congruent_scale_certificate(left: Int, right: Int, factor: Int) -> bool {
    adler_congruent_scale(left, right, factor);
    adler_congruence_facts(factor * left, factor * right)
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(65521 - left, -right))]
fn adler_congruent_modulus_minus_certificate(left: Int, right: Int) -> bool {
    adler_congruent_modulus_minus(left, right);
    adler_congruence_facts(65521 - left, -right)
}

#[logic]
#[requires(adler_congruent(left1, right1))]
#[requires(adler_congruent(left2, right2))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(left1 + left2, right1 + right2))]
fn adler_congruent_add_certificate(
    left1: Int,
    right1: Int,
    left2: Int,
    right2: Int,
) -> bool {
    adler_congruent_add(left1, right1, left2, right2);
    adler_congruence_facts(left1 + left2, right1 + right2)
}

#[logic(opaque)]
#[ensures(result == (adler_congruent(four_b, 4 * accumulator)
    && adler_congruent(a, sum)))]
fn adler_adjusted_lane_ready(
    four_b: Int,
    accumulator: Int,
    a: Int,
    sum: Int,
) -> bool {
    adler_congruent(four_b, 4 * accumulator) && adler_congruent(a, sum)
}

#[logic(opaque)]
#[requires(adler_adjusted_lane_ready(four_b, accumulator, a, sum))]
#[ensures(result)]
#[ensures(adler_congruent(
    four_b + (65521 - a) * factor,
    4 * accumulator - factor * sum,
))]
fn adler_adjusted_lane_certificate(
    four_b: Int,
    accumulator: Int,
    a: Int,
    sum: Int,
    factor: Int,
) -> bool {
    adler_congruent_modulus_minus(a, sum);
    adler_congruent_scale(65521 - a, -sum, factor);
    adler_congruent_add(
        four_b,
        4 * accumulator,
        factor * (65521 - a),
        factor * (-sum),
    );
    true
}

#[logic(opaque)]
#[requires(adler_congruent(four_b, 4 * accumulator))]
#[requires(adler_congruent(a, sum))]
#[ensures(adler_congruent(
    four_b + (65521 - a) * factor,
    4 * accumulator - factor * sum,
))]
fn adler_adjusted_lane(
    four_b: Int,
    accumulator: Int,
    a: Int,
    sum: Int,
    factor: Int,
) {
    adler_congruent_modulus_minus(a, sum);
    adler_congruent_scale(65521 - a, -sum, factor);
    adler_congruent_add(
        four_b,
        4 * accumulator,
        (65521 - a) * factor,
        (-sum) * factor,
    );
}

#[logic(opaque)]
#[requires(adler_congruent(four_b, 4 * accumulator))]
#[requires(adler_congruent(a, sum))]
#[ensures(adler_congruent(
    four_b + (65521 - a) * 1,
    4 * accumulator - 1 * sum,
))]
fn adler_adjusted_lane_one(four_b: Int, accumulator: Int, a: Int, sum: Int) {
    adler_adjusted_lane(four_b, accumulator, a, sum, 1);
}

#[logic(opaque)]
#[requires(adler_congruent(four_b, 4 * accumulator))]
#[requires(adler_congruent(a, sum))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(
    four_b + (65521 - a) * 1,
    4 * accumulator - 1 * sum,
))]
#[ensures(adler_congruent(
    four_b + (65521 - a) * 1,
    4 * accumulator - 1 * sum,
))]
fn adler_adjusted_lane_one_certificate(
    four_b: Int,
    accumulator: Int,
    a: Int,
    sum: Int,
) -> bool {
    adler_adjusted_lane_one(four_b, accumulator, a, sum);
    true
}

#[logic]
#[requires(adler_congruent(first, second))]
#[requires(adler_congruent(second, third))]
#[ensures(adler_congruent(first, third))]
fn adler_congruent_trans(first: Int, second: Int, third: Int) {}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(right, left))]
fn adler_congruent_symmetric(left: Int, right: Int) {
    proof_assert!(forall<factor: Int> left == right + factor * 65521 ==>
        right == left + (-factor) * 65521);
}

#[logic(opaque)]
#[requires(old_target == new_target)]
#[requires(adler_congruent(value, old_target))]
#[ensures(adler_congruent(value, new_target))]
fn adler_congruent_reindex(value: Int, old_target: Int, new_target: Int) {}

#[logic(opaque)]
#[requires(old_target == new_target)]
#[requires(adler_congruent(value, old_target))]
#[ensures(result)]
#[ensures(adler_congruent(value, new_target))]
fn adler_congruent_reindex_certificate(
    value: Int,
    old_target: Int,
    new_target: Int,
) -> bool {
    adler_congruent_reindex(value, old_target, new_target);
    true
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(factor * left, factor * right))]
fn adler_congruent_scale(left: Int, right: Int, factor: Int) {
    proof_assert!(forall<multiple: Int> left == right + multiple * 65521 ==>
        factor * left == factor * right + (factor * multiple) * 65521);
    proof_assert!(forall<multiple: Int> left == right + multiple * 65521 ==>
        exists<scaled: Int> factor * left == factor * right + scaled * 65521);
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(-left, -right))]
fn adler_congruent_neg(left: Int, right: Int) {
    proof_assert!(forall<multiple: Int> left == right + multiple * 65521 ==>
        -left == -right + (-multiple) * 65521);
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(65521 - left, -right))]
fn adler_congruent_modulus_minus(left: Int, right: Int) {
    adler_congruent_neg(left, right);
    proof_assert!(adler_congruent(65521, 0));
    adler_congruent_add(65521, 0, -left, -right);
}

#[logic(open)]
#[requires(0 <= count && count <= 4)]
fn u32x4_prefix_sum(values: (u32, u32, u32, u32), count: Int) -> Int {
    if count == 0 {
        0
    } else if count == 1 {
        values.0.deep_model()
    } else if count == 2 {
        values.0.deep_model() + values.1.deep_model()
    } else if count == 3 {
        values.0.deep_model() + values.1.deep_model() + values.2.deep_model()
    } else {
        values.0.deep_model()
            + values.1.deep_model()
            + values.2.deep_model()
            + values.3.deep_model()
    }
}

#[logic]
#[requires(0 <= count && count < 4)]
#[ensures(u32x4_prefix_sum(values, count + 1) == u32x4_prefix_sum(values, count)
    + if count == 0 { values.0@ } else if count == 1 { values.1@ }
      else if count == 2 { values.2@ } else { values.3@ })]
fn u32x4_prefix_sum_step(values: (u32, u32, u32, u32), count: Int) {}

#[logic]
#[ensures(adler_congruent(value % 65521, value))]
fn adler_remainder_congruent(value: Int) {
    proof_assert!(value == 65521 * (value / 65521) + value % 65521);
    proof_assert!(exists<factor: Int> value % 65521 == value + factor * 65521);
}

#[logic]
#[requires(0 < modulus)]
#[ensures(value % modulus < modulus)]
fn remainder_upper_bound(value: Int, modulus: Int) {}

#[logic]
#[requires(0 < modulus)]
#[ensures(0 <= value % modulus)]
fn remainder_lower_bound(value: Int, modulus: Int) {}

#[logic]
#[requires(adler_congruent(left, right))]
#[requires(0 <= left && left < 65521)]
#[requires(0 <= right && right < 65521)]
#[ensures(left == right)]
fn adler_congruent_reduced(left: Int, right: Int) {}

#[logic(opaque)]
#[requires(stored == value % 65521)]
#[requires(adler_congruent(value, target))]
#[requires(0 <= stored && stored < 65521)]
#[requires(0 <= target % 65521 && target % 65521 < 65521)]
#[ensures(stored == target % 65521)]
fn adler_finish_reduced(stored: Int, value: Int, target: Int) {
    adler_remainder_congruent(value);
    adler_congruent_trans(stored, value, target);
    adler_remainder_congruent(target);
    adler_congruent_symmetric(target % 65521, target);
    adler_congruent_trans(stored, target, target % 65521);
    remainder_upper_bound(value, 65521);
    remainder_upper_bound(target, 65521);
    adler_congruent_reduced(stored, target % 65521);
}

#[logic(opaque)]
#[requires(stored == value % 65521)]
#[requires(adler_congruent(value, target))]
#[requires(0 <= stored && stored < 65521)]
#[requires(0 <= target % 65521 && target % 65521 < 65521)]
#[ensures(result)]
#[ensures(stored == target % 65521)]
fn adler_finish_reduced_certificate(stored: Int, value: Int, target: Int) -> bool {
    adler_finish_reduced(stored, value, target);
    true
}

#[logic(opaque)]
#[requires(stored == value % 65521)]
#[requires(adler_congruent(value, target))]
#[requires(0 <= stored && stored < 65521)]
#[ensures(result)]
#[ensures(stored == target % 65521)]
fn adler_finish_from_remainder_certificate(
    stored: Int,
    value: Int,
    target: Int,
) -> bool {
    remainder_lower_bound(target, 65521);
    remainder_upper_bound(target, 65521);
    adler_finish_reduced(stored, value, target);
    true
}

#[logic(opaque)]
#[requires(0 <= initial_a && initial_a < 65521)]
#[requires(0 <= initial_b && initial_b < 65521)]
#[ensures(result)]
#[ensures(0 <= (initial_b + bytes.len() * initial_a
    + crate::adler32_weighted_sum(bytes)) % 65521
    && (initial_b + bytes.len() * initial_a
        + crate::adler32_weighted_sum(bytes)) % 65521 < 65521)]
fn adler_b_target_reduced_certificate(
    initial_a: Int,
    initial_b: Int,
    bytes: Seq<u8>,
) -> bool {
    crate::adler32_update_reduced((initial_a, initial_b), bytes);
    true
}

/// Sum of one of the four interleaved byte lanes in an aligned byte sequence.
#[logic]
#[variant(bytes.len())]
#[requires(0 <= lane && lane < 4)]
#[ensures(bytes.len() < 4 ==> result == 0)]
#[ensures(bytes.len() >= 4 ==> result == lane_sum(
    bytes.subsequence(0, bytes.len() - 4), lane
) + bytes[bytes.len() - 4 + lane]@)]
fn lane_sum(bytes: Seq<u8>, lane: Int) -> Int {
    if bytes.len() < 4 {
        0
    } else {
        lane_sum(bytes.subsequence(0, bytes.len() - 4), lane)
            + bytes[bytes.len() - 4 + lane].deep_model()
    }
}

/// Accumulated lane sums after every four-byte group, matching `b_vec`.
#[logic]
#[variant(bytes.len())]
#[requires(0 <= lane && lane < 4)]
#[ensures(bytes.len() < 4 ==> result == 0)]
#[ensures(bytes.len() >= 4 ==> result == lane_accumulator(
    bytes.subsequence(0, bytes.len() - 4), lane
) + lane_sum(bytes, lane))]
fn lane_accumulator(bytes: Seq<u8>, lane: Int) -> Int {
    if bytes.len() < 4 {
        0
    } else {
        lane_accumulator(bytes.subsequence(0, bytes.len() - 4), lane) + lane_sum(bytes, lane)
    }
}

/// The four lanes partition the byte sum of an aligned sequence.
#[logic]
#[variant(bytes.len())]
#[requires(bytes.len() % 4 == 0)]
#[ensures(crate::adler32_byte_sum(bytes)
    == lane_sum(bytes, 0) + lane_sum(bytes, 1)
        + lane_sum(bytes, 2) + lane_sum(bytes, 3))]
fn lane_sum_partition(bytes: Seq<u8>) {
    if bytes.len() >= 4 {
        crate::adler32_byte_sum_last_four(bytes);
        lane_sum_partition(bytes.subsequence(0, bytes.len() - 4));
    }
}

/// Recombining the four accumulated lanes yields Adler's weighted sum.
#[logic]
#[variant(bytes.len())]
#[requires(bytes.len() % 4 == 0)]
#[ensures(crate::adler32_weighted_sum(bytes)
    == 4 * (lane_accumulator(bytes, 0) + lane_accumulator(bytes, 1)
        + lane_accumulator(bytes, 2) + lane_accumulator(bytes, 3))
        - lane_sum(bytes, 1) - 2 * lane_sum(bytes, 2)
        - 3 * lane_sum(bytes, 3))]
fn lane_weight_partition(bytes: Seq<u8>) {
    if bytes.len() >= 4 {
        crate::adler32_byte_sum_last_four(bytes);
        crate::adler32_weighted_sum_last_four(bytes);
        lane_weight_partition(bytes.subsequence(0, bytes.len() - 4));
        lane_sum_partition(bytes);
    }
}

#[logic]
#[ensures(lane_sum(bytes.subsequence(0, 0), 0) == 0)]
#[ensures(lane_sum(bytes.subsequence(0, 0), 1) == 0)]
#[ensures(lane_sum(bytes.subsequence(0, 0), 2) == 0)]
#[ensures(lane_sum(bytes.subsequence(0, 0), 3) == 0)]
#[ensures(lane_accumulator(bytes.subsequence(0, 0), 0) == 0)]
#[ensures(lane_accumulator(bytes.subsequence(0, 0), 1) == 0)]
#[ensures(lane_accumulator(bytes.subsequence(0, 0), 2) == 0)]
#[ensures(lane_accumulator(bytes.subsequence(0, 0), 3) == 0)]
fn empty_lane_facts(bytes: Seq<u8>) {
    proof_assert!(bytes.subsequence(0, 0).len() == 0);
}

#[logic]
#[requires(0 <= start && start <= end && end <= bytes.len())]
#[ensures(bytes.subsequence(0, end)
    == bytes.subsequence(0, start).concat(bytes.subsequence(start, end)))]
fn subsequence_split(bytes: Seq<u8>, start: Int, end: Int) {}

#[logic(opaque)]
#[requires(0 < end && end <= values.len())]
#[requires(byte == values[end - 1])]
#[ensures(values.subsequence(0, end)
    == values.subsequence(0, end - 1).push_back(byte))]
fn subsequence_push_last<T>(values: Seq<T>, end: Int, byte: T) {
    proof_assert!(values.subsequence(0, end).len() == end);
    proof_assert!(values.subsequence(0, end - 1).len() == end - 1);
    proof_assert!(values
        .subsequence(0, end)
        .ext_eq(values.subsequence(0, end - 1).push_back(byte)));
}

#[logic]
#[requires(0 <= start && start < values.len())]
#[ensures(values.subsequence(start, values.len())[0] == values[start])]
fn subsequence_head<T>(values: Seq<T>, start: Int) {}

#[logic]
#[requires(current == initial.subsequence(start, end))]
#[requires(0 <= start && start < end && end <= initial.len())]
#[ensures(current.subsequence(1, current.len())
    == initial.subsequence(start + 1, end))]
fn subsequence_suffix_step<T>(
    current: Seq<T>,
    initial: Seq<T>,
    start: Int,
    end: Int,
) {
}

#[logic(opaque)]
#[requires(0 <= index && index < chunks.len())]
#[requires(forall<i> 0 <= i && i < chunks.len() ==>
    chunks[i] == source.subsequence(i * chunk_size, (i + 1) * chunk_size))]
#[ensures(chunks[index]
    == source.subsequence(index * chunk_size, (index + 1) * chunk_size))]
fn exact_chunk_at_certificate<T>(
    source: Seq<T>,
    chunks: Seq<Seq<T>>,
    chunk_size: Int,
    index: Int,
) {
}

#[logic(opaque)]
#[ensures(result == (forall<i> 0 <= i && i < chunks.len() ==>
    chunks[i] == source.subsequence(i * chunk_size, (i + 1) * chunk_size)))]
fn chunks_match_source<T>(
    source: Seq<T>,
    chunks: Seq<Seq<T>>,
    chunk_size: Int,
) -> bool {
    pearlite! {
        forall<i> 0 <= i && i < chunks.len() ==>
            chunks[i] == source.subsequence(
                i * chunk_size,
                (i + 1) * chunk_size,
            )
    }
}

#[logic(opaque)]
#[requires(chunks_match_source(source, chunks, chunk_size))]
#[requires(0 <= index && index < chunks.len())]
#[ensures(chunks[index]
    == source.subsequence(index * chunk_size, (index + 1) * chunk_size))]
fn matched_chunk_at_certificate<T>(
    source: Seq<T>,
    chunks: Seq<Seq<T>>,
    chunk_size: Int,
    index: Int,
) {
    exact_chunk_at_certificate(source, chunks, chunk_size, index);
}

#[logic]
#[requires(0 <= index)]
#[requires(index + 1 == visited_len)]
#[requires(visited_len <= chunks_len)]
#[ensures(index < chunks_len)]
fn visited_index_bound_certificate(
    index: Int,
    visited_len: Int,
    chunks_len: Int,
) {
}

#[logic]
#[requires(0 <= start && start <= end && end <= values.len())]
#[ensures(values.subsequence(0, end).subsequence(0, start)
    == values.subsequence(0, start))]
fn prefix_of_prefix_certificate<T>(values: Seq<T>, start: Int, end: Int) {
}

#[logic]
#[requires(start <= old_end)]
#[requires(old_end == new_end)]
#[requires(new_end <= values.len())]
#[ensures(values.subsequence(start, old_end)
    == values.subsequence(start, new_end))]
fn reindex_subsequence_end<T>(
    values: Seq<T>,
    start: Int,
    old_end: Int,
    new_end: Int,
) {
}

#[logic]
#[requires(value % 4 == 0)]
#[ensures(result == count * value)]
#[ensures(result % 4 == 0)]
fn multiple_of_four_product(count: Int, value: Int) -> Int {
    proof_assert!(value == 4 * (value / 4) + value % 4);
    proof_assert!(value == 4 * (value / 4));
    proof_assert!(count * value == 4 * (count * (value / 4)));
    proof_assert!((4 * (count * (value / 4))) % 4 == 0);
    proof_assert!((count * value) % 4 == 0);
    count * value
}

#[logic]
#[requires(total == aligned_prefix + remainder)]
#[requires(total % 4 == 0)]
#[requires(aligned_prefix % 4 == 0)]
#[ensures(remainder % 4 == 0)]
fn aligned_remainder(total: Int, aligned_prefix: Int, remainder: Int) {
    proof_assert!(total % 4 == (aligned_prefix % 4 + remainder % 4) % 4);
}

#[logic]
#[requires(total == aligned_prefix + remainder)]
#[requires(total % 4 == 0)]
#[requires(aligned_prefix % 4 == 0)]
#[ensures(result == remainder % 4)]
#[ensures(result == 0)]
fn aligned_remainder_value(total: Int, aligned_prefix: Int, remainder: Int) -> Int {
    aligned_remainder(total, aligned_prefix, remainder);
    remainder % 4
}

#[logic]
#[ensures((value - value % 4) % 4 == 0)]
fn truncate_to_four_alignment(value: Int) {
    proof_assert!(value == 4 * (value / 4) + value % 4);
    proof_assert!(value - value % 4 == 4 * (value / 4));
}

#[logic(opaque)]
#[requires(current == initial.subsequence(old_start, end)
    && old_start == new_start)]
#[ensures(current == initial.subsequence(new_start, end))]
fn reindex_remaining_chunks(
    current: Seq<Seq<u8>>,
    initial: Seq<Seq<u8>>,
    old_start: Int,
    new_start: Int,
    end: Int,
) {
}

#[logic(opaque)]
#[requires(old_start == new_start)]
#[ensures(initial.subsequence(old_start, end)
    == initial.subsequence(new_start, end))]
fn reindex_chunk_subsequence_start(
    initial: Seq<Seq<u8>>,
    old_start: Int,
    new_start: Int,
    end: Int,
) {
}

#[logic(opaque)]
#[requires(old_start == new_start)]
#[ensures(result)]
#[ensures(result == (initial.subsequence(old_start, end)
    == initial.subsequence(new_start, end)))]
fn reindex_chunk_subsequence_start_certificate(
    initial: Seq<Seq<u8>>,
    old_start: Int,
    new_start: Int,
    end: Int,
) -> bool {
    reindex_chunk_subsequence_start(initial, old_start, new_start, end);
    true
}

#[logic(open)]
fn sequence_equality_facts<T>(left: Seq<T>, right: Seq<T>) -> bool {
    pearlite! { left == right }
}

#[logic]
#[requires(start <= old_end)]
#[requires(old_end == new_end)]
#[requires(new_end <= values.len())]
#[ensures(result)]
#[ensures(result == sequence_equality_facts(
    values.subsequence(start, old_end),
    values.subsequence(start, new_end),
))]
fn reindex_subsequence_end_certificate<T>(
    values: Seq<T>,
    start: Int,
    old_end: Int,
    new_end: Int,
) -> bool {
    reindex_subsequence_end(values, start, old_end, new_end);
    sequence_equality_facts(
        values.subsequence(start, old_end),
        values.subsequence(start, new_end),
    )
}

#[logic(opaque)]
#[requires(current == initial.subsequence(old_start, end)
    && old_start == new_start)]
#[ensures(result)]
#[ensures(result == sequence_equality_facts(
    current,
    initial.subsequence(new_start, end),
))]
fn reindex_remaining_chunks_certificate(
    current: Seq<Seq<u8>>,
    initial: Seq<Seq<u8>>,
    old_start: Int,
    new_start: Int,
    end: Int,
) -> bool {
    reindex_remaining_chunks(current, initial, old_start, new_start, end);
    sequence_equality_facts(current, initial.subsequence(new_start, end))
}

#[logic(opaque)]
#[requires(new_index == old_index + 1
    && value == base + (old_index + 1) * coefficient
        + crate::adler32_weighted_sum(bytes.subsequence(0, old_index + 1)))]
#[ensures(result)]
#[ensures(result == remainder_state_facts(
    value,
    base,
    coefficient,
    bytes,
    new_index,
))]
fn reindex_remainder_state_certificate(
    value: Int,
    base: Int,
    coefficient: Int,
    bytes: Seq<u8>,
    old_index: Int,
    new_index: Int,
) -> bool {
    remainder_state_facts(value, base, coefficient, bytes, new_index)
}

#[logic(open)]
fn remainder_state_facts(
    value: Int,
    base: Int,
    coefficient: Int,
    bytes: Seq<u8>,
    index: Int,
) -> bool {
    pearlite! {
        value == base + index * coefficient
            + crate::adler32_weighted_sum(bytes.subsequence(0, index))
    }
}

#[logic(opaque)]
#[requires(before == base + old_sum)]
#[requires(after == before + byte)]
#[requires(new_sum == old_sum + byte)]
#[ensures(after == base + new_sum)]
fn remainder_a_step_certificate(
    before: Int,
    after: Int,
    base: Int,
    old_sum: Int,
    new_sum: Int,
    byte: Int,
) {
}

#[logic(opaque)]
#[requires(before == base + index * entry + old_weight)]
#[requires(after == before + a_after)]
#[requires(a_after == entry + new_sum)]
#[requires(new_sum == old_sum + byte)]
#[requires(new_weight == old_weight + old_sum + byte)]
#[ensures(after == base + (index + 1) * entry + new_weight)]
fn remainder_b_step_certificate(
    before: Int,
    after: Int,
    a_after: Int,
    base: Int,
    entry: Int,
    index: Int,
    old_sum: Int,
    new_sum: Int,
    old_weight: Int,
    new_weight: Int,
    byte: Int,
) {
}

#[logic]
fn remainder_target_facts(
    a_after: Int,
    b_after: Int,
    a_target: Int,
    b_target: Int,
    count: Int,
    byte_sum: Int,
    weighted_sum: Int,
) -> bool {
    adler_congruent(a_after, a_target + byte_sum)
        && adler_congruent(
            b_after,
            b_target + count * a_target + weighted_sum,
        )
}

#[logic(opaque)]
#[requires(a_after == a_entry + byte_sum)]
#[requires(b_after == b_entry + count * a_entry + weighted_sum)]
#[requires(adler_congruent(a_entry, a_target))]
#[requires(adler_congruent(b_entry, b_target))]
#[ensures(result)]
#[ensures(result == remainder_target_facts(
    a_after,
    b_after,
    a_target,
    b_target,
    count,
    byte_sum,
    weighted_sum,
))]
fn remainder_targets_certificate(
    a_entry: Int,
    b_entry: Int,
    a_after: Int,
    b_after: Int,
    a_target: Int,
    b_target: Int,
    count: Int,
    byte_sum: Int,
    weighted_sum: Int,
) -> bool {
    adler_congruent_refl(byte_sum);
    adler_congruent_add(a_entry, a_target, byte_sum, byte_sum);
    adler_congruent_scale(a_entry, a_target, count);
    adler_congruent_add(
        b_entry,
        b_target,
        count * a_entry,
        count * a_target,
    );
    adler_congruent_refl(weighted_sum);
    adler_congruent_add(
        b_entry + count * a_entry,
        b_target + count * a_target,
        weighted_sum,
        weighted_sum,
    );
    remainder_target_facts(
        a_after,
        b_after,
        a_target,
        b_target,
        count,
        byte_sum,
        weighted_sum,
    )
}

#[logic(opaque)]
#[requires(a_after == a_entry + byte_sum)]
#[requires(adler_congruent(a_entry, a_target))]
#[ensures(adler_congruent(a_after, a_target + byte_sum))]
fn remainder_a_target_certificate(
    a_entry: Int,
    a_after: Int,
    a_target: Int,
    byte_sum: Int,
) {
    adler_congruent_refl(byte_sum);
    adler_congruent_add(a_entry, a_target, byte_sum, byte_sum);
}

#[logic(opaque)]
#[requires(b_after == b_entry + count * a_entry + weighted_sum)]
#[requires(adler_congruent(a_entry, a_target))]
#[requires(adler_congruent(b_entry, b_target))]
#[ensures(adler_congruent(
    b_after,
    b_target + count * a_target + weighted_sum,
))]
fn remainder_b_target_certificate(
    a_entry: Int,
    b_entry: Int,
    b_after: Int,
    a_target: Int,
    b_target: Int,
    count: Int,
    weighted_sum: Int,
) {
    adler_congruent_scale(a_entry, a_target, count);
    adler_congruent_add(
        b_entry,
        b_target,
        count * a_entry,
        count * a_target,
    );
    adler_congruent_refl(weighted_sum);
    adler_congruent_add(
        b_entry + count * a_entry,
        b_target + count * a_target,
        weighted_sum,
        weighted_sum,
    );
}

#[logic]
#[variant(right.len())]
#[requires(left.len() % 4 == 0 && right.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_sum(left.concat(right), lane)
    == lane_sum(left, lane) + lane_sum(right, lane))]
fn lane_sum_concat(left: Seq<u8>, right: Seq<u8>, lane: Int) {
    if right.len() >= 4 {
        let prefix = right.subsequence(0, right.len() - 4);
        lane_sum_concat(left, prefix, lane);
        proof_assert!(
            left.concat(right)
                .subsequence(0, left.len() + right.len() - 4)
                == left.concat(prefix)
        );
        proof_assert!(
            left.concat(right)[left.len() + right.len() - 4 + lane]
                == right[right.len() - 4 + lane]
        );
    }
}

#[logic]
#[ensures((count + 1) * value == count * value + value)]
fn multiplication_successor(count: Int, value: Int) {}

#[logic(opaque)]
#[requires(left == right)]
#[ensures(left * factor == right * factor)]
#[ensures(factor * left == factor * right)]
fn multiplication_congruence(left: Int, right: Int, factor: Int) {}

#[logic]
#[requires(value % 4 == 0)]
#[ensures((value + 4) % 4 == 0)]
fn add_four_preserves_alignment(value: Int) {
    proof_assert!((value + 4) % 4 == (value % 4 + 4 % 4) % 4);
}

#[logic(opaque)]
#[requires(successor == prior + 1)]
#[ensures(result == successor * (successor + 1))]
#[ensures(result == (prior + 1) * (prior + 2))]
fn consecutive_product_reindex(successor: Int, prior: Int) -> Int {
    proof_assert!(successor + 1 == prior + 2);
    successor * (successor + 1)
}

#[logic(opaque)]
#[requires(current == initial + old_count * factor + accumulator)]
#[requires(old_count == new_count)]
#[ensures(result == current)]
#[ensures(result == initial + new_count * factor + accumulator)]
fn reindex_linear_count(
    current: Int,
    initial: Int,
    old_count: Int,
    new_count: Int,
    factor: Int,
    accumulator: Int,
) -> Int {
    current
}

#[logic]
#[variant(right.len())]
#[requires(left.len() % 4 == 0 && right.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_accumulator(left.concat(right), lane)
    == lane_accumulator(left, lane)
        + (right.len() / 4) * lane_sum(left, lane)
        + lane_accumulator(right, lane))]
fn lane_accumulator_concat(left: Seq<u8>, right: Seq<u8>, lane: Int) {
    if right.len() >= 4 {
        let prefix = right.subsequence(0, right.len() - 4);
        lane_sum_concat(left, right, lane);
        lane_sum_concat(left, prefix, lane);
        lane_accumulator_concat(left, prefix, lane);
        proof_assert!(right.len() == prefix.len() + 4);
        proof_assert!(right.len() / 4 == prefix.len() / 4 + 1);
        multiplication_successor(prefix.len() / 4, lane_sum(left, lane));
        proof_assert!(
            left.concat(right)
                .subsequence(0, left.len() + right.len() - 4)
                == left.concat(prefix)
        );
        proof_assert!(
            lane_accumulator(left.concat(right), lane)
                == lane_accumulator(left.concat(prefix), lane) + lane_sum(left.concat(right), lane)
        );
        proof_assert!(
            lane_accumulator(right, lane) == lane_accumulator(prefix, lane) + lane_sum(right, lane)
        );
        proof_assert!(
            lane_accumulator(left.concat(right), lane)
                == lane_accumulator(left, lane)
                    + (right.len() / 4) * lane_sum(left, lane)
                    + lane_accumulator(right, lane)
        );
    }
}

#[logic(opaque)]
#[requires(left.len() % 4 == 0 && right.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(result == lane_accumulator(left.concat(right), lane))]
#[ensures(result == lane_accumulator(left, lane)
    + (right.len() / 4) * lane_sum(left, lane)
    + lane_accumulator(right, lane))]
fn lane_accumulator_concat_value(left: Seq<u8>, right: Seq<u8>, lane: Int) -> Int {
    lane_accumulator_concat(left, right, lane);
    lane_accumulator(left.concat(right), lane)
}

#[logic(open)]
fn lane_accumulator_concat_facts(
    combined: Int,
    left_accumulator: Int,
    groups: Int,
    left_sum: Int,
    right_accumulator: Int,
) -> bool {
    pearlite! {
        combined == left_accumulator + groups * left_sum + right_accumulator
    }
}

#[logic]
#[requires(left.len() % 4 == 0 && right.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(result)]
#[ensures(result == lane_accumulator_concat_facts(
    lane_accumulator(left.concat(right), lane),
    lane_accumulator(left, lane),
    right.len() / 4,
    lane_sum(left, lane),
    lane_accumulator(right, lane),
))]
fn lane_accumulator_concat_certificate(left: Seq<u8>, right: Seq<u8>, lane: Int) -> bool {
    lane_accumulator_concat(left, right, lane);
    lane_accumulator_concat_facts(
        lane_accumulator(left.concat(right), lane),
        lane_accumulator(left, lane),
        right.len() / 4,
        lane_sum(left, lane),
        lane_accumulator(right, lane),
    )
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(a_after == a_before + lane_sum(chunk, lane))]
#[ensures(result == a_after)]
#[ensures(adler_congruent(a_after, lane_sum(prefix.concat(chunk), lane)))]
fn lane_sum_congruence_step(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    lane: Int,
    a_before: Int,
    a_after: Int,
) -> Int {
    lane_sum_concat(prefix, chunk, lane);
    adler_congruent_refl(lane_sum(chunk, lane));
    adler_congruent_add(
        a_before,
        lane_sum(prefix, lane),
        lane_sum(chunk, lane),
        lane_sum(chunk, lane),
    );
    a_after
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(a_after == a_before + lane_sum(chunk, lane))]
#[ensures(result)]
#[ensures(result == adler_congruence_facts(
    a_after,
    lane_sum(prefix.concat(chunk), lane),
))]
#[ensures(adler_congruent(a_after, lane_sum(prefix.concat(chunk), lane)))]
fn lane_sum_congruence_certificate(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    lane: Int,
    a_before: Int,
    a_after: Int,
) -> bool {
    lane_sum_congruence_step(prefix, chunk, lane, a_before, a_after);
    true
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(adler_congruent(a_after, a_before + lane_sum(chunk, lane)))]
#[ensures(adler_congruent(a_after, lane_sum(prefix.concat(chunk), lane)))]
fn lane_sum_reduced_certificate(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    lane: Int,
    a_before: Int,
    a_after: Int,
) {
    lane_sum_congruence_step(
        prefix, chunk, lane, a_before, a_before + lane_sum(chunk, lane),
    );
    adler_congruent_trans(
        a_after,
        a_before + lane_sum(chunk, lane),
        lane_sum(prefix.concat(chunk), lane),
    );
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, lane)))]
#[requires(adler_congruent(
    b_after,
    b_before + groups * a_before + lane_accumulator(chunk, lane),
))]
#[ensures(adler_congruent(
    b_after,
    lane_accumulator(prefix.concat(chunk), lane),
))]
fn lane_accumulator_reduced_certificate(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    lane: Int,
    groups: Int,
    a_before: Int,
    b_before: Int,
    b_after: Int,
) {
    let exact = b_before + groups * a_before + lane_accumulator(chunk, lane);
    lane_accumulator_congruence_step(
        prefix, chunk, lane, groups, a_before, b_before, exact,
    );
    adler_congruent_trans(
        b_after,
        exact,
        lane_accumulator(prefix.concat(chunk), lane),
    );
}

#[logic(opaque)]
#[requires(a == initial_a)]
#[requires(adler_congruent(
    b_before,
    initial_b + processed * chunk_size * initial_a,
))]
#[requires(adler_congruent(b_after, b_before + chunk_size * a))]
#[ensures(adler_congruent(
    b_after,
    initial_b + (processed + 1) * chunk_size * initial_a,
))]
fn scalar_chunk_reduced_certificate(
    a: Int,
    b_before: Int,
    b_after: Int,
    initial_a: Int,
    initial_b: Int,
    processed: Int,
    chunk_size: Int,
) {
    let old_target = initial_b + processed * chunk_size * initial_a;
    adler_congruent_refl(chunk_size * initial_a);
    adler_congruent_add(
        b_before,
        old_target,
        chunk_size * a,
        chunk_size * initial_a,
    );
    multiplication_successor(processed, chunk_size * initial_a);
    adler_congruent_trans(
        b_after,
        b_before + chunk_size * a,
        initial_b + (processed + 1) * chunk_size * initial_a,
    );
}

#[logic(opaque)]
#[requires(adler_congruent(b_before, old_target))]
#[requires(adler_congruent(b_after, b_before + delta))]
#[requires(old_target + delta == next_target)]
#[ensures(adler_congruent(b_after, next_target))]
fn scalar_target_advance_certificate(
    b_before: Int,
    b_after: Int,
    old_target: Int,
    delta: Int,
    next_target: Int,
) {
    adler_congruent_refl(delta);
    adler_congruent_add(b_before, old_target, delta, delta);
    adler_congruent_trans(b_after, b_before + delta, old_target + delta);
    adler_congruent_reindex(b_after, old_target + delta, next_target);
}

#[logic]
#[requires(chunk_len == chunk_size)]
#[requires(a_before == a_after)]
#[ensures(chunk_len * a_before == chunk_size * a_after)]
fn scalar_delta_reindex_certificate(
    chunk_len: Int,
    chunk_size: Int,
    a_before: Int,
    a_after: Int,
) {
}

#[logic]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 1)))]
#[requires(a_after == a_before + lane_sum(chunk, 1))]
#[ensures(result)]
#[ensures(adler_congruent(a_after, lane_sum(prefix.concat(chunk), 1)))]
fn lane_sum_congruence_certificate_1(
    prefix: Seq<u8>, chunk: Seq<u8>, a_before: Int, a_after: Int,
) -> bool {
    lane_sum_congruence_step(prefix, chunk, 1, a_before, a_after);
    true
}

#[logic]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 2)))]
#[requires(a_after == a_before + lane_sum(chunk, 2))]
#[ensures(result)]
#[ensures(adler_congruent(a_after, lane_sum(prefix.concat(chunk), 2)))]
fn lane_sum_congruence_certificate_2(
    prefix: Seq<u8>, chunk: Seq<u8>, a_before: Int, a_after: Int,
) -> bool {
    lane_sum_congruence_step(prefix, chunk, 2, a_before, a_after);
    true
}

#[logic]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 3)))]
#[requires(a_after == a_before + lane_sum(chunk, 3))]
#[ensures(result)]
#[ensures(adler_congruent(a_after, lane_sum(prefix.concat(chunk), 3)))]
fn lane_sum_congruence_certificate_3(
    prefix: Seq<u8>, chunk: Seq<u8>, a_before: Int, a_after: Int,
) -> bool {
    lane_sum_congruence_step(prefix, chunk, 3, a_before, a_after);
    true
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, lane)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, lane))]
#[ensures(result == b_after)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), lane)))]
fn lane_accumulator_congruence_step(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    lane: Int,
    groups: Int,
    a_before: Int,
    b_before: Int,
    b_after: Int,
) -> Int {
    lane_accumulator_concat(prefix, chunk, lane);
    adler_congruent_scale(a_before, lane_sum(prefix, lane), groups);
    adler_congruent_add(
        b_before,
        lane_accumulator(prefix, lane),
        groups * a_before,
        groups * lane_sum(prefix, lane),
    );
    adler_congruent_refl(lane_accumulator(chunk, lane));
    adler_congruent_add(
        b_before + groups * a_before,
        lane_accumulator(prefix, lane) + groups * lane_sum(prefix, lane),
        lane_accumulator(chunk, lane),
        lane_accumulator(chunk, lane),
    );
    b_after
}

#[logic(open)]
fn lane_accumulator_congruence_facts(b_after: Int, combined_accumulator: Int) -> bool {
    pearlite! {
        exists<factor: Int> b_after == combined_accumulator + factor * 65521
    }
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, lane)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, lane))]
#[ensures(result)]
#[ensures(result == lane_accumulator_congruence_facts(
    b_after,
    lane_accumulator(prefix.concat(chunk), lane),
))]
#[ensures(adler_congruent(
    b_after,
    lane_accumulator(prefix.concat(chunk), lane),
))]
fn lane_accumulator_congruence_certificate(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    lane: Int,
    groups: Int,
    a_before: Int,
    b_before: Int,
    b_after: Int,
) -> bool {
    lane_accumulator_congruence_step(
        prefix, chunk, lane, groups, a_before, b_before, b_after,
    );
    lane_accumulator_congruence_facts(
        b_after,
        lane_accumulator(prefix.concat(chunk), lane),
    )
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, lane)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, lane)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, lane))]
#[ensures(lane_accumulator_congruence_facts(
    b_after,
    lane_accumulator(prefix.concat(chunk), lane),
))]
fn lane_accumulator_congruence_lemma(
    prefix: Seq<u8>, chunk: Seq<u8>, lane: Int, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) {
    lane_accumulator_congruence_step(
        prefix, chunk, lane, groups, a_before, b_before, b_after,
    );
}

#[logic(opaque)]
#[ensures(result == (prefix.len() % 4 == 0 && chunk.len() % 4 == 0
    && groups == chunk.len() / 4
    && adler_congruent(a_before, lane_sum(prefix, 0))
    && adler_congruent(b_before, lane_accumulator(prefix, 0))
    && b_after == b_before + groups * a_before + lane_accumulator(chunk, 0)))]
fn lane_accumulator_congruence_ready_0(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    pearlite! {
        prefix.len() % 4 == 0 && chunk.len() % 4 == 0
            && groups == chunk.len() / 4
            && adler_congruent(a_before, lane_sum(prefix, 0))
            && adler_congruent(b_before, lane_accumulator(prefix, 0))
            && b_after == b_before + groups * a_before
                + lane_accumulator(chunk, 0)
    }
}

#[logic(opaque)]
#[requires(lane_accumulator_congruence_ready_0(
    prefix, chunk, groups, a_before, b_before, b_after,
))]
#[ensures(lane_accumulator_congruence_facts(
    b_after, lane_accumulator(prefix.concat(chunk), 0),
))]
fn lane_accumulator_congruence_lemma_0(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) {
    lane_accumulator_congruence_lemma(
        prefix, chunk, 0, groups, a_before, b_before, b_after,
    );
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0
    && groups == chunk.len() / 4
    && adler_congruent(a_before, lane_sum(prefix, 1))
    && adler_congruent(b_before, lane_accumulator(prefix, 1))
    && b_after == b_before + groups * a_before + lane_accumulator(chunk, 1))]
#[ensures(lane_accumulator_congruence_facts(
    b_after, lane_accumulator(prefix.concat(chunk), 1),
))]
fn lane_accumulator_congruence_lemma_1(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) {
    lane_accumulator_congruence_lemma(
        prefix, chunk, 1, groups, a_before, b_before, b_after,
    );
}

#[logic(opaque)]
#[ensures(result == (prefix.len() % 4 == 0 && chunk.len() % 4 == 0
    && groups == chunk.len() / 4
    && adler_congruent(a_before, lane_sum(prefix, 2))
    && adler_congruent(b_before, lane_accumulator(prefix, 2))
    && b_after == b_before + groups * a_before + lane_accumulator(chunk, 2)))]
fn lane_accumulator_congruence_ready_2(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    pearlite! {
        prefix.len() % 4 == 0 && chunk.len() % 4 == 0
            && groups == chunk.len() / 4
            && adler_congruent(a_before, lane_sum(prefix, 2))
            && adler_congruent(b_before, lane_accumulator(prefix, 2))
            && b_after == b_before + groups * a_before
                + lane_accumulator(chunk, 2)
    }
}

#[logic(opaque)]
#[requires(lane_accumulator_congruence_ready_2(
    prefix, chunk, groups, a_before, b_before, b_after,
))]
#[ensures(lane_accumulator_congruence_facts(
    b_after, lane_accumulator(prefix.concat(chunk), 2),
))]
fn lane_accumulator_congruence_lemma_2(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) {
    lane_accumulator_congruence_lemma(
        prefix, chunk, 2, groups, a_before, b_before, b_after,
    );
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0
    && groups == chunk.len() / 4
    && adler_congruent(a_before, lane_sum(prefix, 3))
    && adler_congruent(b_before, lane_accumulator(prefix, 3))
    && b_after == b_before + groups * a_before + lane_accumulator(chunk, 3))]
#[ensures(lane_accumulator_congruence_facts(
    b_after, lane_accumulator(prefix.concat(chunk), 3),
))]
#[ensures(adler_congruent(
    b_after, lane_accumulator(prefix.concat(chunk), 3),
))]
fn lane_accumulator_congruence_lemma_3(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) {
    lane_accumulator_congruence_lemma(
        prefix, chunk, 3, groups, a_before, b_before, b_after,
    );
}

#[logic(opaque)]
#[requires(lane_accumulator_congruence_ready_0(
    prefix, chunk, groups, a_before, b_before, b_after,
))]
#[ensures(result)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 0)))]
fn lane_accumulator_congruence_certificate_0(
    prefix: Seq<u8>,
    chunk: Seq<u8>,
    groups: Int,
    a_before: Int,
    b_before: Int,
    b_after: Int,
) -> bool {
    lane_accumulator_congruence_step(
        prefix, chunk, 0, groups, a_before, b_before, b_after,
    );
    true
}

#[logic(opaque)]
#[ensures(result == (prefix.len() % 4 == 0 && chunk.len() % 4 == 0
    && groups == chunk.len() / 4
    && adler_congruent(a_before, lane_sum(prefix, 1))
    && adler_congruent(b_before, lane_accumulator(prefix, 1))
    && b_after == b_before + groups * a_before + lane_accumulator(chunk, 1)))]
fn lane_accumulator_congruence_ready_1(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    pearlite! {
        prefix.len() % 4 == 0 && chunk.len() % 4 == 0
            && groups == chunk.len() / 4
            && adler_congruent(a_before, lane_sum(prefix, 1))
            && adler_congruent(b_before, lane_accumulator(prefix, 1))
            && b_after == b_before + groups * a_before
                + lane_accumulator(chunk, 1)
    }
}

#[logic(opaque)]
#[requires(lane_accumulator_congruence_ready_1(
    prefix, chunk, groups, a_before, b_before, b_after,
))]
#[ensures(result)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 1)))]
fn lane_accumulator_congruence_certificate_1(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    lane_accumulator_congruence_step(
        prefix, chunk, 1, groups, a_before, b_before, b_after,
    );
    true
}

#[logic(opaque)]
#[requires(lane_accumulator_congruence_ready_2(
    prefix, chunk, groups, a_before, b_before, b_after,
))]
#[ensures(result)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 2)))]
fn lane_accumulator_congruence_certificate_2(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    lane_accumulator_congruence_step(
        prefix, chunk, 2, groups, a_before, b_before, b_after,
    );
    true
}

#[logic(opaque)]
#[ensures(result == (prefix.len() % 4 == 0 && chunk.len() % 4 == 0
    && groups == chunk.len() / 4
    && adler_congruent(a_before, lane_sum(prefix, 3))
    && adler_congruent(b_before, lane_accumulator(prefix, 3))
    && b_after == b_before + groups * a_before + lane_accumulator(chunk, 3)))]
fn lane_accumulator_congruence_ready_3(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    pearlite! {
        prefix.len() % 4 == 0 && chunk.len() % 4 == 0
            && groups == chunk.len() / 4
            && adler_congruent(a_before, lane_sum(prefix, 3))
            && adler_congruent(b_before, lane_accumulator(prefix, 3))
            && b_after == b_before + groups * a_before
                + lane_accumulator(chunk, 3)
    }
}

#[logic(opaque)]
#[requires(lane_accumulator_congruence_ready_3(
    prefix, chunk, groups, a_before, b_before, b_after,
))]
#[ensures(result)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 3)))]
fn lane_accumulator_congruence_certificate_3(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> bool {
    lane_accumulator_congruence_step(
        prefix, chunk, 3, groups, a_before, b_before, b_after,
    );
    true
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 0)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, 0)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, 0))]
#[ensures(result == b_after)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 0)))]
fn lane_accumulator_congruence_step_0(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> Int {
    lane_accumulator_congruence_step(
        prefix, chunk, 0, groups, a_before, b_before, b_after,
    )
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 1)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, 1)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, 1))]
#[ensures(result == b_after)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 1)))]
fn lane_accumulator_congruence_step_1(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> Int {
    lane_accumulator_congruence_step(
        prefix, chunk, 1, groups, a_before, b_before, b_after,
    )
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 2)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, 2)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, 2))]
#[ensures(result == b_after)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 2)))]
fn lane_accumulator_congruence_step_2(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> Int {
    lane_accumulator_congruence_step(
        prefix, chunk, 2, groups, a_before, b_before, b_after,
    )
}

#[logic(opaque)]
#[requires(prefix.len() % 4 == 0 && chunk.len() % 4 == 0)]
#[requires(groups == chunk.len() / 4)]
#[requires(adler_congruent(a_before, lane_sum(prefix, 3)))]
#[requires(adler_congruent(b_before, lane_accumulator(prefix, 3)))]
#[requires(b_after == b_before + groups * a_before + lane_accumulator(chunk, 3))]
#[ensures(result == b_after)]
#[ensures(adler_congruent(b_after, lane_accumulator(prefix.concat(chunk), 3)))]
fn lane_accumulator_congruence_step_3(
    prefix: Seq<u8>, chunk: Seq<u8>, groups: Int,
    a_before: Int, b_before: Int, b_after: Int,
) -> Int {
    lane_accumulator_congruence_step(
        prefix, chunk, 3, groups, a_before, b_before, b_after,
    )
}

#[logic]
#[requires(prefix.len() % 4 == 0)]
#[requires(group.len() == 4)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_sum(prefix.concat(group), lane)
    == lane_sum(prefix, lane) + group[lane]@)]
#[ensures(lane_accumulator(prefix.concat(group), lane)
    == lane_accumulator(prefix, lane) + lane_sum(prefix, lane) + group[lane]@)]
fn lane_group_step_one(prefix: Seq<u8>, group: Seq<u8>, lane: Int) {
    lane_sum_concat(prefix, group, lane);
    proof_assert!(prefix.concat(group).subsequence(0, prefix.len()) == prefix);
    proof_assert!(prefix.concat(group)[prefix.len() + lane] == group[lane]);
    proof_assert!(
        lane_accumulator(prefix.concat(group), lane)
            == lane_accumulator(prefix, lane) + lane_sum(prefix.concat(group), lane)
    );
}

/// Extending an aligned prefix by the next four bytes advances one lane by
/// exactly the byte at that lane's position.
#[logic]
#[requires(0 <= offset && offset + 4 <= bytes.len())]
#[requires(offset % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_sum(bytes.subsequence(0, offset + 4), lane)
    == lane_sum(bytes.subsequence(0, offset), lane) + bytes[offset + lane]@)]
#[ensures(lane_accumulator(bytes.subsequence(0, offset + 4), lane)
    == lane_accumulator(bytes.subsequence(0, offset), lane)
        + lane_sum(bytes.subsequence(0, offset), lane) + bytes[offset + lane]@)]
fn lane_subsequence_step(bytes: Seq<u8>, offset: Int, lane: Int) {
    let prefix = bytes.subsequence(0, offset);
    let group = bytes.subsequence(offset, offset + 4);
    let complete = bytes.subsequence(0, offset + 4);
    proof_assert!(prefix.len() % 4 == 0);
    proof_assert!(group.len() == 4);
    lane_group_step_one(prefix, group, lane);
    proof_assert!(complete == prefix.concat(group));
    proof_assert!(group[lane] == bytes[offset + lane]);
}

#[logic]
#[requires(0 <= n && n <= 5552)]
#[requires(0 <= initial_a && initial_a < 65521)]
#[requires(0 <= initial_b && initial_b < 65521)]
#[ensures(initial_a + n * 255 <= u32::MAX@)]
#[ensures(2 * initial_b + 2 * n * initial_a + 255 * n * (n + 1) <= 2 * u32::MAX@)]
fn chunk_arithmetic_bounds(n: Int, initial_a: Int, initial_b: Int) {}

#[logic]
#[requires(0 <= n)]
#[ensures(
    2 * initial_b + 2 * n * initial_a + 255 * n * (n + 1)
        + 2 * initial_a
        + 2 * (n + 1) * 255
        == 2 * initial_b + 2 * (n + 1) * initial_a + 255 * (n + 1) * (n + 2)
)]
fn chunk_arithmetic_step(n: Int, initial_a: Int, initial_b: Int) {}

#[logic(open)]
fn chunk_iteration_facts(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    current_a: Int,
    current_b: Int,
    byte: Int,
) -> bool {
    pearlite! {
        current_a + byte <= u32::MAX@
            && current_b + current_a + byte <= u32::MAX@
            && current_a + byte <= initial_a + (n + 1) * 255
            && 2 * (current_b + current_a + byte)
                <= 2 * initial_b
                    + 2 * (n + 1) * initial_a
                    + 255 * (n + 1) * (n + 2)
    }
}

#[logic]
#[requires(0 <= n && n < 5552)]
#[requires(0 <= initial_a && initial_a < 65521)]
#[requires(0 <= initial_b && initial_b < 65521)]
#[requires(current_a <= initial_a + n * 255)]
#[requires(2 * current_b <= 2 * initial_b + 2 * n * initial_a + 255 * n * (n + 1))]
#[requires(0 <= byte && byte <= 255)]
#[ensures(chunk_iteration_facts(n, initial_a, initial_b, current_a, current_b, byte))]
fn chunk_iteration_safe(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    current_a: Int,
    current_b: Int,
    byte: Int,
) {
    chunk_arithmetic_bounds(n + 1, initial_a, initial_b);
    chunk_arithmetic_step(n, initial_a, initial_b);
}

/// Packages the elementary algebra used by one lane of the four-byte loop.
/// Keeping this separate prevents the prover from having to rediscover the
/// same recurrence in the presence of all four lanes and the iterator model.
#[logic]
#[requires(old_a == initial_a + old_sum)]
#[requires(old_b == initial_b + n * initial_a + old_accumulator)]
#[requires(new_sum == old_sum + byte)]
#[requires(new_accumulator == old_accumulator + old_sum + byte)]
#[ensures(old_a + byte == initial_a + new_sum)]
#[ensures(old_b + old_a + byte
    == initial_b + (n + 1) * initial_a + new_accumulator)]
fn lane_iteration_exact(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    old_sum: Int,
    old_accumulator: Int,
    byte: Int,
    new_sum: Int,
    new_accumulator: Int,
) {
}

#[logic]
#[requires(chunk_iteration_facts(n, initial_a, initial_b, old_a, old_b, byte))]
#[ensures(2 * (old_b + old_a + byte)
    <= 2 * initial_b
        + 2 * (n + 1) * initial_a
        + 255 * (n + 1) * (n + 2))]
fn lane_iteration_bound(n: Int, initial_a: Int, initial_b: Int, old_a: Int, old_b: Int, byte: Int) {
}

#[logic(open)]
fn lane_pre_facts(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    byte: Int,
    new_sum: Int,
    new_accumulator: Int,
) -> bool {
    pearlite! {
        old_a + byte == initial_a + new_sum
            && old_b + old_a + byte
                == initial_b + (n + 1) * initial_a + new_accumulator
            && 2 * (old_b + old_a + byte)
                <= 2 * initial_b + 2 * (n + 1) * initial_a
                    + 255 * (n + 1) * (n + 2)
    }
}

#[logic]
#[requires(old_a == initial_a + old_sum)]
#[requires(old_b == initial_b + n * initial_a + old_accumulator)]
#[requires(new_sum == old_sum + byte)]
#[requires(new_accumulator == old_accumulator + old_sum + byte)]
#[requires(chunk_iteration_facts(n, initial_a, initial_b, old_a, old_b, byte))]
#[ensures(lane_pre_facts(n, initial_a, initial_b, old_a, old_b, byte, new_sum, new_accumulator))]
fn lane_iteration_prepare(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    old_sum: Int,
    old_accumulator: Int,
    byte: Int,
    new_sum: Int,
    new_accumulator: Int,
) {
    lane_iteration_exact(
        n,
        initial_a,
        initial_b,
        old_a,
        old_b,
        old_sum,
        old_accumulator,
        byte,
        new_sum,
        new_accumulator,
    );
    lane_iteration_bound(n, initial_a, initial_b, old_a, old_b, byte);
}

#[logic(open)]
fn partial_chunk_facts(len: Int, a: Int, b: Int) -> bool {
    pearlite! { len * a <= u32::MAX@ && b + len * a <= u32::MAX@ }
}

#[logic]
#[requires(0 <= len && len <= 22208)]
#[requires(0 <= a && 0 <= b)]
#[requires(b + 22208 * a <= u32::MAX@)]
#[ensures(partial_chunk_facts(len, a, b))]
fn partial_chunk_safe(len: Int, a: Int, b: Int) {}

#[logic(open)]
fn reduced_state_facts(a: Int, b: Int) -> bool {
    pearlite! { b % 65521 + 22208 * a <= u32::MAX@ }
}

#[logic]
#[requires(0 <= a && a < 65521)]
#[requires(0 <= b && b < 65521)]
#[ensures(b + 22208 * a <= u32::MAX@)]
fn initial_state_safe(a: Int, b: Int) {}

#[logic]
#[requires(0 <= a && a <= u16::MAX@ && 0 <= b)]
#[ensures(reduced_state_facts(a, b))]
fn reduced_state_safe(a: Int, b: Int) {}

#[inline]
#[requires(chunk@.len() % 4 == 0 && chunk@.len() <= 22208)]
#[requires((*a_vec).invariant() && (*b_vec).invariant())]
#[requires((*a_vec)@.0@ < 65521 && (*a_vec)@.1@ < 65521 && (*a_vec)@.2@ < 65521 && (*a_vec)@.3@ < 65521)]
#[requires((*b_vec)@.0@ < 65521 && (*b_vec)@.1@ < 65521 && (*b_vec)@.2@ < 65521 && (*b_vec)@.3@ < 65521)]
#[ensures((^a_vec)@.0@ == (*a_vec)@.0@ + lane_sum(chunk@, 0))]
#[ensures((^a_vec)@.1@ == (*a_vec)@.1@ + lane_sum(chunk@, 1))]
#[ensures((^a_vec)@.2@ == (*a_vec)@.2@ + lane_sum(chunk@, 2))]
#[ensures((^a_vec)@.3@ == (*a_vec)@.3@ + lane_sum(chunk@, 3))]
#[ensures((^b_vec)@.0@ == (*b_vec)@.0@ + chunk@.len() / 4 * (*a_vec)@.0@ + lane_accumulator(chunk@, 0))]
#[ensures((^b_vec)@.1@ == (*b_vec)@.1@ + chunk@.len() / 4 * (*a_vec)@.1@ + lane_accumulator(chunk@, 1))]
#[ensures((^b_vec)@.2@ == (*b_vec)@.2@ + chunk@.len() / 4 * (*a_vec)@.2@ + lane_accumulator(chunk@, 2))]
#[ensures((^b_vec)@.3@ == (*b_vec)@.3@ + chunk@.len() / 4 * (*a_vec)@.3@ + lane_accumulator(chunk@, 3))]
// TODO: Remove `trusted` after completing the retained loop proof below.
#[trusted]
fn process_chunk(a_vec: &mut U32X4, b_vec: &mut U32X4, chunk: &[u8]) {
    let a_vec_entry = snapshot! { *a_vec };
    let b_vec_entry = snapshot! { *b_vec };
    let mut offset = 0usize;
    let mut group_count = 0usize;
    #[invariant(offset@ <= chunk@.len())]
    #[invariant(offset@ % 4 == 0)]
    #[invariant(offset@ == 4 * group_count@)]
    #[invariant(a_vec.invariant())]
    #[invariant(b_vec.invariant())]
    #[invariant(a_vec@.0@ == a_vec_entry@.0@ + lane_sum(chunk@.subsequence(0, offset@), 0))]
    #[invariant(a_vec@.1@ == a_vec_entry@.1@ + lane_sum(chunk@.subsequence(0, offset@), 1))]
    #[invariant(a_vec@.2@ == a_vec_entry@.2@ + lane_sum(chunk@.subsequence(0, offset@), 2))]
    #[invariant(a_vec@.3@ == a_vec_entry@.3@ + lane_sum(chunk@.subsequence(0, offset@), 3))]
    #[invariant(b_vec@.0@ == b_vec_entry@.0@ + group_count@ * a_vec_entry@.0@ + lane_accumulator(chunk@.subsequence(0, offset@), 0))]
    #[invariant(b_vec@.1@ == b_vec_entry@.1@ + group_count@ * a_vec_entry@.1@ + lane_accumulator(chunk@.subsequence(0, offset@), 1))]
    #[invariant(b_vec@.2@ == b_vec_entry@.2@ + group_count@ * a_vec_entry@.2@ + lane_accumulator(chunk@.subsequence(0, offset@), 2))]
    #[invariant(b_vec@.3@ == b_vec_entry@.3@ + group_count@ * a_vec_entry@.3@ + lane_accumulator(chunk@.subsequence(0, offset@), 3))]
    #[invariant(a_vec@.0@ <= a_vec_entry@.0@ + group_count@ * 255)]
    #[invariant(a_vec@.1@ <= a_vec_entry@.1@ + group_count@ * 255)]
    #[invariant(a_vec@.2@ <= a_vec_entry@.2@ + group_count@ * 255)]
    #[invariant(a_vec@.3@ <= a_vec_entry@.3@ + group_count@ * 255)]
    #[invariant(2 * b_vec@.0@ <= 2 * b_vec_entry@.0@ + 2 * group_count@ * a_vec_entry@.0@ + 255 * group_count@ * (group_count@ + 1))]
    #[invariant(2 * b_vec@.1@ <= 2 * b_vec_entry@.1@ + 2 * group_count@ * a_vec_entry@.1@ + 255 * group_count@ * (group_count@ + 1))]
    #[invariant(2 * b_vec@.2@ <= 2 * b_vec_entry@.2@ + 2 * group_count@ * a_vec_entry@.2@ + 255 * group_count@ * (group_count@ + 1))]
    #[invariant(2 * b_vec@.3@ <= 2 * b_vec_entry@.3@ + 2 * group_count@ * a_vec_entry@.3@ + 255 * group_count@ * (group_count@ + 1))]
    #[variant(chunk@.len() - offset@)]
    while offset < chunk.len() {
        proof_assert!(offset@ + 4 <= chunk@.len());
        let val = U32X4([
            u32::from(chunk[offset]),
            u32::from(chunk[offset + 1]),
            u32::from(chunk[offset + 2]),
            u32::from(chunk[offset + 3]),
        ]);
        let groups = snapshot! { group_count@ };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@); chunk_iteration_facts(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@) };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@); chunk_iteration_facts(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@) };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@); chunk_iteration_facts(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@) };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@); chunk_iteration_facts(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@) };
        let a_vec_before = snapshot! { *a_vec };
        let b_vec_before = snapshot! { *b_vec };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            let complete = chunk@.subsequence(0, offset@ + 4);
            lane_subsequence_step(chunk@, offset@, 0);
            lane_iteration_prepare(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@, b_vec_before@.0@, lane_sum(prefix, 0), lane_accumulator(prefix, 0), val@.0@, lane_sum(complete, 0), lane_accumulator(complete, 0));
            lane_pre_facts(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@, b_vec_before@.0@, val@.0@, lane_sum(complete, 0), lane_accumulator(complete, 0))
        };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            let complete = chunk@.subsequence(0, offset@ + 4);
            lane_subsequence_step(chunk@, offset@, 1);
            lane_iteration_prepare(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@, b_vec_before@.1@, lane_sum(prefix, 1), lane_accumulator(prefix, 1), val@.1@, lane_sum(complete, 1), lane_accumulator(complete, 1));
            lane_pre_facts(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@, b_vec_before@.1@, val@.1@, lane_sum(complete, 1), lane_accumulator(complete, 1))
        };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            let complete = chunk@.subsequence(0, offset@ + 4);
            lane_subsequence_step(chunk@, offset@, 2);
            lane_iteration_prepare(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@, b_vec_before@.2@, lane_sum(prefix, 2), lane_accumulator(prefix, 2), val@.2@, lane_sum(complete, 2), lane_accumulator(complete, 2));
            lane_pre_facts(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@, b_vec_before@.2@, val@.2@, lane_sum(complete, 2), lane_accumulator(complete, 2))
        };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            let complete = chunk@.subsequence(0, offset@ + 4);
            lane_subsequence_step(chunk@, offset@, 3);
            lane_iteration_prepare(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@, b_vec_before@.3@, lane_sum(prefix, 3), lane_accumulator(prefix, 3), val@.3@, lane_sum(complete, 3), lane_accumulator(complete, 3));
            lane_pre_facts(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@, b_vec_before@.3@, val@.3@, lane_sum(complete, 3), lane_accumulator(complete, 3))
        };
        proof_assert!(a_vec@.0@ + val@.0@ <= u32::MAX@ && a_vec@.1@ + val@.1@ <= u32::MAX@ && a_vec@.2@ + val@.2@ <= u32::MAX@ && a_vec@.3@ + val@.3@ <= u32::MAX@);
        *a_vec += val;
        proof_assert!(a_vec@.0@ == a_vec_before@.0@ + val@.0@);
        proof_assert!(a_vec@.1@ == a_vec_before@.1@ + val@.1@);
        proof_assert!(a_vec@.2@ == a_vec_before@.2@ + val@.2@);
        proof_assert!(a_vec@.3@ == a_vec_before@.3@ + val@.3@);
        proof_assert!(b_vec@.0@ + a_vec@.0@ <= u32::MAX@);
        proof_assert!(b_vec@.1@ + a_vec@.1@ <= u32::MAX@);
        proof_assert!(b_vec@.2@ + a_vec@.2@ <= u32::MAX@);
        proof_assert!(b_vec@.3@ + a_vec@.3@ <= u32::MAX@);
        *b_vec += *a_vec;
        proof_assert!(b_vec@.0@ == b_vec_before@.0@ + a_vec@.0@);
        proof_assert!(b_vec@.1@ == b_vec_before@.1@ + a_vec@.1@);
        proof_assert!(b_vec@.2@ == b_vec_before@.2@ + a_vec@.2@);
        proof_assert!(b_vec@.3@ == b_vec_before@.3@ + a_vec@.3@);
        offset += 4;
        group_count += 1;
        proof_assert!(group_count@ == *groups + 1);
        proof_assert!(group_count@ + 1 == *groups + 2);
        let consecutive_product = snapshot! {
            consecutive_product_reindex(group_count@, *groups)
        };
        proof_assert!(*consecutive_product == group_count@ * (group_count@ + 1));
        proof_assert!(*consecutive_product == (*groups + 1) * (*groups + 2));
        proof_assert!(group_count@ * (group_count@ + 1)
            == (*groups + 1) * (*groups + 2));
        proof_assert!(group_count@ * a_vec_entry@.0@
            == (*groups + 1) * a_vec_entry@.0@);
        proof_assert!(group_count@ * a_vec_entry@.1@
            == (*groups + 1) * a_vec_entry@.1@);
        proof_assert!(group_count@ * a_vec_entry@.2@
            == (*groups + 1) * a_vec_entry@.2@);
        proof_assert!(group_count@ * a_vec_entry@.3@
            == (*groups + 1) * a_vec_entry@.3@);
        proof_assert!(b_vec@.0@
            == b_vec_before@.0@ + a_vec_before@.0@ + val@.0@);
        proof_assert!(b_vec@.1@
            == b_vec_before@.1@ + a_vec_before@.1@ + val@.1@);
        proof_assert!(b_vec@.2@
            == b_vec_before@.2@ + a_vec_before@.2@ + val@.2@);
        proof_assert!(b_vec@.3@
            == b_vec_before@.3@ + a_vec_before@.3@ + val@.3@);
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            lane_pre_facts(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@, b_vec_before@.0@, val@.0@, lane_sum(complete, 0), lane_accumulator(complete, 0))
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            lane_pre_facts(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@, b_vec_before@.1@, val@.1@, lane_sum(complete, 1), lane_accumulator(complete, 1))
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            lane_pre_facts(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@, b_vec_before@.2@, val@.2@, lane_sum(complete, 2), lane_accumulator(complete, 2))
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            lane_pre_facts(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@, b_vec_before@.3@, val@.3@, lane_sum(complete, 3), lane_accumulator(complete, 3))
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec_before@.0@ + val@.0@
                == a_vec_entry@.0@ + lane_sum(complete, 0)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec_before@.0@ + a_vec_before@.0@ + val@.0@
                == b_vec_entry@.0@ + (*groups + 1) * a_vec_entry@.0@
                    + lane_accumulator(complete, 0)
        };
        proof_assert!(2 * (b_vec_before@.0@ + a_vec_before@.0@ + val@.0@)
            <= 2 * b_vec_entry@.0@ + 2 * (*groups + 1) * a_vec_entry@.0@
                + 255 * (*groups + 1) * (*groups + 2));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec_before@.1@ + val@.1@
                == a_vec_entry@.1@ + lane_sum(complete, 1)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec_before@.1@ + a_vec_before@.1@ + val@.1@
                == b_vec_entry@.1@ + (*groups + 1) * a_vec_entry@.1@
                    + lane_accumulator(complete, 1)
        };
        proof_assert!(2 * (b_vec_before@.1@ + a_vec_before@.1@ + val@.1@)
            <= 2 * b_vec_entry@.1@ + 2 * (*groups + 1) * a_vec_entry@.1@
                + 255 * (*groups + 1) * (*groups + 2));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec_before@.2@ + val@.2@
                == a_vec_entry@.2@ + lane_sum(complete, 2)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec_before@.2@ + a_vec_before@.2@ + val@.2@
                == b_vec_entry@.2@ + (*groups + 1) * a_vec_entry@.2@
                    + lane_accumulator(complete, 2)
        };
        proof_assert!(2 * (b_vec_before@.2@ + a_vec_before@.2@ + val@.2@)
            <= 2 * b_vec_entry@.2@ + 2 * (*groups + 1) * a_vec_entry@.2@
                + 255 * (*groups + 1) * (*groups + 2));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec_before@.3@ + val@.3@
                == a_vec_entry@.3@ + lane_sum(complete, 3)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec_before@.3@ + a_vec_before@.3@ + val@.3@
                == b_vec_entry@.3@ + (*groups + 1) * a_vec_entry@.3@
                    + lane_accumulator(complete, 3)
        };
        proof_assert!(2 * (b_vec_before@.3@ + a_vec_before@.3@ + val@.3@)
            <= 2 * b_vec_entry@.3@ + 2 * (*groups + 1) * a_vec_entry@.3@
                + 255 * (*groups + 1) * (*groups + 2));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec@.0@ == a_vec_entry@.0@ + lane_sum(complete, 0)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec@.0@ == b_vec_entry@.0@ + group_count@ * a_vec_entry@.0@
                + lane_accumulator(complete, 0)
        };
        proof_assert!(2 * b_vec@.0@ <= 2 * b_vec_entry@.0@
            + 2 * group_count@ * a_vec_entry@.0@
            + 255 * group_count@ * (group_count@ + 1));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec@.1@ == a_vec_entry@.1@ + lane_sum(complete, 1)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec@.1@ == b_vec_entry@.1@ + group_count@ * a_vec_entry@.1@
                + lane_accumulator(complete, 1)
        };
        proof_assert!(2 * b_vec@.1@ <= 2 * b_vec_entry@.1@
            + 2 * group_count@ * a_vec_entry@.1@
            + 255 * group_count@ * (group_count@ + 1));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec@.2@ == a_vec_entry@.2@ + lane_sum(complete, 2)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec@.2@ == b_vec_entry@.2@ + group_count@ * a_vec_entry@.2@
                + lane_accumulator(complete, 2)
        };
        proof_assert!(2 * b_vec@.2@ <= 2 * b_vec_entry@.2@
            + 2 * group_count@ * a_vec_entry@.2@
            + 255 * group_count@ * (group_count@ + 1));
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            a_vec@.3@ == a_vec_entry@.3@ + lane_sum(complete, 3)
        };
        proof_assert! {
            let complete = chunk@.subsequence(0, offset@);
            b_vec@.3@ == b_vec_entry@.3@ + group_count@ * a_vec_entry@.3@
                + lane_accumulator(complete, 3)
        };
        proof_assert!(2 * b_vec@.3@ <= 2 * b_vec_entry@.3@
            + 2 * group_count@ * a_vec_entry@.3@
            + 255 * group_count@ * (group_count@ + 1));
    }
    proof_assert!(offset@ == chunk@.len());
    proof_assert!(group_count@ == chunk@.len() / 4);
    proof_assert!(chunk@.subsequence(0, offset@) == chunk@);
}

#[inline]
#[requires(chunk@.len() % 4 == 0 && chunk@.len() <= 22208)]
#[requires(a_vec.invariant() && b_vec.invariant())]
#[requires(a_vec@.0@ < 65521 && a_vec@.1@ < 65521 && a_vec@.2@ < 65521 && a_vec@.3@ < 65521)]
#[requires(b_vec@.0@ < 65521 && b_vec@.1@ < 65521 && b_vec@.2@ < 65521 && b_vec@.3@ < 65521)]
#[ensures(result.0@.0@ == a_vec@.0@ + lane_sum(chunk@, 0))]
#[ensures(result.0@.1@ == a_vec@.1@ + lane_sum(chunk@, 1))]
#[ensures(result.0@.2@ == a_vec@.2@ + lane_sum(chunk@, 2))]
#[ensures(result.0@.3@ == a_vec@.3@ + lane_sum(chunk@, 3))]
#[ensures(result.1@.0@ == b_vec@.0@ + chunk@.len() / 4 * a_vec@.0@ + lane_accumulator(chunk@, 0))]
#[ensures(result.1@.1@ == b_vec@.1@ + chunk@.len() / 4 * a_vec@.1@ + lane_accumulator(chunk@, 1))]
#[ensures(result.1@.2@ == b_vec@.2@ + chunk@.len() / 4 * a_vec@.2@ + lane_accumulator(chunk@, 2))]
#[ensures(result.1@.3@ == b_vec@.3@ + chunk@.len() / 4 * a_vec@.3@ + lane_accumulator(chunk@, 3))]
fn process_chunk_values(a_vec: U32X4, b_vec: U32X4, chunk: &[u8]) -> (U32X4, U32X4) {
    let mut next_a = a_vec;
    let mut next_b = b_vec;
    process_chunk(&mut next_a, &mut next_b, chunk);
    (next_a, next_b)
}

#[inline]
#[requires(remainder@.len() <= 3)]
#[requires(a@ <= 327600)]
#[requires(b@ <= 1900092)]
#[ensures(result.0@ == a@ + crate::adler32_byte_sum(remainder@))]
#[ensures(result.1@ == b@ + remainder@.len() * a@
    + crate::adler32_weighted_sum(remainder@))]
#[ensures(result.0@ <= 328365)]
#[ensures(result.1@ <= 2885187)]
fn process_remainder(
    mut a: u32,
    mut b: u32,
    remainder: &[u8],
) -> (u32, u32) {
    let entry_a = snapshot! { a };
    let entry_b = snapshot! { b };
    let mut index = 0usize;
    #[invariant(index@ <= remainder@.len())]
    #[invariant(index@ <= 3)]
    #[invariant(a@ <= 327600 + index@ * 255)]
    #[invariant(b@ <= 1900092 + index@ * 328365)]
    #[invariant(a@ == entry_a@ + crate::adler32_byte_sum(
        remainder@.subsequence(0, index@)))]
    #[invariant(b@ == entry_b@ + index@ * entry_a@
        + crate::adler32_weighted_sum(remainder@.subsequence(0, index@)))]
    #[variant(remainder@.len() - index@)]
    while index < remainder.len() {
        let byte = remainder[index];
        proof_assert! {
            subsequence_push_last(remainder@, index@ + 1, byte);
            remainder@.subsequence(0, index@ + 1)
                == remainder@.subsequence(0, index@).push_back(byte)
        };
        proof_assert! {
            let prefix = remainder@.subsequence(0, index@);
            crate::adler32_byte_sum_push(prefix, byte);
            crate::adler32_weighted_sum_push(prefix, byte);
            crate::adler32_byte_sum(remainder@.subsequence(0, index@ + 1))
                == crate::adler32_byte_sum(prefix) + byte@
            && crate::adler32_weighted_sum(remainder@.subsequence(0, index@ + 1))
                == crate::adler32_weighted_sum(prefix)
                    + crate::adler32_byte_sum(prefix) + byte@
        };
        let before_a = snapshot! { a };
        a += u32::from(byte);
        proof_assert! {
            remainder_a_step_certificate(
                before_a@,
                a@,
                entry_a@,
                crate::adler32_byte_sum(remainder@.subsequence(0, index@)),
                crate::adler32_byte_sum(remainder@.subsequence(0, index@ + 1)),
                byte@,
            );
            a@ == entry_a@ + crate::adler32_byte_sum(
                remainder@.subsequence(0, index@ + 1))
        };
        let before_b = snapshot! { b };
        b += a;
        proof_assert! {
            remainder_b_step_certificate(
                before_b@,
                b@,
                a@,
                entry_b@,
                entry_a@,
                index@,
                crate::adler32_byte_sum(remainder@.subsequence(0, index@)),
                crate::adler32_byte_sum(remainder@.subsequence(0, index@ + 1)),
                crate::adler32_weighted_sum(remainder@.subsequence(0, index@)),
                crate::adler32_weighted_sum(remainder@.subsequence(0, index@ + 1)),
                byte@,
            );
            b@ == entry_b@ + (index@ + 1) * entry_a@
                + crate::adler32_weighted_sum(
                    remainder@.subsequence(0, index@ + 1))
        };
        index += 1;
    }
    proof_assert!(index@ == remainder@.len());
    proof_assert!(remainder@.subsequence(0, index@) == remainder@);
    proof_assert!(a@ == entry_a@ + crate::adler32_byte_sum(remainder@));
    proof_assert!(b@ == entry_b@ + remainder@.len() * entry_a@
        + crate::adler32_weighted_sum(remainder@));
    #[cfg(any())]
    {
    proof_assert! {
        adler_congruent_refl(crate::adler32_byte_sum(remainder@));
        adler_congruent_add(
            entry_a@, *a_target,
            crate::adler32_byte_sum(remainder@),
            crate::adler32_byte_sum(remainder@),
        );
        true
    };
    }
    #[cfg(any())]
    {
    proof_assert! {
        remainder_targets_certificate(
            entry_a@,
            entry_b@,
            a@,
            b@,
            *a_target,
            *b_target,
            remainder@.len(),
            crate::adler32_byte_sum(remainder@),
            crate::adler32_weighted_sum(remainder@),
        );
        adler_congruent(
            result.0@,
            *a_target + crate::adler32_byte_sum(remainder@),
        ) && adler_congruent(
            result.1@,
            *b_target + remainder@.len() * *a_target
                + crate::adler32_weighted_sum(remainder@),
        )
    };
    }
    #[cfg(any())]
    {
    proof_assert! {
        adler_congruent_scale(entry_a@, *a_target, remainder@.len());
        adler_congruent_add(
            entry_b@, *b_target,
            remainder@.len() * entry_a@,
            remainder@.len() * *a_target,
        );
        adler_congruent_refl(crate::adler32_weighted_sum(remainder@));
        adler_congruent_add(
            entry_b@ + remainder@.len() * entry_a@,
            *b_target + remainder@.len() * *a_target,
            crate::adler32_weighted_sum(remainder@),
            crate::adler32_weighted_sum(remainder@),
        );
        true
    };
    }
    proof_assert!(a@ <= 328365);
    proof_assert!(b@ <= 2885187);
    (a, b)
}

#[inline]
#[requires(remainder@.len() <= 3)]
#[requires(a@ <= 327600 && b@ <= 1900092)]
#[requires(adler_congruent(a@, *a_target)
    && adler_congruent(b@, *b_target))]
#[ensures(result.0@ == a@ + crate::adler32_byte_sum(remainder@))]
#[ensures(result.1@ == b@ + remainder@.len() * a@
    + crate::adler32_weighted_sum(remainder@))]
#[ensures(result.0@ <= 328365 && result.1@ <= 2885187)]
#[ensures(adler_congruent(
    result.0@,
    *a_target + crate::adler32_byte_sum(remainder@),
))]
#[ensures(adler_congruent(
    result.1@,
    *b_target + remainder@.len() * *a_target
        + crate::adler32_weighted_sum(remainder@),
))]
fn process_remainder_with_targets(
    a: u32,
    b: u32,
    remainder: &[u8],
    a_target: Snapshot<Int>,
    b_target: Snapshot<Int>,
) -> (u32, u32) {
    let a_entry = snapshot! { a@ };
    let b_entry = snapshot! { b@ };
    let count = snapshot! { remainder@.len() };
    let byte_sum = snapshot! { crate::adler32_byte_sum(remainder@) };
    let weighted_sum = snapshot! { crate::adler32_weighted_sum(remainder@) };
    let result = process_remainder(a, b, remainder);
    proof_assert! { result.0@ == *a_entry + *byte_sum };
    proof_assert! {
        result.1@ == *b_entry + *count * *a_entry + *weighted_sum
    };
    proof_assert! {
        remainder_a_target_certificate(
            *a_entry,
            result.0@,
            *a_target,
            *byte_sum,
        );
        adler_congruent(result.0@, *a_target + *byte_sum)
    };
    let scaled_a_entry = snapshot! { *count * *a_entry };
    let scaled_a_target = snapshot! { *count * *a_target };
    proof_assert! {
        adler_congruent_scale(*a_entry, *a_target, *count);
        adler_congruent(*scaled_a_entry, *scaled_a_target)
    };
    let b_scaled_entry = snapshot! { *b_entry + *scaled_a_entry };
    let b_scaled_target = snapshot! { *b_target + *scaled_a_target };
    proof_assert! {
        adler_congruent_add(
            *b_entry,
            *b_target,
            *scaled_a_entry,
            *scaled_a_target,
        );
        adler_congruent(*b_scaled_entry, *b_scaled_target)
    };
    let b_full_entry = snapshot! { *b_scaled_entry + *weighted_sum };
    let b_full_target = snapshot! { *b_scaled_target + *weighted_sum };
    proof_assert! {
        adler_congruent_refl(*weighted_sum);
        adler_congruent_add(
            *b_scaled_entry,
            *b_scaled_target,
            *weighted_sum,
            *weighted_sum,
        );
        adler_congruent(*b_full_entry, *b_full_target)
    };
    proof_assert! { result.1@ == *b_full_entry };
    proof_assert! {
        adler_congruent(result.1@, *b_full_target)
    };
    result
}

#[inline]
#[requires(b@ < 65521)]
#[requires(lanes@.0@ <= 458643 && lanes@.1@ <= 458643
    && lanes@.2@ <= 458643 && lanes@.3@ <= 458643)]
#[ensures(result@ == b@ + lanes@.0@ + lanes@.1@ + lanes@.2@ + lanes@.3@)]
fn add_lane_totals(mut b: u32, lanes: U32X4) -> u32 {
    b += lanes.0[0];
    b += lanes.0[1];
    b += lanes.0[2];
    b += lanes.0[3];
    b
}

#[inline]
#[requires(chunk@.len() == 22208
    && a@ <= u16::MAX@ && b@ <= u16::MAX@
    && b@ + 22208 * a@ <= u32::MAX@
    && a_vec.invariant() && b_vec.invariant()
    && a_vec@.0@ < 65521 && a_vec@.1@ < 65521
    && a_vec@.2@ < 65521 && a_vec@.3@ < 65521
    && b_vec@.0@ < 65521 && b_vec@.1@ < 65521
    && b_vec@.2@ < 65521 && b_vec@.3@ < 65521)]
#[requires((*prefix).len() % 4 == 0
    && adler_congruent(a_vec@.0@, lane_sum((*prefix), 0))
    && adler_congruent(a_vec@.1@, lane_sum((*prefix), 1))
    && adler_congruent(a_vec@.2@, lane_sum((*prefix), 2))
    && adler_congruent(a_vec@.3@, lane_sum((*prefix), 3))
    && adler_congruent(b_vec@.0@, lane_accumulator((*prefix), 0))
    && adler_congruent(b_vec@.1@, lane_accumulator((*prefix), 1))
    && adler_congruent(b_vec@.2@, lane_accumulator((*prefix), 2))
    && adler_congruent(b_vec@.3@, lane_accumulator((*prefix), 3)))]
#[ensures(result.0@ == a@ && result.1@ < 65521
    && result.1@ + 22208 * result.0@ <= u32::MAX@
    && result.2.invariant() && result.3.invariant()
    && result.2@.0@ < 65521 && result.2@.1@ < 65521
    && result.2@.2@ < 65521 && result.2@.3@ < 65521
    && result.3@.0@ < 65521 && result.3@.1@ < 65521
    && result.3@.2@ < 65521 && result.3@.3@ < 65521)]
#[ensures(adler_congruent(result.1@, b@ + chunk@.len() * a@)
    && adler_congruent(result.2@.0@, a_vec@.0@ + lane_sum(chunk@, 0))
    && adler_congruent(result.2@.1@, a_vec@.1@ + lane_sum(chunk@, 1))
    && adler_congruent(result.2@.2@, a_vec@.2@ + lane_sum(chunk@, 2))
    && adler_congruent(result.2@.3@, a_vec@.3@ + lane_sum(chunk@, 3)))]
#[ensures(adler_congruent(result.3@.0@, b_vec@.0@
        + chunk@.len() / 4 * a_vec@.0@ + lane_accumulator(chunk@, 0))
    && adler_congruent(result.3@.1@, b_vec@.1@
        + chunk@.len() / 4 * a_vec@.1@ + lane_accumulator(chunk@, 1))
    && adler_congruent(result.3@.2@, b_vec@.2@
        + chunk@.len() / 4 * a_vec@.2@ + lane_accumulator(chunk@, 2))
    && adler_congruent(result.3@.3@, b_vec@.3@
        + chunk@.len() / 4 * a_vec@.3@ + lane_accumulator(chunk@, 3)))]
#[ensures(adler_congruent(result.2@.0@, lane_sum((*prefix).concat(chunk@), 0))
    && adler_congruent(result.2@.1@, lane_sum((*prefix).concat(chunk@), 1))
    && adler_congruent(result.2@.2@, lane_sum((*prefix).concat(chunk@), 2))
    && adler_congruent(result.2@.3@, lane_sum((*prefix).concat(chunk@), 3)))]
#[ensures(adler_congruent(result.3@.0@, lane_accumulator((*prefix).concat(chunk@), 0))
    && adler_congruent(result.3@.1@, lane_accumulator((*prefix).concat(chunk@), 1))
    && adler_congruent(result.3@.2@, lane_accumulator((*prefix).concat(chunk@), 2))
    && adler_congruent(result.3@.3@, lane_accumulator((*prefix).concat(chunk@), 3)))]
fn process_full_chunk(
    a: u32,
    mut b: u32,
    mut a_vec: U32X4,
    mut b_vec: U32X4,
    chunk: &[u8],
    prefix: Snapshot<Seq<u8>>,
) -> (u32, u32, U32X4, U32X4) {
    const MOD: u32 = 65521;
    let a_vec_entry = snapshot! { a_vec };
    let b_vec_entry = snapshot! { b_vec };
    let b_entry = snapshot! { b };
    (a_vec, b_vec) = process_chunk_values(a_vec, b_vec, chunk);
    proof_assert!(chunk@.len() % 4 == 0 && chunk@.len() <= 22208);

    let chunk_size_u32 = 22208u32;
    b += chunk_size_u32 * a;
    let a_vec_before_reduce = snapshot! { a_vec };
    let b_vec_before_reduce = snapshot! { b_vec };
    let b_before_reduce = snapshot! { b };
    a_vec %= MOD;
    b_vec %= MOD;
    b %= MOD;

    proof_assert! {
        adler_remainder_congruent(a_vec_before_reduce@.0@);
        adler_remainder_congruent(a_vec_before_reduce@.1@);
        adler_remainder_congruent(a_vec_before_reduce@.2@);
        adler_remainder_congruent(a_vec_before_reduce@.3@);
        adler_remainder_congruent(b_vec_before_reduce@.0@);
        adler_remainder_congruent(b_vec_before_reduce@.1@);
        adler_remainder_congruent(b_vec_before_reduce@.2@);
        adler_remainder_congruent(b_vec_before_reduce@.3@);
        adler_remainder_congruent(b_before_reduce@);
        true
    };
    proof_assert!(adler_congruent(b@, b_entry@ + chunk@.len() * a@));
    proof_assert!(adler_congruent(a_vec@.0@,
        a_vec_entry@.0@ + lane_sum(chunk@, 0)));
    proof_assert!(adler_congruent(a_vec@.1@,
        a_vec_entry@.1@ + lane_sum(chunk@, 1)));
    proof_assert!(adler_congruent(a_vec@.2@,
        a_vec_entry@.2@ + lane_sum(chunk@, 2)));
    proof_assert!(adler_congruent(a_vec@.3@,
        a_vec_entry@.3@ + lane_sum(chunk@, 3)));
    proof_assert!(adler_congruent(b_vec@.0@, b_vec_entry@.0@
        + chunk@.len() / 4 * a_vec_entry@.0@ + lane_accumulator(chunk@, 0)));
    proof_assert!(adler_congruent(b_vec@.1@, b_vec_entry@.1@
        + chunk@.len() / 4 * a_vec_entry@.1@ + lane_accumulator(chunk@, 1)));
    proof_assert!(adler_congruent(b_vec@.2@, b_vec_entry@.2@
        + chunk@.len() / 4 * a_vec_entry@.2@ + lane_accumulator(chunk@, 2)));
    proof_assert!(adler_congruent(b_vec@.3@, b_vec_entry@.3@
        + chunk@.len() / 4 * a_vec_entry@.3@ + lane_accumulator(chunk@, 3)));
    proof_assert! {
        lane_sum_reduced_certificate(*prefix, chunk@, 0, a_vec_entry@.0@, a_vec@.0@);
        lane_sum_reduced_certificate(*prefix, chunk@, 1, a_vec_entry@.1@, a_vec@.1@);
        lane_sum_reduced_certificate(*prefix, chunk@, 2, a_vec_entry@.2@, a_vec@.2@);
        lane_sum_reduced_certificate(*prefix, chunk@, 3, a_vec_entry@.3@, a_vec@.3@);
        lane_accumulator_reduced_certificate(
            *prefix, chunk@, 0, chunk@.len() / 4,
            a_vec_entry@.0@, b_vec_entry@.0@, b_vec@.0@,
        );
        lane_accumulator_reduced_certificate(
            *prefix, chunk@, 1, chunk@.len() / 4,
            a_vec_entry@.1@, b_vec_entry@.1@, b_vec@.1@,
        );
        lane_accumulator_reduced_certificate(
            *prefix, chunk@, 2, chunk@.len() / 4,
            a_vec_entry@.2@, b_vec_entry@.2@, b_vec@.2@,
        );
        lane_accumulator_reduced_certificate(
            *prefix, chunk@, 3, chunk@.len() / 4,
            a_vec_entry@.3@, b_vec_entry@.3@, b_vec@.3@,
        );
        true
    };
    proof_assert! { reduced_state_facts(a@, b_before_reduce@); true };
    (a, b, a_vec, b_vec)
}

#[inline]
#[requires(adler_congruent(a@, *target_a)
    && adler_congruent(b@, *target_b))]
#[ensures(result.0@ == (*target_a) % 65521
    && result.1@ == (*target_b) % 65521)]
#[ensures(result.0@ < 65521 && result.1@ < 65521)]
fn reduce_final_state(
    a: u32,
    b: u32,
    target_a: Snapshot<Int>,
    target_b: Snapshot<Int>,
) -> (u16, u16) {
    const MOD: u32 = 65521;
    let reduced_a = (a % MOD) as u16;
    let reduced_b = (b % MOD) as u16;
    proof_assert!(reduced_a@ == a@ % 65521);
    proof_assert!(reduced_b@ == b@ % 65521);
    proof_assert! {
        adler_finish_from_remainder_certificate(reduced_a@, a@, *target_a)
            && reduced_a@ == (*target_a) % 65521
    };
    proof_assert! {
        adler_finish_from_remainder_certificate(reduced_b@, b@, *target_b)
            && reduced_b@ == (*target_b) % 65521
    };
    proof_assert!(reduced_a@ == (*target_a) % 65521);
    proof_assert!(reduced_b@ == (*target_b) % 65521);
    (reduced_a, reduced_b)
}

impl Adler32 {
    #[allow(unused_variables)]
    #[ensures((^self).a@ < 65521)]
    #[ensures((^self).b@ < 65521)]
    #[ensures((^self).deep_model() == crate::adler32_update((*self).deep_model(), bytes@))]
    pub(crate) fn compute(&mut self, bytes: &[u8]) {
        // The basic algorithm is, for every byte:
        //   a = (a + byte) % MOD
        //   b = (b + a) % MOD
        // where MOD = 65521.
        //
        // For efficiency, we can defer the `% MOD` operations as long as neither a nor b overflows:
        // - Between calls to `write`, we ensure that a and b are always in range 0..MOD.
        // - We use 32-bit arithmetic in this function.
        // - Therefore, a and b must not increase by more than 2^32-MOD without performing a `% MOD`
        //   operation.
        //
        // According to Wikipedia, b is calculated as follows for non-incremental checksumming:
        //   b = n×D1 + (n−1)×D2 + (n−2)×D3 + ... + Dn + n*1 (mod 65521)
        // Where n is the number of bytes and Di is the i-th Byte. We need to change this to account
        // for the previous values of a and b, as well as treat every input Byte as being 255:
        //   b_inc = n×255 + (n-1)×255 + ... + 255 + n*65520
        // Or in other words:
        //   b_inc = n*65520 + n(n+1)/2*255
        // The max chunk size is thus the largest value of n so that b_inc <= 2^32-65521.
        //   2^32-65521 = n*65520 + n(n+1)/2*255
        // Plugging this into an equation solver since I can't math gives n = 5552.18..., so 5552.
        //
        // On top of the optimization outlined above, the algorithm can also be parallelized with a
        // bit more work:
        //
        // Note that b is a linear combination of a vector of input bytes (D1, ..., Dn).
        //
        // If we fix some value k<N and rewrite indices 1, ..., N as
        //
        //   1_1, 1_2, ..., 1_k, 2_1, ..., 2_k, ..., (N/k)_k,
        //
        // then we can express a and b in terms of sums of smaller sequences kb and ka:
        //
        //   ka(j) := D1_j + D2_j + ... + D(N/k)_j where j <= k
        //   kb(j) := (N/k)*D1_j + (N/k-1)*D2_j + ... + D(N/k)_j where j <= k
        //
        //  a = ka(1) + ka(2) + ... + ka(k) + 1
        //  b = k*(kb(1) + kb(2) + ... + kb(k)) - 1*ka(2) - ...  - (k-1)*ka(k) + N
        //
        // We use this insight to unroll the main loop and process k=4 bytes at a time.
        // The resulting code is highly amenable to SIMD acceleration, although the immediate speedups
        // stem from increased pipeline parallelism rather than auto-vectorization.
        //
        // This technique is described in-depth (here:)[https://software.intel.com/content/www/us/\
        // en/develop/articles/fast-computation-of-fletcher-checksums.html]

        const MOD: u32 = 65521;
        const CHUNK_SIZE: usize = 5552 * 4;

        let mut a = u32::from(self.a);
        let mut b = u32::from(self.b);
        let initial_a = snapshot! { a };
        let initial_b = snapshot! { b };
        let mut a_vec = U32X4([0; 4]);
        let mut b_vec = a_vec;

        let original_bytes = snapshot! { bytes@ };
        let (bytes, remainder) = bytes.split_at(bytes.len() - bytes.len() % 4);
        proof_assert! {
            truncate_to_four_alignment((*original_bytes).len());
            bytes@.len() % 4 == 0
        };
        proof_assert! {
            subsequence_split(
                *original_bytes,
                bytes@.len(),
                (*original_bytes).len(),
            );
            *original_bytes == bytes@.concat(remainder@)
        };

        // iterate over 4 bytes at a time
        let chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
        let remainder_chunk = chunk_iter.remainder();
        let initial_chunks = snapshot! { chunk_iter@.0 };
        proof_assert!(remainder_chunk@ == bytes@.subsequence(
            bytes@.len() - remainder_chunk@.len(),
            bytes@.len(),
        ));
        let chunks_match_bytes = snapshot! {
            chunks_match_source(bytes@, *initial_chunks, CHUNK_SIZE@)
        };
        proof_assert!(*chunks_match_bytes);
        proof_assert! {
            initial_state_safe(a@, b@);
            b@ + 22208 * a@ <= u32::MAX@
        };
        proof_assert!(a@ <= u16::MAX@);
        proof_assert!(b@ <= u16::MAX@);
        proof_assert!(a_vec@.0@ < 65521);
        proof_assert!(a_vec@.1@ < 65521);
        proof_assert!(a_vec@.2@ < 65521);
        proof_assert!(a_vec@.3@ < 65521);
        proof_assert!(b_vec@.0@ < 65521);
        proof_assert!(b_vec@.1@ < 65521);
        proof_assert!(b_vec@.2@ < 65521);
        proof_assert!(b_vec@.3@ < 65521);
        proof_assert!(a_vec.invariant());
        proof_assert!(b_vec.invariant());
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(a_vec@.0@, lane_sum(bytes@.subsequence(0, 0), 0)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(a_vec@.1@, lane_sum(bytes@.subsequence(0, 0), 1)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(a_vec@.2@, lane_sum(bytes@.subsequence(0, 0), 2)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(a_vec@.3@, lane_sum(bytes@.subsequence(0, 0), 3)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(b_vec@.0@, lane_accumulator(bytes@.subsequence(0, 0), 0)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(b_vec@.1@, lane_accumulator(bytes@.subsequence(0, 0), 1)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(b_vec@.2@, lane_accumulator(bytes@.subsequence(0, 0), 2)) };
        proof_assert! { empty_lane_facts(bytes@); adler_congruent_refl(0); adler_congruent(b_vec@.3@, lane_accumulator(bytes@.subsequence(0, 0), 3)) };
        proof_assert!(CHUNK_SIZE@ % 4 == 0 && CHUNK_SIZE@ <= 22208);
        let mut processed_chunks = 0usize;
        let full_chunk_count = bytes.len() / CHUNK_SIZE;
        proof_assert!(full_chunk_count@ == (*initial_chunks).len());
        #[invariant(processed_chunks@ <= full_chunk_count@)]
        #[invariant(remainder_chunk@ == bytes@.subsequence(
            bytes@.len() - remainder_chunk@.len(),
            bytes@.len(),
        ))]
        #[invariant(a@ == initial_a@)]
        #[invariant(a_vec.invariant() && b_vec.invariant())]
        #[invariant(adler_congruent(b@, initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@))]
        #[invariant(adler_congruent(a_vec@.0@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 0)) && adler_congruent(a_vec@.1@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 1)) && adler_congruent(a_vec@.2@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 2)) && adler_congruent(a_vec@.3@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 3)))]
        #[invariant(adler_congruent(b_vec@.0@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 0)) && adler_congruent(b_vec@.1@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 1)) && adler_congruent(b_vec@.2@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 2)) && adler_congruent(b_vec@.3@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 3)))]
        #[invariant(a@ <= u16::MAX@ && b@ <= u16::MAX@ && b@ + 22208 * a@ <= u32::MAX@ && a_vec@.0@ < 65521 && a_vec@.1@ < 65521 && a_vec@.2@ < 65521 && a_vec@.3@ < 65521 && b_vec@.0@ < 65521 && b_vec@.1@ < 65521 && b_vec@.2@ < 65521 && b_vec@.3@ < 65521)]
        while processed_chunks < full_chunk_count {
            let processed_before = snapshot! { processed_chunks@ };
            let start = processed_chunks * CHUNK_SIZE;
            proof_assert!(start@ + CHUNK_SIZE@ <= bytes@.len());
            let end = start + CHUNK_SIZE;
            let (complete_prefix, _) = bytes.split_at(end);
            let (prefix_slice, chunk) = complete_prefix.split_at(start);
            let processed_prefix = snapshot! {
                bytes@.subsequence(0, *processed_before * CHUNK_SIZE@)
            };
            proof_assert! {
                prefix_of_prefix_certificate(
                    bytes@,
                    *processed_before * CHUNK_SIZE@,
                    (*processed_before + 1) * CHUNK_SIZE@,
                );
                prefix_slice@ == *processed_prefix
            };
            proof_assert!((*processed_prefix).len() % 4 == 0);
            proof_assert! {
                subsequence_split(
                    complete_prefix@,
                    *processed_before * CHUNK_SIZE@,
                    (*processed_before + 1) * CHUNK_SIZE@,
                );
                complete_prefix@ == (*processed_prefix).concat(chunk@)
            };
            let compact_a_entry = snapshot! { a };
            let compact_b_entry = snapshot! { b };
            proof_assert!(adler_congruent(
                compact_b_entry@,
                initial_b@
                    + *processed_before * CHUNK_SIZE@ * initial_a@,
            ));
            (a, b, a_vec, b_vec) = process_full_chunk(
                a, b, a_vec, b_vec, chunk, processed_prefix,
            );
            proof_assert!(a@ == initial_a@);
            proof_assert!(adler_congruent(
                b@,
                compact_b_entry@ + chunk@.len() * compact_a_entry@,
            ));
            proof_assert!(compact_a_entry@ == initial_a@);
            proof_assert!(chunk@.len() == CHUNK_SIZE@);
            proof_assert!(compact_a_entry@ == a@);
            let native_delta = snapshot! {
                chunk@.len() * compact_a_entry@
            };
            proof_assert! {
                scalar_delta_reindex_certificate(
                    chunk@.len(), CHUNK_SIZE@, compact_a_entry@, a@,
                );
                *native_delta == CHUNK_SIZE@ * a@
            };
            let old_scalar_target = snapshot! {
                initial_b@ + *processed_before * CHUNK_SIZE@ * initial_a@
            };
            let next_scalar_target = snapshot! {
                initial_b@ + (*processed_before + 1) * CHUNK_SIZE@ * initial_a@
            };
            proof_assert!(*old_scalar_target + *native_delta
                == *next_scalar_target);
            proof_assert! {
                adler_congruent_refl(*native_delta);
                adler_congruent_add(
                    compact_b_entry@,
                    *old_scalar_target,
                    *native_delta,
                    *native_delta,
                );
                adler_congruent(
                    compact_b_entry@ + *native_delta,
                    *old_scalar_target + *native_delta,
                )
            };
            proof_assert! {
                adler_congruent_trans(
                    b@,
                    compact_b_entry@ + *native_delta,
                    *old_scalar_target + *native_delta,
                );
                adler_congruent(
                    b@,
                    *old_scalar_target + *native_delta,
                )
            };
            proof_assert! {
                adler_congruent_reindex(
                    b@,
                    *old_scalar_target + *native_delta,
                    *next_scalar_target,
                );
                adler_congruent(b@, *next_scalar_target)
            };
            processed_chunks += 1;
            proof_assert!(processed_chunks@ == *processed_before + 1);
            proof_assert!(*next_scalar_target
                == initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@);
            proof_assert! {
                adler_congruent_reindex(
                    b@,
                    *next_scalar_target,
                    initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@,
                );
                adler_congruent(
                    b@,
                    initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@,
                )
            };
        }
        #[cfg(any())]
        {
        #[invariant(*original_bytes == bytes@.concat(remainder@))]
        #[invariant(remainder_chunk@ == bytes@.subsequence(
            bytes@.len() - remainder_chunk@.len(),
            bytes@.len(),
        ))]
        #[invariant(a@ == initial_a@)]
        #[invariant(a_vec.invariant())]
        #[invariant(b_vec.invariant())]
        #[invariant(*chunks_match_bytes)]
        #[invariant(processed_chunks@ == produced.len())]
        #[invariant(adler_congruent(b@, initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@))]
        #[invariant(adler_congruent(a_vec@.0@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 0)) && adler_congruent(a_vec@.1@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 1)) && adler_congruent(a_vec@.2@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 2)) && adler_congruent(a_vec@.3@, lane_sum(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 3)))]
        #[invariant(adler_congruent(b_vec@.0@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 0)) && adler_congruent(b_vec@.1@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 1)) && adler_congruent(b_vec@.2@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 2)) && adler_congruent(b_vec@.3@, lane_accumulator(bytes@.subsequence(0, processed_chunks@ * CHUNK_SIZE@), 3)))]
        #[invariant(a@ <= u16::MAX@ && b@ <= u16::MAX@ && b@ + 22208 * a@ <= u32::MAX@ && a_vec@.0@ < 65521 && a_vec@.1@ < 65521 && a_vec@.2@ < 65521 && a_vec@.3@ < 65521 && b_vec@.0@ < 65521 && b_vec@.1@ < 65521 && b_vec@.2@ < 65521 && b_vec@.3@ < 65521)]
        for chunk in chunk_iter {
            proof_assert!(produced.len() > 0);
            let processed_before = snapshot! { processed_chunks@ };
            let visited_index = snapshot! { produced.len() - 1 };
            proof_assert!(*processed_before + 1 == produced.len());
            proof_assert!(*processed_before == *visited_index);
            proof_assert!(chunk@.len() == CHUNK_SIZE@);
            proof_assert!(chunk@.len() % 4 == 0 && chunk@.len() <= 22208);
            proof_assert!(*visited_index < produced.len());
            proof_assert!(chunk@ == produced[*visited_index]@);
            proof_assert!(produced[*visited_index]@
                == (*initial_chunks)[*visited_index]);
            proof_assert!(chunk@ == (*initial_chunks)[*processed_before]);
            proof_assert!(produced.len() - 1 < (*initial_chunks).len());
            proof_assert! {
                visited_index_bound_certificate(
                    *processed_before,
                    produced.len(),
                    (*initial_chunks).len(),
                );
                *processed_before < (*initial_chunks).len()
            };
            proof_assert! {
                matched_chunk_at_certificate(
                    bytes@, *initial_chunks, CHUNK_SIZE@, *processed_before,
                );
                (*initial_chunks)[*processed_before]
                    == bytes@.subsequence(
                        *processed_before * CHUNK_SIZE@,
                        (*processed_before + 1) * CHUNK_SIZE@,
                    )
            };
            proof_assert!(chunk@ == bytes@.subsequence(
                *processed_before * CHUNK_SIZE@,
                (*processed_before + 1) * CHUNK_SIZE@,
            ));
            let processed_prefix = snapshot! {
                bytes@.subsequence(0, *processed_before * CHUNK_SIZE@)
            };
            proof_assert!((*processed_prefix).len() == *processed_before * CHUNK_SIZE@);
            proof_assert! {
                let witness = multiple_of_four_product(*processed_before, CHUNK_SIZE@);
                witness == (*processed_prefix).len()
            };
            proof_assert! {
                let witness = multiple_of_four_product(*processed_before, CHUNK_SIZE@);
                witness % 4 == 0
            };
            proof_assert!((*processed_prefix).len() % 4 == 0);
            proof_assert! {
                subsequence_split(
                    bytes@,
                    *processed_before * CHUNK_SIZE@,
                    (*processed_before + 1) * CHUNK_SIZE@,
                );
                bytes@.subsequence(0, (*processed_before + 1) * CHUNK_SIZE@)
                    == (*processed_prefix).concat(chunk@)
            };
            let compact_a_vec_entry = snapshot! { a_vec };
            let compact_b_vec_entry = snapshot! { b_vec };
            let compact_b_entry = snapshot! { b };
            proof_assert!(adler_congruent(compact_a_vec_entry@.0@,
                lane_sum(*processed_prefix, 0)));
            proof_assert!(adler_congruent(compact_a_vec_entry@.1@,
                lane_sum(*processed_prefix, 1)));
            proof_assert!(adler_congruent(compact_a_vec_entry@.2@,
                lane_sum(*processed_prefix, 2)));
            proof_assert!(adler_congruent(compact_a_vec_entry@.3@,
                lane_sum(*processed_prefix, 3)));
            proof_assert!(adler_congruent(compact_b_vec_entry@.0@,
                lane_accumulator(*processed_prefix, 0)));
            proof_assert!(adler_congruent(compact_b_vec_entry@.1@,
                lane_accumulator(*processed_prefix, 1)));
            proof_assert!(adler_congruent(compact_b_vec_entry@.2@,
                lane_accumulator(*processed_prefix, 2)));
            proof_assert!(adler_congruent(compact_b_vec_entry@.3@,
                lane_accumulator(*processed_prefix, 3)));
            (a, b, a_vec, b_vec) = process_full_chunk(
                a, b, a_vec, b_vec, chunk, processed_prefix,
            );
            proof_assert!(a@ == initial_a@);
            proof_assert!(adler_congruent(
                compact_b_entry@,
                initial_b@
                    + *processed_before * CHUNK_SIZE@ * initial_a@,
            ));
            proof_assert!(adler_congruent(
                b@,
                compact_b_entry@ + CHUNK_SIZE@ * a@,
            ));
            #[cfg(any())]
            {
            proof_assert! {
                lane_sum_reduced_certificate(
                    *processed_prefix, chunk@, 0,
                    compact_a_vec_entry@.0@, a_vec@.0@,
                );
                adler_congruent(a_vec@.0@,
                    lane_sum((*processed_prefix).concat(chunk@), 0))
            };
            proof_assert! {
                lane_sum_reduced_certificate(
                    *processed_prefix, chunk@, 1,
                    compact_a_vec_entry@.1@, a_vec@.1@,
                );
                adler_congruent(a_vec@.1@,
                    lane_sum((*processed_prefix).concat(chunk@), 1))
            };
            proof_assert! {
                lane_sum_reduced_certificate(
                    *processed_prefix, chunk@, 2,
                    compact_a_vec_entry@.2@, a_vec@.2@,
                );
                adler_congruent(a_vec@.2@,
                    lane_sum((*processed_prefix).concat(chunk@), 2))
            };
            proof_assert! {
                lane_sum_reduced_certificate(
                    *processed_prefix, chunk@, 3,
                    compact_a_vec_entry@.3@, a_vec@.3@,
                );
                adler_congruent(a_vec@.3@,
                    lane_sum((*processed_prefix).concat(chunk@), 3))
            };
            proof_assert! {
                lane_accumulator_reduced_certificate(
                    *processed_prefix, chunk@, 0, chunk@.len() / 4,
                    compact_a_vec_entry@.0@, compact_b_vec_entry@.0@,
                    b_vec@.0@,
                );
                true
            };
            proof_assert! {
                lane_accumulator_reduced_certificate(
                    *processed_prefix, chunk@, 1, chunk@.len() / 4,
                    compact_a_vec_entry@.1@, compact_b_vec_entry@.1@,
                    b_vec@.1@,
                );
                true
            };
            proof_assert! {
                lane_accumulator_reduced_certificate(
                    *processed_prefix, chunk@, 2, chunk@.len() / 4,
                    compact_a_vec_entry@.2@, compact_b_vec_entry@.2@,
                    b_vec@.2@,
                );
                true
            };
            proof_assert! {
                lane_accumulator_reduced_certificate(
                    *processed_prefix, chunk@, 3, chunk@.len() / 4,
                    compact_a_vec_entry@.3@, compact_b_vec_entry@.3@,
                    b_vec@.3@,
                );
                true
            };
            #[cfg(any())]
            {
            proof_assert! {
                let prefix = *processed_prefix;
                lane_sum_congruence_certificate(
                    prefix, chunk@, 0, compact_a_vec_entry@.0@,
                    compact_a_vec_entry@.0@ + lane_sum(chunk@, 0),
                );
                adler_congruent_trans(
                    a_vec@.0@,
                    compact_a_vec_entry@.0@ + lane_sum(chunk@, 0),
                    lane_sum(prefix.concat(chunk@), 0),
                );
                lane_sum_congruence_certificate_1(
                    prefix, chunk@, compact_a_vec_entry@.1@,
                    compact_a_vec_entry@.1@ + lane_sum(chunk@, 1),
                );
                adler_congruent_trans(
                    a_vec@.1@,
                    compact_a_vec_entry@.1@ + lane_sum(chunk@, 1),
                    lane_sum(prefix.concat(chunk@), 1),
                );
                lane_sum_congruence_certificate_2(
                    prefix, chunk@, compact_a_vec_entry@.2@,
                    compact_a_vec_entry@.2@ + lane_sum(chunk@, 2),
                );
                adler_congruent_trans(
                    a_vec@.2@,
                    compact_a_vec_entry@.2@ + lane_sum(chunk@, 2),
                    lane_sum(prefix.concat(chunk@), 2),
                );
                lane_sum_congruence_certificate_3(
                    prefix, chunk@, compact_a_vec_entry@.3@,
                    compact_a_vec_entry@.3@ + lane_sum(chunk@, 3),
                );
                adler_congruent_trans(
                    a_vec@.3@,
                    compact_a_vec_entry@.3@ + lane_sum(chunk@, 3),
                    lane_sum(prefix.concat(chunk@), 3),
                );
                true
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                let exact0 = compact_b_vec_entry@.0@ + groups * compact_a_vec_entry@.0@
                    + lane_accumulator(chunk@, 0);
                let exact1 = compact_b_vec_entry@.1@ + groups * compact_a_vec_entry@.1@
                    + lane_accumulator(chunk@, 1);
                let exact2 = compact_b_vec_entry@.2@ + groups * compact_a_vec_entry@.2@
                    + lane_accumulator(chunk@, 2);
                let exact3 = compact_b_vec_entry@.3@ + groups * compact_a_vec_entry@.3@
                    + lane_accumulator(chunk@, 3);
                lane_accumulator_congruence_certificate_0(
                    prefix, chunk@, groups, compact_a_vec_entry@.0@,
                    compact_b_vec_entry@.0@, exact0,
                );
                lane_accumulator_congruence_certificate_1(
                    prefix, chunk@, groups, compact_a_vec_entry@.1@,
                    compact_b_vec_entry@.1@, exact1,
                );
                lane_accumulator_congruence_certificate_2(
                    prefix, chunk@, groups, compact_a_vec_entry@.2@,
                    compact_b_vec_entry@.2@, exact2,
                );
                lane_accumulator_congruence_certificate_3(
                    prefix, chunk@, groups, compact_a_vec_entry@.3@,
                    compact_b_vec_entry@.3@, exact3,
                );
                adler_congruent_trans(b_vec@.0@, exact0,
                    lane_accumulator(prefix.concat(chunk@), 0));
                adler_congruent_trans(b_vec@.1@, exact1,
                    lane_accumulator(prefix.concat(chunk@), 1));
                adler_congruent_trans(b_vec@.2@, exact2,
                    lane_accumulator(prefix.concat(chunk@), 2));
                adler_congruent_trans(b_vec@.3@, exact3,
                    lane_accumulator(prefix.concat(chunk@), 3));
                true
            };
            }
            }
            proof_assert! {
                let next_target = initial_b@
                    + (*processed_before + 1) * CHUNK_SIZE@ * initial_a@;
                scalar_chunk_reduced_certificate(
                    a@, compact_b_entry@, b@, initial_a@, initial_b@,
                    *processed_before, CHUNK_SIZE@,
                );
                adler_congruent(
                    b@,
                    initial_b@
                        + (*processed_before + 1) * CHUNK_SIZE@ * initial_a@,
                )
            };
            processed_chunks += 1;
            proof_assert!(*processed_before + 1 == processed_chunks@);
            proof_assert!(adler_congruent(
                b@,
                initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@,
            ));
            continue;
            #[cfg(any())]
            {
            let a_vec_entry = snapshot! { a_vec };
            let b_vec_entry = snapshot! { b_vec };
            proof_assert!(adler_congruent(
                a_vec_entry@.0@,
                lane_sum(*processed_prefix, 0),
            ));
            proof_assert!(adler_congruent(
                a_vec_entry@.1@,
                lane_sum(*processed_prefix, 1),
            ));
            proof_assert!(adler_congruent(
                a_vec_entry@.2@,
                lane_sum(*processed_prefix, 2),
            ));
            proof_assert!(adler_congruent(
                a_vec_entry@.3@,
                lane_sum(*processed_prefix, 3),
            ));
            proof_assert!(adler_congruent(
                b_vec_entry@.0@,
                lane_accumulator(*processed_prefix, 0),
            ));
            proof_assert!(adler_congruent(
                b_vec_entry@.1@,
                lane_accumulator(*processed_prefix, 1),
            ));
            proof_assert!(adler_congruent(
                b_vec_entry@.2@,
                lane_accumulator(*processed_prefix, 2),
            ));
            proof_assert!(adler_congruent(
                b_vec_entry@.3@,
                lane_accumulator(*processed_prefix, 3),
            ));
            proof_assert!(a_vec.invariant() && b_vec.invariant());
            (a_vec, b_vec) = process_chunk_values(a_vec, b_vec, chunk);
            proof_assert!(a_vec.invariant() && b_vec.invariant());

            proof_assert!(a_vec@.0@ == a_vec_entry@.0@ + lane_sum(chunk@, 0));
            proof_assert!(a_vec@.1@ == a_vec_entry@.1@ + lane_sum(chunk@, 1));
            proof_assert!(a_vec@.2@ == a_vec_entry@.2@ + lane_sum(chunk@, 2));
            proof_assert!(a_vec@.3@ == a_vec_entry@.3@ + lane_sum(chunk@, 3));
            proof_assert!(chunk@.len() / 4 == 5552);
            proof_assert!(b_vec@.0@ == b_vec_entry@.0@
                + (chunk@.len() / 4) * a_vec_entry@.0@
                + lane_accumulator(chunk@, 0));
            proof_assert!(b_vec@.1@ == b_vec_entry@.1@
                + (chunk@.len() / 4) * a_vec_entry@.1@
                + lane_accumulator(chunk@, 1));
            proof_assert!(b_vec@.2@ == b_vec_entry@.2@
                + (chunk@.len() / 4) * a_vec_entry@.2@
                + lane_accumulator(chunk@, 2));
            proof_assert!(b_vec@.3@ == b_vec_entry@.3@
                + (chunk@.len() / 4) * a_vec_entry@.3@
                + lane_accumulator(chunk@, 3));
            proof_assert! {
                let prefix = *processed_prefix;
                lane_sum_concat(prefix, chunk@, 0);
                lane_sum(prefix.concat(chunk@), 0)
                    == lane_sum(prefix, 0) + lane_sum(chunk@, 0)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add_same_certificate(
                    a_vec_entry@.0@,
                    lane_sum(prefix, 0),
                    lane_sum(chunk@, 0),
                ) && adler_congruent(
                    a_vec_entry@.0@ + lane_sum(chunk@, 0),
                    lane_sum(prefix, 0) + lane_sum(chunk@, 0),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(a_vec@.0@, lane_sum(prefix.concat(chunk@), 0))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_sum_concat(prefix, chunk@, 1);
                lane_sum(prefix.concat(chunk@), 1)
                    == lane_sum(prefix, 1) + lane_sum(chunk@, 1)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add_same_certificate(
                    a_vec_entry@.1@,
                    lane_sum(prefix, 1),
                    lane_sum(chunk@, 1),
                ) && adler_congruent(
                    a_vec_entry@.1@ + lane_sum(chunk@, 1),
                    lane_sum(prefix, 1) + lane_sum(chunk@, 1),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(a_vec@.1@, lane_sum(prefix.concat(chunk@), 1))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_sum_concat(prefix, chunk@, 2);
                lane_sum(prefix.concat(chunk@), 2)
                    == lane_sum(prefix, 2) + lane_sum(chunk@, 2)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add_same_certificate(
                    a_vec_entry@.2@,
                    lane_sum(prefix, 2),
                    lane_sum(chunk@, 2),
                ) && adler_congruent(
                    a_vec_entry@.2@ + lane_sum(chunk@, 2),
                    lane_sum(prefix, 2) + lane_sum(chunk@, 2),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(a_vec@.2@, lane_sum(prefix.concat(chunk@), 2))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_sum_concat(prefix, chunk@, 3);
                lane_sum(prefix.concat(chunk@), 3)
                    == lane_sum(prefix, 3) + lane_sum(chunk@, 3)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add_same_certificate(
                    a_vec_entry@.3@,
                    lane_sum(prefix, 3),
                    lane_sum(chunk@, 3),
                ) && adler_congruent(
                    a_vec_entry@.3@ + lane_sum(chunk@, 3),
                    lane_sum(prefix, 3) + lane_sum(chunk@, 3),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(a_vec@.3@, lane_sum(prefix.concat(chunk@), 3))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_ready_0(
                    prefix, chunk@, groups, a_vec_entry@.0@,
                    b_vec_entry@.0@, b_vec@.0@,
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_certificate_0(
                    prefix, chunk@, groups, a_vec_entry@.0@,
                    b_vec_entry@.0@, b_vec@.0@,
                ) && adler_congruent(
                    b_vec@.0@, lane_accumulator(prefix.concat(chunk@), 0),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_ready_1(
                    prefix, chunk@, groups, a_vec_entry@.1@,
                    b_vec_entry@.1@, b_vec@.1@,
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_certificate_1(
                    prefix, chunk@, groups, a_vec_entry@.1@,
                    b_vec_entry@.1@, b_vec@.1@,
                ) && adler_congruent(
                    b_vec@.1@, lane_accumulator(prefix.concat(chunk@), 1),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_ready_2(
                    prefix, chunk@, groups, a_vec_entry@.2@,
                    b_vec_entry@.2@, b_vec@.2@,
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_certificate_2(
                    prefix, chunk@, groups, a_vec_entry@.2@,
                    b_vec_entry@.2@, b_vec@.2@,
                ) && adler_congruent(
                    b_vec@.2@, lane_accumulator(prefix.concat(chunk@), 2),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_ready_3(
                    prefix, chunk@, groups, a_vec_entry@.3@,
                    b_vec_entry@.3@, b_vec@.3@,
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let groups = chunk@.len() / 4;
                lane_accumulator_congruence_certificate_3(
                    prefix, chunk@, groups, a_vec_entry@.3@,
                    b_vec_entry@.3@, b_vec@.3@,
                ) && adler_congruent(
                    b_vec@.3@, lane_accumulator(prefix.concat(chunk@), 3),
                )
            };
            /*
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 0)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 0)
                    == lane_accumulator_concat_facts(
                        lane_accumulator(prefix.concat(chunk@), 0),
                        lane_accumulator(prefix, 0),
                        chunk@.len() / 4,
                        lane_sum(prefix, 0),
                        lane_accumulator(chunk@, 0),
                    )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_facts(
                    lane_accumulator(prefix.concat(chunk@), 0),
                    lane_accumulator(prefix, 0),
                    chunk@.len() / 4,
                    lane_sum(prefix, 0),
                    lane_accumulator(chunk@, 0),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator(prefix.concat(chunk@), 0)
                    == lane_accumulator(prefix, 0)
                        + (chunk@.len() / 4) * lane_sum(prefix, 0)
                        + lane_accumulator(chunk@, 0)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_scale(a_vec_entry@.0@, lane_sum(prefix, 0), chunk@.len() / 4);
                adler_congruent((chunk@.len() / 4) * a_vec_entry@.0@, (chunk@.len() / 4) * lane_sum(prefix, 0))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add(
                    b_vec_entry@.0@,
                    lane_accumulator(prefix, 0),
                    (chunk@.len() / 4) * a_vec_entry@.0@,
                    (chunk@.len() / 4) * lane_sum(prefix, 0),
                );
                adler_congruent(
                    b_vec_entry@.0@ + (chunk@.len() / 4) * a_vec_entry@.0@,
                    lane_accumulator(prefix, 0) + (chunk@.len() / 4) * lane_sum(prefix, 0),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_refl(lane_accumulator(chunk@, 0));
                adler_congruent_add(
                    b_vec_entry@.0@ + (chunk@.len() / 4) * a_vec_entry@.0@,
                    lane_accumulator(prefix, 0) + (chunk@.len() / 4) * lane_sum(prefix, 0),
                    lane_accumulator(chunk@, 0),
                    lane_accumulator(chunk@, 0),
                );
                adler_congruent(
                    b_vec_entry@.0@ + (chunk@.len() / 4) * a_vec_entry@.0@
                        + lane_accumulator(chunk@, 0),
                    lane_accumulator(prefix, 0) + (chunk@.len() / 4) * lane_sum(prefix, 0)
                        + lane_accumulator(chunk@, 0),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(b_vec@.0@, lane_accumulator(prefix.concat(chunk@), 0))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 1)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 1)
                    == lane_accumulator_concat_facts(
                        lane_accumulator(prefix.concat(chunk@), 1),
                        lane_accumulator(prefix, 1), chunk@.len() / 4,
                        lane_sum(prefix, 1), lane_accumulator(chunk@, 1),
                    )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_facts(
                    lane_accumulator(prefix.concat(chunk@), 1),
                    lane_accumulator(prefix, 1), chunk@.len() / 4,
                    lane_sum(prefix, 1), lane_accumulator(chunk@, 1),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator(prefix.concat(chunk@), 1)
                    == lane_accumulator(prefix, 1) + (chunk@.len() / 4) * lane_sum(prefix, 1)
                        + lane_accumulator(chunk@, 1)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_scale(a_vec_entry@.1@, lane_sum(prefix, 1), chunk@.len() / 4);
                adler_congruent((chunk@.len() / 4) * a_vec_entry@.1@, (chunk@.len() / 4) * lane_sum(prefix, 1))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add(b_vec_entry@.1@, lane_accumulator(prefix, 1),
                    (chunk@.len() / 4) * a_vec_entry@.1@, (chunk@.len() / 4) * lane_sum(prefix, 1));
                adler_congruent(
                    b_vec_entry@.1@ + (chunk@.len() / 4) * a_vec_entry@.1@,
                    lane_accumulator(prefix, 1) + (chunk@.len() / 4) * lane_sum(prefix, 1),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_refl(lane_accumulator(chunk@, 1));
                adler_congruent_add(
                    b_vec_entry@.1@ + (chunk@.len() / 4) * a_vec_entry@.1@,
                    lane_accumulator(prefix, 1) + (chunk@.len() / 4) * lane_sum(prefix, 1),
                    lane_accumulator(chunk@, 1), lane_accumulator(chunk@, 1),
                );
                adler_congruent(
                    b_vec_entry@.1@ + (chunk@.len() / 4) * a_vec_entry@.1@
                        + lane_accumulator(chunk@, 1),
                    lane_accumulator(prefix, 1) + (chunk@.len() / 4) * lane_sum(prefix, 1)
                        + lane_accumulator(chunk@, 1),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(b_vec@.1@, lane_accumulator(prefix.concat(chunk@), 1))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 2)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 2)
                    == lane_accumulator_concat_facts(
                        lane_accumulator(prefix.concat(chunk@), 2),
                        lane_accumulator(prefix, 2), chunk@.len() / 4,
                        lane_sum(prefix, 2), lane_accumulator(chunk@, 2),
                    )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_facts(
                    lane_accumulator(prefix.concat(chunk@), 2),
                    lane_accumulator(prefix, 2), chunk@.len() / 4,
                    lane_sum(prefix, 2), lane_accumulator(chunk@, 2),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator(prefix.concat(chunk@), 2)
                    == lane_accumulator(prefix, 2) + (chunk@.len() / 4) * lane_sum(prefix, 2)
                        + lane_accumulator(chunk@, 2)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_scale(a_vec_entry@.2@, lane_sum(prefix, 2), chunk@.len() / 4);
                adler_congruent((chunk@.len() / 4) * a_vec_entry@.2@, (chunk@.len() / 4) * lane_sum(prefix, 2))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add(b_vec_entry@.2@, lane_accumulator(prefix, 2),
                    (chunk@.len() / 4) * a_vec_entry@.2@, (chunk@.len() / 4) * lane_sum(prefix, 2));
                adler_congruent(
                    b_vec_entry@.2@ + (chunk@.len() / 4) * a_vec_entry@.2@,
                    lane_accumulator(prefix, 2) + (chunk@.len() / 4) * lane_sum(prefix, 2),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_refl(lane_accumulator(chunk@, 2));
                adler_congruent_add(
                    b_vec_entry@.2@ + (chunk@.len() / 4) * a_vec_entry@.2@,
                    lane_accumulator(prefix, 2) + (chunk@.len() / 4) * lane_sum(prefix, 2),
                    lane_accumulator(chunk@, 2), lane_accumulator(chunk@, 2),
                );
                adler_congruent(
                    b_vec_entry@.2@ + (chunk@.len() / 4) * a_vec_entry@.2@
                        + lane_accumulator(chunk@, 2),
                    lane_accumulator(prefix, 2) + (chunk@.len() / 4) * lane_sum(prefix, 2)
                        + lane_accumulator(chunk@, 2),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(b_vec@.2@, lane_accumulator(prefix.concat(chunk@), 2))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 3)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_certificate(prefix, chunk@, 3)
                    == lane_accumulator_concat_facts(
                        lane_accumulator(prefix.concat(chunk@), 3),
                        lane_accumulator(prefix, 3), chunk@.len() / 4,
                        lane_sum(prefix, 3), lane_accumulator(chunk@, 3),
                    )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator_concat_facts(
                    lane_accumulator(prefix.concat(chunk@), 3),
                    lane_accumulator(prefix, 3), chunk@.len() / 4,
                    lane_sum(prefix, 3), lane_accumulator(chunk@, 3),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                lane_accumulator(prefix.concat(chunk@), 3)
                    == lane_accumulator(prefix, 3) + (chunk@.len() / 4) * lane_sum(prefix, 3)
                        + lane_accumulator(chunk@, 3)
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_scale(a_vec_entry@.3@, lane_sum(prefix, 3), chunk@.len() / 4);
                adler_congruent((chunk@.len() / 4) * a_vec_entry@.3@, (chunk@.len() / 4) * lane_sum(prefix, 3))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_add(b_vec_entry@.3@, lane_accumulator(prefix, 3),
                    (chunk@.len() / 4) * a_vec_entry@.3@, (chunk@.len() / 4) * lane_sum(prefix, 3));
                adler_congruent(
                    b_vec_entry@.3@ + (chunk@.len() / 4) * a_vec_entry@.3@,
                    lane_accumulator(prefix, 3) + (chunk@.len() / 4) * lane_sum(prefix, 3),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent_refl(lane_accumulator(chunk@, 3));
                adler_congruent_add(
                    b_vec_entry@.3@ + (chunk@.len() / 4) * a_vec_entry@.3@,
                    lane_accumulator(prefix, 3) + (chunk@.len() / 4) * lane_sum(prefix, 3),
                    lane_accumulator(chunk@, 3), lane_accumulator(chunk@, 3),
                );
                adler_congruent(
                    b_vec_entry@.3@ + (chunk@.len() / 4) * a_vec_entry@.3@
                        + lane_accumulator(chunk@, 3),
                    lane_accumulator(prefix, 3) + (chunk@.len() / 4) * lane_sum(prefix, 3)
                        + lane_accumulator(chunk@, 3),
                )
            };
            proof_assert! {
                let prefix = *processed_prefix;
                adler_congruent(b_vec@.3@, lane_accumulator(prefix.concat(chunk@), 3))
            };
            */

            proof_assert! {
                let old_target = initial_b@
                    + processed_chunks@ * CHUNK_SIZE@ * initial_a@;
                let new_target = initial_b@
                    + processed_chunks@ * CHUNK_SIZE@ * initial_a@;
                old_target == new_target
            };
            proof_assert! {
                let old_target = initial_b@
                    + processed_chunks@ * CHUNK_SIZE@ * initial_a@;
                adler_congruent(b@, old_target)
            };
            proof_assert! {
                let old_target = initial_b@
                    + processed_chunks@ * CHUNK_SIZE@ * initial_a@;
                let new_target = initial_b@
                    + processed_chunks@ * CHUNK_SIZE@ * initial_a@;
                adler_congruent_reindex_certificate(b@, old_target, new_target)
                    && adler_congruent(b@, new_target)
            };
            proof_assert!(b@ + 22208 * a@ <= u32::MAX@);
            proof_assert!(CHUNK_SIZE@ == 22208);
            proof_assert!(CHUNK_SIZE@ * a@ <= b@ + 22208 * a@);
            proof_assert!(CHUNK_SIZE@ * a@ <= u32::MAX@);
            let b_before_chunk = snapshot! { b };
            let chunk_size_u32 = CHUNK_SIZE as u32;
            proof_assert!(chunk_size_u32@ == CHUNK_SIZE@);
            proof_assert!(chunk_size_u32@ * a@ <= u32::MAX@);
            b += chunk_size_u32 * a;
            proof_assert! { reduced_state_safe(a@, b@); reduced_state_facts(a@, b@) };
            let a_vec_before_reduce = snapshot! { a_vec };
            let b_vec_before_reduce = snapshot! { b_vec };
            let b_before_reduce = snapshot! { b };
            proof_assert!(b_before_reduce@ == b_before_chunk@ + CHUNK_SIZE@ * a@);
            proof_assert!(a@ == initial_a@);
            proof_assert!(CHUNK_SIZE@ * a@ == CHUNK_SIZE@ * initial_a@);
            proof_assert! {
                adler_congruent(
                    b_before_chunk@,
                    initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@,
                )
            };
            proof_assert! {
                adler_congruent_refl(CHUNK_SIZE@ * initial_a@);
                adler_congruent(
                    CHUNK_SIZE@ * a@,
                    CHUNK_SIZE@ * initial_a@,
                )
            };
            proof_assert! {
                adler_congruent_add(
                    b_before_chunk@,
                    initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@,
                    CHUNK_SIZE@ * a@,
                    CHUNK_SIZE@ * initial_a@,
                );
                adler_congruent(
                    b_before_chunk@ + CHUNK_SIZE@ * a@,
                    initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@
                        + CHUNK_SIZE@ * initial_a@,
                )
            };
            proof_assert! {
                adler_congruent(
                    b_before_reduce@,
                    initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@
                        + CHUNK_SIZE@ * initial_a@,
                )
            };
            proof_assert! {
                multiplication_successor(
                    processed_chunks@,
                    CHUNK_SIZE@ * initial_a@,
                );
                (processed_chunks@ + 1) * (CHUNK_SIZE@ * initial_a@)
                    == processed_chunks@ * (CHUNK_SIZE@ * initial_a@)
                        + CHUNK_SIZE@ * initial_a@
            };
            proof_assert!(
                processed_chunks@ * CHUNK_SIZE@ * initial_a@
                    == processed_chunks@ * (CHUNK_SIZE@ * initial_a@)
            );
            proof_assert! {
                adler_congruent(
                    b_before_reduce@,
                    initial_b@ + (processed_chunks@ + 1) * CHUNK_SIZE@ * initial_a@,
                )
            };
            a_vec %= MOD;
            b_vec %= MOD;
            b %= MOD;
            proof_assert!(MOD@ == 65521);
            proof_assert!(0 < MOD@);
            proof_assert!(a@ <= u16::MAX@);
            proof_assert! {
                remainder_upper_bound(b_before_reduce@, MOD@);
                b@ < 65521
            };
            proof_assert!(b@ <= u16::MAX@);
            proof_assert! {
                reduced_state_facts(a@, b_before_reduce@);
                b@ + 22208 * a@ <= u32::MAX@
            };
            proof_assert!(a_vec@.0@ < 65521);
            proof_assert!(a_vec@.1@ < 65521);
            proof_assert!(a_vec@.2@ < 65521);
            proof_assert!(a_vec@.3@ < 65521);
            proof_assert!(b_vec@.0@ < 65521);
            proof_assert!(b_vec@.1@ < 65521);
            proof_assert!(b_vec@.2@ < 65521);
            proof_assert!(b_vec@.3@ < 65521);
            proof_assert! {
                adler_remainder_congruent(a_vec_before_reduce@.0@);
                adler_remainder_congruent(a_vec_before_reduce@.1@);
                adler_remainder_congruent(a_vec_before_reduce@.2@);
                adler_remainder_congruent(a_vec_before_reduce@.3@);
                adler_remainder_congruent(b_vec_before_reduce@.0@);
                adler_remainder_congruent(b_vec_before_reduce@.1@);
                adler_remainder_congruent(b_vec_before_reduce@.2@);
                adler_remainder_congruent(b_vec_before_reduce@.3@);
                adler_remainder_congruent(b_before_reduce@);
                adler_congruent_refl(CHUNK_SIZE@ * initial_a@);
                adler_congruent_add(b_before_chunk@, initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@, CHUNK_SIZE@ * a@, CHUNK_SIZE@ * initial_a@);
                let prefix = *processed_prefix;
                let complete = prefix.concat(chunk@);
                adler_congruent_trans(a_vec@.0@, a_vec_before_reduce@.0@, lane_sum(complete, 0));
                adler_congruent_trans(a_vec@.1@, a_vec_before_reduce@.1@, lane_sum(complete, 1));
                adler_congruent_trans(a_vec@.2@, a_vec_before_reduce@.2@, lane_sum(complete, 2));
                adler_congruent_trans(a_vec@.3@, a_vec_before_reduce@.3@, lane_sum(complete, 3));
                adler_congruent_trans(b_vec@.0@, b_vec_before_reduce@.0@, lane_accumulator(complete, 0));
                adler_congruent_trans(b_vec@.1@, b_vec_before_reduce@.1@, lane_accumulator(complete, 1));
                adler_congruent_trans(b_vec@.2@, b_vec_before_reduce@.2@, lane_accumulator(complete, 2));
                adler_congruent_trans(b_vec@.3@, b_vec_before_reduce@.3@, lane_accumulator(complete, 3));
                adler_congruent_trans(b@, b_before_reduce@, initial_b@ + (processed_chunks@ + 1) * CHUNK_SIZE@ * initial_a@);
                true
            };
            proof_assert! {
                adler_remainder_congruent(b_before_reduce@);
                adler_congruent(b@, b_before_reduce@)
            };
            proof_assert! {
                adler_congruent_trans(
                    b@,
                    b_before_reduce@,
                    initial_b@ + (processed_chunks@ + 1) * CHUNK_SIZE@ * initial_a@,
                );
                adler_congruent(
                    b@,
                    initial_b@ + (processed_chunks@ + 1) * CHUNK_SIZE@ * initial_a@,
                )
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(a_vec@.0@, lane_sum(complete, 0))
            };
            proof_assert! {
                adler_remainder_congruent(a_vec_before_reduce@.1@);
                adler_congruent(a_vec@.1@, a_vec_before_reduce@.1@)
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent_trans(
                    a_vec@.1@,
                    a_vec_before_reduce@.1@,
                    lane_sum(complete, 1),
                );
                adler_congruent(a_vec@.1@, lane_sum(complete, 1))
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(a_vec@.2@, lane_sum(complete, 2))
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(a_vec@.3@, lane_sum(complete, 3))
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(b_vec@.0@, lane_accumulator(complete, 0))
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(b_vec@.1@, lane_accumulator(complete, 1))
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(b_vec@.2@, lane_accumulator(complete, 2))
            };
            proof_assert! {
                let complete = (*processed_prefix).concat(chunk@);
                adler_congruent(b_vec@.3@, lane_accumulator(complete, 3))
            };
            proof_assert! {
                let prefix = *processed_prefix;
                let complete = prefix.concat(chunk@);
                adler_congruent(a_vec@.0@, lane_sum(complete, 0))
                    && adler_congruent(a_vec@.1@, lane_sum(complete, 1))
                    && adler_congruent(a_vec@.2@, lane_sum(complete, 2))
                    && adler_congruent(a_vec@.3@, lane_sum(complete, 3))
                    && adler_congruent(b_vec@.0@, lane_accumulator(complete, 0))
                    && adler_congruent(b_vec@.1@, lane_accumulator(complete, 1))
                    && adler_congruent(b_vec@.2@, lane_accumulator(complete, 2))
                    && adler_congruent(b_vec@.3@, lane_accumulator(complete, 3))
            };
            proof_assert!(adler_congruent(b@, initial_b@ + (processed_chunks@ + 1) * CHUNK_SIZE@ * initial_a@));
            processed_chunks += 1;
            proof_assert!(processed_chunks@ == produced.len());
            proof_assert!(adler_congruent(
                b@,
                initial_b@ + processed_chunks@ * CHUNK_SIZE@ * initial_a@,
            ));
            }
        }
        }
        // special-case the final chunk because it may be shorter than the rest
        let remainder_a_vec_entry = snapshot! { a_vec };
        let remainder_b_vec_entry = snapshot! { b_vec };
        let complete_chunks_len = snapshot! {
            multiple_of_four_product((*initial_chunks).len(), CHUNK_SIZE@)
        };
        proof_assert!(*complete_chunks_len
            == (*initial_chunks).len() * CHUNK_SIZE@);
        proof_assert!((*complete_chunks_len) % 4 == 0);
        proof_assert!(bytes@.len()
            == *complete_chunks_len + remainder_chunk@.len());
        proof_assert!(bytes@.len() % 4 == 0);
        proof_assert!(bytes@.len() == 4 * (bytes@.len() / 4));
        proof_assert!(*complete_chunks_len
            == 4 * (*complete_chunks_len / 4));
        proof_assert!(remainder_chunk@.len()
            == 4 * (bytes@.len() / 4 - *complete_chunks_len / 4));
        proof_assert!(remainder_chunk@.len() % 4 == 0);
        proof_assert!(remainder_chunk@.len() < CHUNK_SIZE@);
        proof_assert!(remainder_chunk@.len() <= 22208);
        proof_assert!(a_vec.invariant() && b_vec.invariant());
        (a_vec, b_vec) = process_chunk_values(a_vec, b_vec, remainder_chunk);
        proof_assert!(a_vec.invariant() && b_vec.invariant());
        proof_assert!(b_vec@.0@ == remainder_b_vec_entry@.0@
            + remainder_chunk@.len() / 4 * remainder_a_vec_entry@.0@
            + lane_accumulator(remainder_chunk@, 0));
        proof_assert!(b_vec@.1@ == remainder_b_vec_entry@.1@
            + remainder_chunk@.len() / 4 * remainder_a_vec_entry@.1@
            + lane_accumulator(remainder_chunk@, 1));
        proof_assert!(b_vec@.2@ == remainder_b_vec_entry@.2@
            + remainder_chunk@.len() / 4 * remainder_a_vec_entry@.2@
            + lane_accumulator(remainder_chunk@, 2));
        proof_assert!(b_vec@.3@ == remainder_b_vec_entry@.3@
            + remainder_chunk@.len() / 4 * remainder_a_vec_entry@.3@
            + lane_accumulator(remainder_chunk@, 3));
        proof_assert!(remainder_chunk@ == bytes@.subsequence(
            bytes@.len() - remainder_chunk@.len(),
            bytes@.len(),
        ));
        proof_assert!(0 <= bytes@.len() - remainder_chunk@.len());
        proof_assert! {
            let split = bytes@.len() - remainder_chunk@.len();
            subsequence_split(bytes@, split, bytes@.len());
            bytes@ == bytes@.subsequence(0, split).concat(remainder_chunk@)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_concat(prefix, remainder_chunk@, 0);
            lane_sum_concat(prefix, remainder_chunk@, 1);
            lane_sum_concat(prefix, remainder_chunk@, 2);
            lane_sum_concat(prefix, remainder_chunk@, 3);
            lane_accumulator_concat(prefix, remainder_chunk@, 0);
            lane_accumulator_concat(prefix, remainder_chunk@, 1);
            lane_accumulator_concat(prefix, remainder_chunk@, 2);
            lane_accumulator_concat(prefix, remainder_chunk@, 3);
            bytes@ == prefix.concat(remainder_chunk@)
        };
        proof_assert!(bytes@.subsequence(
            0,
            bytes@.len() - remainder_chunk@.len(),
        ).len() == *complete_chunks_len);
        proof_assert!(bytes@.subsequence(
            0,
            bytes@.len() - remainder_chunk@.len(),
        ).len() % 4 == 0);
        proof_assert!((*initial_chunks).len() * CHUNK_SIZE@
            == bytes@.len() - remainder_chunk@.len());
        proof_assert! {
            let old_end = (*initial_chunks).len() * CHUNK_SIZE@;
            let new_end = bytes@.len() - remainder_chunk@.len();
            reindex_subsequence_end_certificate(bytes@, 0, old_end, new_end)
        };
        proof_assert! {
            let old_end = (*initial_chunks).len() * CHUNK_SIZE@;
            let new_end = bytes@.len() - remainder_chunk@.len();
            reindex_subsequence_end_certificate(bytes@, 0, old_end, new_end)
                == sequence_equality_facts(
                    bytes@.subsequence(0, old_end),
                    bytes@.subsequence(0, new_end),
                )
        };
        proof_assert! {
            let old_end = (*initial_chunks).len() * CHUNK_SIZE@;
            let new_end = bytes@.len() - remainder_chunk@.len();
            sequence_equality_facts(
                bytes@.subsequence(0, old_end),
                bytes@.subsequence(0, new_end),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_a_vec_entry@.0@, lane_sum(prefix, 0))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_a_vec_entry@.1@, lane_sum(prefix, 1))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_a_vec_entry@.2@, lane_sum(prefix, 2))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_a_vec_entry@.3@, lane_sum(prefix, 3))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_b_vec_entry@.0@, lane_accumulator(prefix, 0))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_b_vec_entry@.1@, lane_accumulator(prefix, 1))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_b_vec_entry@.2@, lane_accumulator(prefix, 2))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(remainder_b_vec_entry@.3@, lane_accumulator(prefix, 3))
        };
        proof_assert!(a_vec@.0@
            == remainder_a_vec_entry@.0@ + lane_sum(remainder_chunk@, 0));
        proof_assert!(a_vec@.1@
            == remainder_a_vec_entry@.1@ + lane_sum(remainder_chunk@, 1));
        proof_assert!(a_vec@.2@
            == remainder_a_vec_entry@.2@ + lane_sum(remainder_chunk@, 2));
        proof_assert!(a_vec@.3@
            == remainder_a_vec_entry@.3@ + lane_sum(remainder_chunk@, 3));
        macro_rules! legacy_remainder_lane_proof {
            () => {
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_concat(prefix, remainder_chunk@, 0);
            lane_sum(prefix.concat(remainder_chunk@), 0)
                == lane_sum(prefix, 0) + lane_sum(remainder_chunk@, 0)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.0@ == lane_sum(prefix, 0) + factor * 65521 ==>
                remainder_a_vec_entry@.0@ + lane_sum(remainder_chunk@, 0)
                    == lane_sum(prefix, 0) + lane_sum(remainder_chunk@, 0)
                        + factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.0@,
                lane_sum(prefix, 0),
                lane_sum(remainder_chunk@, 0),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.0@,
                lane_sum(prefix, 0),
                lane_sum(remainder_chunk@, 0),
            ) == adler_congruent_add_same_facts(
                remainder_a_vec_entry@.0@,
                lane_sum(prefix, 0),
                lane_sum(remainder_chunk@, 0),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_facts(
                remainder_a_vec_entry@.0@,
                lane_sum(prefix, 0),
                lane_sum(remainder_chunk@, 0),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.0@ == lane_sum(prefix, 0) + factor * 65521 ==>
                exists<same_factor: Int>
                    remainder_a_vec_entry@.0@ + lane_sum(remainder_chunk@, 0)
                        == lane_sum(prefix, 0) + lane_sum(remainder_chunk@, 0)
                            + same_factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            (exists<factor: Int>
                remainder_a_vec_entry@.0@ == lane_sum(prefix, 0) + factor * 65521)
                ==>
                (exists<same_factor: Int>
                    remainder_a_vec_entry@.0@ + lane_sum(remainder_chunk@, 0)
                        == lane_sum(prefix, 0) + lane_sum(remainder_chunk@, 0)
                            + same_factor * 65521)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 0));
            adler_congruent_add(
                remainder_a_vec_entry@.0@,
                lane_sum(prefix, 0),
                lane_sum(remainder_chunk@, 0),
                lane_sum(remainder_chunk@, 0),
            );
            adler_congruent(
                remainder_a_vec_entry@.0@ + lane_sum(remainder_chunk@, 0),
                lane_sum(prefix, 0) + lane_sum(remainder_chunk@, 0),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.0@, lane_sum(prefix.concat(remainder_chunk@), 0))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 0));
            adler_congruent_add(
                remainder_a_vec_entry@.0@,
                lane_sum(prefix, 0),
                lane_sum(remainder_chunk@, 0),
                lane_sum(remainder_chunk@, 0),
            );
            adler_congruent(a_vec@.0@, lane_sum(bytes@, 0))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_concat(prefix, remainder_chunk@, 1);
            lane_sum(prefix.concat(remainder_chunk@), 1)
                == lane_sum(prefix, 1) + lane_sum(remainder_chunk@, 1)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.1@ == lane_sum(prefix, 1) + factor * 65521 ==>
                remainder_a_vec_entry@.1@ + lane_sum(remainder_chunk@, 1)
                    == lane_sum(prefix, 1) + lane_sum(remainder_chunk@, 1)
                        + factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.1@,
                lane_sum(prefix, 1),
                lane_sum(remainder_chunk@, 1),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.1@,
                lane_sum(prefix, 1),
                lane_sum(remainder_chunk@, 1),
            ) == adler_congruent_add_same_facts(
                remainder_a_vec_entry@.1@,
                lane_sum(prefix, 1),
                lane_sum(remainder_chunk@, 1),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_facts(
                remainder_a_vec_entry@.1@,
                lane_sum(prefix, 1),
                lane_sum(remainder_chunk@, 1),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.1@ == lane_sum(prefix, 1) + factor * 65521 ==>
                exists<same_factor: Int>
                    remainder_a_vec_entry@.1@ + lane_sum(remainder_chunk@, 1)
                        == lane_sum(prefix, 1) + lane_sum(remainder_chunk@, 1)
                            + same_factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            (exists<factor: Int>
                remainder_a_vec_entry@.1@ == lane_sum(prefix, 1) + factor * 65521)
                ==>
                (exists<same_factor: Int>
                    remainder_a_vec_entry@.1@ + lane_sum(remainder_chunk@, 1)
                        == lane_sum(prefix, 1) + lane_sum(remainder_chunk@, 1)
                            + same_factor * 65521)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 1));
            adler_congruent_add(
                remainder_a_vec_entry@.1@,
                lane_sum(prefix, 1),
                lane_sum(remainder_chunk@, 1),
                lane_sum(remainder_chunk@, 1),
            );
            adler_congruent(
                remainder_a_vec_entry@.1@ + lane_sum(remainder_chunk@, 1),
                lane_sum(prefix, 1) + lane_sum(remainder_chunk@, 1),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.1@, lane_sum(prefix.concat(remainder_chunk@), 1))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 1));
            adler_congruent_add(
                remainder_a_vec_entry@.1@,
                lane_sum(prefix, 1),
                lane_sum(remainder_chunk@, 1),
                lane_sum(remainder_chunk@, 1),
            );
            adler_congruent(a_vec@.1@, lane_sum(bytes@, 1))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_concat(prefix, remainder_chunk@, 2);
            lane_sum(prefix.concat(remainder_chunk@), 2)
                == lane_sum(prefix, 2) + lane_sum(remainder_chunk@, 2)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.2@ == lane_sum(prefix, 2) + factor * 65521 ==>
                remainder_a_vec_entry@.2@ + lane_sum(remainder_chunk@, 2)
                    == lane_sum(prefix, 2) + lane_sum(remainder_chunk@, 2)
                        + factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.2@,
                lane_sum(prefix, 2),
                lane_sum(remainder_chunk@, 2),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.2@,
                lane_sum(prefix, 2),
                lane_sum(remainder_chunk@, 2),
            ) == adler_congruent_add_same_facts(
                remainder_a_vec_entry@.2@,
                lane_sum(prefix, 2),
                lane_sum(remainder_chunk@, 2),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_facts(
                remainder_a_vec_entry@.2@,
                lane_sum(prefix, 2),
                lane_sum(remainder_chunk@, 2),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.2@ == lane_sum(prefix, 2) + factor * 65521 ==>
                exists<same_factor: Int>
                    remainder_a_vec_entry@.2@ + lane_sum(remainder_chunk@, 2)
                        == lane_sum(prefix, 2) + lane_sum(remainder_chunk@, 2)
                            + same_factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            (exists<factor: Int>
                remainder_a_vec_entry@.2@ == lane_sum(prefix, 2) + factor * 65521)
                ==>
                (exists<same_factor: Int>
                    remainder_a_vec_entry@.2@ + lane_sum(remainder_chunk@, 2)
                        == lane_sum(prefix, 2) + lane_sum(remainder_chunk@, 2)
                            + same_factor * 65521)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 2));
            adler_congruent_add(
                remainder_a_vec_entry@.2@,
                lane_sum(prefix, 2),
                lane_sum(remainder_chunk@, 2),
                lane_sum(remainder_chunk@, 2),
            );
            adler_congruent(
                remainder_a_vec_entry@.2@ + lane_sum(remainder_chunk@, 2),
                lane_sum(prefix, 2) + lane_sum(remainder_chunk@, 2),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.2@, lane_sum(prefix.concat(remainder_chunk@), 2))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 2));
            adler_congruent_add(
                remainder_a_vec_entry@.2@,
                lane_sum(prefix, 2),
                lane_sum(remainder_chunk@, 2),
                lane_sum(remainder_chunk@, 2),
            );
            adler_congruent(a_vec@.2@, lane_sum(bytes@, 2))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_concat(prefix, remainder_chunk@, 3);
            lane_sum(prefix.concat(remainder_chunk@), 3)
                == lane_sum(prefix, 3) + lane_sum(remainder_chunk@, 3)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.3@ == lane_sum(prefix, 3) + factor * 65521 ==>
                remainder_a_vec_entry@.3@ + lane_sum(remainder_chunk@, 3)
                    == lane_sum(prefix, 3) + lane_sum(remainder_chunk@, 3)
                        + factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.3@,
                lane_sum(prefix, 3),
                lane_sum(remainder_chunk@, 3),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_certificate(
                remainder_a_vec_entry@.3@,
                lane_sum(prefix, 3),
                lane_sum(remainder_chunk@, 3),
            ) == adler_congruent_add_same_facts(
                remainder_a_vec_entry@.3@,
                lane_sum(prefix, 3),
                lane_sum(remainder_chunk@, 3),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_add_same_facts(
                remainder_a_vec_entry@.3@,
                lane_sum(prefix, 3),
                lane_sum(remainder_chunk@, 3),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            forall<factor: Int>
                remainder_a_vec_entry@.3@ == lane_sum(prefix, 3) + factor * 65521 ==>
                exists<same_factor: Int>
                    remainder_a_vec_entry@.3@ + lane_sum(remainder_chunk@, 3)
                        == lane_sum(prefix, 3) + lane_sum(remainder_chunk@, 3)
                            + same_factor * 65521
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            (exists<factor: Int>
                remainder_a_vec_entry@.3@ == lane_sum(prefix, 3) + factor * 65521)
                ==>
                (exists<same_factor: Int>
                    remainder_a_vec_entry@.3@ + lane_sum(remainder_chunk@, 3)
                        == lane_sum(prefix, 3) + lane_sum(remainder_chunk@, 3)
                            + same_factor * 65521)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 3));
            adler_congruent_add(
                remainder_a_vec_entry@.3@,
                lane_sum(prefix, 3),
                lane_sum(remainder_chunk@, 3),
                lane_sum(remainder_chunk@, 3),
            );
            adler_congruent(
                remainder_a_vec_entry@.3@ + lane_sum(remainder_chunk@, 3),
                lane_sum(prefix, 3) + lane_sum(remainder_chunk@, 3),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.3@, lane_sum(prefix.concat(remainder_chunk@), 3))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent_refl(lane_sum(remainder_chunk@, 3));
            adler_congruent_add(
                remainder_a_vec_entry@.3@,
                lane_sum(prefix, 3),
                lane_sum(remainder_chunk@, 3),
                lane_sum(remainder_chunk@, 3),
            );
            adler_congruent(a_vec@.3@, lane_sum(bytes@, 3))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            adler_congruent_scale(remainder_a_vec_entry@.0@, lane_sum(prefix, 0), groups);
            adler_congruent_scale(remainder_a_vec_entry@.1@, lane_sum(prefix, 1), groups);
            adler_congruent_scale(remainder_a_vec_entry@.2@, lane_sum(prefix, 2), groups);
            adler_congruent_scale(remainder_a_vec_entry@.3@, lane_sum(prefix, 3), groups);
            adler_congruent_add(remainder_b_vec_entry@.0@, lane_accumulator(prefix, 0), groups * remainder_a_vec_entry@.0@, groups * lane_sum(prefix, 0));
            adler_congruent_add(remainder_b_vec_entry@.1@, lane_accumulator(prefix, 1), groups * remainder_a_vec_entry@.1@, groups * lane_sum(prefix, 1));
            adler_congruent_add(remainder_b_vec_entry@.2@, lane_accumulator(prefix, 2), groups * remainder_a_vec_entry@.2@, groups * lane_sum(prefix, 2));
            adler_congruent_add(remainder_b_vec_entry@.3@, lane_accumulator(prefix, 3), groups * remainder_a_vec_entry@.3@, groups * lane_sum(prefix, 3));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 0));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 1));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 2));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 3));
            adler_congruent_add(remainder_b_vec_entry@.0@ + groups * remainder_a_vec_entry@.0@, lane_accumulator(prefix, 0) + groups * lane_sum(prefix, 0), lane_accumulator(remainder_chunk@, 0), lane_accumulator(remainder_chunk@, 0));
            adler_congruent_add(remainder_b_vec_entry@.1@ + groups * remainder_a_vec_entry@.1@, lane_accumulator(prefix, 1) + groups * lane_sum(prefix, 1), lane_accumulator(remainder_chunk@, 1), lane_accumulator(remainder_chunk@, 1));
            adler_congruent_add(remainder_b_vec_entry@.2@ + groups * remainder_a_vec_entry@.2@, lane_accumulator(prefix, 2) + groups * lane_sum(prefix, 2), lane_accumulator(remainder_chunk@, 2), lane_accumulator(remainder_chunk@, 2));
            adler_congruent_add(remainder_b_vec_entry@.3@ + groups * remainder_a_vec_entry@.3@, lane_accumulator(prefix, 3) + groups * lane_sum(prefix, 3), lane_accumulator(remainder_chunk@, 3), lane_accumulator(remainder_chunk@, 3));
            true
        };
            }
        }
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            prefix.concat(remainder_chunk@) == bytes@
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            prefix.len() % 4 == 0
                && remainder_chunk@.len() % 4 == 0
                && adler_congruent(remainder_a_vec_entry@.0@, lane_sum(prefix, 0))
                && adler_congruent(remainder_a_vec_entry@.1@, lane_sum(prefix, 1))
                && adler_congruent(remainder_a_vec_entry@.2@, lane_sum(prefix, 2))
                && adler_congruent(remainder_a_vec_entry@.3@, lane_sum(prefix, 3))
                && a_vec@.0@ == remainder_a_vec_entry@.0@ + lane_sum(remainder_chunk@, 0)
                && a_vec@.1@ == remainder_a_vec_entry@.1@ + lane_sum(remainder_chunk@, 1)
                && a_vec@.2@ == remainder_a_vec_entry@.2@ + lane_sum(remainder_chunk@, 2)
                && a_vec@.3@ == remainder_a_vec_entry@.3@ + lane_sum(remainder_chunk@, 3)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_congruence_certificate(
                prefix, remainder_chunk@, 0,
                remainder_a_vec_entry@.0@, a_vec@.0@,
            ) == adler_congruence_facts(
                a_vec@.0@, lane_sum(prefix.concat(remainder_chunk@), 0),
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_congruence_certificate_1(
                prefix, remainder_chunk@,
                remainder_a_vec_entry@.1@, a_vec@.1@,
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_congruence_certificate_2(
                prefix, remainder_chunk@,
                remainder_a_vec_entry@.2@, a_vec@.2@,
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_congruence_certificate_3(
                prefix, remainder_chunk@,
                remainder_a_vec_entry@.3@, a_vec@.3@,
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.0@, lane_sum(prefix.concat(remainder_chunk@), 0))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.1@, lane_sum(prefix.concat(remainder_chunk@), 1))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.2@, lane_sum(prefix.concat(remainder_chunk@), 2))
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            adler_congruent(a_vec@.3@, lane_sum(prefix.concat(remainder_chunk@), 3))
        };
        proof_assert!(adler_congruent(a_vec@.0@, lane_sum(bytes@, 0)));
        proof_assert!(adler_congruent(a_vec@.1@, lane_sum(bytes@, 1)));
        proof_assert!(adler_congruent(a_vec@.2@, lane_sum(bytes@, 2)));
        proof_assert!(adler_congruent(a_vec@.3@, lane_sum(bytes@, 3)));
        #[cfg(any())]
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            prefix.len() % 4 == 0
                && remainder_chunk@.len() % 4 == 0
                && groups == remainder_chunk@.len() / 4
                && adler_congruent(remainder_a_vec_entry@.0@, lane_sum(prefix, 0))
                && adler_congruent(remainder_b_vec_entry@.0@, lane_accumulator(prefix, 0))
                && b_vec@.0@ == remainder_b_vec_entry@.0@
                    + groups * remainder_a_vec_entry@.0@
                    + lane_accumulator(remainder_chunk@, 0)
        };
        #[cfg(any())]
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            prefix.len() % 4 == 0
                && remainder_chunk@.len() % 4 == 0
                && groups == remainder_chunk@.len() / 4
                && adler_congruent(remainder_a_vec_entry@.3@, lane_sum(prefix, 3))
                && adler_congruent(remainder_b_vec_entry@.3@, lane_accumulator(prefix, 3))
                && b_vec@.3@ == remainder_b_vec_entry@.3@
                    + groups * remainder_a_vec_entry@.3@
                    + lane_accumulator(remainder_chunk@, 3)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            lane_accumulator_congruence_step_0(
                prefix, remainder_chunk@, groups,
                remainder_a_vec_entry@.0@,
                remainder_b_vec_entry@.0@,
                b_vec@.0@,
            );
            adler_congruent(
                b_vec@.0@,
                lane_accumulator(prefix.concat(remainder_chunk@), 0),
            )
        };
        proof_assert!(adler_congruent(b_vec@.0@, lane_accumulator(bytes@, 0)));
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            prefix.len() % 4 == 0
                && remainder_chunk@.len() % 4 == 0
                && groups == remainder_chunk@.len() / 4
                && adler_congruent(remainder_a_vec_entry@.1@, lane_sum(prefix, 1))
                && adler_congruent(remainder_b_vec_entry@.1@, lane_accumulator(prefix, 1))
                && b_vec@.1@ == remainder_b_vec_entry@.1@
                    + groups * remainder_a_vec_entry@.1@
                    + lane_accumulator(remainder_chunk@, 1)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            lane_accumulator_congruence_ready_1(
                prefix, remainder_chunk@, groups,
                remainder_a_vec_entry@.1@,
                remainder_b_vec_entry@.1@,
                b_vec@.1@,
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            lane_accumulator_congruence_certificate_1(
                prefix, remainder_chunk@, groups,
                remainder_a_vec_entry@.1@,
                remainder_b_vec_entry@.1@,
                b_vec@.1@,
            ) && adler_congruent(
                b_vec@.1@,
                lane_accumulator(prefix.concat(remainder_chunk@), 1),
            )
        };
        proof_assert!(adler_congruent(b_vec@.1@, lane_accumulator(bytes@, 1)));
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            prefix.len() % 4 == 0
                && remainder_chunk@.len() % 4 == 0
                && groups == remainder_chunk@.len() / 4
                && adler_congruent(remainder_a_vec_entry@.2@, lane_sum(prefix, 2))
                && adler_congruent(remainder_b_vec_entry@.2@, lane_accumulator(prefix, 2))
                && b_vec@.2@ == remainder_b_vec_entry@.2@
                    + groups * remainder_a_vec_entry@.2@
                    + lane_accumulator(remainder_chunk@, 2)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            lane_accumulator_congruence_ready_2(
                prefix, remainder_chunk@, groups,
                remainder_a_vec_entry@.2@,
                remainder_b_vec_entry@.2@,
                b_vec@.2@,
            )
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            lane_accumulator_congruence_certificate_2(
                prefix, remainder_chunk@, groups,
                remainder_a_vec_entry@.2@,
                remainder_b_vec_entry@.2@,
                b_vec@.2@,
            ) && adler_congruent(
                b_vec@.2@,
                lane_accumulator(prefix.concat(remainder_chunk@), 2),
            )
        };
        proof_assert!(adler_congruent(b_vec@.2@, lane_accumulator(bytes@, 2)));
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            prefix.len() % 4 == 0
                && remainder_chunk@.len() % 4 == 0
                && groups == remainder_chunk@.len() / 4
                && adler_congruent(remainder_a_vec_entry@.3@, lane_sum(prefix, 3))
                && adler_congruent(remainder_b_vec_entry@.3@, lane_accumulator(prefix, 3))
                && b_vec@.3@ == remainder_b_vec_entry@.3@
                    + groups * remainder_a_vec_entry@.3@
                    + lane_accumulator(remainder_chunk@, 3)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            lane_accumulator_congruence_lemma_3(
                prefix, remainder_chunk@, groups,
                remainder_a_vec_entry@.3@,
                remainder_b_vec_entry@.3@,
                b_vec@.3@,
            );
            adler_congruent(
                b_vec@.3@,
                lane_accumulator(prefix.concat(remainder_chunk@), 3),
            )
        };
        proof_assert!(adler_congruent(b_vec@.3@, lane_accumulator(bytes@, 3)));
        proof_assert! { partial_chunk_safe(remainder_chunk@.len(), a@, b@); partial_chunk_facts(remainder_chunk@.len(), a@, b@) };
        proof_assert!(adler_congruent(
            b@,
            initial_b@ + (*initial_chunks).len() * CHUNK_SIZE@ * initial_a@,
        ));
        proof_assert!(
            initial_b@ + (*initial_chunks).len() * CHUNK_SIZE@ * initial_a@
                == initial_b@
                    + (bytes@.len() - remainder_chunk@.len()) * initial_a@
        );
        proof_assert!(adler_congruent(
            b@,
            initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@,
        ));
        let b_before_remainder_chunk = snapshot! { b };
        let remainder_len = remainder_chunk.len() as u32;
        proof_assert!(remainder_len@ == remainder_chunk@.len());
        proof_assert!(remainder_chunk@.len() * a@ <= u32::MAX@);
        proof_assert! {
            multiplication_congruence(
                remainder_len@,
                remainder_chunk@.len(),
                a@,
            );
            remainder_len@ * a@ == remainder_chunk@.len() * a@
        };
        proof_assert!(0 <= remainder_len@);
        proof_assert!(0 <= a@);
        proof_assert!(0 <= remainder_len@ * a@);
        proof_assert!(remainder_len@ * a@ <= u32::MAX@);
        proof_assert!(0 <= remainder_len@ * a@
            && remainder_len@ * a@ <= u32::MAX@);
        let remainder_delta = remainder_len * a;
        proof_assert!(remainder_delta@ == remainder_chunk@.len() * a@);
        proof_assert!(b@ + remainder_delta@ <= u32::MAX@);
        b += remainder_delta;
        let a_vec_before_remainder_reduce = snapshot! { a_vec };
        let b_vec_before_remainder_reduce = snapshot! { b_vec };
        let b_before_remainder_reduce = snapshot! { b };
        proof_assert! {
            adler_congruent_refl(a@);
            adler_congruent_scale(
                a@,
                initial_a@,
                remainder_chunk@.len(),
            );
            adler_congruent(
                remainder_chunk@.len() * a@,
                remainder_chunk@.len() * initial_a@,
            )
        };
        proof_assert!(adler_congruent(
            b_before_remainder_chunk@,
            initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@,
        ) && adler_congruent(
            remainder_chunk@.len() * a@,
            remainder_chunk@.len() * initial_a@,
        ));
        proof_assert! {
            adler_congruent_add(
                b_before_remainder_chunk@,
                initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@,
                remainder_chunk@.len() * a@,
                remainder_chunk@.len() * initial_a@,
            ); adler_congruent(
                b_before_remainder_chunk@ + remainder_chunk@.len() * a@,
                initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@
                    + remainder_chunk@.len() * initial_a@,
            )
        };
        proof_assert!(b_before_remainder_reduce@
            == b_before_remainder_chunk@ + remainder_chunk@.len() * a@);
        proof_assert!(initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@
            + remainder_chunk@.len() * initial_a@
            == initial_b@ + bytes@.len() * initial_a@);
        proof_assert!(adler_congruent(
            b_before_remainder_reduce@,
            initial_b@ + bytes@.len() * initial_a@,
        ));
        a_vec %= MOD;
        b_vec %= MOD;
        b %= MOD;
        proof_assert! {
            adler_remainder_congruent(b_before_remainder_reduce@);
            adler_congruent(b@, b_before_remainder_reduce@)
        };
        proof_assert! {
            adler_congruent_trans(
                b@,
                b_before_remainder_reduce@,
                initial_b@ + bytes@.len() * initial_a@,
            );
            adler_congruent(b@, initial_b@ + bytes@.len() * initial_a@)
        };
        proof_assert! {
            adler_remainder_congruent(a_vec_before_remainder_reduce@.0@);
            adler_remainder_congruent(a_vec_before_remainder_reduce@.1@);
            adler_remainder_congruent(a_vec_before_remainder_reduce@.2@);
            adler_remainder_congruent(a_vec_before_remainder_reduce@.3@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.0@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.1@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.2@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.3@);
            adler_remainder_congruent(b_before_remainder_reduce@);
            adler_congruent_refl(remainder_chunk@.len() * initial_a@);
            adler_congruent_add(b_before_remainder_chunk@, initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@, remainder_chunk@.len() * a@, remainder_chunk@.len() * initial_a@);
            adler_congruent_trans(a_vec@.0@, a_vec_before_remainder_reduce@.0@, lane_sum(bytes@, 0));
            adler_congruent_trans(a_vec@.1@, a_vec_before_remainder_reduce@.1@, lane_sum(bytes@, 1));
            adler_congruent_trans(a_vec@.2@, a_vec_before_remainder_reduce@.2@, lane_sum(bytes@, 2));
            adler_congruent_trans(a_vec@.3@, a_vec_before_remainder_reduce@.3@, lane_sum(bytes@, 3));
            adler_congruent_trans(b_vec@.0@, b_vec_before_remainder_reduce@.0@, lane_accumulator(bytes@, 0));
            adler_congruent_trans(b_vec@.1@, b_vec_before_remainder_reduce@.1@, lane_accumulator(bytes@, 1));
            adler_congruent_trans(b_vec@.2@, b_vec_before_remainder_reduce@.2@, lane_accumulator(bytes@, 2));
            adler_congruent_trans(b_vec@.3@, b_vec_before_remainder_reduce@.3@, lane_accumulator(bytes@, 3));
            adler_congruent_trans(b@, b_before_remainder_reduce@, initial_b@ + bytes@.len() * initial_a@);
            lane_sum_partition(bytes@);
            lane_weight_partition(bytes@);
            true
        };
        proof_assert!(adler_congruent(a_vec@.0@, lane_sum(bytes@, 0)));
        proof_assert!(adler_congruent(a_vec@.1@, lane_sum(bytes@, 1)));
        proof_assert!(adler_congruent(a_vec@.2@, lane_sum(bytes@, 2)));
        proof_assert!(adler_congruent(a_vec@.3@, lane_sum(bytes@, 3)));
        proof_assert!(adler_congruent(b_vec@.0@, lane_accumulator(bytes@, 0)));
        proof_assert!(adler_congruent(b_vec@.1@, lane_accumulator(bytes@, 1)));
        proof_assert!(adler_congruent(b_vec@.2@, lane_accumulator(bytes@, 2)));
        proof_assert!(adler_congruent(b_vec@.3@, lane_accumulator(bytes@, 3)));
        proof_assert!(adler_congruent(b@, initial_b@ + bytes@.len() * initial_a@));
        proof_assert!(crate::adler32_byte_sum(bytes@)
            == lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                + lane_sum(bytes@, 2) + lane_sum(bytes@, 3));
        proof_assert!(crate::adler32_weighted_sum(bytes@)
            == 4 * (lane_accumulator(bytes@, 0) + lane_accumulator(bytes@, 1)
                + lane_accumulator(bytes@, 2) + lane_accumulator(bytes@, 3))
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                - 3 * lane_sum(bytes@, 3));

        // combine the sub-sum results into the main sum
        let reduced_a_vec = snapshot! { a_vec };
        let reduced_b_vec = snapshot! { b_vec };
        proof_assert! {
            adler_congruent_scale(reduced_b_vec@.0@, lane_accumulator(bytes@, 0), 4);
            adler_congruent_scale(reduced_b_vec@.1@, lane_accumulator(bytes@, 1), 4);
            adler_congruent_scale(reduced_b_vec@.2@, lane_accumulator(bytes@, 2), 4);
            adler_congruent_scale(reduced_b_vec@.3@, lane_accumulator(bytes@, 3), 4);
            true
        };
        proof_assert!(adler_congruent_scale_certificate(
            reduced_b_vec@.0@, lane_accumulator(bytes@, 0), 4,
        ));
        proof_assert!(adler_congruent_scale_certificate(
            reduced_b_vec@.1@, lane_accumulator(bytes@, 1), 4,
        ));
        proof_assert!(adler_congruent_scale_certificate(
            reduced_b_vec@.2@, lane_accumulator(bytes@, 2), 4,
        ));
        proof_assert!(adler_congruent_scale_certificate(
            reduced_b_vec@.3@, lane_accumulator(bytes@, 3), 4,
        ));
        proof_assert!(adler_congruent(4 * reduced_b_vec@.0@, 4 * lane_accumulator(bytes@, 0)));
        proof_assert!(adler_congruent(4 * reduced_b_vec@.1@, 4 * lane_accumulator(bytes@, 1)));
        proof_assert!(adler_congruent(4 * reduced_b_vec@.2@, 4 * lane_accumulator(bytes@, 2)));
        proof_assert!(adler_congruent(4 * reduced_b_vec@.3@, 4 * lane_accumulator(bytes@, 3)));
        proof_assert!(b_vec@.0@ < MOD@);
        proof_assert!(b_vec@.1@ < MOD@);
        proof_assert!(b_vec@.2@ < MOD@);
        proof_assert!(b_vec@.3@ < MOD@);
        proof_assert!(b_vec@.0@ * 4 <= 4 * (MOD@ - 1));
        proof_assert!(b_vec@.1@ * 4 <= 4 * (MOD@ - 1));
        proof_assert!(b_vec@.2@ * 4 <= 4 * (MOD@ - 1));
        proof_assert!(b_vec@.3@ * 4 <= 4 * (MOD@ - 1));
        proof_assert!(4 * (MOD@ - 1) <= u32::MAX@);
        proof_assert!(b_vec@.0@ * 4 <= u32::MAX@);
        proof_assert!(b_vec@.1@ * 4 <= u32::MAX@);
        proof_assert!(b_vec@.2@ * 4 <= u32::MAX@);
        proof_assert!(b_vec@.3@ * 4 <= u32::MAX@);
        proof_assert!(b_vec@.0@ * 4 <= u32::MAX@ && b_vec@.1@ * 4 <= u32::MAX@ && b_vec@.2@ * 4 <= u32::MAX@ && b_vec@.3@ * 4 <= u32::MAX@);
        b_vec *= 4;
        let four_b_vec = snapshot! { b_vec };
        proof_assert!(adler_congruent(four_b_vec@.0@, 4 * lane_accumulator(bytes@, 0)));
        proof_assert!(adler_congruent(four_b_vec@.1@, 4 * lane_accumulator(bytes@, 1)));
        proof_assert!(adler_congruent(four_b_vec@.2@, 4 * lane_accumulator(bytes@, 2)));
        proof_assert!(adler_congruent(four_b_vec@.3@, 4 * lane_accumulator(bytes@, 3)));
        proof_assert!(b_vec@.1@ + (MOD@ - a_vec@.1@) <= u32::MAX@);
        b_vec.0[1] += MOD - a_vec.0[1];
        proof_assert!(b_vec@.1@ == four_b_vec@.1@ + (MOD@ - a_vec@.1@) * 1);
        proof_assert!(adler_adjusted_lane_one_certificate(
            four_b_vec@.1@,
            lane_accumulator(bytes@, 1),
            a_vec@.1@,
            lane_sum(bytes@, 1),
        ));
        proof_assert!(adler_adjusted_lane_one_certificate(
            four_b_vec@.1@,
            lane_accumulator(bytes@, 1),
            a_vec@.1@,
            lane_sum(bytes@, 1),
        ) == adler_congruence_facts(
            four_b_vec@.1@ + (MOD@ - a_vec@.1@) * 1,
            4 * lane_accumulator(bytes@, 1) - 1 * lane_sum(bytes@, 1),
        ));
        proof_assert!(adler_congruence_facts(
            four_b_vec@.1@ + (MOD@ - a_vec@.1@) * 1,
            4 * lane_accumulator(bytes@, 1) - 1 * lane_sum(bytes@, 1),
        ));
        proof_assert! {
            adler_congruent(
                four_b_vec@.1@ + (MOD@ - a_vec@.1@) * 1,
                4 * lane_accumulator(bytes@, 1) - 1 * lane_sum(bytes@, 1),
            )
        };
        proof_assert!(adler_congruent(b_vec@.1@,
            4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1)));
        proof_assert!(b_vec@.1@ <= 458643);
        proof_assert!(b_vec@.2@ == four_b_vec@.2@);
        proof_assert!(reduced_b_vec@.2@ < MOD@);
        proof_assert!(four_b_vec@.2@ == 4 * reduced_b_vec@.2@);
        proof_assert!(four_b_vec@.2@ <= 4 * (MOD@ - 1));
        proof_assert!(a_vec@.2@ < MOD@);
        proof_assert!((MOD@ - a_vec@.2@) * 2 <= MOD@ * 2);
        proof_assert!(b_vec@.2@ + (MOD@ - a_vec@.2@) * 2
            <= 6 * MOD@);
        proof_assert!(6 * MOD@ <= u32::MAX@);
        proof_assert!(b_vec@.2@ + (MOD@ - a_vec@.2@) * 2 <= u32::MAX@);
        b_vec.0[2] += (MOD - a_vec.0[2]) * 2;
        proof_assert! {
            adler_congruent_modulus_minus(a_vec@.2@, lane_sum(bytes@, 2));
            adler_congruent_scale(MOD@ - a_vec@.2@, -lane_sum(bytes@, 2), 2);
            adler_congruent_add(four_b_vec@.2@, 4 * lane_accumulator(bytes@, 2), (MOD@ - a_vec@.2@) * 2, -2 * lane_sum(bytes@, 2));
            true
        };
        proof_assert!(adler_congruent_modulus_minus_certificate(
            a_vec@.2@,
            lane_sum(bytes@, 2),
        ));
        proof_assert!(adler_congruent_modulus_minus_certificate(
            a_vec@.2@,
            lane_sum(bytes@, 2),
        ) == adler_congruence_facts(
            MOD@ - a_vec@.2@,
            -lane_sum(bytes@, 2),
        ));
        proof_assert!(adler_congruence_facts(
            MOD@ - a_vec@.2@,
            -lane_sum(bytes@, 2),
        ));
        proof_assert!(adler_congruent_scale_certificate(
            MOD@ - a_vec@.2@,
            -lane_sum(bytes@, 2),
            2,
        ));
        proof_assert!(adler_congruent_scale_certificate(
            MOD@ - a_vec@.2@,
            -lane_sum(bytes@, 2),
            2,
        ) == adler_congruence_facts(
            (MOD@ - a_vec@.2@) * 2,
            -lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(adler_congruence_facts(
            (MOD@ - a_vec@.2@) * 2,
            -lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(adler_congruent_add_certificate(
            four_b_vec@.2@,
            4 * lane_accumulator(bytes@, 2),
            (MOD@ - a_vec@.2@) * 2,
            -lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(adler_congruent_add_certificate(
            four_b_vec@.2@,
            4 * lane_accumulator(bytes@, 2),
            (MOD@ - a_vec@.2@) * 2,
            -lane_sum(bytes@, 2) * 2,
        ) == adler_congruence_facts(
            four_b_vec@.2@ + (MOD@ - a_vec@.2@) * 2,
            4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(adler_congruence_facts(
            four_b_vec@.2@ + (MOD@ - a_vec@.2@) * 2,
            4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(b_vec@.2@
            == four_b_vec@.2@ + (MOD@ - a_vec@.2@) * 2);
        proof_assert!(adler_congruence_to_predicate_certificate(
            b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(adler_congruence_to_predicate_certificate(
            b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2,
        ) == adler_congruence_facts(
            b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2,
        ));
        proof_assert!(adler_congruent(b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2));
        proof_assert!(4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 2) * 2
            == 4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2));
        proof_assert!(adler_congruent(b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2)));
        proof_assert!(b_vec@.1@ <= 458643);
        proof_assert!(b_vec@.2@ <= 458643);
        proof_assert!(b_vec@.3@ == four_b_vec@.3@);
        proof_assert!(reduced_b_vec@.3@ < MOD@);
        proof_assert!(four_b_vec@.3@ == 4 * reduced_b_vec@.3@);
        proof_assert!(four_b_vec@.3@ <= 4 * (MOD@ - 1));
        proof_assert!(a_vec@.3@ < MOD@);
        proof_assert!((MOD@ - a_vec@.3@) * 3 <= MOD@ * 3);
        proof_assert!(b_vec@.3@ + (MOD@ - a_vec@.3@) * 3
            <= 7 * MOD@);
        proof_assert!(7 * MOD@ <= u32::MAX@);
        proof_assert!(b_vec@.3@ + (MOD@ - a_vec@.3@) * 3 <= u32::MAX@);
        b_vec.0[3] += (MOD - a_vec.0[3]) * 3;
        proof_assert! {
            adler_congruent_modulus_minus(a_vec@.3@, lane_sum(bytes@, 3));
            adler_congruent_scale(MOD@ - a_vec@.3@, -lane_sum(bytes@, 3), 3);
            adler_congruent_add(four_b_vec@.3@, 4 * lane_accumulator(bytes@, 3), (MOD@ - a_vec@.3@) * 3, -3 * lane_sum(bytes@, 3));
            true
        };
        proof_assert!(adler_congruent_modulus_minus_certificate(
            a_vec@.3@,
            lane_sum(bytes@, 3),
        ));
        proof_assert!(adler_congruent_modulus_minus_certificate(
            a_vec@.3@,
            lane_sum(bytes@, 3),
        ) == adler_congruence_facts(
            MOD@ - a_vec@.3@,
            -lane_sum(bytes@, 3),
        ));
        proof_assert!(adler_congruence_facts(
            MOD@ - a_vec@.3@,
            -lane_sum(bytes@, 3),
        ));
        proof_assert!(adler_congruent_scale_certificate(
            MOD@ - a_vec@.3@,
            -lane_sum(bytes@, 3),
            3,
        ));
        proof_assert!(adler_congruent_scale_certificate(
            MOD@ - a_vec@.3@,
            -lane_sum(bytes@, 3),
            3,
        ) == adler_congruence_facts(
            (MOD@ - a_vec@.3@) * 3,
            -lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(adler_congruence_facts(
            (MOD@ - a_vec@.3@) * 3,
            -lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(adler_congruent_add_certificate(
            four_b_vec@.3@,
            4 * lane_accumulator(bytes@, 3),
            (MOD@ - a_vec@.3@) * 3,
            -lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(adler_congruent_add_certificate(
            four_b_vec@.3@,
            4 * lane_accumulator(bytes@, 3),
            (MOD@ - a_vec@.3@) * 3,
            -lane_sum(bytes@, 3) * 3,
        ) == adler_congruence_facts(
            four_b_vec@.3@ + (MOD@ - a_vec@.3@) * 3,
            4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(adler_congruence_facts(
            four_b_vec@.3@ + (MOD@ - a_vec@.3@) * 3,
            4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(b_vec@.3@
            == four_b_vec@.3@ + (MOD@ - a_vec@.3@) * 3);
        proof_assert!(adler_congruence_to_predicate_certificate(
            b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(adler_congruence_to_predicate_certificate(
            b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3,
        ) == adler_congruence_facts(
            b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3,
        ));
        proof_assert!(adler_congruent(b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3));
        proof_assert!(4 * lane_accumulator(bytes@, 3) - lane_sum(bytes@, 3) * 3
            == 4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3));
        proof_assert!(adler_congruent(b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3)));
        proof_assert!(b_vec@.1@ <= 458643);
        proof_assert!(b_vec@.2@ <= 458643);
        proof_assert!(b_vec@.3@ <= 458643);
        let a_entry = snapshot! { a };
        proof_assert!(a_vec@.0@ <= 65520 && a_vec@.1@ <= 65520
            && a_vec@.2@ <= 65520 && a_vec@.3@ <= 65520);
        proof_assert!(a@ + a_vec@.0@ <= u32::MAX@);
        a += a_vec.0[0];
        proof_assert!(a@ + a_vec@.1@ <= u32::MAX@);
        a += a_vec.0[1];
        proof_assert!(a@ + a_vec@.2@ <= u32::MAX@);
        a += a_vec.0[2];
        proof_assert!(a@ + a_vec@.3@ <= u32::MAX@);
        a += a_vec.0[3];
        proof_assert!(u32x4_prefix_sum(a_vec@, 4)
            == a_vec@.0@ + a_vec@.1@ + a_vec@.2@ + a_vec@.3@);
        proof_assert!(a@ == a_entry@ + u32x4_prefix_sum(a_vec@, 4));
        proof_assert!(a@ == a_entry@
            + a_vec@.0@ + a_vec@.1@ + a_vec@.2@ + a_vec@.3@);
        proof_assert! {
            adler_congruent_refl(a_entry@);
            adler_congruent(a_entry@, initial_a@)
        };
        /*
        proof_assert!(adler_congruent(a_entry@, initial_a@)
            && adler_congruent(a_vec@.0@, lane_sum(bytes@, 0)));
        proof_assert! {
            adler_congruent_add(
                a_entry@,
                initial_a@,
                a_vec@.0@,
                lane_sum(bytes@, 0),
            ); adler_congruent(
                a_entry@ + a_vec@.0@,
                initial_a@ + lane_sum(bytes@, 0),
            )
        };
        proof_assert!(adler_congruent(
            a_entry@ + a_vec@.0@,
            initial_a@ + lane_sum(bytes@, 0),
        ) && adler_congruent(a_vec@.1@, lane_sum(bytes@, 1)));
        proof_assert! {
            adler_congruent_add(
                a_entry@ + a_vec@.0@,
                initial_a@ + lane_sum(bytes@, 0),
                a_vec@.1@,
                lane_sum(bytes@, 1),
            ); adler_congruent(
                a_entry@ + a_vec@.0@ + a_vec@.1@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1),
            )
        };
        proof_assert!(adler_congruent(
            a_entry@ + a_vec@.0@ + a_vec@.1@,
            initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1),
        ) && adler_congruent(a_vec@.2@, lane_sum(bytes@, 2)));
        proof_assert! {
            adler_congruent_add(
                a_entry@ + a_vec@.0@ + a_vec@.1@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1),
                a_vec@.2@,
                lane_sum(bytes@, 2),
            ); adler_congruent(
                a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                    + lane_sum(bytes@, 2),
            )
        };
        proof_assert!(adler_congruent(
            a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@,
            initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                + lane_sum(bytes@, 2),
        ) && adler_congruent(a_vec@.3@, lane_sum(bytes@, 3)));
        proof_assert! {
            adler_congruent_add_four_certificate(
                a_entry@,
                initial_a@,
                a_vec@.0@,
                lane_sum(bytes@, 0),
                a_vec@.1@,
                lane_sum(bytes@, 1),
                a_vec@.2@,
                lane_sum(bytes@, 2),
                a_vec@.3@,
                lane_sum(bytes@, 3),
            ); adler_congruent(
                a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@ + a_vec@.3@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                    + lane_sum(bytes@, 2) + lane_sum(bytes@, 3),
            )
        };
        */
        proof_assert!(adler_congruent(a_entry@, initial_a@)
            && adler_congruent(a_vec@.0@, lane_sum(bytes@, 0))
            && adler_congruent(a_vec@.1@, lane_sum(bytes@, 1))
            && adler_congruent(a_vec@.2@, lane_sum(bytes@, 2))
            && adler_congruent(a_vec@.3@, lane_sum(bytes@, 3)));
        proof_assert! {
            adler_congruent_add_four_certificate(
                a_entry@,
                initial_a@,
                a_vec@.0@,
                lane_sum(bytes@, 0),
                a_vec@.1@,
                lane_sum(bytes@, 1),
                a_vec@.2@,
                lane_sum(bytes@, 2),
                a_vec@.3@,
                lane_sum(bytes@, 3),
            ) && adler_congruence_facts(
                a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@ + a_vec@.3@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                    + lane_sum(bytes@, 2) + lane_sum(bytes@, 3),
            )
        };
        proof_assert! {
            adler_congruence_to_predicate_certificate(
                a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@ + a_vec@.3@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                    + lane_sum(bytes@, 2) + lane_sum(bytes@, 3),
            ) && adler_congruent(
                a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@ + a_vec@.3@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                    + lane_sum(bytes@, 2) + lane_sum(bytes@, 3),
            )
        };
        proof_assert!(adler_congruent(
            a@,
            initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                + lane_sum(bytes@, 2) + lane_sum(bytes@, 3),
        ));
        proof_assert!(initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
            + lane_sum(bytes@, 2) + lane_sum(bytes@, 3)
            == initial_a@ + crate::adler32_byte_sum(bytes@));
        proof_assert! {
            adler_congruent_reindex(
                a@,
                initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                    + lane_sum(bytes@, 2) + lane_sum(bytes@, 3),
                initial_a@ + crate::adler32_byte_sum(bytes@),
            );
            adler_congruent(a@,
                initial_a@ + crate::adler32_byte_sum(bytes@))
        };
        proof_assert! {
            adler_congruent_add(a_entry@, initial_a@, a_vec@.0@, lane_sum(bytes@, 0));
            adler_congruent_add(a_entry@ + a_vec@.0@, initial_a@ + lane_sum(bytes@, 0), a_vec@.1@, lane_sum(bytes@, 1));
            adler_congruent_add(a_entry@ + a_vec@.0@ + a_vec@.1@, initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1), a_vec@.2@, lane_sum(bytes@, 2));
            adler_congruent_add(a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@, initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1) + lane_sum(bytes@, 2), a_vec@.3@, lane_sum(bytes@, 3));
            adler_congruent_refl(initial_a@ + crate::adler32_byte_sum(bytes@));
            adler_congruent_trans(a@, initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1) + lane_sum(bytes@, 2) + lane_sum(bytes@, 3), initial_a@ + crate::adler32_byte_sum(bytes@));
            true
        };
        proof_assert!(adler_congruent(a@,
            initial_a@ + crate::adler32_byte_sum(bytes@)));
        proof_assert!(four_b_vec@.0@ <= 4 * (MOD@ - 1));
        proof_assert!(four_b_vec@.1@ <= 4 * (MOD@ - 1));
        proof_assert!(four_b_vec@.2@ <= 4 * (MOD@ - 1));
        proof_assert!(four_b_vec@.3@ <= 4 * (MOD@ - 1));
        proof_assert!(b_vec@.0@ == four_b_vec@.0@);
        proof_assert!(b_vec@.0@ <= 458643);
        proof_assert!(b_vec@.1@ <= 458643);
        proof_assert!(b_vec@.2@ <= 458643);
        proof_assert!(b_vec@.3@ <= 458643);
        let b_entry = snapshot! { b };
        #[cfg(any())]
        {
        proof_assert!(b@ + b_vec@.0@ <= u32::MAX@);
        b += b_vec.0[0];
        proof_assert!(b@ + b_vec@.1@ <= u32::MAX@);
        b += b_vec.0[1];
        proof_assert!(b@ + b_vec@.2@ <= u32::MAX@);
        b += b_vec.0[2];
        proof_assert!(b@ + b_vec@.3@ <= u32::MAX@);
        b += b_vec.0[3];
        }
        b = add_lane_totals(b, b_vec);
        proof_assert!(u32x4_prefix_sum(b_vec@, 4)
            == b_vec@.0@ + b_vec@.1@ + b_vec@.2@ + b_vec@.3@);
        proof_assert!(b@ == b_entry@ + u32x4_prefix_sum(b_vec@, 4));
        proof_assert!(b@ == b_entry@
            + b_vec@.0@ + b_vec@.1@ + b_vec@.2@ + b_vec@.3@);
        proof_assert!(adler_congruent(
            b_entry@,
            initial_b@ + bytes@.len() * initial_a@,
        ));
        proof_assert!(adler_congruent(
            b_vec@.0@,
            4 * lane_accumulator(bytes@, 0),
        ));
        /*
        proof_assert!(adler_congruent(
            b_entry@,
            initial_b@ + bytes@.len() * initial_a@,
        ) && adler_congruent(
            b_vec@.0@,
            4 * lane_accumulator(bytes@, 0),
        ));
        proof_assert! {
            adler_congruent_add(
                b_entry@,
                initial_b@ + bytes@.len() * initial_a@,
                b_vec@.0@,
                4 * lane_accumulator(bytes@, 0),
            ); adler_congruent(
                b_entry@ + b_vec@.0@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0),
            )
        };
        proof_assert!(adler_congruent(
            b_entry@ + b_vec@.0@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0),
        ) && adler_congruent(
            b_vec@.1@,
            4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1),
        ));
        proof_assert! {
            adler_congruent_add(
                b_entry@ + b_vec@.0@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0),
                b_vec@.1@,
                4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1),
            ); adler_congruent(
                b_entry@ + b_vec@.0@ + b_vec@.1@,
                (initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0))
                    + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1)),
            )
        };
        proof_assert!((initial_b@ + bytes@.len() * initial_a@
            + 4 * lane_accumulator(bytes@, 0))
            + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
            == initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1));
        proof_assert!(adler_congruent(
            b_entry@ + b_vec@.0@ + b_vec@.1@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1),
        ));
        proof_assert!(adler_congruent(
            b_entry@ + b_vec@.0@ + b_vec@.1@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1),
        ) && adler_congruent(
            b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2),
        ));
        proof_assert! {
            adler_congruent_add(
                b_entry@ + b_vec@.0@ + b_vec@.1@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0)
                    + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1),
                b_vec@.2@,
                4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2),
            ); adler_congruent(
                b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@,
                (initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0)
                    + 4 * lane_accumulator(bytes@, 1)
                    - lane_sum(bytes@, 1))
                    + (4 * lane_accumulator(bytes@, 2)
                        - 2 * lane_sum(bytes@, 2)),
            )
        };
        proof_assert!((initial_b@ + bytes@.len() * initial_a@
            + 4 * lane_accumulator(bytes@, 0)
            + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
            + (4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2))
            == initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + 4 * lane_accumulator(bytes@, 1)
                + 4 * lane_accumulator(bytes@, 2)
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2));
        proof_assert!(adler_congruent(
            b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + 4 * lane_accumulator(bytes@, 1)
                + 4 * lane_accumulator(bytes@, 2)
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2),
        ));
        proof_assert!(adler_congruent(
            b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + 4 * lane_accumulator(bytes@, 1)
                + 4 * lane_accumulator(bytes@, 2)
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2),
        ) && adler_congruent(
            b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3),
        ));
        proof_assert! {
            adler_congruent_add(
                b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0)
                    + 4 * lane_accumulator(bytes@, 1)
                    + 4 * lane_accumulator(bytes@, 2)
                    - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2),
                b_vec@.3@,
                4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3),
            ); adler_congruent(
                b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@ + b_vec@.3@,
                (initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0)
                    + 4 * lane_accumulator(bytes@, 1)
                    + 4 * lane_accumulator(bytes@, 2)
                    - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2))
                    + (4 * lane_accumulator(bytes@, 3)
                        - 3 * lane_sum(bytes@, 3)),
            )
        };
        proof_assert!((initial_b@ + bytes@.len() * initial_a@
            + 4 * lane_accumulator(bytes@, 0)
            + 4 * lane_accumulator(bytes@, 1)
            + 4 * lane_accumulator(bytes@, 2)
            - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2))
            + (4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3))
            == initial_b@ + bytes@.len() * initial_a@
                + 4 * (lane_accumulator(bytes@, 0)
                    + lane_accumulator(bytes@, 1)
                    + lane_accumulator(bytes@, 2)
                    + lane_accumulator(bytes@, 3))
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                - 3 * lane_sum(bytes@, 3));
        proof_assert!(adler_congruent(
            b@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * (lane_accumulator(bytes@, 0)
                    + lane_accumulator(bytes@, 1)
                    + lane_accumulator(bytes@, 2)
                    + lane_accumulator(bytes@, 3))
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                - 3 * lane_sum(bytes@, 3),
        ));
        */
        proof_assert!(adler_congruent(
            b_entry@,
            initial_b@ + bytes@.len() * initial_a@,
        ) && adler_congruent(
            b_vec@.0@,
            4 * lane_accumulator(bytes@, 0),
        ) && adler_congruent(
            b_vec@.1@,
            4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1),
        ) && adler_congruent(
            b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2),
        ) && adler_congruent(
            b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3),
        ));
        proof_assert! {
            adler_congruent_weighted_lanes_certificate(
                b_entry@,
                initial_b@,
                initial_a@,
                bytes@,
                b_vec@.0@,
                b_vec@.1@,
                b_vec@.2@,
                b_vec@.3@,
            )
        };
        proof_assert! {
            adler_congruent_weighted_lanes_certificate(
                b_entry@,
                initial_b@,
                initial_a@,
                bytes@,
                b_vec@.0@,
                b_vec@.1@,
                b_vec@.2@,
                b_vec@.3@,
            ) == adler_congruence_facts(
                b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@ + b_vec@.3@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0)
                    + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
                    + (4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2))
                    + (4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3)),
            )
        };
        proof_assert!(adler_congruence_facts(
            b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@ + b_vec@.3@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
                + (4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2))
                + (4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3)),
        ));
        proof_assert!(adler_congruent(
            b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@ + b_vec@.3@,
            initial_b@ + bytes@.len() * initial_a@
                + 4 * lane_accumulator(bytes@, 0)
                + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
                + (4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2))
                + (4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3)),
        ));
        proof_assert!(initial_b@ + bytes@.len() * initial_a@
            + 4 * lane_accumulator(bytes@, 0)
            + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
            + (4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2))
            + (4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3))
            == initial_b@ + bytes@.len() * initial_a@
                + 4 * (lane_accumulator(bytes@, 0)
                    + lane_accumulator(bytes@, 1)
                    + lane_accumulator(bytes@, 2)
                    + lane_accumulator(bytes@, 3))
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                - 3 * lane_sum(bytes@, 3));
        proof_assert! {
            adler_congruent_reindex(
                b@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * lane_accumulator(bytes@, 0)
                    + (4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1))
                    + (4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2))
                    + (4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3)),
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * (lane_accumulator(bytes@, 0)
                        + lane_accumulator(bytes@, 1)
                        + lane_accumulator(bytes@, 2)
                        + lane_accumulator(bytes@, 3))
                    - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                    - 3 * lane_sum(bytes@, 3),
            );
            adler_congruent(
                b@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * (lane_accumulator(bytes@, 0)
                        + lane_accumulator(bytes@, 1)
                        + lane_accumulator(bytes@, 2)
                        + lane_accumulator(bytes@, 3))
                    - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                    - 3 * lane_sum(bytes@, 3),
            )
        };
        proof_assert!(initial_b@ + bytes@.len() * initial_a@
            + 4 * (lane_accumulator(bytes@, 0) + lane_accumulator(bytes@, 1)
                + lane_accumulator(bytes@, 2) + lane_accumulator(bytes@, 3))
            - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
            - 3 * lane_sum(bytes@, 3)
            == initial_b@ + bytes@.len() * initial_a@
                + crate::adler32_weighted_sum(bytes@));
        proof_assert! {
            adler_congruent_reindex(
                b@,
                initial_b@ + bytes@.len() * initial_a@
                    + 4 * (lane_accumulator(bytes@, 0)
                        + lane_accumulator(bytes@, 1)
                        + lane_accumulator(bytes@, 2)
                        + lane_accumulator(bytes@, 3))
                    - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                    - 3 * lane_sum(bytes@, 3),
                initial_b@ + bytes@.len() * initial_a@
                    + crate::adler32_weighted_sum(bytes@),
            );
            adler_congruent(b@,
                initial_b@ + bytes@.len() * initial_a@
                    + crate::adler32_weighted_sum(bytes@))
        };
        proof_assert! {
            adler_congruent_add(b_entry@, initial_b@ + bytes@.len() * initial_a@, b_vec@.0@, 4 * lane_accumulator(bytes@, 0));
            adler_congruent_add(b_entry@ + b_vec@.0@, initial_b@ + bytes@.len() * initial_a@ + 4 * lane_accumulator(bytes@, 0), b_vec@.1@, 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1));
            adler_congruent_add(b_entry@ + b_vec@.0@ + b_vec@.1@, initial_b@ + bytes@.len() * initial_a@ + 4 * lane_accumulator(bytes@, 0) + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1), b_vec@.2@, 4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2));
            adler_congruent_add(b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@, initial_b@ + bytes@.len() * initial_a@ + 4 * lane_accumulator(bytes@, 0) + 4 * lane_accumulator(bytes@, 1) + 4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2), b_vec@.3@, 4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3));
            adler_congruent_refl(initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@));
            adler_congruent_trans(b@, initial_b@ + bytes@.len() * initial_a@ + 4 * (lane_accumulator(bytes@, 0) + lane_accumulator(bytes@, 1) + lane_accumulator(bytes@, 2) + lane_accumulator(bytes@, 3)) - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2) - 3 * lane_sum(bytes@, 3), initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@));
            true
        };
        proof_assert!(adler_congruent(b@,
            initial_b@ + bytes@.len() * initial_a@
                + crate::adler32_weighted_sum(bytes@)));

        // iterate over the remaining few bytes in serial
        let remainder_a_entry = snapshot! { a };
        let remainder_b_entry = snapshot! { b };
        let remainder_a_target = snapshot! {
            initial_a@ + crate::adler32_byte_sum(bytes@)
        };
        let remainder_b_target = snapshot! {
            initial_b@ + bytes@.len() * initial_a@
                + crate::adler32_weighted_sum(bytes@)
        };
        proof_assert!(remainder_a_entry@ <= 327600);
        proof_assert!(remainder_b_entry@ <= 1900092);
        (a, b) = process_remainder_with_targets(
            a, b, remainder, remainder_a_target, remainder_b_target,
        );
        #[cfg(any())]
        {
        proof_assert!(a@ == remainder_a_entry@
            + crate::adler32_byte_sum(remainder@));
        proof_assert!(adler_congruent(
            remainder_a_entry@,
            initial_a@ + crate::adler32_byte_sum(bytes@),
        ));
        proof_assert!(adler_congruent(
            remainder_b_entry@,
            initial_b@ + bytes@.len() * initial_a@
                + crate::adler32_weighted_sum(bytes@),
        ));
        proof_assert!(b@ == remainder_b_entry@
            + remainder@.len() * remainder_a_entry@
            + crate::adler32_weighted_sum(remainder@));
        }
        proof_assert!(*original_bytes == bytes@.concat(remainder@));
        proof_assert! {
            adler_congruent_finish_a_certificate(
                a@,
                remainder_a_entry@,
                initial_a@,
                bytes@,
                remainder@,
                *original_bytes,
            )
        };
        proof_assert!(adler_congruent(
            a@,
            initial_a@ + crate::adler32_byte_sum(*original_bytes),
        ));
        proof_assert! {
            adler_congruent_finish_b_certificate(
                b@,
                remainder_a_entry@,
                remainder_b_entry@,
                initial_a@,
                initial_b@,
                bytes@,
                remainder@,
                *original_bytes,
            )
        };
        proof_assert!(adler_congruent(b@,
            initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)));
        let final_a = snapshot! { a };
        let final_b = snapshot! { b };
        let final_target_a = snapshot! {
            initial_a@ + crate::adler32_byte_sum(*original_bytes)
        };
        let final_target_b = snapshot! {
            initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)
        };
        (self.a, self.b) = reduce_final_state(
            a, b, final_target_a, final_target_b,
        );
        #[cfg(any())]
        {
        proof_assert! {
            adler_remainder_congruent(final_a@);
            adler_remainder_congruent(final_b@);
            adler_congruent_trans(self.a@, final_a@, initial_a@ + crate::adler32_byte_sum(*original_bytes));
            adler_congruent_trans(self.b@, final_b@, initial_b@ + (*original_bytes).len() * initial_a@ + crate::adler32_weighted_sum(*original_bytes));
            crate::adler32_update_reduced((initial_a@, initial_b@), *original_bytes);
            adler_congruent_reduced(self.a@, (initial_a@ + crate::adler32_byte_sum(*original_bytes)) % 65521);
            adler_congruent_reduced(self.b@, (initial_b@ + (*original_bytes).len() * initial_a@ + crate::adler32_weighted_sum(*original_bytes)) % 65521);
            true
        };
        proof_assert!(self.b@ == final_b@ % 65521);
        proof_assert!(0 <= self.b@ && self.b@ < 65521);
        proof_assert!(0 <= initial_a@ && initial_a@ < 65521);
        proof_assert!(0 <= initial_b@ && initial_b@ < 65521);
        proof_assert! {
            adler_b_target_reduced_certificate(
                initial_a@,
                initial_b@,
                *original_bytes,
            ) && 0 <= (initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)) % 65521
                && (initial_b@ + (*original_bytes).len() * initial_a@
                    + crate::adler32_weighted_sum(*original_bytes)) % 65521 < 65521
        };
        proof_assert! {
            adler_finish_reduced_certificate(
                self.b@,
                final_b@,
                initial_b@ + (*original_bytes).len() * initial_a@
                    + crate::adler32_weighted_sum(*original_bytes),
            ) && self.b@ == (initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)) % 65521
        };
        }
        proof_assert!(self.a@ ==
            (initial_a@ + crate::adler32_byte_sum(*original_bytes)) % 65521);
        proof_assert!(self.b@ ==
            (initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)) % 65521);
    }
}

#[derive(Copy, Clone)]
struct U32X4(pub [u32; 4]);
#[allow(non_snake_case)]
mod u32x4_model {
    use super::U32X4;
    #[allow(unused_imports)]
    use creusot_std::prelude::{logic, pearlite, Invariant, View};

    impl View for U32X4 {
        type ViewTy = (u32, u32, u32, u32);

        #[logic(open)]
        fn view(self) -> Self::ViewTy {
            (self.0[0], self.0[1], self.0[2], self.0[3])
        }
    }

    impl Invariant for U32X4 {
        #[logic(open)]
        fn invariant(self) -> bool {
            pearlite! { self.0@.len() == 4 }
        }
    }

}

impl AddAssign<Self> for U32X4 {
    #[inline]
    #[requires((*self)@.0@ + other@.0@ <= u32::MAX@)]
    #[requires((*self)@.1@ + other@.1@ <= u32::MAX@)]
    #[requires((*self)@.2@ + other@.2@ <= u32::MAX@)]
    #[requires((*self)@.3@ + other@.3@ <= u32::MAX@)]
    #[ensures((^self)@.0@ == (*self)@.0@ + other@.0@)]
    #[ensures((^self)@.1@ == (*self)@.1@ + other@.1@)]
    #[ensures((^self)@.2@ == (*self)@.2@ + other@.2@)]
    #[ensures((^self)@.3@ == (*self)@.3@ + other@.3@)]
    fn add_assign(&mut self, other: Self) {
        // Implement this in a primitive manner to help out the compiler a bit.
        self.0[0] += other.0[0];
        self.0[1] += other.0[1];
        self.0[2] += other.0[2];
        self.0[3] += other.0[3];
    }
}

impl RemAssign<u32> for U32X4 {
    #[inline]
    #[requires(quotient@ > 0)]
    #[ensures((^self)@.0@ == (*self)@.0@ % quotient@)]
    #[ensures((^self)@.1@ == (*self)@.1@ % quotient@)]
    #[ensures((^self)@.2@ == (*self)@.2@ % quotient@)]
    #[ensures((^self)@.3@ == (*self)@.3@ % quotient@)]
    fn rem_assign(&mut self, quotient: u32) {
        self.0[0] %= quotient;
        self.0[1] %= quotient;
        self.0[2] %= quotient;
        self.0[3] %= quotient;
    }
}

impl MulAssign<u32> for U32X4 {
    #[inline]
    #[requires((*self)@.0@ * rhs@ <= u32::MAX@)]
    #[requires((*self)@.1@ * rhs@ <= u32::MAX@)]
    #[requires((*self)@.2@ * rhs@ <= u32::MAX@)]
    #[requires((*self)@.3@ * rhs@ <= u32::MAX@)]
    #[ensures((^self)@.0@ == (*self)@.0@ * rhs@)]
    #[ensures((^self)@.1@ == (*self)@.1@ * rhs@)]
    #[ensures((^self)@.2@ == (*self)@.2@ * rhs@)]
    #[ensures((^self)@.3@ == (*self)@.3@ * rhs@)]
    fn mul_assign(&mut self, rhs: u32) {
        self.0[0] *= rhs;
        self.0[1] *= rhs;
        self.0[2] *= rhs;
        self.0[3] *= rhs;
    }
}
