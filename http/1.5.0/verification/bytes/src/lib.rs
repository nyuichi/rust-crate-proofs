#![allow(dead_code, unexpected_cfgs)]

#[path = "../../../src/bytes_model.rs"]
mod bytes_model;

use bytes::{BufMut, Bytes, BytesMut};
#[cfg(creusot)]
use bytes_model::{bytes_mut_capacity, bytes_mut_seq, bytes_seq};
#[cfg(creusot)]
use creusot_std::logic::OrdLogic as _;
#[cfg(creusot)]
use creusot_std::prelude::Seq;
use creusot_std::prelude::{ensures, requires};
use std::ops::Deref;

#[ensures(bytes_seq(result) == data@)]
pub fn copy_from_slice(data: &[u8]) -> Bytes {
    Bytes::copy_from_slice(data)
}

#[ensures(bytes_seq(result) == bytes_seq(*source))]
pub fn clone_bytes(source: &Bytes) -> Bytes {
    source.clone()
}

#[ensures(result@ == bytes_seq(*source))]
pub fn borrow_bytes(source: &Bytes) -> &[u8] {
    source.as_ref()
}

#[requires(data@.len() <= isize::MAX@)]
#[ensures(bytes_seq(result) == data@)]
pub fn append_and_freeze(data: &[u8]) -> Bytes {
    let mut output = BytesMut::with_capacity(data.len());
    output.extend_from_slice(data);
    output.freeze()
}

#[requires(data@.to_bytes().len() <= isize::MAX@)]
#[ensures(bytes_seq(result) == data@.to_bytes())]
pub fn write_and_freeze(data: &str) -> Bytes {
    use std::fmt::Write;

    let mut output = BytesMut::new();
    output.write_str(data).unwrap();
    output.freeze()
}

#[ensures(bytes_seq(result) == data@.to_bytes())]
pub fn copy_string(data: &str) -> Bytes {
    Bytes::from(data.to_owned())
}

#[ensures(bytes_mut_seq(result) == data@)]
pub fn mutable_from_slice(data: &[u8]) -> BytesMut {
    BytesMut::from(data)
}

#[ensures(bytes_seq(result) == data@)]
pub fn from_vec(data: Vec<u8>) -> Bytes {
    Bytes::from(data)
}

#[ensures(bytes_seq(result) == data@)]
pub fn from_static_slice(data: &'static [u8]) -> Bytes {
    Bytes::from(data)
}

#[ensures(bytes_seq(result) == data@.to_bytes())]
pub fn from_static_str(data: &'static str) -> Bytes {
    Bytes::from(data)
}

#[ensures(result@ == bytes_seq(*value).len())]
pub fn bytes_len(value: &Bytes) -> usize {
    value.len()
}

#[ensures(result == (bytes_seq(*value).len() == 0))]
pub fn bytes_is_empty(value: &Bytes) -> bool {
    value.is_empty()
}

#[ensures(result == (bytes_seq(*left) == bytes_seq(*right)))]
pub fn bytes_equal(left: &Bytes, right: &Bytes) -> bool {
    left == right
}

#[ensures(result == bytes_seq(*left).cmp_log(bytes_seq(*right)))]
pub fn bytes_compare(left: &Bytes, right: &Bytes) -> std::cmp::Ordering {
    left.cmp(right)
}

#[ensures(result == Some(bytes_seq(*left).cmp_log(bytes_seq(*right))))]
pub fn bytes_partial_compare(left: &Bytes, right: &Bytes) -> Option<std::cmp::Ordering> {
    left.partial_cmp(right)
}

#[ensures(result@ == bytes_seq(*source))]
pub fn deref_bytes(source: &Bytes) -> &[u8] {
    Deref::deref(source)
}

#[requires(at@ <= bytes_seq(*source).len())]
#[ensures(bytes_seq(result) == bytes_seq(*source).subsequence(0, at@))]
#[ensures(bytes_seq(^source) == bytes_seq(*source).subsequence(at@, bytes_seq(*source).len()))]
pub fn split_bytes_to(source: &mut Bytes, at: usize) -> Bytes {
    source.split_to(at)
}

#[requires(at@ <= bytes_seq(*source).len())]
#[ensures(bytes_seq(result) == bytes_seq(*source).subsequence(at@, bytes_seq(*source).len()))]
#[ensures(bytes_seq(^source) == bytes_seq(*source).subsequence(0, at@))]
pub fn split_bytes_off(source: &mut Bytes, at: usize) -> Bytes {
    source.split_off(at)
}

#[ensures(bytes_seq(^value) == bytes_seq(*value).subsequence(0, if len@ < bytes_seq(*value).len() { len@ } else { bytes_seq(*value).len() }))]
pub fn truncate_bytes(value: &mut Bytes, len: usize) {
    value.truncate(len)
}

#[ensures(bytes_seq(^value) == Seq::empty())]
pub fn clear_bytes(value: &mut Bytes) {
    value.clear()
}

#[requires(capacity@ <= isize::MAX@)]
#[ensures(bytes_mut_seq(result) == Seq::empty())]
#[ensures(bytes_mut_capacity(result) >= capacity@)]
pub fn mutable_with_capacity(capacity: usize) -> BytesMut {
    BytesMut::with_capacity(capacity)
}

#[ensures(result@ == bytes_mut_seq(*value).len())]
pub fn mutable_len(value: &BytesMut) -> usize {
    value.len()
}

#[ensures(result == (bytes_mut_seq(*value).len() == 0))]
pub fn mutable_is_empty(value: &BytesMut) -> bool {
    value.is_empty()
}

#[ensures(result@ == bytes_mut_capacity(*value))]
pub fn mutable_capacity(value: &BytesMut) -> usize {
    value.capacity()
}

#[requires(additional@ <= isize::MAX@ - bytes_mut_seq(*value).len())]
#[ensures(bytes_mut_seq(^value) == bytes_mut_seq(*value))]
#[ensures(bytes_mut_capacity(^value) >= bytes_mut_seq(*value).len() + additional@)]
pub fn reserve_mutable(value: &mut BytesMut, additional: usize) {
    value.reserve(additional)
}

#[ensures(bytes_mut_seq(^value) == bytes_mut_seq(*value).subsequence(0, if len@ < bytes_mut_seq(*value).len() { len@ } else { bytes_mut_seq(*value).len() }))]
pub fn truncate_mutable(value: &mut BytesMut, len: usize) {
    value.truncate(len)
}

#[ensures(bytes_mut_seq(^value) == Seq::empty())]
pub fn clear_mutable(value: &mut BytesMut) {
    value.clear()
}

#[ensures(bytes_mut_seq(result) == bytes_mut_seq(*source))]
pub fn clone_mutable(source: &BytesMut) -> BytesMut {
    source.clone()
}

#[ensures(result@ == bytes_mut_seq(*source))]
pub fn borrow_mutable(source: &BytesMut) -> &[u8] {
    source.as_ref()
}

#[ensures(result@ == bytes_mut_seq(*source))]
pub fn deref_mutable(source: &BytesMut) -> &[u8] {
    Deref::deref(source)
}

#[requires(at@ <= bytes_mut_seq(*source).len())]
#[ensures(bytes_mut_seq(result) == bytes_mut_seq(*source).subsequence(0, at@))]
#[ensures(bytes_mut_seq(^source) == bytes_mut_seq(*source).subsequence(at@, bytes_mut_seq(*source).len()))]
pub fn split_mutable_to(source: &mut BytesMut, at: usize) -> BytesMut {
    source.split_to(at)
}

#[requires(at@ <= bytes_mut_capacity(*source))]
#[ensures(bytes_mut_seq(result) == if at@ <= bytes_mut_seq(*source).len() {
    bytes_mut_seq(*source).subsequence(at@, bytes_mut_seq(*source).len())
} else { Seq::empty() })]
#[ensures(bytes_mut_seq(^source) == if at@ <= bytes_mut_seq(*source).len() {
    bytes_mut_seq(*source).subsequence(0, at@)
} else { bytes_mut_seq(*source) })]
pub fn split_mutable_off(source: &mut BytesMut, at: usize) -> BytesMut {
    source.split_off(at)
}

#[ensures(bytes_mut_seq(result) == bytes_seq(value))]
pub fn mutable_from_bytes(value: Bytes) -> BytesMut {
    BytesMut::from(value)
}

#[ensures(bytes_mut_seq(result) == data@.to_bytes())]
pub fn mutable_from_str(data: &str) -> BytesMut {
    BytesMut::from(data)
}

#[requires(bytes_mut_seq(*output).len() + data@.len() <= isize::MAX@)]
#[ensures(bytes_mut_seq(^output) == bytes_mut_seq(*output).concat(data@))]
pub fn put_slice(output: &mut BytesMut, data: &[u8]) {
    output.put_slice(data)
}

#[requires(bytes_mut_seq(*output).len() + data@.to_bytes().len() <= isize::MAX@)]
#[ensures(result == Ok(()))]
#[ensures(bytes_mut_seq(^output) == bytes_mut_seq(*output).concat(data@.to_bytes()))]
pub fn write_string(output: &mut BytesMut, data: &str) -> std::fmt::Result {
    use std::fmt::Write;

    output.write_str(data)
}
