#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
#[cfg(creusot)]
use creusot_std::prelude::*;

// Static callbacks are unselected in this metadata-only proof. Their source
// signature mentions BytesMut, so retain an opaque local return-type stand-in
// without compiling or claiming the real BytesMut implementation.
pub struct BytesMut;

#[cfg(not(creusot))]
impl From<&[u8]> for BytesMut {
    fn from(_: &[u8]) -> Self {
        Self
    }
}

pub mod actual {
    include!(concat!(env!("OUT_DIR"), "/actual_bytes_observers.rs"));
}

// Generic std constructor boundary used only by constructor scopes. The
// pinned core implementation constructs an AtomicPtr value from its argument;
// this vacuous postcondition states no stored-value relation, permission,
// pointer authority, memory ordering, refcount, or Bytes protocol fact.
#[cfg(all(creusot, any(bytes_observer_scope = "from_static", bytes_observer_scope = "all")))]
extern_spec! {
    impl<T> core::sync::atomic::AtomicPtr<T> {
        #[ensures(true)]
        fn new(p: *mut T) -> Self;
    }
}

#[cfg(all(creusot, bytes_observer_scope = "from_static"))]
#[ensures(result@ == bytes@.len())]
pub fn static_len(bytes: &'static [u8]) -> usize {
    let value = actual::Bytes::from_static(bytes);
    let len = value.len();
    core::mem::forget(value);
    len
}

#[cfg(all(creusot, bytes_observer_scope = "all"))]
#[ensures(result)]
pub fn new_is_empty() -> bool {
    let value = actual::Bytes::new();
    let empty = value.is_empty();
    core::mem::forget(value);
    empty
}

#[cfg(test)]
mod tests {
    use super::actual::Bytes;
    use core::mem;

    #[test]
    fn new_is_empty_without_reading_the_raw_view() {
        let value = Bytes::new();
        assert_eq!(value.len(), 0);
        assert!(value.is_empty());
        mem::forget(value);
    }

    #[test]
    fn from_static_preserves_empty_and_nonempty_lengths() {
        let empty = Bytes::from_static(b"");
        let nonempty = Bytes::from_static(b"\0\xffbytes");
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
        assert_eq!(nonempty.len(), 7);
        assert!(!nonempty.is_empty());
        mem::forget(empty);
        mem::forget(nonempty);
    }
}
