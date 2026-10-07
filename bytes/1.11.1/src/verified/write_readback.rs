//! Cross-module composition of initialized writes with the shared read path.

use super::exclusive::ExclusiveBytes;
use creusot_std::prelude::*;

/// Writes a big-endian value, then reads it through the shared physical
/// allocation and the existing Cursor/close lifecycle.
#[ensures(result.0 == Some(value))]
#[ensures(result.1 == 0usize)]
#[ensures(result.2 != result.3)]
pub fn scoped_write_read_u16_be(
    value: u16,
    reverse: bool,
) -> (Option<u16>, usize, bool, bool) {
    let mut bytes = ExclusiveBytes::from_vec(alloc::vec::Vec::new());
    bytes.write_u16_be(value);
    super::scoped_numeric_read(bytes.into_vec(), reverse)
}
