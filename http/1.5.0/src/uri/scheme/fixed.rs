#[allow(unused_imports)]
use creusot_std::prelude::{Int, Seq, ensures, invariant, logic, pearlite, variant};

/// ASCII lowercase as an integer-valued model.
#[logic(open(super))]
pub(super) fn ascii_lowercase_model(byte: u8) -> Int {
    pearlite! {
        if 65 <= byte@ && byte@ <= 90 { byte@ + 32 } else { byte@ }
    }
}

/// The bytes of `prefix` match the start of `bytes`, ignoring ASCII case.
#[logic(open(super))]
pub(super) fn ascii_prefix_matches(bytes: Seq<u8>, prefix: Seq<u8>) -> bool {
    pearlite! {
        prefix.len() <= bytes.len()
            && forall<i: Int> 0 <= i && i < prefix.len() ==>
                ascii_lowercase_model(bytes[i]) == ascii_lowercase_model(prefix[i])
    }
}

/// Lowercase one ASCII uppercase byte, leaving all other bytes unchanged.
#[ensures(result@ == ascii_lowercase_model(byte))]
pub(super) const fn ascii_lowercase(byte: u8) -> u8 {
    if byte >= b'A' && byte <= b'Z' {
        byte + 32
    } else {
        byte
    }
}

/// Compare a byte slice's prefix using ASCII case-insensitive equality.
#[ensures(result == ascii_prefix_matches(bytes@, prefix@))]
pub(super) fn eq_ascii_prefix(bytes: &[u8], prefix: &[u8]) -> bool {
    let prefix_len = prefix.len();
    if bytes.len() < prefix_len {
        return false;
    }

    let mut i = 0;
    #[invariant(bytes@.len() >= prefix_len@)]
    #[invariant(i@ <= prefix_len@)]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==>
        ascii_lowercase_model(bytes@[j]) == ascii_lowercase_model(prefix@[j]))]
    #[variant(prefix_len - i)]
    while i < prefix_len {
        if ascii_lowercase(bytes[i]) != ascii_lowercase(prefix[i]) {
            return false;
        }
        i += 1;
    }

    true
}
