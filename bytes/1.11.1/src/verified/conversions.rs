//! Standard-library conversions whose contracts preserve the byte sequence.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::ops::Range;
use creusot_std::{prelude::*, std::iter::IteratorSpec};

use super::exclusive::ExclusiveBytes;

impl ExclusiveBytes {
    /// Copies a borrowed byte slice into exclusive storage.
    #[ensures(result@ == source@)]
    pub fn copy_from_slice(source: &[u8]) -> Self {
        Self::from_vec(Vec::from(source))
    }

    /// Creates a separate Vec-backed copy of this sequence.
    #[ensures(result@ == self@)]
    pub fn copy_clone(&self) -> Self {
        Self::copy_from_slice(self.as_slice())
    }

    /// Copies a checked byte range into a separate owner.
    ///
    /// Reversed or out-of-bounds ranges return `None` without changing this owner.
    #[ensures(match result {
        Some(copy) => range.start@ <= range.end@ && range.end@ <= self@.len()
            && copy@ == self@[range.start@..range.end@],
        None => range.start@ > range.end@ || range.end@ > self@.len(),
    })]
    pub fn try_copy_slice(&self, range: Range<usize>) -> Option<Self> {
        let input = self.as_slice();
        if range.start > range.end || range.end > input.len() {
            None
        } else {
            Some(Self::copy_from_slice(&input[range.start..range.end]))
        }
    }

    /// Copies the UTF-8 bytes of a borrowed string into exclusive storage.
    #[ensures(result@ == source@.to_bytes())]
    pub fn copy_from_str(source: &str) -> Self {
        Self::from_vec(Vec::from(source))
    }

    /// Appends a copied UTF-8 string and closes its temporary owner.
    #[ensures((^self)@ == self@.concat(source@.to_bytes()))]
    pub fn append_str(&mut self, source: &str) {
        let copied = Self::copy_from_str(source);
        self.append_owner(copied);
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

    /// Appends an iterator whose actual iterator type has an `IteratorSpec` contract.
    #[requires(I::into_iter.precondition((source,)))]
    #[ensures(exists<initial: <I as IntoIterator>::IntoIter,
                     done: &mut <I as IntoIterator>::IntoIter,
                     produced: Seq<u8>>
        I::into_iter.postcondition((source,), initial) &&
        initial.produces(produced, *done) &&
        done.completed() && resolve(^done) &&
        (^self)@ == self@.concat(produced))]
    pub fn append_iter<I>(&mut self, source: I)
    where
        I: IntoIterator<Item = u8>,
        I::IntoIter: IteratorSpec,
    {
        let appended = Self::from_iter(source);
        self.append_owner(appended);
    }
}

impl Default for ExclusiveBytes {
    #[ensures(result@.len() == 0)]
    fn default() -> Self {
        Self::from_vec(Vec::new())
    }
}

/// Copies an owner's contents and explicitly closes both owners.
pub(crate) fn copy_clone_then_close(owner: ExclusiveBytes) {
    let copy = owner.copy_clone();
    owner.close();
    copy.close();
}

/// Copies a checked range and explicitly closes every owner on both branches.
#[ensures(result == (range.start@ <= range.end@ && range.end@ <= owner@.len()))]
pub(crate) fn copy_slice_then_close(owner: ExclusiveBytes, range: Range<usize>) -> bool {
    match owner.try_copy_slice(range) {
        Some(copy) => {
            owner.close();
            copy.close();
            true
        }
        None => {
            owner.close();
            false
        }
    }
}

/// Copies a slice, updates one byte, and sends the resulting allocation
/// through the physical shared-read tree and explicit cleanup path.
#[cfg(feature = "std")]
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

        let mut copied = ExclusiveBytes::copy_from_slice(b"copy");
        let clone = copied.copy_clone();
        assert_eq!(clone.as_slice(), b"copy");
        copied.set(0, b'C');
        assert_eq!(clone.as_slice(), b"copy");
        clone.close();
        copied.close();

        let mut ranged = ExclusiveBytes::copy_from_slice(b"range");
        let range_copy = ranged.try_copy_slice(1..4).unwrap();
        assert_eq!(range_copy.as_slice(), b"ang");
        ranged.set(1, b'X');
        assert_eq!(range_copy.as_slice(), b"ang");
        range_copy.close();
        assert!(super::copy_slice_then_close(
            ExclusiveBytes::copy_from_slice(b"range"),
            0..5,
        ));
        assert!(super::copy_slice_then_close(
            ExclusiveBytes::copy_from_slice(b"range"),
            5..5,
        ));
        assert!(!super::copy_slice_then_close(
            ExclusiveBytes::copy_from_slice(b"range"),
            4..2,
        ));
        assert!(!super::copy_slice_then_close(
            ExclusiveBytes::copy_from_slice(b"range"),
            0..6,
        ));
        super::copy_clone_then_close(ExclusiveBytes::copy_from_slice(b"closed"));

        let default = ExclusiveBytes::default();
        assert!(default.is_empty());
        default.close();

        let iterated = ExclusiveBytes::from_iter(alloc::vec![b'i', b't', b'e', b'r'].into_iter());
        assert_eq!(iterated.as_slice(), b"iter");
        iterated.close();

        let mut appended_iter = ExclusiveBytes::from_vec(b"before:".to_vec());
        appended_iter.append_iter(alloc::vec![b'a', b'f', b't', b'e', b'r'].into_iter());
        assert_eq!(appended_iter.as_slice(), b"before:after");
        appended_iter.close();

        let equal_left = ExclusiveBytes::copy_from_slice(b"same");
        let equal_right = ExclusiveBytes::from_string(String::from("same"));
        assert!(super::super::observers::equal_then_close(equal_left, equal_right));
    }

    #[test]
    fn appended_string_uses_checked_owned_conversion() {
        let mut bytes = ExclusiveBytes::from_vec(b"prefix:".to_vec());
        bytes.append_str("héllo");
        assert_eq!(bytes.as_slice(), "prefix:héllo".as_bytes());
        bytes.close();
    }

    #[test]
    fn copy_ranges_cover_empty_and_invalid_owners() {
        let mut empty_with_spare = ExclusiveBytes::from_vec(Vec::with_capacity(8));
        let empty = empty_with_spare.try_copy_slice(0..0).unwrap();
        assert!(empty.is_empty());
        empty.close();
        assert!(empty_with_spare.try_copy_slice(0..1).is_none());
        empty_with_spare.close();
    }
}
