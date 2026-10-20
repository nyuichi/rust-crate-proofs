use bytes::{Bytes, BytesMut};
use std::cmp::{PartialEq, PartialOrd};

pub fn bytes_eq_slice(lhs: &Bytes, rhs: &[u8]) -> bool {
    <Bytes as PartialEq<[u8]>>::eq(lhs, rhs)
}

pub fn bytes_cmp_slice(lhs: &Bytes, rhs: &[u8]) -> Option<std::cmp::Ordering> {
    <Bytes as PartialOrd<[u8]>>::partial_cmp(lhs, rhs)
}

pub fn slice_eq_bytes(lhs: &[u8], rhs: &Bytes) -> bool {
    <[u8] as PartialEq<Bytes>>::eq(lhs, rhs)
}

pub fn slice_cmp_bytes(lhs: &[u8], rhs: &Bytes) -> Option<std::cmp::Ordering> {
    <[u8] as PartialOrd<Bytes>>::partial_cmp(lhs, rhs)
}

pub fn bytes_eq_str(lhs: &Bytes, rhs: &str) -> bool {
    <Bytes as PartialEq<str>>::eq(lhs, rhs)
}

pub fn bytes_cmp_str(lhs: &Bytes, rhs: &str) -> Option<std::cmp::Ordering> {
    <Bytes as PartialOrd<str>>::partial_cmp(lhs, rhs)
}

pub fn str_eq_bytes(lhs: &str, rhs: &Bytes) -> bool {
    <str as PartialEq<Bytes>>::eq(lhs, rhs)
}

pub fn str_cmp_bytes(lhs: &str, rhs: &Bytes) -> Option<std::cmp::Ordering> {
    <str as PartialOrd<Bytes>>::partial_cmp(lhs, rhs)
}

pub fn bytes_eq_vec(lhs: &Bytes, rhs: &Vec<u8>) -> bool {
    <Bytes as PartialEq<Vec<u8>>>::eq(lhs, rhs)
}

pub fn bytes_cmp_vec(lhs: &Bytes, rhs: &Vec<u8>) -> Option<std::cmp::Ordering> {
    <Bytes as PartialOrd<Vec<u8>>>::partial_cmp(lhs, rhs)
}

pub fn vec_eq_bytes(lhs: &Vec<u8>, rhs: &Bytes) -> bool {
    <Vec<u8> as PartialEq<Bytes>>::eq(lhs, rhs)
}

pub fn vec_cmp_bytes(lhs: &Vec<u8>, rhs: &Bytes) -> Option<std::cmp::Ordering> {
    <Vec<u8> as PartialOrd<Bytes>>::partial_cmp(lhs, rhs)
}

pub fn bytes_eq_string(lhs: &Bytes, rhs: &String) -> bool {
    <Bytes as PartialEq<String>>::eq(lhs, rhs)
}

pub fn bytes_cmp_string(lhs: &Bytes, rhs: &String) -> Option<std::cmp::Ordering> {
    <Bytes as PartialOrd<String>>::partial_cmp(lhs, rhs)
}

pub fn string_eq_bytes(lhs: &String, rhs: &Bytes) -> bool {
    <String as PartialEq<Bytes>>::eq(lhs, rhs)
}

pub fn string_cmp_bytes(lhs: &String, rhs: &Bytes) -> Option<std::cmp::Ordering> {
    <String as PartialOrd<Bytes>>::partial_cmp(lhs, rhs)
}

pub fn bytesmut_eq_slice(lhs: &BytesMut, rhs: &[u8]) -> bool {
    <BytesMut as PartialEq<[u8]>>::eq(lhs, rhs)
}

pub fn bytesmut_cmp_slice(lhs: &BytesMut, rhs: &[u8]) -> Option<std::cmp::Ordering> {
    <BytesMut as PartialOrd<[u8]>>::partial_cmp(lhs, rhs)
}

pub fn slice_eq_bytesmut(lhs: &[u8], rhs: &BytesMut) -> bool {
    <[u8] as PartialEq<BytesMut>>::eq(lhs, rhs)
}

pub fn slice_cmp_bytesmut(lhs: &[u8], rhs: &BytesMut) -> Option<std::cmp::Ordering> {
    <[u8] as PartialOrd<BytesMut>>::partial_cmp(lhs, rhs)
}

pub fn bytesmut_eq_str(lhs: &BytesMut, rhs: &str) -> bool {
    <BytesMut as PartialEq<str>>::eq(lhs, rhs)
}

pub fn bytesmut_cmp_str(lhs: &BytesMut, rhs: &str) -> Option<std::cmp::Ordering> {
    <BytesMut as PartialOrd<str>>::partial_cmp(lhs, rhs)
}

pub fn str_eq_bytesmut(lhs: &str, rhs: &BytesMut) -> bool {
    <str as PartialEq<BytesMut>>::eq(lhs, rhs)
}

pub fn str_cmp_bytesmut(lhs: &str, rhs: &BytesMut) -> Option<std::cmp::Ordering> {
    <str as PartialOrd<BytesMut>>::partial_cmp(lhs, rhs)
}

pub fn bytesmut_eq_vec(lhs: &BytesMut, rhs: &Vec<u8>) -> bool {
    <BytesMut as PartialEq<Vec<u8>>>::eq(lhs, rhs)
}

pub fn bytesmut_cmp_vec(lhs: &BytesMut, rhs: &Vec<u8>) -> Option<std::cmp::Ordering> {
    <BytesMut as PartialOrd<Vec<u8>>>::partial_cmp(lhs, rhs)
}

pub fn vec_eq_bytesmut(lhs: &Vec<u8>, rhs: &BytesMut) -> bool {
    <Vec<u8> as PartialEq<BytesMut>>::eq(lhs, rhs)
}

pub fn vec_cmp_bytesmut(lhs: &Vec<u8>, rhs: &BytesMut) -> Option<std::cmp::Ordering> {
    <Vec<u8> as PartialOrd<BytesMut>>::partial_cmp(lhs, rhs)
}

pub fn bytesmut_eq_string(lhs: &BytesMut, rhs: &String) -> bool {
    <BytesMut as PartialEq<String>>::eq(lhs, rhs)
}

pub fn bytesmut_cmp_string(lhs: &BytesMut, rhs: &String) -> Option<std::cmp::Ordering> {
    <BytesMut as PartialOrd<String>>::partial_cmp(lhs, rhs)
}

pub fn string_eq_bytesmut(lhs: &String, rhs: &BytesMut) -> bool {
    <String as PartialEq<BytesMut>>::eq(lhs, rhs)
}

pub fn string_cmp_bytesmut(lhs: &String, rhs: &BytesMut) -> Option<std::cmp::Ordering> {
    <String as PartialOrd<BytesMut>>::partial_cmp(lhs, rhs)
}

pub fn bytes_eq_ref_slice(lhs: &Bytes, rhs: &[u8]) -> bool {
    <Bytes as PartialEq<&[u8]>>::eq(lhs, &rhs)
}

pub fn bytes_cmp_ref_slice(lhs: &Bytes, rhs: &[u8]) -> Option<std::cmp::Ordering> {
    <Bytes as PartialOrd<&[u8]>>::partial_cmp(lhs, &rhs)
}

pub fn bytes_eq_ref_str(lhs: &Bytes, rhs: &str) -> bool {
    <Bytes as PartialEq<&str>>::eq(lhs, &rhs)
}

pub fn bytes_cmp_ref_str(lhs: &Bytes, rhs: &str) -> Option<std::cmp::Ordering> {
    <Bytes as PartialOrd<&str>>::partial_cmp(lhs, &rhs)
}

pub fn bytesmut_eq_ref_slice(lhs: &BytesMut, rhs: &[u8]) -> bool {
    <BytesMut as PartialEq<&[u8]>>::eq(lhs, &rhs)
}

pub fn bytesmut_cmp_ref_slice(lhs: &BytesMut, rhs: &[u8]) -> Option<std::cmp::Ordering> {
    <BytesMut as PartialOrd<&[u8]>>::partial_cmp(lhs, &rhs)
}

pub fn bytesmut_eq_ref_str(lhs: &BytesMut, rhs: &str) -> bool {
    <BytesMut as PartialEq<&str>>::eq(lhs, &rhs)
}

pub fn bytesmut_cmp_ref_str(lhs: &BytesMut, rhs: &str) -> Option<std::cmp::Ordering> {
    <BytesMut as PartialOrd<&str>>::partial_cmp(lhs, &rhs)
}

pub fn ref_slice_eq_bytes(lhs: &[u8], rhs: &Bytes) -> bool {
    <&[u8] as PartialEq<Bytes>>::eq(&lhs, rhs)
}

pub fn ref_slice_cmp_bytes(lhs: &[u8], rhs: &Bytes) -> Option<std::cmp::Ordering> {
    <&[u8] as PartialOrd<Bytes>>::partial_cmp(&lhs, rhs)
}

pub fn ref_str_eq_bytes(lhs: &str, rhs: &Bytes) -> bool {
    <&str as PartialEq<Bytes>>::eq(&lhs, rhs)
}

pub fn ref_str_cmp_bytes(lhs: &str, rhs: &Bytes) -> Option<std::cmp::Ordering> {
    <&str as PartialOrd<Bytes>>::partial_cmp(&lhs, rhs)
}

pub fn ref_slice_eq_bytesmut(lhs: &[u8], rhs: &BytesMut) -> bool {
    <&[u8] as PartialEq<BytesMut>>::eq(&lhs, rhs)
}

pub fn ref_slice_cmp_bytesmut(lhs: &[u8], rhs: &BytesMut) -> Option<std::cmp::Ordering> {
    <&[u8] as PartialOrd<BytesMut>>::partial_cmp(&lhs, rhs)
}

pub fn ref_str_eq_bytesmut(lhs: &str, rhs: &BytesMut) -> bool {
    <&str as PartialEq<BytesMut>>::eq(&lhs, rhs)
}

pub fn ref_str_cmp_bytesmut(lhs: &str, rhs: &BytesMut) -> Option<std::cmp::Ordering> {
    <&str as PartialOrd<BytesMut>>::partial_cmp(&lhs, rhs)
}

pub fn bytesmut_eq_bytes(lhs: &BytesMut, rhs: &Bytes) -> bool {
    <BytesMut as PartialEq<Bytes>>::eq(lhs, rhs)
}

pub fn bytes_eq_bytesmut(lhs: &Bytes, rhs: &BytesMut) -> bool {
    <Bytes as PartialEq<BytesMut>>::eq(lhs, rhs)
}
