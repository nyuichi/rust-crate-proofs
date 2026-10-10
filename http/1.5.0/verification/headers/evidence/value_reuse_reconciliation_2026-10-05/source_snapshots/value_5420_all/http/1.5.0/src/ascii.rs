//! Verified bridge from ASCII byte sequences to Creusot's UTF-8 model.
//!
//! The finite byte-to-character table avoids an unchecked cast in logic; the
//! recursive law proves that the resulting `Seq<char>` encodes exactly the
//! supplied ASCII bytes.

#[cfg(creusot)]
#[allow(unused_imports)]
use creusot_std::prelude::{Seq, ensures, logic, pearlite, proof_assert, requires, variant};
#[cfg(creusot)]
#[allow(unused_imports)]
use creusot_std::std::char::CharExt as _;

/// Map an ASCII byte to the character with the same scalar value. Values above
/// ASCII use an irrelevant fallback; callers proving UTF-8 provide the bound.
#[cfg(creusot)]
#[logic(opaque)]
#[ensures(byte@ < 128 ==> result@ == byte@)]
pub(crate) fn ascii_char(byte: u8) -> char {
    pearlite! {
    if byte@ == 0 { '\u{0}' }
        else if byte@ == 1 { '\u{1}' }
        else if byte@ == 2 { '\u{2}' }
        else if byte@ == 3 { '\u{3}' }
        else if byte@ == 4 { '\u{4}' }
        else if byte@ == 5 { '\u{5}' }
        else if byte@ == 6 { '\u{6}' }
        else if byte@ == 7 { '\u{7}' }
        else if byte@ == 8 { '\u{8}' }
        else if byte@ == 9 { '\u{9}' }
        else if byte@ == 10 { '\u{a}' }
        else if byte@ == 11 { '\u{b}' }
        else if byte@ == 12 { '\u{c}' }
        else if byte@ == 13 { '\u{d}' }
        else if byte@ == 14 { '\u{e}' }
        else if byte@ == 15 { '\u{f}' }
        else if byte@ == 16 { '\u{10}' }
        else if byte@ == 17 { '\u{11}' }
        else if byte@ == 18 { '\u{12}' }
        else if byte@ == 19 { '\u{13}' }
        else if byte@ == 20 { '\u{14}' }
        else if byte@ == 21 { '\u{15}' }
        else if byte@ == 22 { '\u{16}' }
        else if byte@ == 23 { '\u{17}' }
        else if byte@ == 24 { '\u{18}' }
        else if byte@ == 25 { '\u{19}' }
        else if byte@ == 26 { '\u{1a}' }
        else if byte@ == 27 { '\u{1b}' }
        else if byte@ == 28 { '\u{1c}' }
        else if byte@ == 29 { '\u{1d}' }
        else if byte@ == 30 { '\u{1e}' }
        else if byte@ == 31 { '\u{1f}' }
        else if byte@ == 32 { '\u{20}' }
        else if byte@ == 33 { '\u{21}' }
        else if byte@ == 34 { '\u{22}' }
        else if byte@ == 35 { '\u{23}' }
        else if byte@ == 36 { '\u{24}' }
        else if byte@ == 37 { '\u{25}' }
        else if byte@ == 38 { '\u{26}' }
        else if byte@ == 39 { '\u{27}' }
        else if byte@ == 40 { '\u{28}' }
        else if byte@ == 41 { '\u{29}' }
        else if byte@ == 42 { '\u{2a}' }
        else if byte@ == 43 { '\u{2b}' }
        else if byte@ == 44 { '\u{2c}' }
        else if byte@ == 45 { '\u{2d}' }
        else if byte@ == 46 { '\u{2e}' }
        else if byte@ == 47 { '\u{2f}' }
        else if byte@ == 48 { '\u{30}' }
        else if byte@ == 49 { '\u{31}' }
        else if byte@ == 50 { '\u{32}' }
        else if byte@ == 51 { '\u{33}' }
        else if byte@ == 52 { '\u{34}' }
        else if byte@ == 53 { '\u{35}' }
        else if byte@ == 54 { '\u{36}' }
        else if byte@ == 55 { '\u{37}' }
        else if byte@ == 56 { '\u{38}' }
        else if byte@ == 57 { '\u{39}' }
        else if byte@ == 58 { '\u{3a}' }
        else if byte@ == 59 { '\u{3b}' }
        else if byte@ == 60 { '\u{3c}' }
        else if byte@ == 61 { '\u{3d}' }
        else if byte@ == 62 { '\u{3e}' }
        else if byte@ == 63 { '\u{3f}' }
        else if byte@ == 64 { '\u{40}' }
        else if byte@ == 65 { '\u{41}' }
        else if byte@ == 66 { '\u{42}' }
        else if byte@ == 67 { '\u{43}' }
        else if byte@ == 68 { '\u{44}' }
        else if byte@ == 69 { '\u{45}' }
        else if byte@ == 70 { '\u{46}' }
        else if byte@ == 71 { '\u{47}' }
        else if byte@ == 72 { '\u{48}' }
        else if byte@ == 73 { '\u{49}' }
        else if byte@ == 74 { '\u{4a}' }
        else if byte@ == 75 { '\u{4b}' }
        else if byte@ == 76 { '\u{4c}' }
        else if byte@ == 77 { '\u{4d}' }
        else if byte@ == 78 { '\u{4e}' }
        else if byte@ == 79 { '\u{4f}' }
        else if byte@ == 80 { '\u{50}' }
        else if byte@ == 81 { '\u{51}' }
        else if byte@ == 82 { '\u{52}' }
        else if byte@ == 83 { '\u{53}' }
        else if byte@ == 84 { '\u{54}' }
        else if byte@ == 85 { '\u{55}' }
        else if byte@ == 86 { '\u{56}' }
        else if byte@ == 87 { '\u{57}' }
        else if byte@ == 88 { '\u{58}' }
        else if byte@ == 89 { '\u{59}' }
        else if byte@ == 90 { '\u{5a}' }
        else if byte@ == 91 { '\u{5b}' }
        else if byte@ == 92 { '\u{5c}' }
        else if byte@ == 93 { '\u{5d}' }
        else if byte@ == 94 { '\u{5e}' }
        else if byte@ == 95 { '\u{5f}' }
        else if byte@ == 96 { '\u{60}' }
        else if byte@ == 97 { '\u{61}' }
        else if byte@ == 98 { '\u{62}' }
        else if byte@ == 99 { '\u{63}' }
        else if byte@ == 100 { '\u{64}' }
        else if byte@ == 101 { '\u{65}' }
        else if byte@ == 102 { '\u{66}' }
        else if byte@ == 103 { '\u{67}' }
        else if byte@ == 104 { '\u{68}' }
        else if byte@ == 105 { '\u{69}' }
        else if byte@ == 106 { '\u{6a}' }
        else if byte@ == 107 { '\u{6b}' }
        else if byte@ == 108 { '\u{6c}' }
        else if byte@ == 109 { '\u{6d}' }
        else if byte@ == 110 { '\u{6e}' }
        else if byte@ == 111 { '\u{6f}' }
        else if byte@ == 112 { '\u{70}' }
        else if byte@ == 113 { '\u{71}' }
        else if byte@ == 114 { '\u{72}' }
        else if byte@ == 115 { '\u{73}' }
        else if byte@ == 116 { '\u{74}' }
        else if byte@ == 117 { '\u{75}' }
        else if byte@ == 118 { '\u{76}' }
        else if byte@ == 119 { '\u{77}' }
        else if byte@ == 120 { '\u{78}' }
        else if byte@ == 121 { '\u{79}' }
        else if byte@ == 122 { '\u{7a}' }
        else if byte@ == 123 { '\u{7b}' }
        else if byte@ == 124 { '\u{7c}' }
        else if byte@ == 125 { '\u{7d}' }
        else if byte@ == 126 { '\u{7e}' }
        else if byte@ == 127 { '\u{7f}' }
        else { '\u{fffd}' }
    }
}

#[cfg(creusot)]
#[logic(open(crate))]
#[variant(bytes.len())]
#[ensures(result.len() == bytes.len())]
#[ensures(bytes.len() == 0 ==> result == Seq::empty())]
#[ensures(bytes.len() > 0 ==> result == ascii_chars(bytes.tail()).push_front(ascii_char(bytes[0])))]
pub(crate) fn ascii_chars(bytes: Seq<u8>) -> Seq<char> {
    pearlite! {
        if bytes.len() == 0 {
            Seq::empty()
        } else {
            ascii_chars(bytes.tail()).push_front(ascii_char(bytes[0]))
        }
    }
}

/// UTF-8 encoding distributes over adding one character at the front.
#[cfg(creusot)]
#[logic(open(crate))]
#[ensures(chars.push_front(character).to_bytes()
    == character.to_utf8().concat(chars.to_bytes()))]
pub(crate) fn utf8_cons(chars: Seq<char>, character: char) {
    let pushed = chars.push_front(character);
    proof_assert! { pushed.len() == chars.len() + 1 };
    proof_assert! { pushed.len() > 0 };
    proof_assert! { pushed[0] == character };
    proof_assert! { pushed.tail().len() == chars.len() };
    proof_assert! {
        forall<i> 0 <= i && i < chars.len() ==> pushed.tail()[i] == chars[i]
    };
    proof_assert! { pushed.tail() == chars };
    proof_assert! {
        pushed.to_bytes()
            == character.to_utf8().concat(pushed.tail().to_bytes())
    };
}

/// The empty character sequence encodes to the empty byte sequence.
#[cfg(creusot)]
#[logic(open(crate))]
#[ensures(Seq::<char>::empty().to_bytes() == Seq::<u8>::empty())]
pub(crate) fn utf8_empty() {}

/// Prove that a sequence with only ASCII bytes is valid UTF-8.
#[cfg(creusot)]
#[logic(open(crate))]
#[requires(forall<i> 0 <= i && i < bytes.len() ==> bytes[i]@ < 128)]
#[ensures(ascii_chars(bytes).to_bytes() == bytes)]
#[ensures(creusot_std::std::string::valid_utf8(bytes))]
#[variant(bytes.len())]
pub(crate) fn ascii_bytes_are_valid_utf8(bytes: Seq<u8>) {
    if bytes.len() > 0 {
        ascii_bytes_are_valid_utf8(bytes.tail());
        utf8_cons(ascii_chars(bytes.tail()), ascii_char(bytes[0]));
        proof_assert! {
            ascii_char(bytes[0]).to_utf8() == Seq::singleton(bytes[0])
        };
        proof_assert! {
            ascii_chars(bytes).to_bytes()
                == ascii_char(bytes[0]).to_utf8().concat(ascii_chars(bytes.tail()).to_bytes())
        };
        let reassembled = Seq::singleton(bytes[0]).concat(bytes.tail());
        proof_assert! { reassembled.len() == bytes.len() };
        proof_assert! {
            forall<i> 0 <= i && i < bytes.len() ==> reassembled[i] == bytes[i]
        };
        proof_assert! { reassembled == bytes };
    } else {
        utf8_empty();
    }
    proof_assert! { ascii_chars(bytes).to_bytes() == bytes };
    proof_assert! {
        exists<s: Seq<char>> s.to_bytes() == bytes
    };
}
