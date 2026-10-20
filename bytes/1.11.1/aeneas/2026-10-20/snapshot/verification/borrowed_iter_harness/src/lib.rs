use bytes::{Bytes, BytesMut};

pub fn bytes_ref_iter<'a>(x: &'a Bytes) -> core::slice::Iter<'a, u8> {
    <&'a Bytes as IntoIterator>::into_iter(x)
}

pub fn bytesmut_ref_iter<'a>(x: &'a BytesMut) -> core::slice::Iter<'a, u8> {
    <&'a BytesMut as IntoIterator>::into_iter(x)
}
