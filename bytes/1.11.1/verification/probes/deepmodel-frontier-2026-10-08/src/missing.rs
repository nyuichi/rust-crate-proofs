//! Negative control: original-like runtime representation with no `DeepModel`.

use core::cmp::{Ordering, PartialEq, PartialOrd};
use creusot_std::prelude::*;

pub struct RawBytes {
    pub ptr: *const u8,
    pub len: usize,
}

impl PartialEq for RawBytes {
    fn eq(&self, other: &RawBytes) -> bool {
        self.ptr == other.ptr && self.len == other.len
    }
}

impl PartialOrd for RawBytes {
    fn partial_cmp(&self, other: &RawBytes) -> Option<Ordering> {
        self.len.partial_cmp(&other.len)
    }
}

// Match bytes 1.11.1's generic reference-forwarding shape. The bound mentions
// comparison on the original type, which is the shape in the archived ICE's
// typing environment.
impl<'a, T: ?Sized> PartialEq<&'a T> for RawBytes
where
    RawBytes: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for RawBytes
where
    RawBytes: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
        self.partial_cmp(&**other)
    }
}
