#![cfg_attr(not(feature = "std"), no_std)]

use creusot_std::prelude::*;

#[check(ghost)]
#[ensures('\u{00E9}'.to_utf8() == seq![0xC3u8, 0xA9u8])]
pub fn utf8_u00e9() {
    let encoded = snapshot!('\u{00E9}'.to_utf8());
    proof_assert!('\u{00E9}'@ == 233);
    proof_assert!(encoded.len() == 2);
    proof_assert!(encoded[0]@ == 195);
    proof_assert!(encoded[1]@ == 169);
    proof_assert!(encoded.ext_eq(seq![0xC3u8, 0xA9u8]));
}

#[check(ghost)]
#[ensures('\u{20AC}'.to_utf8() == seq![0xE2u8, 0x82u8, 0xACu8])]
pub fn utf8_u20ac() {
    let encoded = snapshot!('\u{20AC}'.to_utf8());
    proof_assert!('\u{20AC}'@ == 8364);
    proof_assert!(encoded.len() == 3);
    proof_assert!(encoded[0]@ == 226);
    proof_assert!(encoded[1]@ == 130);
    proof_assert!(encoded[2]@ == 172);
    proof_assert!(encoded.ext_eq(seq![0xE2u8, 0x82u8, 0xACu8]));
}

#[check(ghost)]
#[ensures('\u{D7FF}'.to_utf8() == seq![0xEDu8, 0x9Fu8, 0xBFu8])]
pub fn utf8_u_d7ff() {
    let encoded = snapshot!('\u{D7FF}'.to_utf8());
    proof_assert!('\u{D7FF}'@ == 55295);
    proof_assert!(encoded.len() == 3);
    proof_assert!(encoded[0]@ == 237);
    proof_assert!(encoded[1]@ == 159);
    proof_assert!(encoded[2]@ == 191);
    proof_assert!(encoded.ext_eq(seq![0xEDu8, 0x9Fu8, 0xBFu8]));
}

#[check(ghost)]
#[ensures('\u{E000}'.to_utf8() == seq![0xEEu8, 0x80u8, 0x80u8])]
pub fn utf8_u_e000() {
    let encoded = snapshot!('\u{E000}'.to_utf8());
    proof_assert!('\u{E000}'@ == 57344);
    proof_assert!(encoded.len() == 3);
    proof_assert!(encoded[0]@ == 238);
    proof_assert!(encoded[1]@ == 128);
    proof_assert!(encoded[2]@ == 128);
    proof_assert!(encoded.ext_eq(seq![0xEEu8, 0x80u8, 0x80u8]));
}

#[check(ghost)]
#[ensures('\u{10FFFF}'.to_utf8() == seq![0xF4u8, 0x8Fu8, 0xBFu8, 0xBFu8])]
pub fn utf8_u10ffff() {
    let encoded = snapshot!('\u{10FFFF}'.to_utf8());
    proof_assert!('\u{10FFFF}'@ == 1114111);
    proof_assert!(encoded.len() == 4);
    proof_assert!(encoded[0]@ == 244);
    proof_assert!(encoded[1]@ == 143);
    proof_assert!(encoded[2]@ == 191);
    proof_assert!(encoded[3]@ == 191);
    proof_assert!(encoded.ext_eq(seq![0xF4u8, 0x8Fu8, 0xBFu8, 0xBFu8]));
}

#[logic]
#[ensures(Seq::singleton(c).to_bytes() == c.to_utf8())]
pub fn utf8_singleton(c: char) {
    Seq::flat_map_singleton(c, |x: char| x.to_utf8());
}

#[logic]
#[ensures(s.push_back(c).to_bytes() == s.to_bytes().concat(c.to_utf8()))]
pub fn utf8_push_back(s: Seq<char>, c: char) {
    s.flat_map_push_back(c, |x: char| x.to_utf8());
}
