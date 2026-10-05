#[cfg(creusot)]
use creusot_std::prelude::{check, ensures, requires};

#[repr(C, align(2))]
pub(crate) struct DecimalPairs(pub(crate) [u8; 200]);

// This is the one proof-side scalar initializer. The normal Rust build checks
// it against the original runtime byte string using const evaluation below.
macro_rules! decimal_pairs_scalar {
    () => {
        DecimalPairs([
            48, 48, 48, 49, 48, 50, 48, 51, 48, 52, 48, 53, 48, 54, 48, 55, 48, 56, 48, 57,
            49, 48, 49, 49, 49, 50, 49, 51, 49, 52, 49, 53, 49, 54, 49, 55, 49, 56, 49, 57,
            50, 48, 50, 49, 50, 50, 50, 51, 50, 52, 50, 53, 50, 54, 50, 55, 50, 56, 50, 57,
            51, 48, 51, 49, 51, 50, 51, 51, 51, 52, 51, 53, 51, 54, 51, 55, 51, 56, 51, 57,
            52, 48, 52, 49, 52, 50, 52, 51, 52, 52, 52, 53, 52, 54, 52, 55, 52, 56, 52, 57,
            53, 48, 53, 49, 53, 50, 53, 51, 53, 52, 53, 53, 53, 54, 53, 55, 53, 56, 53, 57,
            54, 48, 54, 49, 54, 50, 54, 51, 54, 52, 54, 53, 54, 54, 54, 55, 54, 56, 54, 57,
            55, 48, 55, 49, 55, 50, 55, 51, 55, 52, 55, 53, 55, 54, 55, 55, 55, 56, 55, 57,
            56, 48, 56, 49, 56, 50, 56, 51, 56, 52, 56, 53, 56, 54, 56, 55, 56, 56, 56, 57,
            57, 48, 57, 49, 57, 50, 57, 51, 57, 52, 57, 53, 57, 54, 57, 55, 57, 56, 57, 57,
        ])
    };
}

#[cfg(not(creusot))]
const DECIMAL_PAIRS_LITERAL: DecimalPairs = DecimalPairs(
    *b"0001020304050607080910111213141516171819\
       2021222324252627282930313233343536373839\
       4041424344454647484950515253545556575859\
       6061626364656667686970717273747576777879\
       8081828384858687888990919293949596979899",
);

// Const-evaluate a bytewise equality check. This ties the scalar proof
// initializer to the exact runtime byte-string initializer without a trust.
#[cfg(not(creusot))]
const fn arrays_equal(a: &[u8; 200], b: &[u8; 200]) -> bool {
    let mut i = 0;
    while i < 200 {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(not(creusot))]
const DECIMAL_PAIRS_SCALAR: DecimalPairs = decimal_pairs_scalar!();
#[cfg(not(creusot))]
const _: () = assert!(arrays_equal(&DECIMAL_PAIRS_LITERAL.0, &DECIMAL_PAIRS_SCALAR.0));

#[cfg(not(creusot))]
pub(crate) static DECIMAL_PAIRS: DecimalPairs = DECIMAL_PAIRS_LITERAL;

// Creusot 0.11.0-dev cannot translate an immutable static or the byte-string
// array constant, so verify this scalar initializer. Its equality to the
// runtime literal is enforced by the normal-build const assertion above.
#[cfg(creusot)]
pub(crate) const DECIMAL_PAIRS: DecimalPairs = decimal_pairs_scalar!();

#[cfg(creusot)]
#[check(terminates)]
#[requires(n@ < 100)]
#[ensures(2 * n@ + 1 < 200)]
#[ensures(DECIMAL_PAIRS.0[2 * n@]@ == 48 + n@ / 10)]
#[ensures(DECIMAL_PAIRS.0[2 * n@ + 1]@ == 48 + n@ % 10)]
#[ensures(result.0@ == DECIMAL_PAIRS.0[2 * n@]@)]
#[ensures(result.1@ == DECIMAL_PAIRS.0[2 * n@ + 1]@)]
pub(crate) fn decimal_pair_correct(n: u32) -> (u8, u8) {
    let index = n as usize * 2;
    (DECIMAL_PAIRS.0[index], DECIMAL_PAIRS.0[index + 1])
}
