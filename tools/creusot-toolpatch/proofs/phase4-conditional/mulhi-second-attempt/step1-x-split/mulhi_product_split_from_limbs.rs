// Standalone, untrusted program lemma for substituting one exact limb split
// under an arbitrary multiplication factor. Its caller-side use in mulhi
// remains open in the generated nonlinear arithmetic VC.
#[cfg(creusot)]
#[requires(y@ == y_hi@ * 64.pow2() + y_lo@)]
#[ensures(factor@ * y@ == factor@ * (y_hi@ * 64.pow2() + y_lo@))]
fn mulhi_factor_y_split_product(factor: u128, y: u128, y_hi: u64, y_lo: u64) {
    proof_assert!(factor@ * y@ == factor@ * (y_hi@ * 64.pow2() + y_lo@));
}

// Symmetric untrusted program lemma for substituting the left operand's
// exact limb decomposition under an arbitrary multiplication factor.
#[cfg(creusot)]
#[requires(x@ == x_hi@ * 64.pow2() + x_lo@)]
#[ensures(x@ * factor@ == (x_hi@ * 64.pow2() + x_lo@) * factor@)]
fn mulhi_x_split_product(x: u128, factor: u128, x_hi: u64, x_lo: u64) {
    proof_assert!(x@ * factor@ == (x_hi@ * 64.pow2() + x_lo@) * factor@);
}

// Small representative caller for checking that the two one-sided split
// contracts compose into the exact product of both limb decompositions.
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
