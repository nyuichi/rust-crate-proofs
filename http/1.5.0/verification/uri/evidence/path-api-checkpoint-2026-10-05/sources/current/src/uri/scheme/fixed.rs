#[allow(unused_imports)]
use creusot_std::prelude::{ensures, invariant, logic, pearlite, variant, Int, Seq};

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

/// The sequence is exactly the lowercase ASCII spelling `http`.
#[logic(open(super))]
pub(super) fn is_http_bytes(bytes: Seq<u8>) -> bool {
    pearlite! {
        bytes.len() == 4
            && bytes[0]@ == 104 && bytes[1]@ == 116
            && bytes[2]@ == 116 && bytes[3]@ == 112
    }
}

/// The sequence is exactly the lowercase ASCII spelling `https`.
#[logic(open(super))]
pub(super) fn is_https_bytes(bytes: Seq<u8>) -> bool {
    pearlite! {
        bytes.len() == 5
            && bytes[0]@ == 104 && bytes[1]@ == 116
            && bytes[2]@ == 116 && bytes[3]@ == 112
            && bytes[4]@ == 115
    }
}

/// Compare a slice to the exact lowercase ASCII spelling `http`.
#[ensures(result == is_http_bytes(bytes@))]
pub(super) fn is_http(bytes: &[u8]) -> bool {
    bytes.len() == 4
        && bytes[0] == b'h'
        && bytes[1] == b't'
        && bytes[2] == b't'
        && bytes[3] == b'p'
}

/// Compare a slice to the exact lowercase ASCII spelling `https`.
#[ensures(result == is_https_bytes(bytes@))]
pub(super) fn is_https(bytes: &[u8]) -> bool {
    bytes.len() == 5
        && bytes[0] == b'h'
        && bytes[1] == b't'
        && bytes[2] == b't'
        && bytes[3] == b'p'
        && bytes[4] == b's'
}

/// The slice begins with an ASCII-case-insensitive `http://` prefix.
#[logic(open(super))]
pub(super) fn has_http_scheme_prefix(bytes: Seq<u8>) -> bool {
    pearlite! {
        bytes.len() >= 7
            && ascii_lowercase_model(bytes[0]) == 104
            && ascii_lowercase_model(bytes[1]) == 116
            && ascii_lowercase_model(bytes[2]) == 116
            && ascii_lowercase_model(bytes[3]) == 112
            && bytes[4]@ == 58 && bytes[5]@ == 47 && bytes[6]@ == 47
    }
}

/// The slice begins with an ASCII-case-insensitive `https://` prefix.
#[logic(open(super))]
pub(super) fn has_https_scheme_prefix(bytes: Seq<u8>) -> bool {
    pearlite! {
        bytes.len() >= 8
            && ascii_lowercase_model(bytes[0]) == 104
            && ascii_lowercase_model(bytes[1]) == 116
            && ascii_lowercase_model(bytes[2]) == 116
            && ascii_lowercase_model(bytes[3]) == 112
            && ascii_lowercase_model(bytes[4]) == 115
            && bytes[5]@ == 58 && bytes[6]@ == 47 && bytes[7]@ == 47
    }
}

/// Recognize an ASCII-case-insensitive `http://` prefix without creating a
/// temporary string slice.
#[ensures(result == has_http_scheme_prefix(bytes@))]
pub(super) fn has_http_scheme(bytes: &[u8]) -> bool {
    if bytes.len() < 7 {
        return false;
    }

    let prefix = [b'h', b't', b't', b'p', b':', b'/', b'/'];
    eq_ascii_prefix(bytes, &prefix)
}

/// Recognize an ASCII-case-insensitive `https://` prefix without creating a
/// temporary string slice.
#[ensures(result == has_https_scheme_prefix(bytes@))]
pub(super) fn has_https_scheme(bytes: &[u8]) -> bool {
    if bytes.len() < 8 {
        return false;
    }

    let prefix = [b'h', b't', b't', b'p', b's', b':', b'/', b'/'];
    eq_ascii_prefix(bytes, &prefix)
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
