#[cfg(creusot)]
use creusot_std::prelude::{bitwise_proof, check, ensures, logic, proof_assert, requires, Int};

/// Arithmetic bound for the reciprocal-multiply quotient used by `divmod100`.
#[cfg(creusot)]
#[logic]
#[requires(0 <= n && n < 10_000)]
#[ensures((n * 5_243) / 524_288 == n / 100)]
fn magic_quotient(n: Int) {
    let q = n / 100;
    let r = n % 100;
    proof_assert!(0 <= q && q < 100);
    proof_assert!(0 <= r && r < 100);
    proof_assert!(n == 100 * q + r);
    proof_assert!(n * 5_243 == q * 524_288 + 12 * q + 5_243 * r);
    proof_assert!(0 <= 12 * q + 5_243 * r);
    proof_assert!(12 * q + 5_243 * r <= 520_245);
    proof_assert!(12 * q + 5_243 * r < 524_288);
    proof_assert!(q * 524_288 <= n * 5_243);
    proof_assert!(n * 5_243 < (q + 1) * 524_288);
    proof_assert!((n * 5_243) / 524_288 == q);
}

// Returns {value / 100, value % 100} correct for values of up to 4 digits.
#[cfg_attr(creusot, check(terminates))]
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, requires(value@ < 10_000))]
#[cfg_attr(creusot, ensures(result.0@ == value@ / 100))]
#[cfg_attr(creusot, ensures(result.1@ == value@ % 100))]
#[cfg_attr(creusot, ensures(result.1@ < 100))]
#[cfg_attr(creusot, ensures(result.0@ * 100 + result.1@ == value@))]
pub(crate) fn divmod100(value: u32) -> (u32, u32) {
    debug_assert!(value < 10_000);
    const EXP: u32 = 19; // 19 is faster or equal to 12 even for 3 digits.
    const SIG: u32 = (1 << EXP) / 100 + 1;
    #[cfg(creusot)]
    {
        proof_assert!(SIG@ == 5_243);
        proof_assert!(value@ * SIG@ <= 52_424_757);
    }
    let div = (value * SIG) >> EXP; // value / 100
    #[cfg(creusot)]
    {
        proof_assert!((1u32 << EXP)@ == 524_288);
        proof_assert!(div@ == (value * SIG)@ / 524_288);
        proof_assert! {
            let _ = magic_quotient(value@);
            div@ == value@ / 100
        };
        proof_assert!(div@ < 100);
        proof_assert!(div@ * 100 <= value@);
        proof_assert!(value@ - div@ * 100 == value@ % 100);
        proof_assert!(value@ - div@ * 100 < 100);
    }
    (div, value - div * 100)
}
