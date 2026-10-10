#![allow(dead_code, unexpected_cfgs)]

#[cfg(creusot)]
use creusot_std::logic::Seq;
#[cfg(creusot)]
use creusot_std::prelude::proof_assert;
#[cfg(creusot)]
use creusot_std::std::str_index::utf8::{prefix_at_ascii_byte, prefix_through_ascii_byte};
#[cfg(creusot)]
use creusot_std::std::char::CharExt;
use creusot_std::prelude::{ensures, requires};

/// Exercise byte offsets around a two-byte UTF-8 character (`é`).
/// The character-view precondition gives explicit valid boundary witnesses:
/// byte offsets 1 and 3 correspond to character indices 1 and 2.
#[requires(source@ == Seq::<char>::singleton('a').push_back('é').push_back('b'))]
#[ensures(result@ == Seq::<char>::singleton('é'))]
pub fn select_multibyte_character(source: &str) -> &str {
    proof_assert! { source@.subsequence(0, 0).to_bytes().len() == 0 };
    proof_assert! { source@.subsequence(0, 1).to_bytes().len() == 1 };
    proof_assert! { source@.subsequence(0, 2).to_bytes().len() == 3 };
    proof_assert! { 'b'.to_utf8().len() == 1 };
    proof_assert! { Seq::<char>::singleton('b').to_bytes().len() == 1 };
    proof_assert! {
        source@ == source@.subsequence(0, 2).concat(Seq::<char>::singleton('b'))
    };
    proof_assert! {
        creusot_std::std::str_index::utf8::concat(
            source@.subsequence(0, 2),
            Seq::<char>::singleton('b'),
        );
        source@.to_bytes().len() == 4
    };
    proof_assert! { source@.subsequence(0, 3) == source@ };
    proof_assert! { source@.subsequence(0, 3).to_bytes().len() == 4 };
    proof_assert! {
        forall<i> 0 <= i && i <= source@.len() ==>
            (source@.subsequence(0, i).to_bytes().len() == 1 ==> i == 1)
            && (source@.subsequence(0, i).to_bytes().len() == 3 ==> i == 2)
    };
    proof_assert! { source@.subsequence(1, 2) == Seq::<char>::singleton('é') };
    &source[1..3]
}

/// The full byte range has no UTF-8 boundary side condition and preserves the
/// complete character view.
#[ensures(result@ == source@)]
pub fn select_full_range(source: &str) -> &str {
    &source[..]
}

/// Exercise the URI path/query split boundaries around an ASCII `?` with
/// non-ASCII characters on both sides. The offsets are bytes: `é` takes two
/// bytes, `?` takes one, and the following pizza scalar takes four.
#[requires(source@ == Seq::<char>::singleton('é').push_back('?').push_back('🍕'))]
pub fn split_utf8_query_delimiter(source: &str) {
    proof_assert! { prefix_at_ascii_byte(source@, 2).0.to_bytes().len() == 2 };
    proof_assert! { prefix_at_ascii_byte(source@, 2).1[0] == '?' };
    let (_path, _query_with_delimiter) = source.split_at(2);

    proof_assert! { prefix_through_ascii_byte(source@, 2).0.to_bytes().len() == 3 };
    proof_assert! { prefix_through_ascii_byte(source@, 2).1 == Seq::<char>::singleton('🍕') };
    let (_path_and_query, _query) = source.split_at(3);
}
