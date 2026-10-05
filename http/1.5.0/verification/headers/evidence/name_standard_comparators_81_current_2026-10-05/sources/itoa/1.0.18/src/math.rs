use creusot_std::prelude::{logic, proof_assert, requires, ensures, Int};

#[logic]
#[requires(a >= 0 && b >= 0)]
#[ensures((a + b).pow2() == a.pow2() * b.pow2())]
fn power_two_sum(a: Int, b: Int) {}

#[logic]
#[ensures(179.pow2() == 128.pow2() * 51.pow2())]
fn pow2_179_decomposition() {
    let _ = power_two_sum(128, 51);
    proof_assert!(179 == 128 + 51);
    proof_assert!(179.pow2() == 128.pow2() * 51.pow2());
}

#[logic]
#[ensures(128.pow2() == 64.pow2() * 64.pow2())]
fn pow2_128_decomposition() {
    let _ = power_two_sum(64, 64);
    proof_assert!(128 == 64 + 64);
    proof_assert!(128.pow2() == 64.pow2() * 64.pow2());
}

#[logic]
#[ensures(10_000_000_000_000_000
    * 76_624_777_043_294_442_917_917_351_357_515_459_181
    == 179.pow2() + 630_438_908_198_912)]
fn magic_multiplier_identity() {
    let _ = pow2_179_decomposition();
    let _ = pow2_128_decomposition();
    proof_assert!(51.pow2() == 2_251_799_813_685_248);
    proof_assert!(64.pow2() == 18_446_744_073_709_551_616);
    proof_assert!(10_000_000_000_000_000
        * 76_624_777_043_294_442_917_917_351_357_515_459_181
        == 179.pow2() + 630_438_908_198_912);
}

/// Exact Euclidean floor division for the Granlund–Montgomery multiplier.
#[logic]
#[requires(0 <= n && n < 128.pow2())]
#[ensures(n * 76_624_777_043_294_442_917_917_351_357_515_459_181
    / 179.pow2()
    == n / 10_000_000_000_000_000)]
fn magic_division_floor(n: Int) {
    let q = n / 10_000_000_000_000_000;
    let r = n % 10_000_000_000_000_000;
    proof_assert!(n == q * 10_000_000_000_000_000 + r);
    proof_assert!(0 <= q && 0 <= r && r < 10_000_000_000_000_000);

    let _ = magic_multiplier_identity();
    proof_assert!(0 < 630_438_908_198_912 && 630_438_908_198_912 < 50.pow2());
    proof_assert!(128.pow2() * 50.pow2() < 179.pow2());
    proof_assert!(n * 630_438_908_198_912 < 179.pow2());

    proof_assert!(10_000_000_000_000_000
        * (q * 630_438_908_198_912
            + r * 76_624_777_043_294_442_917_917_351_357_515_459_181)
        == n * 630_438_908_198_912 + r * 179.pow2());
    proof_assert!(n * 630_438_908_198_912 + r * 179.pow2()
        < 10_000_000_000_000_000 * 179.pow2());
    proof_assert!(q * 630_438_908_198_912
        + r * 76_624_777_043_294_442_917_917_351_357_515_459_181
        < 179.pow2());
    proof_assert!(q * 179.pow2()
        <= n * 76_624_777_043_294_442_917_917_351_357_515_459_181);
    proof_assert!(n * 76_624_777_043_294_442_917_917_351_357_515_459_181
        < (q + 1) * 179.pow2());
    proof_assert!(n * 76_624_777_043_294_442_917_917_351_357_515_459_181
        / 179.pow2()
        == q);
}

#[logic]
#[requires(n >= 0 && d > 0)]
#[requires(n == q * d + r && 0 <= r && r < d)]
#[ensures(q == n / d)]
pub(crate) fn exact_floor_from_split(n: Int, q: Int, r: Int, d: Int) {
    proof_assert!(q * d <= n);
    proof_assert!(n < (q + 1) * d);
    proof_assert!(q == n / d);
}

#[logic(opaque)]
#[requires(n >= 0 && d > 0)]
#[ensures(n == (n / d) * d + n % d)]
#[ensures(0 <= n % d && n % d < d)]
pub(crate) fn euclidean_div_mod(n: Int, d: Int) {
    let q = n / d;
    let r = n % d;
    proof_assert!(n == q * d + r);
    proof_assert!(0 <= r && r < d);
}

#[logic]
#[requires(n >= 0 && p > 0 && s > 0)]
#[ensures((n / p) / s == n / (p * s))]
fn nested_floor(n: Int, p: Int, s: Int) {
    let h = n / p;
    let t = n % p;
    let q = h / s;
    let r = h % s;
    proof_assert!(n == h * p + t);
    proof_assert!(0 <= t && t < p);
    proof_assert!(h == q * s + r);
    proof_assert!(0 <= r && r < s);
    proof_assert!(h >= 0);
    proof_assert!(q >= 0);
    proof_assert!(r * p + t < p * s);
    proof_assert!(n == q * (p * s) + (r * p + t));
    let _ = exact_floor_from_split(n, q, r * p + t, p * s);
}

/// Pure-integer bridge from the exact high product and post-shift to `/ 10^16`.
#[logic(opaque)]
#[requires(0 <= n && n < 128.pow2())]
#[requires(high == n * 76_624_777_043_294_442_917_917_351_357_515_459_181
    / 128.pow2())]
#[requires(q == high / 51.pow2())]
#[ensures(result == q && result == n / 10_000_000_000_000_000)]
pub(crate) fn magic_shift_floor(n: Int, high: Int, q: Int) -> Int {
    let _ = pow2_179_decomposition();
    let _ = nested_floor(
        n * 76_624_777_043_294_442_917_917_351_357_515_459_181,
        128.pow2(),
        51.pow2(),
    );
    let _ = magic_division_floor(n);
    proof_assert!(q == n / 10_000_000_000_000_000);
    q
}
