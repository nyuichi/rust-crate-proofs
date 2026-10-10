#[cfg(creusot)]
use creusot_std::prelude::ensures;

/// Returns whether a header value byte can be exposed as visible ASCII text.
#[cfg_attr(creusot, ensures(result == (b@ >= 32 && b@ < 127 || b@ == 9)))]
pub(crate) const fn is_visible_ascii(b: u8) -> bool {
    b >= 32 && b < 127 || b == b'\t'
}

/// Returns whether a byte is permitted in an opaque HTTP header value.
#[cfg_attr(creusot, ensures(result == (b@ >= 32 && b@ != 127 || b@ == 9)))]
pub(crate) fn is_valid(b: u8) -> bool {
    b >= 32 && b != 127 || b == b'\t'
}
