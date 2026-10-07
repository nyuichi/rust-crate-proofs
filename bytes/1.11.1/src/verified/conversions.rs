//! Standard-library conversions whose contracts preserve the byte sequence.

use alloc::{boxed::Box, string::String, vec::Vec};
use creusot_std::{prelude::*, std::iter::IteratorSpec};

use super::exclusive::ExclusiveBytes;

impl ExclusiveBytes {
    /// Copies a borrowed byte slice into exclusive storage.
    #[ensures(result@ == source@)]
    pub fn copy_from_slice(source: &[u8]) -> Self {
        Self::from_vec(Vec::from(source))
    }

    /// Copies the UTF-8 bytes of a borrowed string into exclusive storage.
    #[ensures(result@ == source@.to_bytes())]
    pub fn copy_from_str(source: &str) -> Self {
        Self::from_vec(Vec::from(source))
    }

    /// Consumes a `String` and transfers its byte vector into exclusive storage.
    #[ensures(result@ == source@.to_bytes())]
    pub fn from_string(source: String) -> Self {
        Self::from_vec(Vec::from(source))
    }

    /// Consumes a boxed byte slice and transfers its allocation into exclusive storage.
    #[ensures(result@ == source@)]
    pub fn from_boxed_slice(source: Box<[u8]>) -> Self {
        Self::from_vec(Vec::from(source))
    }

    /// Collects an iterator whose actual iterator type has an `IteratorSpec` contract.
    #[requires(I::into_iter.precondition((source,)))]
    #[ensures(exists<initial: <I as IntoIterator>::IntoIter,
                     done: &mut <I as IntoIterator>::IntoIter,
                     produced: Seq<u8>>
        I::into_iter.postcondition((source,), initial) &&
        initial.produces(produced, *done) &&
        done.completed() && resolve(^done) &&
        result@ == produced)]
    pub fn from_iter<I>(source: I) -> Self
    where
        I: IntoIterator<Item = u8>,
        I::IntoIter: IteratorSpec,
    {
        let bytes: Vec<u8> = Vec::from_iter(source);
        Self::from_vec(bytes)
    }
}

/// Copies a slice, updates one byte, and sends the resulting allocation
/// through the physical shared-read tree and explicit cleanup path.
#[cfg(feature = "std")]
#[ensures(result.0 == (write_index@ < source@.len()))]
#[ensures((result.1 == None) == (leaves < 2usize))]
#[ensures(result.1 != None ==> result.1.unwrap_logic().0 ==
    if read_index@ < source@.len() {
        Some(if read_index == write_index { value } else { source@[read_index@] })
    } else { None })]
#[ensures(result.1 != None ==> result.1.unwrap_logic().1 == leaves)]
#[ensures(result.1 != None ==> result.1.unwrap_logic().2 != result.1.unwrap_logic().3)]
pub fn copy_slice_then_set_and_share(
    source: &[u8],
    write_index: usize,
    value: u8,
    read_index: usize,
    leaves: usize,
) -> (bool, Option<(Option<u8>, usize, bool, bool)>) {
    let mut bytes = ExclusiveBytes::copy_from_slice(source);
    let changed = bytes.set(write_index, value);
    let result = super::scoped_tree(bytes.into_vec(), read_index, leaves);
    (changed, result)
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn standard_conversions_feed_explicit_cleanup_paths() {
        let static_input: &'static [u8] = b"static";
        let (changed, read) = copy_slice_then_set_and_share(static_input, 1, b'X', 1, 4);
        assert!(changed);
        assert_eq!(read.unwrap().0, Some(b'X'));

        let borrowed_string = ExclusiveBytes::copy_from_str("borrowed");
        assert_eq!(borrowed_string.as_slice(), b"borrowed");
        borrowed_string.close();

        let owned_string = ExclusiveBytes::from_string(String::from("owned"));
        assert_eq!(owned_string.as_slice(), b"owned");
        owned_string.close();

        let boxed = Vec::from([b'B', b'o', b'x']).into_boxed_slice();
        let boxed_bytes = ExclusiveBytes::from_boxed_slice(boxed);
        assert_eq!(boxed_bytes.as_slice(), b"Box");
        boxed_bytes.close();

        let iterated = ExclusiveBytes::from_iter(alloc::vec![b'i', b't', b'e', b'r'].into_iter());
        assert_eq!(iterated.as_slice(), b"iter");
        iterated.close();

        let equal_left = ExclusiveBytes::copy_from_slice(b"same");
        let equal_right = ExclusiveBytes::from_string(String::from("same"));
        assert!(super::super::observers::equal_then_close(equal_left, equal_right));
    }
}
