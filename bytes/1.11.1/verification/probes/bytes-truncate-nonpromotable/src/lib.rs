#![allow(unexpected_cfgs, dead_code)]
extern crate alloc;
#[cfg(creusot)]
use creusot_std::prelude::*;

// Callback return-type placeholder. Callbacks are not selected or called.
pub struct BytesMut;

#[cfg(not(creusot))]
impl From<&[u8]> for BytesMut {
    fn from(_: &[u8]) -> Self {
        Self
    }
}

pub mod actual {
    include!(concat!(env!("OUT_DIR"), "/actual_bytes_truncate.rs"));
}

#[cfg(test)]
mod tests {
    use super::actual::Bytes;
    use core::mem;

    #[test]
    fn truncate_and_clear_keep_metadata_coherent_on_static_backing() {
        let mut bytes = Bytes::from_static(b"abcdef");
        bytes.truncate(3);
        assert_eq!(bytes.len(), 3);
        bytes.truncate(10);
        assert_eq!(bytes.len(), 3);
        bytes.clear();
        assert_eq!(bytes.len(), 0);
        mem::forget(bytes);
    }
}
