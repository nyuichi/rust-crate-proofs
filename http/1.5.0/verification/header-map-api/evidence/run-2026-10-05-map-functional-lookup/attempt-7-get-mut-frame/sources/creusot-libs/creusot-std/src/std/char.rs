use crate::{ghost::Plain, prelude::*};

impl View for char {
    type ViewTy = Int;
    #[logic]
    #[builtin("creusot.prelude.Char.to_int")]
    fn view(self) -> Self::ViewTy {
        dead
    }
}

impl DeepModel for char {
    type DeepModelTy = Int;
    #[logic(open, inline)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@ }
    }
}

extern_spec! {
    impl Default for char {
        #[check(ghost)]
        #[ensures(result@ == 0)]
        fn default() -> char;
    }
}

impl Plain for char {
    #[trusted]
    #[ensures(*result == *snap)]
    #[check(ghost)]
    #[allow(unused_variables)]
    fn into_ghost(snap: Snapshot<Self>) -> Ghost<Self> {
        Ghost::conjure()
    }
}

/// Extra methods for `char`
pub trait CharExt {
    #[logic]
    pub fn to_utf8(self) -> Seq<u8>;
}

impl CharExt for char {
    #[logic(open)]
    #[ensures(1 <= result.len() && result.len() <= 4)]
    fn to_utf8(self) -> Seq<u8> {
        let code = pearlite! { self@ };
        pearlite! {
            if code < 0x80 {
                Seq::singleton(utf8_byte(code))
            } else if code < 0x800 {
                Seq::singleton(utf8_byte(0xC0 + code / 64))
                    .push_back(utf8_byte(0x80 + code % 64))
            } else if code < 0x10000 {
                Seq::singleton(utf8_byte(0xE0 + code / 4096))
                    .push_back(utf8_byte(0x80 + (code / 64) % 64))
                    .push_back(utf8_byte(0x80 + code % 64))
            } else {
                Seq::singleton(utf8_byte(0xF0 + code / 262144))
                    .push_back(utf8_byte(0x80 + (code / 4096) % 64))
                    .push_back(utf8_byte(0x80 + (code / 64) % 64))
                    .push_back(utf8_byte(0x80 + code % 64))
            }
        }
    }
}

/// Construct a byte from its mathematical value by successor steps.
///
/// Logic `Int` values cannot be cast back to machine integers. This recursive
/// constructor keeps that conversion explicit and proves the result's value.
#[logic]
#[requires(0 <= value && value <= 255)]
#[ensures(result@ == value)]
#[variant(value)]
#[doc(hidden)]
pub fn utf8_byte(value: Int) -> u8 {
    if value == 0 {
        0u8
    } else {
        proof_assert!(0 <= value - 1 && value - 1 <= 255);
        let previous = utf8_byte(value - 1);
        proof_assert!(previous@ == value - 1);
        proof_assert!(previous@ < 255);
        pearlite! { previous + 1u8 }
    }
}

#[trusted]
#[logic(open)]
#[ensures(forall<c1: char, c2: char> c1.to_utf8() == c2.to_utf8() ==> c1 == c2)]
pub fn injective_to_utf8() {}
