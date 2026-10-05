//! Exact logical contract for byte-range indexing of UTF-8 strings.
//!
//! Rust's `str` range indexing uses byte offsets, and panics unless both
//! endpoints are in bounds and on UTF-8 character boundaries. The model below
//! records those boundary conditions as character indices in the `str` view.
//! It intentionally does not identify a byte offset with a character index.

use crate::prelude::*;
use core::ops::{Index, Range, RangeFull};

/// Proof lemmas connecting a string's character view to UTF-8 byte offsets.
///
/// These helpers are available only to Creusot clients. They describe the
/// mathematical UTF-8 model and do not add runtime code.
#[doc(hidden)]
pub mod utf8 {
    use crate::prelude::*;
    use crate::std::char::CharExt;

    /// Associativity of logical sequence concatenation, proved by equal
    /// lengths and pointwise equality of the resulting elements.
    #[logic(open)]
    #[ensures(first.concat(second).concat(third)
        == first.concat(second.concat(third)))]
    pub fn seq_concat_assoc<T>(first: Seq<T>, second: Seq<T>, third: Seq<T>) {
        let left = first.concat(second).concat(third);
        let right = first.concat(second.concat(third));
        proof_assert! { left.len() == right.len() };
        proof_assert! {
            forall<i> 0 <= i && i < left.len() ==> left[i] == right[i]
        };
        proof_assert! { left.ext_eq(right) };
        proof_assert! { left == right };
    }

    /// Decompose a nonempty logical sequence into its head and tail.
    #[logic(open)]
    #[requires(sequence.len() > 0)]
    #[ensures(sequence == Seq::singleton(sequence[0]).concat(sequence.tail()))]
    pub fn seq_head_tail<T>(sequence: Seq<T>) {
        let rebuilt = Seq::singleton(sequence[0]).concat(sequence.tail());
        proof_assert! { sequence.len() == rebuilt.len() };
        proof_assert! {
            forall<i> 0 <= i && i < sequence.len() ==> sequence[i] == rebuilt[i]
        };
        proof_assert! { sequence.ext_eq(rebuilt) };
        proof_assert! { sequence == rebuilt };
    }

    /// Decompose a concatenation at the first element of its nonempty left
    /// half. This exposes the first element and tail needed to unfold the
    /// recursive UTF-8 encoding definition.
    #[logic(open)]
    #[requires(left.len() > 0)]
    #[ensures(left.concat(right)
        == Seq::singleton(left[0]).concat(left.tail().concat(right)))]
    #[ensures((left.concat(right))[0] == left[0])]
    #[ensures((left.concat(right)).tail() == left.tail().concat(right))]
    pub fn seq_concat_head_tail<T>(left: Seq<T>, right: Seq<T>) {
        let head = left[0];
        let tail = left.tail();
        seq_head_tail(left);
        seq_concat_assoc(Seq::singleton(head), tail, right);

        let joined = left.concat(right);
        let joined_tail = tail.concat(right);
        let head_tail_form = Seq::singleton(head).concat(tail);
        let concatenated_form = head_tail_form.concat(right);
        let associative_form = Seq::singleton(head).concat(joined_tail);
        proof_assert! { left == head_tail_form };
        proof_assert! { joined == concatenated_form };
        proof_assert! { concatenated_form == associative_form };
        proof_assert! {
            joined == associative_form
        };
        proof_assert! { joined[0] == head };
        proof_assert! { joined.tail().len() == joined_tail.len() };
        proof_assert! {
            forall<i> 0 <= i && i < joined.tail().len()
                ==> joined.tail()[i] == joined_tail[i]
        };
        proof_assert! { joined.tail().ext_eq(joined_tail) };
        proof_assert! { joined.tail() == joined_tail };
    }

    /// Splitting a concatenated sequence at the left length recovers both
    /// exact halves. The proof uses each half's length and pointwise elements
    /// before invoking sequence extensionality.
    #[logic(open)]
    #[ensures(left.concat(right).subsequence(0, left.len()) == left)]
    #[ensures(left.concat(right).subsequence(left.len(),
        left.concat(right).len()) == right)]
    pub fn seq_concat_halves<T>(left: Seq<T>, right: Seq<T>) {
        let joined = left.concat(right);
        let joined_prefix = joined.subsequence(0, left.len());
        let joined_suffix = joined.subsequence(left.len(), joined.len());

        proof_assert! { joined_prefix.len() == left.len() };
        proof_assert! {
            forall<i> 0 <= i && i < joined_prefix.len()
                ==> joined_prefix[i] == left[i]
        };
        proof_assert! { joined_prefix.ext_eq(left) };
        proof_assert! { joined_prefix == left };

        proof_assert! { joined_suffix.len() == right.len() };
        proof_assert! {
            forall<i> 0 <= i && i < joined_suffix.len()
                ==> joined_suffix[i] == right[i]
        };
        proof_assert! { joined_suffix.ext_eq(right) };
        proof_assert! { joined_suffix == right };
    }

    /// Encoding a concatenation of character sequences concatenates their
    /// UTF-8 bytes and adds their byte lengths.
    #[logic(open)]
    #[variant(left.len())]
    #[ensures(left.concat(right).to_bytes()
        == left.to_bytes().concat(right.to_bytes()))]
    #[ensures(left.concat(right).to_bytes().len()
        == left.to_bytes().len() + right.to_bytes().len())]
    pub fn concat(left: Seq<char>, right: Seq<char>) {
        if left.len() == 0 {
            proof_assert! { left == Seq::<char>::empty() };
            proof_assert! { left.concat(right) == right };
            proof_assert! { Seq::<u8>::empty().concat(right.to_bytes()) == right.to_bytes() };
        } else {
            let head = left[0];
            let tail = left.tail();
            concat(tail, right);
            seq_concat_head_tail(left, right);
            proof_assert! {
                left.concat(right).to_bytes()
                    == head.to_utf8().concat(tail.concat(right).to_bytes())
            };
            proof_assert! {
                tail.concat(right).to_bytes()
                    == tail.to_bytes().concat(right.to_bytes())
            };
            proof_assert! {
                left.to_bytes() == head.to_utf8().concat(tail.to_bytes())
            };
            seq_concat_assoc(head.to_utf8(), tail.to_bytes(), right.to_bytes());
        }
    }

    /// UTF-8 encodes an ASCII character as its single byte. Every byte after
    /// the first byte of a non-ASCII character is a continuation byte.
    #[logic(open)]
    #[ensures(character@ < 128 ==> character.to_utf8().len() == 1)]
    #[ensures(character@ < 128 ==> character.to_utf8()[0]@ == character@)]
    #[ensures(character@ >= 128 ==> character.to_utf8()[0]@ >= 128)]
    #[ensures(character.to_utf8()[0]@ < 128 ==> character@ < 128)]
    #[ensures(character@ >= 128 ==> forall<i>
        0 < i && i < character.to_utf8().len()
            ==> character.to_utf8()[i]@ >= 128)]
    pub fn character_layout(character: char) {
        let code = pearlite! { character@ };
        if code < 128 {
            proof_assert! { character.to_utf8().len() == 1 };
            proof_assert! { character.to_utf8()[0]@ == code };
        } else if code < 2048 {
            proof_assert! { character.to_utf8()[0]@ >= 128 };
            proof_assert! {
                forall<i> 0 < i && i < character.to_utf8().len()
                    ==> character.to_utf8()[i]@ >= 128
            };
        } else if code < 65536 {
            proof_assert! { character.to_utf8()[0]@ >= 128 };
            proof_assert! {
                forall<i> 0 < i && i < character.to_utf8().len()
                    ==> character.to_utf8()[i]@ >= 128
            };
        } else {
            proof_assert! { character.to_utf8()[0]@ >= 128 };
            proof_assert! {
                forall<i> 0 < i && i < character.to_utf8().len()
                    ==> character.to_utf8()[i]@ >= 128
            };
        }
    }

    /// Split a character sequence at the beginning of an ASCII-encoded
    /// character, using a byte offset rather than a character index.
    ///
    /// The input may contain arbitrary Unicode before and after the selected
    /// byte. Since UTF-8 continuation bytes are non-ASCII, an ASCII byte can
    /// only occur at the first byte of its encoded character.
    #[logic(open)]
    #[variant(chars.len())]
    #[requires(0 <= offset && offset < chars.to_bytes().len())]
    #[requires(chars.to_bytes()[offset]@ < 128)]
    #[ensures(result.0.concat(result.1) == chars)]
    #[ensures(result.0 == chars.subsequence(0, result.0.len()))]
    #[ensures(result.1 == chars.subsequence(result.0.len(), chars.len()))]
    #[ensures(result.0.to_bytes().len() == offset)]
    #[ensures(result.0.to_bytes() == chars.to_bytes().subsequence(0, offset))]
    #[ensures(result.1.to_bytes()
        == chars.to_bytes().subsequence(offset, chars.to_bytes().len()))]
    #[ensures(result.1.len() > 0)]
    #[ensures(result.1[0]@ == chars.to_bytes()[offset]@)]
    #[ensures(result.1[0].to_utf8().len() == 1)]
    pub fn prefix_at_ascii_byte(
        chars: Seq<char>,
        offset: Int,
    ) -> (Seq<char>, Seq<char>) {
        if chars.len() == 0 {
            (Seq::empty(), Seq::empty())
        } else {
            let head = chars[0];
            let tail = chars.tail();
            let head_bytes = head.to_utf8();
            let head_len = head_bytes.len();

            if offset < head_len {
                character_layout(head);
                proof_assert! { chars.to_bytes() == head_bytes.concat(tail.to_bytes()) };
                proof_assert! {
                    chars.to_bytes()[offset] == head_bytes[offset]
                };
                proof_assert! { offset == 0 };
                proof_assert! { chars.to_bytes()[0] == head_bytes[0] };
                proof_assert! { head@ < 128 };
                proof_assert! { head_bytes.len() == 1 };
                proof_assert! { head_bytes[0]@ == head@ };
                let empty = Seq::<char>::empty();
                concat(empty, chars);
                seq_concat_halves(empty, chars);
                seq_concat_halves(empty.to_bytes(), chars.to_bytes());
                (empty, chars)
            } else {
                let tail_offset = offset - head_len;
                proof_assert! { chars.to_bytes() == head_bytes.concat(tail.to_bytes()) };
                proof_assert! {
                    chars.to_bytes().len() == head_len + tail.to_bytes().len()
                };
                proof_assert! {
                    0 <= tail_offset && tail_offset < tail.to_bytes().len()
                };
                proof_assert! {
                    chars.to_bytes()[offset] == tail.to_bytes()[tail_offset]
                };
                let split = prefix_at_ascii_byte(tail, tail_offset);
                concat(Seq::singleton(head), split.0);
                seq_head_tail(chars);
                seq_concat_assoc(Seq::singleton(head), split.0, split.1);
                proof_assert! { split.0.concat(split.1) == tail };
                let prefix = Seq::singleton(head).concat(split.0);
                let suffix = split.1;
                proof_assert! { chars == Seq::singleton(head).concat(tail) };
                proof_assert! { prefix.concat(suffix) == chars };
                seq_concat_halves(prefix, suffix);

                concat(prefix, suffix);
                proof_assert! {
                    chars.to_bytes() == prefix.to_bytes().concat(suffix.to_bytes())
                };
                seq_concat_halves(prefix.to_bytes(), suffix.to_bytes());
                proof_assert! {
                    split.0.to_bytes().len() == tail_offset
                };
                proof_assert! {
                    Seq::<char>::singleton(head).to_bytes().len() == head_len
                };
                proof_assert! {
                    prefix.to_bytes().len() == head_len + split.0.to_bytes().len()
                };
                proof_assert! { head_len + tail_offset == offset };
                (prefix, suffix)
            }
        }
    }

    /// Return the character prefix through an ASCII byte and the remaining
    /// suffix. This gives `str::split_at(offset + 1)` a boundary witness even
    /// when the next character is multibyte.
    #[logic(open)]
    #[requires(0 <= offset && offset < chars.to_bytes().len())]
    #[requires(chars.to_bytes()[offset]@ < 128)]
    #[ensures(result.0.concat(result.1) == chars)]
    #[ensures(result.0 == chars.subsequence(0, result.0.len()))]
    #[ensures(result.1 == chars.subsequence(result.0.len(), chars.len()))]
    #[ensures(result.0.to_bytes().len() == offset + 1)]
    #[ensures(result.0.to_bytes()
        == chars.to_bytes().subsequence(0, offset + 1))]
    #[ensures(result.1.to_bytes()
        == chars.to_bytes().subsequence(offset + 1, chars.to_bytes().len()))]
    pub fn prefix_through_ascii_byte(
        chars: Seq<char>,
        offset: Int,
    ) -> (Seq<char>, Seq<char>) {
        let split = prefix_at_ascii_byte(chars, offset);
        let delimiter = split.1[0];
        let delimiter_seq = Seq::singleton(delimiter);
        let prefix = split.0.concat(delimiter_seq);
        let suffix = split.1.tail();

        seq_head_tail(split.1);
        seq_concat_assoc(split.0, delimiter_seq, suffix);
        concat(split.0, delimiter_seq);
        proof_assert! { split.1 == delimiter_seq.concat(suffix) };
        proof_assert! { prefix.concat(suffix) == chars };
        seq_concat_halves(prefix, suffix);

        concat(prefix, suffix);
        proof_assert! {
            chars.to_bytes() == prefix.to_bytes().concat(suffix.to_bytes())
        };
        seq_concat_halves(prefix.to_bytes(), suffix.to_bytes());
        proof_assert! { delimiter.to_utf8().len() == 1 };
        proof_assert! { delimiter_seq.to_bytes().len() == 1 };
        proof_assert! { prefix.to_bytes().len() == offset + 1 };
        proof_assert! {
            prefix.to_bytes() == chars.to_bytes().subsequence(0, offset + 1)
        };
        proof_assert! {
            suffix.to_bytes()
                == chars.to_bytes().subsequence(offset + 1, chars.to_bytes().len())
        };

        (prefix, suffix)
    }
}

/// A range of byte offsets is valid for a string precisely when each endpoint
/// is the byte length of a character prefix and the endpoints are ordered.
pub trait StrSliceIndexSpec: core::slice::SliceIndex<str, Output = str> {
    #[logic]
    fn in_bounds(self, chars: Seq<char>) -> bool;

    #[logic]
    fn has_value(self, chars: Seq<char>, output: Seq<char>) -> bool;
}

impl StrSliceIndexSpec for Range<usize> {
    #[logic(open)]
    fn in_bounds(self, chars: Seq<char>) -> bool {
        pearlite! {
            exists<first, last>
                0 <= first && first <= last && last <= chars.len() &&
                chars.subsequence(0, first).to_bytes().len() == self.start@ &&
                chars.subsequence(0, last).to_bytes().len() == self.end@
        }
    }

    #[logic(open)]
    fn has_value(self, chars: Seq<char>, output: Seq<char>) -> bool {
        pearlite! {
            exists<first, last>
                0 <= first && first <= last && last <= chars.len() &&
                chars.subsequence(0, first).to_bytes().len() == self.start@ &&
                chars.subsequence(0, last).to_bytes().len() == self.end@ &&
                output == chars.subsequence(first, last)
        }
    }
}

impl StrSliceIndexSpec for RangeFull {
    #[logic(open, inline)]
    fn in_bounds(self, _chars: Seq<char>) -> bool {
        true
    }

    #[logic(open, inline)]
    fn has_value(self, chars: Seq<char>, output: Seq<char>) -> bool {
        pearlite! { output == chars }
    }
}

extern_spec! {
    impl<I: StrSliceIndexSpec> Index<I> for str {
        #[check(ghost)]
        #[requires(ix.in_bounds(self@))]
        #[ensures(ix.has_value(self@, result@))]
        fn index(&self, ix: I) -> &<str as Index<I>>::Output;
    }
}
