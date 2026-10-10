#[cfg(feature = "no-panic")]
use no_panic::no_panic;

#[cfg(creusot)]
use creusot_std::prelude::{bitwise_proof, ensures, logic, opaque, proof_assert, requires, snapshot, Int};


#[cfg(creusot)]
#[logic]
#[requires(a >= 0 && b >= 0)]
#[ensures((a + b).pow2() == a.pow2() * b.pow2())]
pub(super) fn power_two_sum(a: Int, b: Int) {}

#[cfg(creusot)]
#[logic]
#[ensures(64.pow2() * 64.pow2() == 128.pow2())]
fn limb_base_square() {
    let _ = power_two_sum(64, 64);
    proof_assert!(64.pow2() * 64.pow2() == 128.pow2());
}

#[cfg(creusot)]
#[logic]
#[requires(n >= 0 && n < 128.pow2())]
#[ensures(n.div_euclid(64.pow2()) < 64.pow2())]
fn euclidean_u128_half_bound(n: Int) {
    let _ = limb_base_square();
    let d = 64.pow2();
    let q = n.div_euclid(d);
    let r = n.rem_euclid(d);
    proof_assert!(0 < d);
    proof_assert!(n == q * d + r);
    proof_assert!(0 <= r && r < d);
    proof_assert!(n < d * d);
    proof_assert!(q * d <= n);
    proof_assert!(q < d);
}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(n >= 0 && d > 0)]
#[requires(n == q * d + r)]
#[requires(0 <= r && r < d)]
#[ensures(result == q && result == n / d)]
pub(super) fn exact_floor_from_split(n: Int, q: Int, r: Int, d: Int) -> Int {
    proof_assert!(q * d <= n);
    proof_assert!(n < (q + 1) * d);
    proof_assert!(q == n / d);
    q
}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(n >= 0)]
#[requires(hi == n / 64.pow2())]
#[requires(lo == n % 64.pow2())]
#[ensures(result == n)]
#[ensures(result == hi * 64.pow2() + lo)]
fn split_u128_at_64(n: Int, hi: Int, lo: Int) -> Int {
    proof_assert!(n == (n / 64.pow2()) * 64.pow2() + n % 64.pow2());
    proof_assert!(n == hi * 64.pow2() + lo);
    n
}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(x * y == x_hi * y_hi * 128.pow2() + x_hi * y_lo * 64.pow2() + x_lo * y_hi * 64.pow2() + x_lo * y_lo)]
#[requires(x_lo * y_hi * 64.pow2() + x_lo * y_lo == high1 * 128.pow2() + m_lo * 64.pow2() + r00)]
#[requires(x_hi * y_lo * 64.pow2() + m_lo * 64.pow2() == high2 * 128.pow2() + r10 * 64.pow2())]
#[requires(final_result == x_hi * y_hi + high1 + high2)]
#[requires(0 <= r00 && r00 < 64.pow2())]
#[requires(0 <= r10 && r10 < 64.pow2())]
#[ensures(result == r10 * 64.pow2() + r00)]
#[ensures(x * y == final_result * 128.pow2() + result)]
#[ensures(0 <= result && result < 128.pow2())]
fn mulhi_limb_reconstruction(
    x: Int,
    y: Int,
    x_hi: Int,
    x_lo: Int,
    y_hi: Int,
    y_lo: Int,
    r00: Int,
    high1: Int,
    m_lo: Int,
    high2: Int,
    r10: Int,
    final_result: Int,
) -> Int {
    let _ = limb_base_square();
    let residual = r10 * 64.pow2() + r00;
    proof_assert!(
        x * y == final_result * 128.pow2() + residual
    );
    proof_assert!(0 <= residual);
    proof_assert!(r00 <= 64.pow2() - 1 && r10 <= 64.pow2() - 1);
    proof_assert!(residual <= (64.pow2() - 1) * 64.pow2() + (64.pow2() - 1));
    proof_assert!((64.pow2() - 1) * 64.pow2() + (64.pow2() - 1) < 128.pow2());
    proof_assert!(residual < 128.pow2());
    residual
}

/// Return the low and high limbs using the same cast and shift as mulhi.
#[inline(always)]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, ensures(result.0@ == n@ % 64.pow2()))]
#[cfg_attr(creusot, ensures(result.1@ == n@ / 64.pow2()))]
#[cfg_attr(creusot, ensures(result.0@ + result.1@ * 64.pow2() == n@))]
#[cfg_attr(creusot, ensures(0 <= result.0@ && result.0@ < 64.pow2()))]
#[cfg_attr(creusot, ensures(0 <= result.1@ && result.1@ < 64.pow2()))]
pub(crate) fn u128_halves(n: u128) -> (u64, u64) {
    (n as u64, (n >> 64) as u64)
}

#[inline(always)]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, requires(carry@ < 64.pow2()))]
#[cfg_attr(creusot, ensures(result@ == x_lo@ * y_hi@ + carry@))]
#[cfg_attr(creusot, ensures(result@ < 128.pow2()))]
fn cross_product_plus_carry(x_lo: u64, y_hi: u64, carry: u128) -> u128 {
    let x_wide = u128::from(x_lo);
    let y_wide = u128::from(y_hi);
    #[cfg(creusot)]
    proof_assert!(x_wide@ == x_lo@);
    #[cfg(creusot)]
    proof_assert!(y_wide@ == y_hi@);
    #[cfg(creusot)]
    proof_assert!(0 <= x_lo@ && x_lo@ < 64.pow2());
    #[cfg(creusot)]
    proof_assert!(0 <= y_hi@ && y_hi@ < 64.pow2());
    #[cfg(creusot)]
    proof_assert!(0 <= carry@ && carry@ < 64.pow2());
    #[cfg(creusot)]
    proof_assert! {
        let _ = limb_product_plus_limb_fits(x_lo@, y_hi@, carry@);
        x_lo@ * y_hi@ + carry@ < 128.pow2()
    };
    #[cfg(creusot)]
    proof_assert!(x_wide@ * y_wide@ == x_lo@ * y_hi@);
    let result = x_wide * y_wide + carry;
    #[cfg(creusot)]
    proof_assert!(result@ == x_wide@ * y_wide@ + carry@);
    #[cfg(creusot)]
    proof_assert!(result@ == x_lo@ * y_hi@ + carry@);
    result
}

/// Accumulate the three high limbs using the exact widened operations in mulhi.
#[inline(always)]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, requires(x_hi@ * y_hi@ + high1@ + high2@ < 128.pow2()))]
#[cfg_attr(creusot, ensures(result@ == x_hi@ * y_hi@ + high1@ + high2@))]
fn final_accumulation(x_hi: u64, y_hi: u64, high1: u64, high2: u64) -> u128 {
    let x_wide = u128::from(x_hi);
    let y_wide = u128::from(y_hi);
    let high1_wide = u128::from(high1);
    let high2_wide = u128::from(high2);
    #[cfg(creusot)]
    proof_assert!(x_wide@ == x_hi@);
    #[cfg(creusot)]
    proof_assert!(y_wide@ == y_hi@);
    #[cfg(creusot)]
    proof_assert!(high1_wide@ == high1@);
    #[cfg(creusot)]
    proof_assert!(high2_wide@ == high2@);
    let result = x_wide * y_wide + high1_wide + high2_wide;
    #[cfg(creusot)]
    proof_assert!(result@ == x_wide@ * y_wide@ + high1_wide@ + high2_wide@);
    #[cfg(creusot)]
    proof_assert!(result@ == x_hi@ * y_hi@ + high1@ + high2@);
    result
}

#[cfg(creusot)]
#[logic]
#[requires(0 <= a && a < 64.pow2())]
#[requires(0 <= b && b < 64.pow2())]
#[ensures(a * b < 128.pow2())]
#[ensures(a * b <= 18_446_744_073_709_551_615 * 18_446_744_073_709_551_615)]
fn limb_product_fits(a: Int, b: Int) {
    let top: Int = 18_446_744_073_709_551_615;
    proof_assert!(0 <= top - a && 0 <= top - b);
    proof_assert!(0 <= b && 0 <= top);
    proof_assert!((top - a) * b >= 0);
    proof_assert!(a * b <= top * b);
    proof_assert!(top * (top - b) >= 0);
    proof_assert!(top * b <= top * top);
    proof_assert!(a * b <= top * top);
    proof_assert!(top * top < 128.pow2());
}

#[cfg(creusot)]
#[logic]
#[requires(0 <= x && x < 64.pow2())]
#[ensures(x <= 18_446_744_073_709_551_615)]
fn limb_at_most_max(x: Int) {
    proof_assert!(18_446_744_073_709_551_615 + 1 == 64.pow2());
    proof_assert!(x <= 18_446_744_073_709_551_615);
}

#[cfg(creusot)]
#[logic]
#[requires(n >= 0 && n < 128.pow2())]
#[ensures(n / 64.pow2() < 64.pow2())]
#[ensures(n / 64.pow2() >= 0)]
fn quotient_half_fits(n: Int) {
    let _ = limb_base_square();
    let d = 64.pow2();
    let q = n / d;
    let r = n % d;
    proof_assert!(0 < d);
    proof_assert!(n == q * d + r);
    proof_assert!(0 <= r && r < d);
    proof_assert!(n < d * d);
    proof_assert!(q * d <= n);
    proof_assert!(q >= 0);
    proof_assert!(q < d);
}

#[cfg(creusot)]
#[logic]
#[requires(0 <= x_lo && x_lo < 64.pow2())]
#[requires(0 <= y_lo && y_lo < 64.pow2())]
#[requires(0 <= y_hi && y_hi < 64.pow2())]
#[ensures(x_lo * y_hi + (x_lo * y_lo) / 64.pow2() < 128.pow2())]
fn cross_product_plus_carry_fits(x_lo: Int, y_lo: Int, y_hi: Int) {
    let _ = limb_base_square();
    let _ = limb_product_fits(x_lo, y_lo);
    proof_assert!(0 <= x_lo * y_lo && x_lo * y_lo < 128.pow2());
    let _ = quotient_half_fits(x_lo * y_lo);
    let carry = (x_lo * y_lo) / 64.pow2();
    proof_assert!(0 <= carry && carry < 64.pow2());
    let _ = limb_at_most_max(carry);
    let _ = limb_product_fits(x_lo, y_hi);
    let top: Int = 18_446_744_073_709_551_615;
    proof_assert!(x_lo * y_hi <= top * top);
    proof_assert!(x_lo * y_hi + carry <= top * top + top);
    proof_assert!(top * top + top < 128.pow2());
}

#[cfg(creusot)]
#[logic]
#[requires(0 <= a && a < 64.pow2())]
#[requires(0 <= b && b < 64.pow2())]
#[requires(0 <= c && c < 64.pow2())]
#[ensures(a * b < 128.pow2())]
#[ensures(a * b + c < 128.pow2())]
fn limb_product_plus_limb_fits(a: Int, b: Int, c: Int) {
    let top: Int = 18_446_744_073_709_551_615;
    let _ = limb_product_fits(a, b);
    proof_assert!(c <= top);
    proof_assert!(a * b + c <= top * top + c);
    proof_assert!(a * b + c <= top * top + top);
    proof_assert!(top * top + top < 128.pow2());
}

#[cfg(creusot)]
#[logic]
#[requires(0 <= a && a < 64.pow2())]
#[requires(0 <= b && b < 64.pow2())]
#[requires(0 <= c && c < 64.pow2())]
#[requires(0 <= d && d < 64.pow2())]
#[ensures(a * b < 128.pow2())]
#[ensures(a * b + c < 128.pow2())]
#[ensures(a * b + c + d < 128.pow2())]
fn limb_product_plus_two_limbs_fits(a: Int, b: Int, c: Int, d: Int) {
    let top: Int = 18_446_744_073_709_551_615;
    let _ = limb_product_fits(a, b);
    proof_assert!(c + d <= 2 * top);
    proof_assert!(a * b + c + d <= top * top + c + d);
    proof_assert!(a * b + c + d <= top * top + 2 * top);
    proof_assert!(top * top + 2 * top < 128.pow2());
}

// Reusable bridge for substituting exact limb decompositions inside products.
// Retained as an independently proved arithmetic helper; mulhi_core now uses
// the joint program lemma below to compose its local split facts.
#[cfg(creusot)]
#[logic]
#[requires(x == a)]
#[requires(y == b)]
#[ensures(result == x * y)]
#[ensures(result == a * b)]
fn mulhi_product_congruence(x: Int, y: Int, a: Int, b: Int) -> Int {
    proof_assert!(x * y == a * b);
    x * y
}

// Proved program lemma for substituting one exact limb split under an
// arbitrary multiplication factor. The joint helper below composes it with
// the symmetric x-split lemma.
#[cfg(creusot)]
#[requires(y@ == y_hi@ * 64.pow2() + y_lo@)]
#[ensures(factor@ * y@ == factor@ * (y_hi@ * 64.pow2() + y_lo@))]
fn mulhi_factor_y_split_product(factor: u128, y: u128, y_hi: u64, y_lo: u64) {
    proof_assert!(factor@ * y@ == factor@ * (y_hi@ * 64.pow2() + y_lo@));
}

// Symmetric proved program lemma for substituting the left operand's exact
// limb decomposition under an arbitrary multiplication factor.
#[cfg(creusot)]
#[requires(x@ == x_hi@ * 64.pow2() + x_lo@)]
#[ensures(x@ * factor@ == (x_hi@ * 64.pow2() + x_lo@) * factor@)]
fn mulhi_x_split_product(x: u128, factor: u128, x_hi: u64, x_lo: u64) {
    proof_assert!(x@ * factor@ == (x_hi@ * 64.pow2() + x_lo@) * factor@);
}

// Compose the untrusted one-sided split lemmas into the joint product fact
// consumed by `mulhi_core`.
#[cfg(creusot)]
#[requires(x@ == x_hi@ * 64.pow2() + x_lo@)]
#[requires(y@ == y_hi@ * 64.pow2() + y_lo@)]
#[ensures(x@ * y@ == (x_hi@ * 64.pow2() + x_lo@) * (y_hi@ * 64.pow2() + y_lo@))]
fn mulhi_product_split_from_limbs(
    x: u128,
    y: u128,
    x_hi: u64,
    x_lo: u64,
    y_hi: u64,
    y_lo: u64,
) {
    let _ = mulhi_x_split_product(x, y, x_hi, x_lo);
    let _ = mulhi_factor_y_split_product(x, y, y_hi, y_lo);
    proof_assert!(x@ * y@ == (x_hi@ * 64.pow2() + x_lo@) * (y_hi@ * 64.pow2() + y_lo@));
}

// Independently checked four-limb ring expansion used as a small algebraic
// building block while composing the exact mulhi result equation.
#[cfg(creusot)]
#[logic(opaque)]
#[ensures(result == (x_hi * 64.pow2() + x_lo) * (y_hi * 64.pow2() + y_lo))]
#[ensures(result == x_hi * y_hi * 128.pow2()
    + x_hi * y_lo * 64.pow2()
    + x_lo * y_hi * 64.pow2()
    + x_lo * y_lo)]
fn mulhi_product_expansion(x_hi: Int, x_lo: Int, y_hi: Int, y_lo: Int) -> Int {
    let _ = limb_base_square();
    let product = (x_hi * 64.pow2() + x_lo) * (y_hi * 64.pow2() + y_lo);
    proof_assert!(product
        == x_hi * y_hi * 128.pow2()
            + x_hi * y_lo * 64.pow2()
            + x_lo * y_hi * 64.pow2()
            + x_lo * y_lo);
    product
}

/// Multiply unsigned 128 bit integers, return upper 128 bits of the result
#[inline(always)]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, ensures(result@ == x@ * y@ / 128.pow2()))]
fn mulhi_core(x: u128, y: u128) -> u128 {
    let (x_lo, x_hi) = u128_halves(x);
    let (y_lo, y_hi) = u128_halves(y);

    #[cfg(creusot)]
    proof_assert! {
        let _ = limb_base_square();
        64.pow2() * 64.pow2() == 128.pow2()
    };

    #[cfg(creusot)]
    let x_hi_shifted = shift_by_64_actual(x);
    #[cfg(creusot)]
    proof_assert!(x_hi_shifted == x >> 64i32);
    #[cfg(creusot)]
    proof_assert!(x_hi_shifted@ == x@.div_euclid(64.pow2()));
    #[cfg(creusot)]
    proof_assert!(x@ >= 0);
    #[cfg(creusot)]
    proof_assert!(x@ < 128.pow2());
    #[cfg(creusot)]
    proof_assert! {
        let _ = euclidean_u128_half_bound(x@);
        x_hi_shifted@ < 64.pow2()
    };
    // Keep the proof connection in the integer model; the cast's logical
    // bitvector embedding is partial even though its Rust implementation is
    // the modulo cast handled by the executable cast model.
    #[cfg(creusot)]
    proof_assert!(0 < 64.pow2());
    #[cfg(creusot)]
    proof_assert!(computer_division_matches_shift64(x@)
        && x@ / 64.pow2() == x@.div_euclid(64.pow2()));
    #[cfg(creusot)]
    proof_assert!(x_hi@ == x@ / 64.pow2());
    #[cfg(creusot)]
    let y_hi_shifted = shift_by_64_actual(y);
    #[cfg(creusot)]
    proof_assert!(y_hi_shifted == y >> 64i32);
    #[cfg(creusot)]
    proof_assert!(y_hi_shifted@ == y@.div_euclid(64.pow2()));
    #[cfg(creusot)]
    proof_assert!(y@ >= 0);
    #[cfg(creusot)]
    proof_assert!(y@ < 128.pow2());
    #[cfg(creusot)]
    proof_assert! {
        let _ = euclidean_u128_half_bound(y@);
        y_hi_shifted@ < 64.pow2()
    };
    #[cfg(creusot)]
    proof_assert!(0 < 64.pow2());
    #[cfg(creusot)]
    proof_assert!(computer_division_matches_shift64(y@)
        && y@ / 64.pow2() == y@.div_euclid(64.pow2()));
    #[cfg(creusot)]
    proof_assert!(y_hi@ == y@ / 64.pow2());

    #[cfg(creusot)]
    proof_assert!(x_hi@ == x@ / 64.pow2() && x_lo@ == x@ % 64.pow2());
    #[cfg(creusot)]
    proof_assert!(y_hi@ == y@ / 64.pow2() && y_lo@ == y@ % 64.pow2());

    // handle possibility of overflow
    #[cfg(creusot)]
    proof_assert! {
        let _ = limb_product_fits(x_lo@, y_lo@);
        x_lo@ * y_lo@ < 128.pow2()
    };
    let carry = (u128::from(x_lo) * u128::from(y_lo)) >> 64;

    #[cfg(creusot)]
    let carry_product = u128::from(x_lo) * u128::from(y_lo);
    #[cfg(creusot)]
    let carry_shifted = shift_by_64_actual(carry_product);
    #[cfg(creusot)]
    proof_assert!(carry_product@ == x_lo@ * y_lo@);
    #[cfg(creusot)]
    proof_assert!(carry_product@ < 128.pow2());
    #[cfg(creusot)]
    proof_assert!(carry_shifted == carry_product >> 64i32);
    #[cfg(creusot)]
    proof_assert!(carry_shifted@ == carry_product@.div_euclid(64.pow2()));
    #[cfg(creusot)]
    proof_assert! {
        let _ = euclidean_u128_half_bound(carry_product@);
        carry_shifted@ < 64.pow2()
    };
    #[cfg(creusot)]
    proof_assert!(carry == carry_shifted);
    #[cfg(creusot)]
    proof_assert!(carry@ < 64.pow2());
    #[cfg(creusot)]
    proof_assert!(carry_product@ >= 0);
    #[cfg(creusot)]
    proof_assert!(0 < 64.pow2());
    #[cfg(creusot)]
    proof_assert!(computer_division_matches_shift64(carry_product@)
        && carry_product@ / 64.pow2() == carry_product@.div_euclid(64.pow2()));
    #[cfg(creusot)]
    proof_assert!(carry@ == carry_product@ / 64.pow2());
    let m = cross_product_plus_carry(x_lo, y_hi, carry);
    #[cfg(creusot)]
    proof_assert!(m@ == x_lo@ * y_hi@ + carry@);

    let (m_lo, high1) = u128_halves(m);

    #[cfg(creusot)]
    proof_assert! {
        let _ = limb_product_plus_limb_fits(x_hi@, y_lo@, m_lo@);
        x_hi@ * y_lo@ + m_lo@ < 128.pow2()
    };
    let high2_product = cross_product_plus_carry(x_hi, y_lo, u128::from(m_lo));
    let (high2_lo, high2) = u128_halves(high2_product);
    #[cfg(creusot)]
    proof_assert!(high2_product@ == x_hi@ * y_lo@ + m_lo@);

    #[cfg(creusot)]
    proof_assert! {
        let _ = limb_product_plus_two_limbs_fits(x_hi@, y_hi@, high1@, high2@);
        x_hi@ * y_hi@ + high1@ + high2@ < 128.pow2()
    };
    let result = final_accumulation(x_hi, y_hi, high1, high2);

    #[cfg(creusot)]
    proof_assert!(split_u128_at_64(x@, x_hi@, x_lo@) == x@);
    #[cfg(creusot)]
    proof_assert!(split_u128_at_64(y@, y_hi@, y_lo@) == y@);
    #[cfg(creusot)]
    proof_assert!(x_lo@ * y_lo@ == carry@ * 64.pow2() + (x_lo@ * y_lo@) % 64.pow2());
    #[cfg(creusot)]
    proof_assert!(x_lo@ * y_hi@ + carry@ == high1@ * 64.pow2() + m_lo@);
    #[cfg(creusot)]
    proof_assert!(x_hi@ * y_lo@ + m_lo@ == high2@ * 64.pow2() + high2_lo@);
    #[cfg(creusot)]
    proof_assert! {
        let r00 = (x_lo@ * y_lo@) % 64.pow2();
        0 <= r00 && r00 < 64.pow2()
    };
    #[cfg(creusot)]
    proof_assert!(0 <= high2_lo@ && high2_lo@ < 64.pow2());

    #[cfg(creusot)]
    proof_assert!(x@ == x_hi@ * 64.pow2() + x_lo@);
    #[cfg(creusot)]
    proof_assert!(y@ == y_hi@ * 64.pow2() + y_lo@);
    #[cfg(creusot)]
    proof_assert!(carry@ * 64.pow2() + (x_lo@ * y_lo@) % 64.pow2()
        == x_lo@ * y_lo@);
    #[cfg(creusot)]
    proof_assert!(m@ == x_lo@ * y_hi@ + carry@);
    #[cfg(creusot)]
    proof_assert!(high1@ * 64.pow2() + m_lo@ == m@);
    #[cfg(creusot)]
    proof_assert!(high2_product@ == x_hi@ * y_lo@ + m_lo@);
    #[cfg(creusot)]
    proof_assert!(high2@ * 64.pow2() + high2_lo@ == high2_product@);
    #[cfg(creusot)]
    proof_assert!(result@ == x_hi@ * y_hi@ + high1@ + high2@);

    #[cfg(creusot)]
    let _ = mulhi_product_split_from_limbs(x, y, x_hi, x_lo, y_hi, y_lo);
    #[cfg(creusot)]
    proof_assert!(x@ * y@ == (x_hi@ * 64.pow2() + x_lo@) * (y_hi@ * 64.pow2() + y_lo@));
    #[cfg(creusot)]
    let expanded_product = snapshot!(mulhi_product_expansion(x_hi@, x_lo@, y_hi@, y_lo@));
    #[cfg(creusot)]
    proof_assert!(*expanded_product
        == (x_hi@ * 64.pow2() + x_lo@) * (y_hi@ * 64.pow2() + y_lo@));
    #[cfg(creusot)]
    proof_assert!(*expanded_product == x_hi@ * y_hi@ * 128.pow2()
        + x_hi@ * y_lo@ * 64.pow2()
        + x_lo@ * y_hi@ * 64.pow2()
        + x_lo@ * y_lo@);
    #[cfg(creusot)]
    proof_assert!(x@ * y@ == x_hi@ * y_hi@ * 128.pow2()
        + x_hi@ * y_lo@ * 64.pow2()
        + x_lo@ * y_hi@ * 64.pow2()
        + x_lo@ * y_lo@);
    #[cfg(creusot)]
    proof_assert!(x_lo@ * y_hi@ * 64.pow2() + x_lo@ * y_lo@
        == high1@ * 128.pow2() + m_lo@ * 64.pow2()
            + (x_lo@ * y_lo@) % 64.pow2());
    #[cfg(creusot)]
    proof_assert!(high2@ * 64.pow2() + high2_lo@ == x_hi@ * y_lo@ + m_lo@);
    #[cfg(creusot)]
    proof_assert!(64.pow2() * 64.pow2() == 128.pow2());
    #[cfg(creusot)]
    proof_assert!((x_hi@ * y_lo@ + m_lo@) * 64.pow2()
        == (high2@ * 64.pow2() + high2_lo@) * 64.pow2());
    #[cfg(creusot)]
    proof_assert!(x_hi@ * y_lo@ * 64.pow2() + m_lo@ * 64.pow2()
        == high2@ * 64.pow2() * 64.pow2() + high2_lo@ * 64.pow2());
    #[cfg(creusot)]
    proof_assert!(x_hi@ * y_lo@ * 64.pow2() + m_lo@ * 64.pow2()
        == high2@ * 128.pow2() + high2_lo@ * 64.pow2());
    #[cfg(creusot)]
    proof_assert!(x@ * y@ == (x_hi@ * y_hi@ + high1@ + high2@)
        * 128.pow2() + high2_lo@ * 64.pow2() + (x_lo@ * y_lo@) % 64.pow2());
    #[cfg(creusot)]
    proof_assert!(high2_lo@ * 64.pow2() + (x_lo@ * y_lo@) % 64.pow2()
        < 128.pow2());
    #[cfg(creusot)]
    proof_assert! {
        let _ = exact_floor_from_split(
            x@ * y@,
            result@,
            high2_lo@ * 64.pow2() + (x_lo@ * y_lo@) % 64.pow2(),
            128.pow2(),
        );
        result@ == x@ * y@ / 128.pow2()
    };
    result
}

/// Return the high half of the full-width `u128` product.
///
/// `mulhi_core` proves this result equation together with its limb-operation
/// range obligations.
#[inline]
#[cfg_attr(all(feature = "no-panic", not(creusot)), no_panic)]
#[cfg_attr(creusot, ensures(result@ == x@ * y@ / 128.pow2()))]
pub(crate) fn mulhi(x: u128, y: u128) -> u128 {
    mulhi_core(x, y)
}

#[cfg(creusot)]
#[bitwise_proof]
#[ensures(result@ == n@.div_euclid(64.pow2()))]
#[ensures(result == n >> 64i32)]
pub(crate) fn shift_by_64_actual(n: u128) -> u128 {
    n >> 64
}

#[cfg(creusot)]
#[logic]
#[requires(x <= y && z >= 0)]
#[ensures(x * z <= y * z)]
fn positive_multiplication_monotone(x: Int, y: Int, z: Int) {
    proof_assert!((y - x) * z >= 0);
}

#[cfg(creusot)]
#[logic]
#[requires(n >= 0 && d > 0)]
#[requires(n == q * d + r)]
#[requires(n == qe * d + re)]
#[requires(0 <= r && r < d)]
#[requires(0 <= re && re < d)]
#[ensures(q == qe)]
fn quotient_equal_from_remainders(n: Int, d: Int, q: Int, r: Int, qe: Int, re: Int) {
    if q < qe {
        proof_assert!(q + 1 <= qe);
        let _ = positive_multiplication_monotone(q + 1, qe, d);
        proof_assert!((q + 1) * d <= qe * d);
        proof_assert!(n < (q + 1) * d);
        proof_assert!(qe * d <= n);
        proof_assert!(n < n);
    }
    if qe < q {
        proof_assert!(qe + 1 <= q);
        let _ = positive_multiplication_monotone(qe + 1, q, d);
        proof_assert!((qe + 1) * d <= q * d);
        proof_assert!(n < (qe + 1) * d);
        proof_assert!(q * d <= n);
        proof_assert!(n < n);
    }
}

#[cfg(creusot)]
#[logic]
#[opaque]
#[requires(n >= 0 && d > 0)]
#[ensures(n / d == n.div_euclid(d))]
#[ensures(n % d == n.rem_euclid(d))]
fn computer_division_euclidean_compat(n: Int, d: Int) {
    let q = n / d;
    let r = n % d;
    let qe = n.div_euclid(d);
    let re = n.rem_euclid(d);
    proof_assert!(n == d * q + r);
    proof_assert!(0 <= r && r < d);
    proof_assert!(n == qe * d + re);
    proof_assert!(0 <= re && re < d);
    let _ = quotient_equal_from_remainders(n, d, q, r, qe, re);
    proof_assert!(q == qe);
    proof_assert!(r == re);
    proof_assert!(n / d == q);
    proof_assert!(n % d == r);
    proof_assert!(n.rem_euclid(d) == re);
}

#[cfg(creusot)]
#[logic]
#[requires(n >= 0)]
#[ensures(result && result == (n / 64.pow2() == n.div_euclid(64.pow2())))]
#[ensures(n % 64.pow2() == n.rem_euclid(64.pow2()))]
pub(super) fn computer_division_matches_shift64(n: Int) -> bool {
    let _ = computer_division_euclidean_compat(n, 64.pow2());
    proof_assert!(n / 64.pow2() == n.div_euclid(64.pow2()));
    proof_assert!(n % 64.pow2() == n.rem_euclid(64.pow2()));
    n / 64.pow2() == n.div_euclid(64.pow2())
}

#[cfg(creusot)]
#[logic]
#[requires(n >= 0)]
#[ensures(result && result == (n / 51.pow2() == n.div_euclid(51.pow2())))]
#[ensures(n % 51.pow2() == n.rem_euclid(51.pow2()))]
pub(super) fn computer_division_matches_shift51(n: Int) -> bool {
    let _ = computer_division_euclidean_compat(n, 51.pow2());
    proof_assert!(n / 51.pow2() == n.div_euclid(51.pow2()));
    proof_assert!(n % 51.pow2() == n.rem_euclid(51.pow2()));
    n / 51.pow2() == n.div_euclid(51.pow2())
}
