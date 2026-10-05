use creusot_std::prelude::*;

/// Map every ASCII byte to the character with the same Unicode scalar value.
#[doc(hidden)]
#[logic(open)]
#[ensures(byte@ < 128 ==> result@ == byte@)]
pub fn ascii_byte_to_char(byte: u8) -> char {
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
        else { '\u{FFFD}' }
    }
}

/// An ASCII byte, interpreted as a Unicode scalar, has that byte's one-byte
/// UTF-8 encoding.
#[doc(hidden)]
#[logic(open)]
#[requires(byte@ < 128)]
#[ensures(ascii_byte_to_char(byte).to_utf8() == Seq::singleton(byte))]
pub fn ascii_byte_to_utf8(byte: u8) {
    let character = ascii_byte_to_char(byte);
    proof_assert!(character@ == byte@);
    proof_assert!(character.to_utf8() == Seq::singleton(byte));
}

/// Mapping an ASCII byte sequence to the corresponding characters and
/// encoding those characters as UTF-8 returns the original bytes.
#[doc(hidden)]
#[logic(open)]
#[requires(forall<i: Int> 0 <= i && i < bytes.len() ==> bytes[i]@ < 128)]
#[ensures(bytes.map(|byte: u8| ascii_byte_to_char(byte)).to_bytes() == bytes)]
#[variant(bytes.len())]
pub fn ascii_byte_map_to_utf8(bytes: Seq<u8>) {
    let chars = bytes.map(|byte: u8| ascii_byte_to_char(byte));

    if bytes.len() > 0 {
        let first = bytes[0];
        let tail = bytes.tail();
        let tail_chars = tail.map(|byte: u8| ascii_byte_to_char(byte));

        proof_assert!(first@ < 128);
        proof_assert!(forall<i: Int>
            0 <= i && i < tail.len() ==> tail[i]@ < 128);

        // `Seq::map` exposes its length and indexing facts through its
        // contract. Establish explicitly that its suffix is the map of the
        // input suffix before unfolding UTF-8 encoding over that suffix.
        proof_assert!(chars.len() == bytes.len());
        proof_assert!(tail_chars.len() == tail.len());
        proof_assert!(chars[0] == ascii_byte_to_char(first));
        proof_assert!(chars.tail().len() == tail_chars.len());
        proof_assert!(forall<i: Int>
            0 <= i && i < chars.tail().len() ==> chars.tail()[i] == tail_chars[i]);
        proof_assert!(chars.tail() == tail_chars);

        let _ = ascii_byte_to_utf8(first);
        let _ = ascii_byte_map_to_utf8(tail);

        let joined = Seq::singleton(first).concat(tail);
        proof_assert!(joined.len() == bytes.len());
        proof_assert!(forall<i: Int>
            0 <= i && i < joined.len() ==>
                (if i == 0 {
                    joined[i] == first
                } else {
                    joined[i] == tail[i - 1]
                }));
        proof_assert!(forall<i: Int>
            0 <= i && i < bytes.len() ==>
                (if i == 0 {
                    bytes[i] == first
                } else {
                    tail[i - 1] == bytes[i]
                }));
        proof_assert!(forall<i: Int>
            0 <= i && i < joined.len() ==> joined[i] == bytes[i]);
        proof_assert!(bytes == joined);
        proof_assert!(chars.to_bytes()
            == ascii_byte_to_char(first).to_utf8().concat(chars.tail().to_bytes()));
        proof_assert!(ascii_byte_to_char(first).to_utf8() == Seq::singleton(first));
        proof_assert!(tail_chars.to_bytes() == tail);
        proof_assert!(chars.tail().to_bytes() == tail_chars.to_bytes());
        proof_assert!(chars.to_bytes() == bytes);
    } else {
        proof_assert!(chars.len() == 0);
        proof_assert!(chars == Seq::empty());
        proof_assert!(bytes == Seq::empty());
        proof_assert!(chars.to_bytes() == Seq::empty());
        proof_assert!(chars.to_bytes() == bytes);
    }
}

/// ASCII bytes are valid UTF-8, witnessed by mapping each byte to the
/// character with the same scalar value.
#[doc(hidden)]
#[logic]
#[requires(forall<i: Int> 0 <= i && i < bytes.len() ==> bytes[i]@ < 128)]
#[ensures(exists<characters: Seq<char>> characters.to_bytes() == bytes)]
pub fn ascii_bytes_are_utf8(bytes: Seq<u8>) {
    let _ = ascii_byte_map_to_utf8(bytes);
    proof_assert!(exists<characters: Seq<char>>
        characters == bytes.map(|byte: u8| ascii_byte_to_char(byte))
            && characters.to_bytes() == bytes);
}
