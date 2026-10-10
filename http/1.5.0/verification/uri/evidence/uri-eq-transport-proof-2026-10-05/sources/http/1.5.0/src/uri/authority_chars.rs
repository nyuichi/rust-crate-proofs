//! ASCII character classifier shared by the authority parser.

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, Int};

/// Exact model of the nonzero entries in the former URI character table.
#[doc(hidden)]
#[logic(open)]
pub fn uri_char_model(byte: Int) -> Int {
    pearlite! {
        if byte == 33 || byte == 35 || byte == 36 || (38 <= byte && byte <= 59)
            || byte == 61 || (63 <= byte && byte <= 91) || byte == 93
            || byte == 95 || (97 <= byte && byte <= 122) || byte == 126
        { byte } else { 0 }
    }
}

/// Runtime classifier used by the authority validator in place of a table
/// lookup.
#[ensures(result@ == uri_char_model(byte@))]
pub(crate) const fn uri_char(byte: u8) -> u8 {
    match byte {
        33 | 35 | 36 | 38..=59 | 61 | 63..=91 | 93 | 95 | 97..=122 | 126 => byte,
        _ => 0,
    }
}
