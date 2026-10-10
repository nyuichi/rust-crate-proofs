use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::{cmp, fmt, str};

use bytes::Bytes;

use super::{authority_chars, ErrorKind, InvalidUri, Port};
use crate::byte_str::ByteStr;

#[cfg(creusot)]
use creusot_std::{
    prelude::{inv, snapshot},
    std::ops::{FnExt as _, FnOnceExt as _},
    std::str_index::{
        utf8::{
            concat as utf8_concat, prefix_at_ascii_byte, prefix_through_ascii_byte,
            seq_concat_assoc as utf8_seq_concat_assoc,
            seq_concat_halves as utf8_seq_concat_halves,
        },
    },
};
#[cfg(creusot)]
use creusot_std::logic::OrdLogic;
#[cfg(creusot)]
use creusot_std::std::partial_eq::PartialEqModel as _;
#[cfg(creusot)]
use creusot_std::std::partial_ord::PartialOrdModel as _;

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, variant, DeepModel, Int, Seq,
    View,
};

/// Tests the authority path/query/fragment delimiter byte classes.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_delimiter(byte: Int) -> bool {
    pearlite! { byte == 47 || byte == 63 || byte == 35 }
}

/// Character and ASCII safety facts for a scanned authority prefix.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn authority_scan_prefix_is_valid(bytes: Seq<u8>, end: Int) -> bool {
    pearlite! {
        end <= bytes.len()
        && (end == bytes.len()
            || (end < bytes.len() && authority_delimiter(bytes[end]@)))
        && forall<j: Int> 0 <= j && j < end ==>
            !authority_delimiter(bytes[j]@)
                && (authority_chars::uri_char_model(bytes[j]@) != 0 || bytes[j]@ == 37)
                && bytes[j]@ < 128
    }
}

/// Number of colons since the most recent `@` or `]` in the prefix.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[variant(index)]
pub fn authority_colons_since_reset(bytes: Seq<u8>, index: Int) -> Int {
    pearlite! {
        if index == 0 { 0 }
        else if bytes[index - 1]@ == 64 || bytes[index - 1]@ == 93 { 0 }
        else if bytes[index - 1]@ == 58 {
            authority_colons_since_reset(bytes, index - 1) + 1
        } else { authority_colons_since_reset(bytes, index - 1) }
    }
}

/// Whether the prefix contains the requested byte.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[variant(index)]
pub fn authority_prefix_has_byte(bytes: Seq<u8>, index: Int, byte: Int) -> bool {
    pearlite! {
        if index == 0 { false }
        else { bytes[index - 1]@ == byte
            || authority_prefix_has_byte(bytes, index - 1, byte) }
    }
}

/// Whether a percent byte remains since the most recent `@` or `]`.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[variant(index)]
pub fn authority_percent_since_reset(bytes: Seq<u8>, index: Int) -> bool {
    pearlite! {
        if index == 0 { false }
        else if bytes[index - 1]@ == 64 || bytes[index - 1]@ == 93 { false }
        else { bytes[index - 1]@ == 37
            || authority_percent_since_reset(bytes, index - 1) }
    }
}

/// Index of the last `@` in a prefix, or the full input length if absent.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[variant(index)]
pub fn authority_last_at(bytes: Seq<u8>, index: Int) -> Int {
    pearlite! {
        if index == 0 { bytes.len() }
        else if bytes[index - 1]@ == 64 { index - 1 }
        else { authority_last_at(bytes, index - 1) }
    }
}

/// Index of the first requested byte at or after `index`, or `bytes.len()`.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[ensures(index <= result && result <= bytes.len())]
#[ensures(result == bytes.len() || bytes[result]@ == byte)]
#[ensures(forall<j: Int> index <= j && j < result ==> bytes[j]@ != byte)]
#[variant(bytes.len() - index)]
pub fn authority_first_byte_from(bytes: Seq<u8>, index: Int, byte: Int) -> Int {
    pearlite! {
        if index == bytes.len() || bytes[index]@ == byte {
            index
        } else {
            authority_first_byte_from(bytes, index + 1, byte)
        }
    }
}

/// Start of the host after the last `@` in a prefix.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[ensures(0 <= result && result <= index)]
#[ensures(result == 0 || bytes[result - 1]@ == 64)]
#[variant(index)]
pub fn authority_host_start_before(bytes: Seq<u8>, index: Int) -> Int {
    pearlite! {
        if index == 0 {
            0
        } else if bytes[index - 1]@ == 64 {
            index
        } else {
            authority_host_start_before(bytes, index - 1)
        }
    }
}

/// Start of the host after the last `@` in the full authority.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_host_start(bytes: Seq<u8>) -> Int {
    authority_host_start_before(bytes, bytes.len())
}

/// Exact panic-free input domain of `Authority::host`.
///
/// The existing implementation scans through the entire stored authority,
/// including any suffix retained by `from_static`. It panics for a missing
/// host byte or for a bracketed host without a later closing bracket.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_host_input_is_safe(bytes: Seq<u8>) -> bool {
    pearlite! {
        authority_host_start(bytes) < bytes.len()
            && (bytes[authority_host_start(bytes)]@ != 91
                || authority_first_byte_from(
                    bytes,
                    authority_host_start(bytes) + 1,
                    93,
                ) < bytes.len())
    }
}

/// End of the host slice selected by the existing byte-scanning behavior.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(authority_host_input_is_safe(bytes))]
pub fn authority_host_end(bytes: Seq<u8>) -> Int {
    pearlite! {
        if bytes[authority_host_start(bytes)]@ == 91 {
            authority_first_byte_from(
                bytes,
                authority_host_start(bytes) + 1,
                93,
            ) + 1
        } else {
            authority_first_byte_from(bytes, authority_host_start(bytes), 58)
        }
    }
}

/// UTF-8 bytes of a character subsequence are the corresponding byte range.
/// This is the bridge between `str` indexing's character model and the
/// authority scanner's byte offsets.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(0 <= first && first <= last && last <= chars.len())]
#[ensures(result)]
#[ensures(chars.subsequence(first, last).to_bytes()
    == chars.to_bytes().subsequence(
        chars.subsequence(0, first).to_bytes().len(),
        chars.subsequence(0, last).to_bytes().len(),
    ))]
pub(crate) fn authority_utf8_subsequence_bytes(
    chars: Seq<char>,
    first: Int,
    last: Int,
) -> bool {
    let prefix = chars.subsequence(0, first);
    let middle = chars.subsequence(first, last);
    let suffix = chars.subsequence(last, chars.len());
    let prefix_and_middle = prefix.concat(middle);

    proof_assert! { prefix.len() == first };
    proof_assert! { middle.len() == last - first };
    proof_assert! { suffix.len() == chars.len() - last };
    proof_assert! { prefix_and_middle.len() == last };
    proof_assert! {
        forall<i: Int> 0 <= i && i < first
            ==> prefix_and_middle[i] == prefix[i]
    };
    proof_assert! {
        forall<i: Int> first <= i && i < last
            ==> prefix_and_middle[i] == middle[i - first]
    };
    proof_assert! {
        forall<i: Int> 0 <= i && i < first ==> prefix[i] == chars[i]
    };
    proof_assert! {
        forall<i: Int> first <= i && i < last
            ==> middle[i - first] == chars[i]
    };
    proof_assert! {
        forall<i: Int> 0 <= i && i < last
            ==> prefix_and_middle[i] == chars[i]
    };
    proof_assert! { prefix_and_middle.ext_eq(chars.subsequence(0, last)) };
    proof_assert! { prefix_and_middle == chars.subsequence(0, last) };
    let whole = prefix_and_middle.concat(suffix);
    proof_assert! { whole.len() == chars.len() };
    proof_assert! {
        forall<i: Int> 0 <= i && i < last ==> whole[i] == prefix_and_middle[i]
    };
    proof_assert! {
        forall<i: Int> last <= i && i < chars.len()
            ==> whole[i] == suffix[i - last]
    };
    proof_assert! {
        forall<i: Int> last <= i && i < chars.len()
            ==> suffix[i - last] == chars[i]
    };
    proof_assert! {
        forall<i: Int> 0 <= i && i < chars.len() ==> whole[i] == chars[i]
    };
    proof_assert! { whole.ext_eq(chars) };
    proof_assert! { prefix_and_middle.concat(suffix) == chars };
    let whole = prefix_and_middle.concat(suffix);
    proof_assert! { whole.len() == chars.len() };
    proof_assert! {
        forall<i: Int> 0 <= i && i < last ==> whole[i] == prefix_and_middle[i]
    };
    proof_assert! {
        forall<i: Int> last <= i && i < chars.len()
            ==> whole[i] == suffix[i - last]
    };
    proof_assert! {
        forall<i: Int> last <= i && i < chars.len()
            ==> suffix[i - last] == chars[i]
    };
    proof_assert! {
        forall<i: Int> 0 <= i && i < chars.len() ==> whole[i] == chars[i]
    };
    proof_assert! { whole.ext_eq(chars) };
    proof_assert! { prefix_and_middle.concat(suffix) == chars };

    utf8_concat(prefix, middle);
    utf8_concat(prefix_and_middle, suffix);
    utf8_seq_concat_assoc(prefix.to_bytes(), middle.to_bytes(), suffix.to_bytes());

    let prefix_bytes = prefix.to_bytes();
    let middle_bytes = middle.to_bytes();
    let prefix_and_middle_bytes = prefix_and_middle.to_bytes();
    let suffix_bytes = suffix.to_bytes();
    let all_bytes = chars.to_bytes();

    proof_assert! {
        prefix_and_middle_bytes == prefix_bytes.concat(middle_bytes)
    };
    proof_assert! {
        all_bytes == prefix_and_middle_bytes.concat(suffix_bytes)
    };
    proof_assert! {
        prefix_and_middle_bytes.len() == prefix_bytes.len() + middle_bytes.len()
    };

    utf8_seq_concat_halves(prefix_bytes, middle_bytes);
    utf8_seq_concat_halves(prefix_and_middle_bytes, suffix_bytes);
    proof_assert! {
        all_bytes.subsequence(0, prefix_and_middle_bytes.len())
            == prefix_and_middle_bytes
    };
    proof_assert! {
        prefix_and_middle_bytes.subsequence(prefix_bytes.len(), prefix_and_middle_bytes.len())
            == middle_bytes
    };
    proof_assert! {
        authority_seq_subsequence_compose(
            all_bytes,
            0,
            prefix_and_middle_bytes.len(),
            prefix_bytes.len(),
            prefix_and_middle_bytes.len(),
        )
    };
    proof_assert! {
        all_bytes.subsequence(prefix_bytes.len(), prefix_and_middle_bytes.len())
            == middle_bytes
    };
    proof_assert! {
        chars.subsequence(first, last).to_bytes() == middle_bytes
    };
    true
}

/// Compose two bounded slices of the same sequence.
///
/// This keeps the character and byte boundary arguments explicit when a
/// prefix or a concatenation is sliced again.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(0 <= first && first <= last && last <= source.len())]
#[requires(0 <= inner_first && inner_first <= inner_last
    && inner_last <= last - first)]
#[ensures(result)]
#[ensures(source.subsequence(first, last).subsequence(inner_first, inner_last)
    == source.subsequence(first + inner_first, first + inner_last))]
pub(crate) fn authority_seq_subsequence_compose<T>(
    source: Seq<T>,
    first: Int,
    last: Int,
    inner_first: Int,
    inner_last: Int,
) -> bool {
    let nested = source.subsequence(first, last).subsequence(inner_first, inner_last);
    let direct = source.subsequence(first + inner_first, first + inner_last);
    proof_assert! { nested.len() == direct.len() };
    proof_assert! {
        forall<i: Int> 0 <= i && i < nested.len()
            ==> nested[i] == source[first + inner_first + i]
                && direct[i] == source[first + inner_first + i]
    };
    proof_assert! { nested.ext_eq(direct) };
    proof_assert! { nested == direct };
    true
}

/// Character prefix ending at a byte boundary selected by the host scanner.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(0 <= end && end <= chars.to_bytes().len())]
#[requires(if end_after_ascii {
    end > 0 && chars.to_bytes()[end - 1]@ < 128
} else {
    end == chars.to_bytes().len()
        || (end < chars.to_bytes().len() && chars.to_bytes()[end]@ < 128)
})]
#[ensures(result.len() <= chars.len())]
#[ensures(result == chars.subsequence(0, result.len()))]
#[ensures(result.to_bytes().len() == end)]
#[ensures(result.to_bytes() == chars.to_bytes().subsequence(0, end))]
pub fn authority_utf8_range_end_prefix(
    chars: Seq<char>,
    end: Int,
    end_after_ascii: bool,
) -> Seq<char> {
    if end_after_ascii {
        prefix_through_ascii_byte(chars, end - 1).0
    } else if end < chars.to_bytes().len() {
        prefix_at_ascii_byte(chars, end).0
    } else {
        chars
    }
}

/// Character prefix ending at the host scanner's start boundary.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(opaque)]
#[requires(0 <= start && start <= end && end <= chars.to_bytes().len())]
#[requires(start == 0 || chars.to_bytes()[start - 1]@ < 128)]
#[requires(if end_after_ascii {
    end > 0 && chars.to_bytes()[end - 1]@ < 128
} else {
    end == chars.to_bytes().len()
        || (end < chars.to_bytes().len() && chars.to_bytes()[end]@ < 128)
})]
#[ensures(result.len() <= authority_utf8_range_end_prefix(
    chars, end, end_after_ascii,
).len())]
#[ensures(result == authority_utf8_range_end_prefix(
    chars, end, end_after_ascii,
).subsequence(0, result.len()))]
#[ensures(result.to_bytes().len() == start)]
#[ensures(result.to_bytes() == chars.to_bytes().subsequence(0, start))]
#[ensures(result.len() <= chars.len())]
#[ensures(result == chars.subsequence(0, result.len()))]
pub fn authority_utf8_range_start_prefix(
    chars: Seq<char>,
    start: Int,
    end: Int,
    end_after_ascii: bool,
) -> Seq<char> {
    if start == 0 {
        Seq::empty()
    } else {
        let end_prefix = authority_utf8_range_end_prefix(chars, end, end_after_ascii);
        let result = prefix_through_ascii_byte(end_prefix, start - 1).0;
        proof_assert! {
            authority_seq_subsequence_compose(
                chars, 0, end_prefix.len(), 0, result.len(),
            )
        };
        proof_assert! { result == chars.subsequence(0, result.len()) };
        result
    }
}

/// Build ordered character-prefix witnesses for the host scanner's byte range.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(0 <= start && start <= end && end <= chars.to_bytes().len())]
#[requires(start == 0 || chars.to_bytes()[start - 1]@ < 128)]
#[requires(if end_after_ascii {
    end > 0 && chars.to_bytes()[end - 1]@ < 128
} else {
    end == chars.to_bytes().len()
        || (end < chars.to_bytes().len() && chars.to_bytes()[end]@ < 128)
})]
#[ensures(result)]
#[ensures(result == exists<first: Int, last: Int>
    first == authority_utf8_range_start_prefix(chars, start, end, end_after_ascii).len()
        && last == authority_utf8_range_end_prefix(chars, end, end_after_ascii).len()
        && 0 <= first && first <= last && last <= chars.len()
        && chars.subsequence(0, first).to_bytes().len() == start
        && chars.subsequence(0, last).to_bytes().len() == end)]
pub(crate) fn authority_utf8_range_is_in_bounds(
    chars: Seq<char>,
    start: Int,
    end: Int,
    end_after_ascii: bool,
) -> bool {
    let end_prefix = authority_utf8_range_end_prefix(chars, end, end_after_ascii);
    proof_assert! { end_prefix == chars.subsequence(0, end_prefix.len()) };
    proof_assert! { end_prefix.to_bytes().len() == end };

    let start_prefix = authority_utf8_range_start_prefix(
        chars, start, end, end_after_ascii,
    );
    proof_assert! {
        start_prefix == end_prefix.subsequence(0, start_prefix.len())
    };
    proof_assert! { start_prefix.to_bytes().len() == start };
    proof_assert! { start_prefix.len() <= end_prefix.len() };
    proof_assert! {
        forall<i: Int> 0 <= i && i < start_prefix.len()
            ==> start_prefix[i] == chars[i]
    };
    proof_assert! {
        start_prefix.ext_eq(chars.subsequence(0, start_prefix.len()))
    };
    proof_assert! {
        start_prefix == chars.subsequence(0, start_prefix.len())
    };

    proof_assert! {
        exists<first: Int, last: Int>
            first == start_prefix.len()
                && last == end_prefix.len()
                && 0 <= first && first <= last && last <= chars.len()
                && chars.subsequence(0, first).to_bytes().len() == start
                && chars.subsequence(0, last).to_bytes().len() == end
    };
    true
}

/// Lift the UTF-8 subsequence lemma through the external `str` range-index
/// contract, which identifies the selected character endpoints existentially.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(opaque)]
#[requires(0 <= start && start <= end && end <= chars.to_bytes().len())]
#[ensures(result)]
#[ensures(result == forall<output: Seq<char>>
    (exists<first: Int, last: Int>
        0 <= first && first <= last && last <= chars.len()
            && chars.subsequence(0, first).to_bytes().len() == start
            && chars.subsequence(0, last).to_bytes().len() == end
            && output == chars.subsequence(first, last))
        ==> output.to_bytes() == chars.to_bytes().subsequence(start, end))]
pub(crate) fn authority_utf8_range_output_bytes(
    chars: Seq<char>,
    start: Int,
    end: Int,
) -> bool {
    proof_assert! {
        forall<output: Seq<char>, first: Int, last: Int>
            0 <= first && first <= last && last <= chars.len()
                && chars.subsequence(0, first).to_bytes().len() == start
                && chars.subsequence(0, last).to_bytes().len() == end
                && output == chars.subsequence(first, last)
            ==> authority_utf8_subsequence_bytes(chars, first, last)
                && output.to_bytes()
                    == chars.to_bytes().subsequence(start, end)
    };
    true
}

/// Index of the first authority delimiter at or after `index`.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= bytes.len())]
#[ensures(index <= result && result <= bytes.len())]
#[ensures(result == bytes.len() || authority_delimiter(bytes[result]@))]
#[ensures(forall<j: Int> index <= j && j < result ==>
    !authority_delimiter(bytes[j]@))]
#[variant(bytes.len() - index)]
pub fn authority_first_delimiter_from(bytes: Seq<u8>, index: Int) -> Int {
    pearlite! {
        if index == bytes.len() || authority_delimiter(bytes[index]@) { index }
        else { authority_first_delimiter_from(bytes, index + 1) }
    }
}

/// Preserve a first-delimiter search across a range already known to contain
/// no delimiter. This is the loop-to-model bridge used by the byte scanner.
#[cfg(creusot)]
#[logic(open(crate))]
#[requires(0 <= start && start <= end && end <= bytes.len())]
#[requires(forall<j: Int> start <= j && j < end ==>
    !authority_delimiter(bytes[j]@))]
#[ensures(result)]
#[ensures(authority_first_delimiter_from(bytes, start)
    == authority_first_delimiter_from(bytes, end))]
#[variant(end - start)]
pub(crate) fn authority_first_delimiter_same_after_empty_prefix(
    bytes: Seq<u8>,
    start: Int,
    end: Int,
) -> bool {
    if start < end {
        proof_assert! { !authority_delimiter(bytes[start]@) };
        authority_first_delimiter_same_after_empty_prefix(bytes, start + 1, end);
        proof_assert! {
            authority_first_delimiter_from(bytes, start)
                == authority_first_delimiter_from(bytes, start + 1)
        };
    }
    true
}

/// Runtime scanner-step conditions for an accepted authority prefix.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn authority_prefix_steps_valid(bytes: Seq<u8>, end: Int) -> bool {
    pearlite! {
        forall<j: Int> 0 <= j && j < end ==>
            (bytes[j]@ == 58 ==> authority_colons_since_reset(bytes, j) < 8)
            && (bytes[j]@ == 91 ==>
                !authority_percent_since_reset(bytes, j)
                    && !authority_prefix_has_byte(bytes, j, 91))
            && (bytes[j]@ == 93 ==>
                authority_prefix_has_byte(bytes, j, 91)
                    && !authority_prefix_has_byte(bytes, j, 93))
    }
}

/// Complete scanner acceptance conditions for the prefix before a delimiter.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn authority_prefix_is_accepted(bytes: Seq<u8>, end: Int) -> bool {
    pearlite! {
        end <= bytes.len()
        && (end == bytes.len()
            || (end < bytes.len() && authority_delimiter(bytes[end]@)))
        && (forall<j: Int> 0 <= j && j < end ==>
            !authority_delimiter(bytes[j]@)
                && (authority_chars::uri_char_model(bytes[j]@) != 0 || bytes[j]@ == 37)
                && bytes[j]@ < 128)
        && authority_prefix_steps_valid(bytes, end)
        && authority_prefix_has_byte(bytes, end, 91)
            == authority_prefix_has_byte(bytes, end, 93)
        && authority_colons_since_reset(bytes, end) <= 1
        && !(end > 0 && authority_last_at(bytes, end) == end - 1)
        && !authority_percent_since_reset(bytes, end)
    }
}

/// State facts for a prefix which contains no delimiter or state-changing
/// authority character. This lets clients establish the accepted-prefix
/// predicate by checking the finite byte vocabulary once.
#[cfg(creusot)]
#[logic(open(crate))]
#[requires(0 <= end && end <= bytes.len())]
#[requires(forall<j: Int> 0 <= j && j < end ==>
    bytes[j]@ != 37 && bytes[j]@ != 58 && bytes[j]@ != 64
        && bytes[j]@ != 91 && bytes[j]@ != 93)]
#[ensures(result)]
#[ensures(authority_colons_since_reset(bytes, end) == 0)]
#[ensures(!authority_prefix_has_byte(bytes, end, 91))]
#[ensures(!authority_prefix_has_byte(bytes, end, 93))]
#[ensures(authority_last_at(bytes, end) == bytes.len())]
#[ensures(!authority_percent_since_reset(bytes, end))]
#[ensures(authority_prefix_steps_valid(bytes, end))]
#[variant(end)]
pub(crate) fn authority_neutral_prefix_states(bytes: Seq<u8>, end: Int) -> bool {
    if end > 0 {
        authority_neutral_prefix_states(bytes, end - 1);
        proof_assert! {
            authority_colons_since_reset(bytes, end)
                == authority_colons_since_reset(bytes, end - 1)
        };
        proof_assert! {
            authority_prefix_has_byte(bytes, end, 91)
                == authority_prefix_has_byte(bytes, end - 1, 91)
        };
        proof_assert! {
            authority_prefix_has_byte(bytes, end, 93)
                == authority_prefix_has_byte(bytes, end - 1, 93)
        };
        proof_assert! {
            authority_last_at(bytes, end) == authority_last_at(bytes, end - 1)
        };
        proof_assert! {
            authority_percent_since_reset(bytes, end)
                == authority_percent_since_reset(bytes, end - 1)
        };
    }
    true
}

/// Establish accepted-prefix state for a prefix whose bytes are all ordinary
/// ASCII URI characters, with no authority delimiter or state transition.
#[cfg(creusot)]
#[logic(open(crate))]
#[requires(0 <= end && end <= bytes.len())]
#[requires(end == bytes.len() || authority_delimiter(bytes[end]@))]
#[requires(forall<j: Int> 0 <= j && j < end ==>
    !authority_delimiter(bytes[j]@)
        && (authority_chars::uri_char_model(bytes[j]@) != 0 || bytes[j]@ == 37)
        && bytes[j]@ < 128)]
#[requires(forall<j: Int> 0 <= j && j < end ==>
    bytes[j]@ != 37 && bytes[j]@ != 58 && bytes[j]@ != 64
        && bytes[j]@ != 91 && bytes[j]@ != 93)]
#[ensures(result && authority_prefix_is_accepted(bytes, end))]
pub(crate) fn authority_neutral_prefix_is_accepted(bytes: Seq<u8>, end: Int) -> bool {
    authority_neutral_prefix_states(bytes, end);
    proof_assert! { authority_prefix_is_accepted(bytes, end) };
    true
}

/// Exact accepted-input domain of `Authority::from_static`.
///
/// The scanner accepts the prefix before the first `/`, `?`, or `#` and then
/// ignores the remaining bytes. This intentionally describes that behavior,
/// including its existing static-input quirks.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_static_input_is_valid(bytes: Seq<u8>) -> bool {
    pearlite! {
        bytes.len() > 0
            && authority_prefix_is_accepted(
                bytes,
                authority_first_delimiter_from(bytes, 0),
            )
    }
}

/// Whole-input acceptance predicate used by checked Authority conversions.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_input_is_fully_valid(bytes: Seq<u8>) -> bool {
    pearlite! {
        authority_static_input_is_valid(bytes)
            && authority_first_delimiter_from(bytes, 0) == bytes.len()
    }
}

/// Relates an Authority parse error to its rejected byte input and error class.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_error_matches_rejection(
    bytes: Seq<u8>,
    error: InvalidUri,
) -> bool {
    pearlite! {
        !authority_input_is_fully_valid(bytes)
            && ((error.deep_model() == 10 && bytes.len() == 0)
                || ((error.deep_model() == 0 || error.deep_model() == 2)
                    && bytes.len() > 0))
    }
}

/// Byte offset immediately after the last `:` in an authority, if present.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(0 <= index && index <= bytes.len())]
#[ensures(match result {
    Some(start) => 0 < start && start <= index && bytes[start - 1]@ == 58
        && forall<j: Int> start <= j && j < index ==> bytes[j]@ != 58,
    None => forall<j: Int> 0 <= j && j < index ==> bytes[j]@ != 58,
})]
#[variant(index)]
pub fn authority_port_start(bytes: Seq<u8>, index: Int) -> Option<Int> {
    pearlite! {
        if index == 0 {
            None
        } else if bytes[index - 1]@ == 58 {
            Some(index)
        } else {
            authority_port_start(bytes, index - 1)
        }
    }
}

/// Parsed port number represented by the suffix after the last `:`.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn authority_port_number(bytes: Seq<u8>) -> Option<Int> {
    pearlite! {
        match authority_port_start(bytes, bytes.len()) {
            Some(start) => creusot_std::std::string::parse_u16_model(
                bytes.subsequence(start, bytes.len()),
            ),
            None => None,
        }
    }
}

/// Find the byte offset after the last colon in a byte slice.
///
/// This is the byte-level equivalent of `str::rfind(':') + 1`: an ASCII colon
/// always occupies one byte, including in a UTF-8 string with multibyte text.
#[cfg_attr(creusot, ensures(match result {
    Some(start) => 0 < start@ && start@ <= bytes@.len()
        && bytes@[start@ - 1]@ == 58
        && forall<j: Int> start@ <= j && j < bytes@.len() ==> bytes@[j]@ != 58
        && authority_port_start(bytes@, bytes@.len()) == Some(start@),
    None => forall<j: Int> 0 <= j && j < bytes@.len() ==> bytes@[j]@ != 58
        && authority_port_start(bytes@, bytes@.len()) == None,
}))]
fn find_port_start(bytes: &[u8]) -> Option<usize> {
    let mut i = bytes.len();
    #[invariant(i@ <= bytes@.len())]
    #[invariant(forall<j: Int> i@ <= j && j < bytes@.len() ==> bytes@[j]@ != 58)]
    #[invariant(authority_port_start(bytes@, bytes@.len())
        == authority_port_start(bytes@, i@))]
    #[variant(i@)]
    while i > 0 {
        i -= 1;
        if bytes[i] == b':' {
            return Some(i + 1);
        }
    }

    None
}

/// Validation result for authority parsing.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(creusot), derive(PartialEq, Eq))]
enum AuthorityError {
    Empty,
    InvalidUriChar,
    InvalidAuthority,
    TooManyColons,
    MismatchedBrackets,
    InvalidBracketUsage,
    EmptyAfterAt,
    InvalidPercent,
}

/// Represents the authority component of a URI.
#[derive(Clone)]
pub struct Authority {
    pub(super) data: ByteStr,
}

#[cfg(creusot)]
impl View for Authority {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.data@ }
    }
}

#[cfg(creusot)]
impl DeepModel for Authority {
    type DeepModelTy = AuthorityCompareModel;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { AuthorityCompareModel { folded: authority_folded_bytes(self@) } }
    }
}

/// Case-folded comparison model for `Authority`. The public view continues to
/// expose the exact stored bytes; only the comparison model folds ASCII bytes.
#[cfg(creusot)]
#[doc(hidden)]
pub struct AuthorityCompareModel {
    pub folded: Seq<Int>,
}

/// Lowercase an ASCII byte in the authority comparison model.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn authority_fold_ascii_byte(byte: Int) -> Int {
    pearlite! { if 65 <= byte && byte <= 90 { byte + 32 } else { byte } }
}

/// Model the runtime's bytewise ASCII folding over UTF-8 storage.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn authority_folded_bytes(bytes: Seq<u8>) -> Seq<Int> {
    pearlite! { bytes.map(|byte: u8| authority_fold_ascii_byte(byte@)) }
}

/// Compare already-folded byte values lexicographically from `index`.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
#[requires(0 <= index && index <= left.len() && index <= right.len())]
#[variant(left.len() + right.len() - 2 * index)]
pub fn authority_folded_cmp_from(
    left: Seq<Int>,
    right: Seq<Int>,
    index: Int,
) -> core::cmp::Ordering {
    pearlite! {
        if index == left.len() || index == right.len() {
            left.len().cmp_log(right.len())
        } else {
            match left[index].cmp_log(right[index]) {
                core::cmp::Ordering::Less => core::cmp::Ordering::Less,
                core::cmp::Ordering::Equal =>
                    authority_folded_cmp_from(left, right, index + 1),
                core::cmp::Ordering::Greater => core::cmp::Ordering::Greater,
            }
        }
    }
}

/// Lexicographic case-insensitive comparison of folded authority bytes.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn authority_folded_cmp(left: Seq<Int>, right: Seq<Int>) -> core::cmp::Ordering {
    authority_folded_cmp_from(left, right, 0)
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<Seq<char>> for AuthorityCompareModel {
    #[logic(open)]
    fn eq_model(self, rhs: Seq<char>) -> bool {
        pearlite! { self.folded == authority_folded_bytes(rhs.to_bytes()) }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<AuthorityCompareModel> for Seq<char> {
    #[logic(open)]
    fn eq_model(self, rhs: AuthorityCompareModel) -> bool {
        pearlite! { authority_folded_bytes(self.to_bytes()) == rhs.folded }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<AuthorityCompareModel>
    for AuthorityCompareModel
{
    #[logic(open)]
    fn partial_cmp_model(self, rhs: AuthorityCompareModel) -> core::cmp::Ordering {
        authority_folded_cmp(self.folded, rhs.folded)
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<Seq<char>> for AuthorityCompareModel {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<char>) -> core::cmp::Ordering {
        authority_folded_cmp(self.folded, authority_folded_bytes(rhs.to_bytes()))
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<AuthorityCompareModel> for Seq<char> {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: AuthorityCompareModel) -> core::cmp::Ordering {
        authority_folded_cmp(authority_folded_bytes(self.to_bytes()), rhs.folded)
    }
}

#[cfg_attr(creusot, ensures(result@ == authority_fold_ascii_byte(byte@)))]
fn authority_ascii_lowercase_byte(byte: u8) -> u8 {
    if byte >= b'A' && byte <= b'Z' {
        byte + (b'a' - b'A')
    } else {
        byte
    }
}

#[cfg_attr(creusot, ensures(result == (
    authority_folded_bytes(left@) == authority_folded_bytes(right@)
)))]
fn authority_ascii_case_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        #[cfg(creusot)]
        proof_assert! {
            authority_folded_bytes(left@).len() != authority_folded_bytes(right@).len()
        };
        return false;
    }

    let mut index = 0;
    #[invariant(index@ <= left@.len())]
    #[invariant(index@ <= right@.len())]
    #[invariant(authority_folded_bytes(left@).subsequence(0, index@)
        == authority_folded_bytes(right@).subsequence(0, index@))]
    #[variant(left@.len() - index@)]
    while index < left.len() {
        let left_byte = authority_ascii_lowercase_byte(left[index]);
        let right_byte = authority_ascii_lowercase_byte(right[index]);
        if left_byte != right_byte {
            #[cfg(creusot)]
            {
                proof_assert! {
                    authority_folded_bytes(left@)[index@] == left_byte@
                };
                proof_assert! {
                    authority_folded_bytes(right@)[index@] == right_byte@
                };
                proof_assert! {
                    authority_folded_bytes(left@) != authority_folded_bytes(right@)
                };
            }
            return false;
        }
        index += 1;
    }

    #[cfg(creusot)]
    {
        proof_assert! { index@ == left@.len() };
        proof_assert! { left@.len() == right@.len() };
        proof_assert! {
            authority_folded_bytes(left@) == authority_folded_bytes(right@)
        };
    }
    true
}

#[cfg_attr(creusot, ensures(result == Some(
    authority_folded_cmp(authority_folded_bytes(left@), authority_folded_bytes(right@))
)))]
fn authority_ascii_case_cmp(left: &[u8], right: &[u8]) -> Option<cmp::Ordering> {
    #[cfg(creusot)]
    let left_folded = snapshot!(authority_folded_bytes(left@));
    #[cfg(creusot)]
    let right_folded = snapshot!(authority_folded_bytes(right@));
    let common_len = left.len().min(right.len());
    let mut index = 0;

    #[invariant(index@ <= left@.len())]
    #[invariant(index@ <= right@.len())]
    #[invariant(authority_folded_cmp_from(*left_folded, *right_folded, 0)
        == authority_folded_cmp_from(*left_folded, *right_folded, index@))]
    #[variant(common_len@ - index@)]
    while index < common_len {
        let left_byte = authority_ascii_lowercase_byte(left[index]);
        let right_byte = authority_ascii_lowercase_byte(right[index]);
        match left_byte.cmp(&right_byte) {
            cmp::Ordering::Less => return Some(cmp::Ordering::Less),
            cmp::Ordering::Greater => return Some(cmp::Ordering::Greater),
            cmp::Ordering::Equal => index += 1,
        }
    }

    Some(left.len().cmp(&right.len()))
}

impl Authority {
    #[cfg_attr(creusot, ensures(result@ == Seq::<u8>::empty()))]
    pub(super) fn empty() -> Self {
        Authority {
            data: ByteStr::new(),
        }
    }

    // Not public while `bytes` is unstable.
    #[cfg_attr(creusot, ensures(match result {
        Ok(authority) => authority_input_is_fully_valid(crate::bytes_model::bytes_seq(s))
            && authority@ == crate::bytes_model::bytes_seq(s),
        Err(error) => authority_error_matches_rejection(
            crate::bytes_model::bytes_seq(s), error,
        ),
    }))]
    pub(super) fn from_shared(s: Bytes) -> Result<Self, InvalidUri> {
        // Precondition on create_authority: trivially satisfied by the
        // identity closure
        create_authority(s, |s| s)
    }

    /// Attempt to convert an `Authority` from a static string.
    ///
    /// This function will not perform any copying, and the string will be
    /// checked if it is empty or contains an invalid character.
    ///
    /// # Panics
    ///
    /// This function panics if the argument contains invalid characters or
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::uri::Authority;
    /// let authority = Authority::from_static("example.com");
    /// assert_eq!(authority.host(), "example.com");
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(authority_static_input_is_valid(src@.to_bytes())))]
    #[cfg_attr(creusot, ensures(result@ == src@.to_bytes()))]
    pub const fn from_static(src: &'static str) -> Self {
        match validate_authority_bytes(src.as_bytes()) {
            Ok(_) => Authority {
                data: ByteStr::from_static(src),
            },
            Err(_) => panic!("static str is not valid authority"),
        }
    }

    /// Attempt to convert a `Bytes` buffer to a `Authority`.
    ///
    /// This will try to prevent a copy if the type passed is the type used
    /// internally, and will copy the data if it is not.
    // The sharing optimization relies on a dyn Any downcast, which Creusot
    // does not model. The validated parsing path is proved separately.
    #[cfg(not(http_uri_leaf))]
    pub fn from_maybe_shared<T>(src: T) -> Result<Self, InvalidUri>
    where
        T: AsRef<[u8]> + 'static,
    {
        if_downcast_into!(T, Bytes, src, {
            return Authority::from_shared(src);
        });

        Authority::try_from(src.as_ref())
    }

    // Note: this may return an *empty* Authority. You might want `parse_non_empty`.
    // Postcondition: for all Ok() returns, s[..ret.unwrap()] is valid UTF-8 where
    // ret is the return value.
    #[cfg_attr(creusot, ensures(match result {
        Ok(end) => s@.len() > 0 && 0 <= end@
            && authority_static_input_is_valid(s@)
            && authority_first_delimiter_from(s@, 0) == end@
            && authority_scan_prefix_is_valid(s@, end@),
        Err(error) => ((error.deep_model() == 10 && s@.len() == 0)
            || ((error.deep_model() == 0 || error.deep_model() == 2)
                && s@.len() > 0))
            && !authority_static_input_is_valid(s@),
    }))]
    pub(super) fn parse(s: &[u8]) -> Result<usize, InvalidUri> {
        match validate_authority_bytes(s) {
            Ok(end) => Ok(end),
            Err(error) => Err(match error {
                AuthorityError::Empty => ErrorKind::Empty,
                AuthorityError::InvalidUriChar => ErrorKind::InvalidUriChar,
                AuthorityError::InvalidAuthority
                | AuthorityError::MismatchedBrackets
                | AuthorityError::InvalidBracketUsage
                | AuthorityError::EmptyAfterAt
                | AuthorityError::InvalidPercent
                | AuthorityError::TooManyColons => ErrorKind::InvalidAuthority,
            }
            .into()),
        }
    }

    // Parse bytes as an Authority, not allowing an empty string.
    //
    // This should be used by functions that allow a user to parse
    // an `Authority` by itself.
    //
    // Postcondition: for all Ok() returns, s[..ret.unwrap()] is valid UTF-8 where
    // ret is the return value.
    #[cfg_attr(creusot, ensures(match result {
        Ok(end) => s@.len() > 0 && 0 <= end@
            && authority_static_input_is_valid(s@)
            && authority_first_delimiter_from(s@, 0) == end@
            && authority_scan_prefix_is_valid(s@, end@),
        Err(error) => ((error.deep_model() == 10 && s@.len() == 0)
            || ((error.deep_model() == 0 || error.deep_model() == 2)
                && s@.len() > 0))
            && !authority_static_input_is_valid(s@),
    }))]
    fn parse_non_empty(s: &[u8]) -> Result<usize, InvalidUri> {
        if s.len() == 0 {
            return Err(ErrorKind::Empty.into());
        }
        Authority::parse(s)
    }

    /// Get the host of this `Authority`.
    ///
    /// The host subcomponent of authority is identified by an IP literal
    /// encapsulated within square brackets, an IPv4 address in dotted- decimal
    /// form, or a registered name.  The host subcomponent is **case-insensitive**.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///                         |---------|
    ///                              |
    ///                             host
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::uri::*;
    /// let authority: Authority = "example.org:80".parse().unwrap();
    ///
    /// assert_eq!(authority.host(), "example.org");
    /// ```
    #[inline]
    #[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_host_leaf))]
    #[cfg_attr(creusot, requires(authority_host_input_is_safe(self@)))]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self@.subsequence(
        authority_host_start(self@), authority_host_end(self@),
    )))]
    pub fn host(&self) -> &str {
        host(self.as_str())
    }

    /// Get the port part of this `Authority`.
    ///
    /// The port subcomponent of authority is designated by an optional port
    /// number following the host and delimited from it by a single colon (":")
    /// character. It can be turned into a decimal port number with the `as_u16`
    /// method or as a `str` with the `as_str` method.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///                                     |-|
    ///                                      |
    ///                                     port
    /// ```
    ///
    /// # Examples
    ///
    /// Authority with port
    ///
    /// ```
    /// # use http::uri::Authority;
    /// let authority: Authority = "example.org:80".parse().unwrap();
    ///
    /// let port = authority.port().unwrap();
    /// assert_eq!(port.as_u16(), 80);
    /// assert_eq!(port.as_str(), "80");
    /// ```
    ///
    /// Authority without port
    ///
    /// ```
    /// # use http::uri::Authority;
    /// let authority: Authority = "example.org".parse().unwrap();
    ///
    /// assert!(authority.port().is_none());
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(port) => exists<start: Int>
            authority_port_start(self@, self@.len()) == Some(start)
                && (*port.repr_model())@.to_bytes()
                    == self@.subsequence(start, self@.len())
                && authority_port_number(self@) == Some(port@@),
        None => authority_port_number(self@) == None,
    }))]
    pub fn port(&self) -> Option<Port<&str>> {
        let authority = self.as_str();
        match find_port_start(authority.as_bytes()) {
            Some(start) => {
                #[cfg(creusot)]
                {
                    proof_assert! { authority@.to_bytes() == self@ };
                    proof_assert! {
                        authority_port_start(
                            authority@.to_bytes(), authority@.to_bytes().len(),
                        ) == Some(start@)
                    };
                    proof_assert! {
                        0 <= start@ - 1
                            && start@ - 1 < authority@.to_bytes().len()
                    };
                    proof_assert! {
                        authority@.to_bytes()[start@ - 1]@ == 58
                    };
                    proof_assert! {
                        prefix_through_ascii_byte(authority@, start@ - 1)
                            .0.to_bytes().len() == start@
                    };
                }
                let (_prefix, suffix) = authority.split_at(start);
                #[cfg(creusot)]
                {
                    proof_assert! { _prefix@.to_bytes().len() == start@ };
                    proof_assert! { _prefix@.concat(suffix@) == authority@ };
                    proof_assert! {
                        suffix@ == authority@.subsequence(
                            _prefix@.len(), authority@.len(),
                        )
                    };
                    proof_assert! {
                        authority_utf8_subsequence_bytes(
                            authority@, _prefix@.len(), authority@.len(),
                        )
                    };
                    proof_assert! {
                        suffix@.to_bytes()
                            == authority@.to_bytes().subsequence(
                                start@, authority@.to_bytes().len(),
                            )
                    };
                    proof_assert! {
                        authority_port_number(self@)
                            == creusot_std::std::string::parse_u16_model(
                                suffix@.to_bytes(),
                            )
                    };
                }
                match Port::from_str(suffix) {
                    Ok(port) => {
                        #[cfg(creusot)]
                        proof_assert! {
                            authority_port_number(self@) == Some(port@@)
                        };
                        Some(port)
                    }
                    Err(_) => None,
                }
            }
            None => None,
        }
    }

    /// Get the port of this `Authority` as a `u16`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::uri::Authority;
    /// let authority: Authority = "example.org:80".parse().unwrap();
    ///
    /// assert_eq!(authority.port_u16(), Some(80));
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(port) => authority_port_number(self@) == Some(port@),
        None => authority_port_number(self@) == None,
    }))]
    pub fn port_u16(&self) -> Option<u16> {
        self.port().map(|p| p.as_u16())
    }

    /// Return a str representation of the authority
    #[inline]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self@))]
    pub fn as_str(&self) -> &str {
        &*self.data
    }
}

// Purposefully not public while `bytes` is unstable.
// impl TryFrom<Bytes> for Authority

impl AsRef<str> for Authority {
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self@))]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for Authority {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &Authority) -> bool {
        authority_ascii_case_eq(self.data.as_bytes(), other.data.as_bytes())
    }
}

impl Eq for Authority {}

/// Case-insensitive equality
///
/// # Examples
///
/// ```
/// # use http::uri::Authority;
/// let authority: Authority = "HELLO.com".parse().unwrap();
/// assert_eq!(authority, "hello.coM");
/// assert_eq!("hello.com", authority);
/// ```
impl PartialEq<str> for Authority {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &str) -> bool {
        authority_ascii_case_eq(self.data.as_bytes(), other.as_bytes())
    }
}

impl PartialEq<Authority> for str {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &Authority) -> bool {
        authority_ascii_case_eq(self.as_bytes(), other.data.as_bytes())
    }
}

impl PartialEq<Authority> for &str {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &Authority) -> bool {
        authority_ascii_case_eq(self.as_bytes(), other.data.as_bytes())
    }
}

impl PartialEq<&str> for Authority {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &&str) -> bool {
        authority_ascii_case_eq(self.data.as_bytes(), other.as_bytes())
    }
}

impl PartialEq<String> for Authority {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &String) -> bool {
        authority_ascii_case_eq(self.data.as_bytes(), other.as_bytes())
    }
}

impl PartialEq<Authority> for String {
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &Authority) -> bool {
        authority_ascii_case_eq(self.as_bytes(), other.data.as_bytes())
    }
}

/// Case-insensitive ordering
///
/// # Examples
///
/// ```
/// # use http::uri::Authority;
/// let authority: Authority = "DEF.com".parse().unwrap();
/// assert!(authority < "ghi.com");
/// assert!(authority > "abc.com");
/// ```
#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd for Authority {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &Authority) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.data.as_bytes(), other.data.as_bytes())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd<str> for Authority {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.data.as_bytes(), other.as_bytes())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd<Authority> for str {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &Authority) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.as_bytes(), other.data.as_bytes())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd<Authority> for &str {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &Authority) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.as_bytes(), other.data.as_bytes())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd<&str> for Authority {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &&str) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.data.as_bytes(), other.as_bytes())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd<String> for Authority {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.data.as_bytes(), other.as_bytes())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_authority_compare_leaf))]
impl PartialOrd<Authority> for String {
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &Authority) -> Option<cmp::Ordering> {
        authority_ascii_case_cmp(self.as_bytes(), other.data.as_bytes())
    }
}

/// Case-insensitive hashing
///
/// # Examples
///
/// ```
/// # use http::uri::Authority;
/// # use std::hash::{Hash, Hasher};
/// # use std::collections::hash_map::DefaultHasher;
///
/// let a: Authority = "HELLO.com".parse().unwrap();
/// let b: Authority = "hello.coM".parse().unwrap();
///
/// let mut s = DefaultHasher::new();
/// a.hash(&mut s);
/// let a = s.finish();
///
/// let mut s = DefaultHasher::new();
/// b.hash(&mut s);
/// let b = s.finish();
///
/// assert_eq!(a, b);
/// ```
impl Hash for Authority {
    #[cfg_attr(creusot, ensures(inv(^state)))]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.data.len().hash(state);
        for &b in self.data.as_bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
    }
}

impl TryFrom<&[u8]> for Authority {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(authority) => authority_input_is_fully_valid(s@)
            && authority@ == s@,
        Err(error) => authority_error_matches_rejection(s@, error),
    }))]
    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        // parse first, and only turn into Bytes if valid

        // Preconditon on create_authority: copy_from_slice() copies all of
        // bytes from the [u8] parameter into a new Bytes
        create_authority(s, Bytes::copy_from_slice)
    }
}

impl TryFrom<&str> for Authority {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(authority) => authority_input_is_fully_valid(s@.to_bytes())
            && authority@ == s@.to_bytes(),
        Err(error) => authority_error_matches_rejection(s@.to_bytes(), error),
    }))]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        TryFrom::try_from(s.as_bytes())
    }
}

impl TryFrom<Vec<u8>> for Authority {
    type Error = InvalidUri;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(authority) => authority_input_is_fully_valid(vec@)
            && authority@ == vec@,
        Err(error) => authority_error_matches_rejection(vec@, error),
    }))]
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Authority::from_shared(vec.into())
    }
}

impl TryFrom<String> for Authority {
    type Error = InvalidUri;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(authority) => authority_input_is_fully_valid(t@.to_bytes())
            && authority@ == t@.to_bytes(),
        Err(error) => authority_error_matches_rejection(t@.to_bytes(), error),
    }))]
    fn try_from(t: String) -> Result<Self, Self::Error> {
        Authority::from_shared(t.into())
    }
}

impl FromStr for Authority {
    type Err = InvalidUri;

    #[cfg_attr(creusot, ensures(match result {
        Ok(authority) => authority_input_is_fully_valid(s@.to_bytes())
            && authority@ == s@.to_bytes(),
        Err(error) => authority_error_matches_rejection(s@.to_bytes(), error),
    }))]
    fn from_str(s: &str) -> Result<Self, InvalidUri> {
        TryFrom::try_from(s)
    }
}

impl fmt::Debug for Authority {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for Authority {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(any(not(http_uri_authority_scanner_leaf), http_uri_host_leaf))]
#[cfg_attr(creusot, requires(authority_host_input_is_safe(auth@.to_bytes())))]
#[cfg_attr(creusot, ensures(result@.to_bytes() == auth@.to_bytes().subsequence(
    authority_host_start(auth@.to_bytes()), authority_host_end(auth@.to_bytes()),
)))]
fn host(auth: &str) -> &str {
    let bytes = auth.as_bytes();
    let mut host_start = 0;
    let mut i = 0;
    #[invariant(i@ <= bytes@.len())]
    #[invariant(host_start@ == authority_host_start_before(bytes@, i@))]
    #[variant(bytes@.len() - i@)]
    while i < bytes.len() {
        if bytes[i] == b'@' {
            host_start = i + 1;
        }
        i += 1;
    }

    #[cfg(creusot)]
    if host_start > 0 {
        proof_assert! {
            prefix_through_ascii_byte(auth@, host_start@ - 1)
                .0.to_bytes().len() == host_start@
        };
    }
    #[cfg(creusot)]
    proof_assert! {
        host_start@ == authority_host_start(auth@.to_bytes())
    };

    if bytes[host_start] == b'[' {
        let mut host_end = host_start + 1;
        #[invariant(host_start@ + 1 <= host_end@)]
        #[invariant(host_end@ <= bytes@.len())]
        #[invariant(host_end@ <= authority_first_byte_from(
            bytes@, host_start@ + 1, 93,
        ))]
        #[invariant(forall<j: Int> host_start@ + 1 <= j && j < host_end@ ==>
            bytes@[j]@ != 93)]
        #[variant(bytes@.len() - host_end@)]
        while host_end < bytes.len() && bytes[host_end] != b']' {
            host_end += 1;
        }
        #[cfg(creusot)]
        {
            proof_assert! {
                host_end@ == authority_first_byte_from(
                    bytes@, host_start@ + 1, 93,
                )
            };
            proof_assert! {
                host_end@ + 1 == authority_host_end(auth@.to_bytes())
            };
            proof_assert! { host_end@ < bytes@.len() };
            proof_assert! {
                prefix_through_ascii_byte(auth@, host_end@)
                    .0.to_bytes().len() == host_end@ + 1
            };
            proof_assert! {
                host_start@ == 0 || bytes@[host_start@ - 1]@ == 64
            };
            proof_assert! {
                bytes@[host_end@]@ == 93
            };
            proof_assert! {
                authority_utf8_range_is_in_bounds(
                    auth@, host_start@, host_end@ + 1, true,
                )
            };
        }
        if host_end == bytes.len() {
            panic!("parsing should validate brackets");
        }
        // ..= ranges aren't available in 1.20, our minimum Rust version...
        let host = &auth[host_start..host_end + 1];
        #[cfg(creusot)]
        {
            proof_assert! { authority_utf8_range_output_bytes(
                auth@,
                host_start@,
                host_end@ + 1,
            ) };
            proof_assert! {
                exists<first: Int, last: Int>
                    0 <= first && first <= last && last <= auth@.len()
                        && auth@.subsequence(0, first).to_bytes().len() == host_start@
                        && auth@.subsequence(0, last).to_bytes().len() == host_end@ + 1
                        && host@ == auth@.subsequence(first, last)
            };
            proof_assert! {
                host@.to_bytes()
                    == auth@.to_bytes().subsequence(host_start@, host_end@ + 1)
            };
        }
        host
    } else {
        let mut host_end = host_start;
        #[invariant(host_start@ <= host_end@)]
        #[invariant(host_end@ <= bytes@.len())]
        #[invariant(host_end@ <= authority_first_byte_from(bytes@, host_start@, 58))]
        #[invariant(forall<j: Int> host_start@ <= j && j < host_end@ ==>
            bytes@[j]@ != 58)]
        #[variant(bytes@.len() - host_end@)]
        while host_end < bytes.len() && bytes[host_end] != b':' {
            host_end += 1;
        }
        #[cfg(creusot)]
        {
            proof_assert! {
                host_end@ == authority_first_byte_from(bytes@, host_start@, 58)
            };
            proof_assert! {
                host_end@ == authority_host_end(auth@.to_bytes())
            };
            if host_end < bytes.len() {
                proof_assert! {
                    prefix_at_ascii_byte(auth@, host_end@)
                        .0.to_bytes().len() == host_end@
                };
            }
            proof_assert! {
                host_start@ == 0 || bytes@[host_start@ - 1]@ == 64
            };
            proof_assert! {
                host_end@ == bytes@.len() || bytes@[host_end@]@ == 58
            };
            proof_assert! {
                authority_utf8_range_is_in_bounds(
                    auth@, host_start@, host_end@, false,
                )
            };
        }
        let host = &auth[host_start..host_end];
        #[cfg(creusot)]
        {
            proof_assert! {
                authority_utf8_range_output_bytes(auth@, host_start@, host_end@)
            };
            proof_assert! {
                exists<first: Int, last: Int>
                    0 <= first && first <= last && last <= auth@.len()
                        && auth@.subsequence(0, first).to_bytes().len() == host_start@
                        && auth@.subsequence(0, last).to_bytes().len() == host_end@
                        && host@ == auth@.subsequence(first, last)
            };
            proof_assert! {
                host@.to_bytes()
                    == auth@.to_bytes().subsequence(host_start@, host_end@)
            };
        }
        host
    }
}

// Precondition: f converts all of the bytes in the passed in B into the
// returned Bytes.
// The explicit closure relation below gives that premise a caller-checkable
// form for the conversions that feed this helper.
#[cfg_attr(creusot, requires(<B as AsRef<[u8]>>::as_ref.precondition((&b,))))]
#[cfg_attr(creusot, requires(forall<source: &[u8]>
    <B as AsRef<[u8]>>::as_ref.postcondition((&b,), source)
        && authority_input_is_fully_valid(source@)
        ==> f.precondition((b,))))]
#[cfg_attr(creusot, requires(forall<source: &[u8], output: Bytes>
    <B as AsRef<[u8]>>::as_ref.postcondition((&b,), source)
        && authority_input_is_fully_valid(source@)
        && f.postcondition_once((b,), output)
        ==> crate::bytes_model::bytes_seq(output) == source@))]
#[cfg_attr(creusot, ensures(exists<source: &[u8]>
    <B as AsRef<[u8]>>::as_ref.postcondition((&b,), source)
        && (match result {
            Ok(authority) => authority_input_is_fully_valid(source@)
                && authority@ == source@,
            Err(error) => authority_error_matches_rejection(source@, error),
        })))]
fn create_authority<B, F>(b: B, f: F) -> Result<Authority, InvalidUri>
where
    B: AsRef<[u8]>,
    F: FnOnce(B) -> Bytes,
{
    let s = b.as_ref();
    #[cfg(creusot)]
    let source = snapshot!(s@);
    let authority_end = Authority::parse_non_empty(s)?;

    if authority_end != s.len() {
        #[cfg(creusot)]
        proof_assert! { !authority_input_is_fully_valid(*source) };
        return Err(ErrorKind::InvalidUriChar.into());
    }

    #[cfg(creusot)]
    {
        proof_assert! { authority_input_is_fully_valid(*source) };
        proof_assert! {
            forall<j: Int> 0 <= j && j < (*source).len() ==> (*source)[j]@ < 128
        };
        proof_assert! {
            crate::ascii::ascii_bytes_are_valid_utf8(*source);
            creusot_std::std::string::valid_utf8(*source)
        };
    }

    let bytes = f(b);

    #[cfg(creusot)]
    {
        proof_assert! { crate::bytes_model::bytes_seq(bytes) == *source };
        proof_assert! {
            creusot_std::std::string::valid_utf8(crate::bytes_model::bytes_seq(bytes))
        };
    }

    Ok(Authority {
        // Safety: the postcondition on parse_non_empty() and the check against
        // s.len() ensure that b is valid UTF-8. The precondition on f ensures
        // that this is carried through to bytes.
        data: unsafe { ByteStr::from_utf8_unchecked(bytes) },
    })
}

/// Shared validation logic for authority bytes.
/// Returns the end position of valid authority bytes, or an error.
#[cfg_attr(creusot, ensures(match result {
    Ok(end) => authority_static_input_is_valid(s@)
        && authority_first_delimiter_from(s@, 0) == end@
        && 0 <= end@ && authority_scan_prefix_is_valid(s@, end@),
    Err(AuthorityError::Empty) => s@.len() == 0,
    Err(_) => s@.len() > 0 && !authority_static_input_is_valid(s@),
}))]
const fn validate_authority_bytes(s: &[u8]) -> Result<usize, AuthorityError> {
    if s.len() == 0 {
        return Err(AuthorityError::Empty);
    }

    let mut colon_cnt: u32 = 0;
    let mut start_bracket = false;
    let mut end_bracket = false;
    let mut has_percent = false;
    let mut end = s.len();
    let mut at_sign_pos: usize = s.len();
    const MAX_COLONS: u32 = 8; // e.g., [FEDC:BA98:7654:3210:FEDC:BA98:7654:3210]:80

    let mut i = 0;
    // Among other things, this loop checks that every byte in s up to the
    // first '/', '?', or '#' is a valid URI character (or in some contexts,
    // a '%'). This means that each such byte is a valid single-byte UTF-8
    // code point.
    #[invariant(i@ <= s@.len())]
    #[invariant(end@ <= s@.len())]
    #[invariant(end@ == s@.len() || end@ == i@)]
    #[invariant(end@ == s@.len() || authority_delimiter(s@[end@]@))]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==>
        !authority_delimiter(s@[j]@))]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==>
        authority_chars::uri_char_model(s@[j]@) != 0 || s@[j]@ == 37)]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==> s@[j]@ < 128)]
    #[invariant(authority_prefix_steps_valid(s@, i@))]
    #[invariant(colon_cnt@ == authority_colons_since_reset(s@, i@))]
    #[invariant(start_bracket == authority_prefix_has_byte(s@, i@, 91))]
    #[invariant(end_bracket == authority_prefix_has_byte(s@, i@, 93))]
    #[invariant(has_percent == authority_percent_since_reset(s@, i@))]
    #[invariant(at_sign_pos@ == authority_last_at(s@, i@))]
    #[variant(s@.len() - i@)]
    while i < s.len() {
        let b = s[i];
        let ch = authority_chars::uri_char(b);

        if ch == b'/' || ch == b'?' || ch == b'#' {
            end = i;
            break;
        }

        if ch == 0 {
            if b == b'%' {
                // Per https://tools.ietf.org/html/rfc3986#section-3.2.1 and
                // https://url.spec.whatwg.org/#authority-state
                // the userinfo can have a percent-encoded username and password,
                // so record that a `%` was found. If this turns out to be
                // part of the userinfo, this flag will be cleared.
                // Also per https://tools.ietf.org/html/rfc6874, percent-encoding can
                // be used to indicate a zone identifier.
                // If the flag hasn't been cleared at the end, that means this
                // was part of the hostname (and not part of an IPv6 address), and
                // will fail with an error.
                has_percent = true;
            } else {
                #[cfg(creusot)]
                {
                    proof_assert! {
                        authority_first_delimiter_same_after_empty_prefix(s@, 0, i@ + 1)
                    };
                    proof_assert! { authority_first_delimiter_from(s@, 0) >= i@ + 1 };
                    proof_assert! { !authority_static_input_is_valid(s@) };
                }
                return Err(AuthorityError::InvalidUriChar);
            }
        } else if ch == b':' {
            if colon_cnt >= MAX_COLONS {
                #[cfg(creusot)]
                {
                    proof_assert! {
                        authority_first_delimiter_same_after_empty_prefix(s@, 0, i@ + 1)
                    };
                    proof_assert! { authority_first_delimiter_from(s@, 0) >= i@ + 1 };
                    proof_assert! { !authority_static_input_is_valid(s@) };
                }
                return Err(AuthorityError::TooManyColons);
            }
            colon_cnt += 1;
        } else if ch == b'[' {
            if has_percent || start_bracket {
                #[cfg(creusot)]
                {
                    proof_assert! {
                        authority_first_delimiter_same_after_empty_prefix(s@, 0, i@ + 1)
                    };
                    proof_assert! { authority_first_delimiter_from(s@, 0) >= i@ + 1 };
                    proof_assert! { !authority_static_input_is_valid(s@) };
                }
                // Something other than the userinfo has a `%`, so reject it.
                return Err(AuthorityError::InvalidBracketUsage);
            }
            start_bracket = true;
        } else if ch == b']' {
            if !start_bracket || end_bracket {
                #[cfg(creusot)]
                {
                    proof_assert! {
                        authority_first_delimiter_same_after_empty_prefix(s@, 0, i@ + 1)
                    };
                    proof_assert! { authority_first_delimiter_from(s@, 0) >= i@ + 1 };
                    proof_assert! { !authority_static_input_is_valid(s@) };
                }
                return Err(AuthorityError::InvalidBracketUsage);
            }
            end_bracket = true;

            // Those were part of an IPv6 hostname, so forget them...
            colon_cnt = 0;
            has_percent = false;
        } else if ch == b'@' {
            at_sign_pos = i;

            // Those weren't a port colon, but part of the
            // userinfo, so it needs to be forgotten.
            colon_cnt = 0;
            has_percent = false;
        }

        i += 1;
    }

    #[cfg(creusot)]
    {
        proof_assert! { authority_first_delimiter_same_after_empty_prefix(s@, 0, i@) };
        proof_assert! { end@ == i@ };
        proof_assert! { authority_first_delimiter_from(s@, i@) == end@ };
        proof_assert! { authority_first_delimiter_from(s@, 0) == end@ };
    }

    if start_bracket != end_bracket {
        #[cfg(creusot)]
        proof_assert! { !authority_static_input_is_valid(s@) };
        return Err(AuthorityError::MismatchedBrackets);
    }

    if colon_cnt > 1 {
        // Things like 'localhost:8080:3030' are rejected.
        #[cfg(creusot)]
        proof_assert! { !authority_static_input_is_valid(s@) };
        return Err(AuthorityError::InvalidAuthority);
    }

    if end > 0 && at_sign_pos == end - 1 {
        // If there's nothing after an `@`, this is bonkers.
        #[cfg(creusot)]
        proof_assert! { !authority_static_input_is_valid(s@) };
        return Err(AuthorityError::EmptyAfterAt);
    }

    if has_percent {
        // Something after the userinfo has a `%`, so reject it.
        #[cfg(creusot)]
        proof_assert! { !authority_static_input_is_valid(s@) };
        return Err(AuthorityError::InvalidPercent);
    }

    #[cfg(creusot)]
    {
        proof_assert! { authority_static_input_is_valid(s@) };
        proof_assert! { authority_first_delimiter_from(s@, 0) == end@ };
    }
    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_character_classifier_matches_uri_table() {
        for byte in 0..=u8::MAX {
            assert_eq!(authority_chars::uri_char(byte), crate::uri::URI_CHARS[byte as usize]);
        }
    }

    #[test]
    fn parse_empty_string_is_error() {
        let err = Authority::parse_non_empty(b"").unwrap_err();
        assert_eq!(err.0, ErrorKind::Empty);
    }

    #[test]
    fn equal_to_self_of_same_authority() {
        let authority1: Authority = "example.com".parse().unwrap();
        let authority2: Authority = "EXAMPLE.COM".parse().unwrap();
        assert_eq!(authority1, authority2);
        assert_eq!(authority2, authority1);
    }

    #[test]
    fn not_equal_to_self_of_different_authority() {
        let authority1: Authority = "example.com".parse().unwrap();
        let authority2: Authority = "test.com".parse().unwrap();
        assert_ne!(authority1, authority2);
        assert_ne!(authority2, authority1);
    }

    #[test]
    fn equates_with_a_str() {
        let authority: Authority = "example.com".parse().unwrap();
        assert_eq!(&authority, "EXAMPLE.com");
        assert_eq!("EXAMPLE.com", &authority);
        assert_eq!(authority, "EXAMPLE.com");
        assert_eq!("EXAMPLE.com", authority);
    }

    #[test]
    fn from_static_equates_with_a_str() {
        let authority = Authority::from_static("example.com");
        assert_eq!(authority, "example.com");
    }

    #[test]
    fn not_equal_with_a_str_of_a_different_authority() {
        let authority: Authority = "example.com".parse().unwrap();
        assert_ne!(&authority, "test.com");
        assert_ne!("test.com", &authority);
        assert_ne!(authority, "test.com");
        assert_ne!("test.com", authority);
    }

    #[test]
    fn equates_with_a_string() {
        let authority: Authority = "example.com".parse().unwrap();
        assert_eq!(authority, "EXAMPLE.com".to_string());
        assert_eq!("EXAMPLE.com".to_string(), authority);
    }

    #[test]
    fn equates_with_a_string_of_a_different_authority() {
        let authority: Authority = "example.com".parse().unwrap();
        assert_ne!(authority, "test.com".to_string());
        assert_ne!("test.com".to_string(), authority);
    }

    #[test]
    fn compares_to_self() {
        let authority1: Authority = "abc.com".parse().unwrap();
        let authority2: Authority = "def.com".parse().unwrap();
        assert!(authority1 < authority2);
        assert!(authority2 > authority1);
    }

    #[test]
    fn compares_with_a_str() {
        let authority: Authority = "def.com".parse().unwrap();
        // with ref
        assert!(&authority < "ghi.com");
        assert!("ghi.com" > &authority);
        assert!(&authority > "abc.com");
        assert!("abc.com" < &authority);

        // no ref
        assert!(authority < "ghi.com");
        assert!("ghi.com" > authority);
        assert!(authority > "abc.com");
        assert!("abc.com" < authority);
    }

    #[test]
    fn compares_with_a_string() {
        let authority: Authority = "def.com".parse().unwrap();
        assert!(authority < "ghi.com".to_string());
        assert!("ghi.com".to_string() > authority);
        assert!(authority > "abc.com".to_string());
        assert!("abc.com".to_string() < authority);
    }

    #[test]
    fn comparison_folds_ascii_and_preserves_non_ascii_bytes() {
        let authority: Authority = "EXAMPLE.com".parse().unwrap();
        assert_eq!(authority, "example.COM");
        assert_eq!(authority.partial_cmp("example.com"), Some(cmp::Ordering::Equal));

        assert_ne!(authority, "example.comé");
        assert_eq!(authority.partial_cmp("example.comé"), Some(cmp::Ordering::Less));
        assert_eq!("example.comé".partial_cmp(&authority), Some(cmp::Ordering::Greater));
    }

    #[test]
    fn allows_percent_in_userinfo() {
        let authority_str = "a%2f:b%2f@example.com";
        let authority: Authority = authority_str.parse().unwrap();
        assert_eq!(authority, authority_str);
    }

    #[test]
    fn rejects_percent_in_hostname() {
        let err = Authority::parse_non_empty(b"example%2f.com").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);

        let err = Authority::parse_non_empty(b"a%2f:b%2f@example%2f.com").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);
    }

    #[test]
    fn allows_percent_in_ipv6_address() {
        let authority_str = "[fe80::1:2:3:4%25eth0]";
        let result: Authority = authority_str.parse().unwrap();
        assert_eq!(result, authority_str);
    }

    #[test]
    fn reject_obviously_invalid_ipv6_address() {
        let err = Authority::parse_non_empty(b"[0:1:2:3:4:5:6:7:8:9:10:11:12:13:14]").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);
    }

    #[test]
    fn rejects_percent_outside_ipv6_address() {
        let err = Authority::parse_non_empty(b"1234%20[fe80::1:2:3:4]").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);

        let err = Authority::parse_non_empty(b"[fe80::1:2:3:4]%20").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);
    }

    #[test]
    fn rejects_invalid_utf8() {
        let err = Authority::try_from([0xc0u8].as_ref()).unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidUriChar);

        let err = Authority::from_shared(Bytes::from_static([0xc0u8].as_ref())).unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidUriChar);
    }

    #[test]
    fn host_keeps_last_userinfo_and_bracketed_ipv6_behavior() {
        assert_eq!(host("first@second@host.example:8443"), "host.example");
        assert_eq!(host("[::1]:80"), "[::1]");
        assert_eq!(host("user@[2001:db8::1]:443"), "[2001:db8::1]");
    }

    #[test]
    fn rejects_invalid_use_of_brackets() {
        let err = Authority::parse_non_empty(b"[]@[").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);

        // reject tie-fighter
        let err = Authority::parse_non_empty(b"]o[").unwrap_err();
        assert_eq!(err.0, ErrorKind::InvalidAuthority);
    }
}
