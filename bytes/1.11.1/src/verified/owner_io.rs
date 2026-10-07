//! Concrete byte-oriented I/O and copied-owner replacements for the verified API.

use alloc::{boxed::Box, string::String, vec::Vec};
use creusot_std::prelude::*;

use super::{cursor::Cursor, exclusive::ExclusiveBytes};

#[cfg(feature = "std")]
impl std::io::BufRead for Cursor<'_> {
    #[ensures(match result {
        Ok(bytes) => bytes@ == self@ && (^self)@ == self@,
        Err(_) => false,
    })]
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.chunk())
    }

    #[ensures((^self)@ == self@[if amount@ < self@.len() {
        amount@
    } else {
        self@.len()
    }..])]
    fn consume(&mut self, amount: usize) {
        let remaining = self.remaining();
        let consumed = if amount < remaining { amount } else { remaining };
        let _ = self.advance(consumed);
    }
}

mod readable_owner_sealed {
    use alloc::{boxed::Box, string::String, vec::Vec};

    pub trait Sealed {}

    impl Sealed for Vec<u8> {}
    impl<'a> Sealed for &'a [u8] {}
    impl Sealed for Box<[u8]> {}
    impl Sealed for String {}
}

/// A closed set of owners that can provide a proved byte-preserving copy.
///
/// Implementations state their byte view and prove that `copy_bytes` returns
/// exactly that initialized sequence. The trait is sealed so arbitrary
/// downstream `AsRef` implementations do not supply an unchecked byte law.
pub trait ReadableOwner: readable_owner_sealed::Sealed {
    /// The bytes represented by this owner.
    #[cfg(creusot)]
    #[logic]
    fn byte_view(self) -> Seq<u8>;

    /// Copies the owner's bytes into a new vector without consuming it.
    #[cfg_attr(creusot, ensures(result@ == (*self).byte_view()))]
    fn copy_bytes(&self) -> Vec<u8>;
}

impl ReadableOwner for Vec<u8> {
    #[cfg(creusot)]
    #[logic(open)]
    fn byte_view(self) -> Seq<u8> {
        pearlite! { self@ }
    }

    #[cfg_attr(creusot, ensures(result@ == (*self).byte_view()))]
    fn copy_bytes(&self) -> Vec<u8> {
        Vec::from(&**self)
    }
}

impl<'a> ReadableOwner for &'a [u8] {
    #[cfg(creusot)]
    #[logic(open)]
    fn byte_view(self) -> Seq<u8> {
        pearlite! { (*self)@ }
    }

    #[cfg_attr(creusot, ensures(result@ == (*self).byte_view()))]
    fn copy_bytes(&self) -> Vec<u8> {
        Vec::from(*self)
    }
}

impl ReadableOwner for Box<[u8]> {
    #[cfg(creusot)]
    #[logic(open)]
    fn byte_view(self) -> Seq<u8> {
        pearlite! { (*self)@ }
    }

    #[cfg_attr(creusot, ensures(result@ == (*self).byte_view()))]
    fn copy_bytes(&self) -> Vec<u8> {
        Vec::from(&**self)
    }
}

impl ReadableOwner for String {
    #[cfg(creusot)]
    #[logic(open)]
    fn byte_view(self) -> Seq<u8> {
        pearlite! { self@.to_bytes() }
    }

    #[cfg_attr(creusot, ensures(result@ == (*self).byte_view()))]
    fn copy_bytes(&self) -> Vec<u8> {
        Vec::from(&**self)
    }
}

/// Copies a supported owner while returning the original value intact.
///
/// The returned `ExclusiveBytes` owns a separate allocation and must be
/// explicitly closed. This does not transfer or reinterpret the source owner.
#[cfg_attr(creusot, ensures(result.0.byte_view() == source.byte_view()))]
#[cfg_attr(creusot, ensures(result.1@ == source.byte_view()))]
pub fn copy_from_owner<T: ReadableOwner>(source: T) -> (T, ExclusiveBytes) {
    let copied = source.copy_bytes();
    (source, ExclusiveBytes::from_vec(copied))
}

/// Copies an owner, reads a prefix through `Cursor`, closes the copy, and
/// returns the untouched source owner to its caller.
#[cfg_attr(creusot, ensures(result.0.byte_view() == source.byte_view()))]
#[ensures(result.1@ == if destination@.len() < source.byte_view().len() {
    destination@.len()
} else {
    source.byte_view().len()
})]
#[ensures((^destination)@ == source.byte_view()[0..result.1@]
    .concat(destination@[result.1@..]))]
pub fn copy_owner_read_prefix_and_close<T: ReadableOwner>(
    source: T,
    destination: &mut [u8],
) -> (T, usize) {
    let (original, copied) = copy_from_owner(source);
    let count = {
        let mut cursor = Cursor::new(copied.as_slice());
        cursor.read_prefix(destination)
    };
    copied.close();
    (original, count)
}

/// Concatenates the initialized bytes in a sequence of borrowed slices.
#[logic(open)]
#[variant(slices.len())]
pub fn initialized_slices<'a>(slices: Seq<&'a [u8]>) -> Seq<u8> {
    pearlite! {
        if slices.len() == 0 {
            Seq::empty()
        } else {
            initialized_slices(slices.subsequence(0, slices.len() - 1))
                .concat(slices[slices.len() - 1]@)
        }
    }
}

/// Concatenates mutable slice views in a half-open interval.
#[logic(open)]
#[variant(end - start)]
#[requires(0 <= start && start <= end && end <= slices.len())]
pub fn initialized_mut_slices_range<'a>(
    slices: Seq<&'a mut [u8]>,
    start: Int,
    end: Int,
) -> Seq<u8> {
    pearlite! {
        if start >= end {
            Seq::empty()
        } else {
            initialized_mut_slices_range(slices, start, end - 1)
                .concat(slices[end - 1]@)
        }
    }
}

/// Concatenates the initialized bytes in a sequence of mutable slice borrows.
#[logic(open)]
pub fn initialized_mut_slices<'a>(slices: Seq<&'a mut [u8]>) -> Seq<u8> {
    initialized_mut_slices_range(slices, 0, slices.len())
}

/// Splits a flattened slice sequence around one valid position.
#[check(ghost)]
#[variant(end@ - index@)]
#[requires(start@ <= index@ && index@ < end@)]
#[requires(end@ <= (*slices).len())]
#[ensures(initialized_mut_slices_range(*slices, start@, end@) ==
    initialized_mut_slices_range(*slices, start@, index@)
        .concat((*slices)[index@]@)
        .concat(initialized_mut_slices_range(*slices, index@ + 1, end@)))]
fn initialized_mut_slices_range_split<'a>(
    slices: Snapshot<Seq<&'a mut [u8]>>,
    start: usize,
    index: usize,
    end: usize,
) {
    if index + 1 < end {
        let shorter = end - 1;
        initialized_mut_slices_range_split(slices, start, index, shorter);
        proof_assert!(initialized_mut_slices_range(*slices, start@, end@) ==
            initialized_mut_slices_range(*slices, start@, index@)
                .concat((*slices)[index@]@)
                .concat(initialized_mut_slices_range(*slices, index@ + 1, end@)));
    } else {
        proof_assert!(index@ + 1 == end@);
        proof_assert!(initialized_mut_slices_range(*slices, index@ + 1, end@)
            == Seq::<u8>::empty());
        proof_assert!(initialized_mut_slices_range(*slices, start@, end@) ==
            initialized_mut_slices_range(*slices, start@, index@)
                .concat((*slices)[index@]@));
    }
}

/// Equal segment views on an interval yield equal flattened byte ranges.
#[check(ghost)]
#[variant(end@ - start@)]
#[requires(start@ <= end@)]
#[requires(end@ <= (*before).len() && end@ <= (*after).len())]
#[requires(forall<j: Int> start@ <= j && j < end@ ==>
    (*before)[j]@ == (*after)[j]@)]
#[ensures(initialized_mut_slices_range(*before, start@, end@)
    == initialized_mut_slices_range(*after, start@, end@))]
fn initialized_mut_slices_range_congruent<'a>(
    before: Snapshot<Seq<&'a mut [u8]>>,
    after: Snapshot<Seq<&'a mut [u8]>>,
    start: usize,
    end: usize,
) {
    if start < end {
        let shorter = end - 1;
        initialized_mut_slices_range_congruent(before, after, start, shorter);
        proof_assert!(initialized_mut_slices_range(*before, start@, end@)
            == initialized_mut_slices_range(*after, start@, end@));
    } else {
        proof_assert!(start@ == end@);
        proof_assert!(initialized_mut_slices_range(*before, start@, end@)
            == initialized_mut_slices_range(*after, start@, end@));
    }
}

/// Reads one destination slice and records its exact initialized prefix,
/// untouched tail, and unchanged shape.
#[ensures(result@ == if cursor@.len() < destination@.len() {
    cursor@.len()
} else {
    destination@.len()
})]
#[ensures((^cursor)@ == cursor@[result@..])]
#[ensures((^destination)@ == cursor@[0..result@]
    .concat(destination@[result@..]))]
#[ensures((^destination)@.len() == destination@.len())]
fn read_scatter_segment(cursor: &mut Cursor<'_>, destination: &mut [u8]) -> usize {
    cursor.read_prefix(destination)
}

/// Composes one initialized-segment overwrite with the flattened byte model.
///
/// This separates the saturated-prefix case (the input ended before this
/// segment) from the case where this segment contributes new input bytes.
#[check(ghost)]
#[requires((*prefix).concat(*segment).concat(*suffix)
    == (*input).subsequence(0, c@).concat((*original).subsequence(c@, (*original).len())))]
#[requires((*input).len() == input_len@)]
#[requires(c@ == if input_len@ < (*prefix).len() {
    input_len@
} else {
    (*prefix).len()
})]
#[requires(n@ == if input_len@ - c@ < (*segment).len() {
    input_len@ - c@
} else {
    (*segment).len()
})]
#[requires(0 <= c@ && c@ <= (*input).len())]
#[requires(0 <= n@ && c@ + n@ <= (*input).len())]
#[requires(n@ <= (*segment).len())]
#[requires((*original).len() == (*prefix).len() + (*segment).len() + (*suffix).len())]
#[ensures((*prefix).concat((*input).subsequence(c@, c@ + n@))
        .concat((*segment).subsequence(n@, (*segment).len()))
        .concat(*suffix)
    == (*input).subsequence(0, c@ + n@)
        .concat((*original).subsequence(c@ + n@, (*original).len())))]
fn scatter_segment_splice(
    input: Snapshot<Seq<u8>>,
    original: Snapshot<Seq<u8>>,
    prefix: Snapshot<Seq<u8>>,
    segment: Snapshot<Seq<u8>>,
    suffix: Snapshot<Seq<u8>>,
    c: usize,
    n: usize,
    input_len: usize,
) {
    if c == input_len {
        proof_assert!(n@ == 0);
        proof_assert!((*input).subsequence(c@, c@ + n@) == Seq::<u8>::empty());
        proof_assert!((*segment).subsequence(n@, (*segment).len()) == *segment);
        proof_assert!((*input).subsequence(0, c@ + n@) ==
            (*input).subsequence(0, c@));
        proof_assert!((*prefix).concat((*input).subsequence(c@, c@ + n@))
            .concat((*segment).subsequence(n@, (*segment).len()))
            .concat(*suffix)
            == (*input).subsequence(0, c@ + n@)
                .concat((*original).subsequence(c@ + n@, (*original).len())));
    } else {
        proof_assert!(c@ < (*input).len());
        proof_assert!(c@ == (*prefix).len());
        let before_tail = snapshot!((*segment).concat(*suffix));
        let input_prefix = snapshot!((*input).subsequence(0, c@));
        let original_tail = snapshot!((*original).subsequence(c@, (*original).len()));
        proof_assert!((*before_tail).len() == (*original_tail).len());
        proof_assert!(forall<i: Int> 0 <= i && i < c@ ==>
            (*prefix)[i] == (*input_prefix)[i]);
        proof_assert!(forall<i: Int> 0 <= i && i < (*before_tail).len() ==>
            (*prefix).concat(*segment).concat(*suffix)[c@ + i]
                == (*input_prefix).concat(*original_tail)[c@ + i]);
        proof_assert!(forall<i: Int> 0 <= i && i < (*before_tail).len() ==>
            (*prefix).concat(*segment).concat(*suffix)[c@ + i]
                == (*before_tail)[i]);
        proof_assert!(forall<i: Int> 0 <= i && i < (*before_tail).len() ==>
            (*input_prefix).concat(*original_tail)[c@ + i]
                == (*original_tail)[i]);
        proof_assert!(forall<i: Int> 0 <= i && i < (*before_tail).len() ==>
            (*before_tail)[i] == (*original_tail)[i]);
        proof_assert!((*before_tail).ext_eq(*original_tail));
        proof_assert!((*segment).concat(*suffix) ==
            (*original).subsequence(c@, (*original).len()));

        let after_segment = snapshot!((*segment).subsequence(n@, (*segment).len())
            .concat(*suffix));
        let after_original = snapshot!((*original).subsequence(c@ + n@, (*original).len()));
        proof_assert!((*after_segment).len() == (*after_original).len());
        proof_assert!(forall<i: Int> 0 <= i && i < (*after_segment).len() ==>
            if i < (*segment).len() - n@ {
                (*after_segment)[i] == (*segment)[n@ + i]
                    && (*before_tail)[n@ + i] == (*segment)[n@ + i]
                    && (*original_tail)[n@ + i] == (*after_original)[i]
            } else {
                (*after_segment)[i] == (*suffix)[n@ + i - (*segment).len()]
                    && (*before_tail)[n@ + i]
                        == (*suffix)[n@ + i - (*segment).len()]
                    && (*original_tail)[n@ + i] == (*after_original)[i]
            });
        proof_assert!(forall<i: Int> 0 <= i && i < (*after_segment).len() ==>
            (*after_segment)[i] == (*after_original)[i]);
        proof_assert!((*after_segment).ext_eq(*after_original));
        proof_assert!((*segment).subsequence(n@, (*segment).len()).concat(*suffix)
            == (*original).subsequence(c@ + n@, (*original).len()));

        let input_part = snapshot!((*input).subsequence(c@, c@ + n@));
        let updated = snapshot!((*prefix).concat(*input_part).concat(*after_segment));
        let expected_prefix = snapshot!((*input).subsequence(0, c@ + n@));
        let expected_tail = snapshot!((*original).subsequence(c@ + n@, (*original).len()));
        let expected = snapshot!((*expected_prefix).concat(*expected_tail));
        proof_assert!((*updated).len() == (*expected).len());
        proof_assert!(forall<i: Int> 0 <= i && i < c@ ==>
            (*updated)[i] == (*expected)[i]);
        proof_assert!(forall<i: Int> c@ <= i && i < c@ + n@ ==>
            (*updated)[i] == (*expected)[i]);
        proof_assert!(forall<i: Int> c@ + n@ <= i && i < (*updated).len() ==>
            (*updated)[i] == (*expected)[i]);
        proof_assert!(forall<i: Int> 0 <= i && i < (*updated).len() ==>
            (*updated)[i] == (*expected)[i]);
        proof_assert!((*updated).ext_eq(*expected));
    }
}

impl ExclusiveBytes {
    /// Appends initialized slices in order, preserving every source byte.
    #[ensures((^self)@ == self@.concat(initialized_slices(sources@)))]
    pub fn extend_from_slices(&mut self, sources: &[&[u8]]) {
        let initial = snapshot!(self@);
        let mut index = 0usize;

        #[invariant(index@ <= sources@.len())]
        #[invariant(self@ == initial.concat(initialized_slices(sources@[0..index@])))]
        #[variant(sources@.len() - index@)]
        while index < sources.len() {
            self.extend_from_slice(sources[index]);
            index += 1;
        }
    }

}

impl<'a> Cursor<'a> {
    /// Reads the longest available input prefix across initialized destination
    /// slices, in order. Destination bytes after the returned prefix are kept.
    #[ensures(result@ == if self@.len() < initialized_mut_slices(destinations@).len() {
        self@.len()
    } else {
        initialized_mut_slices(destinations@).len()
    })]
    #[ensures((^self)@ == self@[result@..])]
    #[ensures(initialized_mut_slices((^destinations)@) ==
        self@[0..result@].concat(initialized_mut_slices(destinations@)[result@..]))]
    pub fn read_scatter_prefix(&mut self, destinations: &mut [&mut [u8]]) -> usize {
        let input_len = self.remaining();
        let input = snapshot!(self@);
        let output = snapshot!(initialized_mut_slices(destinations@));
        let mut index = 0usize;
        let mut count = 0usize;

        #[invariant(index@ <= destinations@.len())]
        #[invariant(input.len() == input_len@)]
        #[invariant(count@ <= input.len())]
        #[invariant(count@ == input_len@ - self@.len())]
        #[invariant(self@ == input[count@..])]
        #[invariant(count@ == if input.len() < initialized_mut_slices_range(destinations@, 0, index@).len() {
            input.len()
        } else {
            initialized_mut_slices_range(destinations@, 0, index@).len()
        })]
        #[invariant(initialized_mut_slices_range(destinations@, 0, destinations@.len()).len() == output.len())]
        #[invariant(initialized_mut_slices_range(destinations@, 0, destinations@.len()) ==
            input[0..count@].concat(output[count@..]))]
        #[variant(destinations@.len() - index@)]
        while index < destinations.len() {
            let before_destinations = snapshot!(destinations@);
            let prefix = snapshot!(initialized_mut_slices_range(
                *before_destinations,
                0,
                index@,
            ));
            let segment = snapshot!((*before_destinations)[index@]@);
            let suffix_start = index + 1;
            let suffix = snapshot!(initialized_mut_slices_range(
                *before_destinations,
                suffix_start@,
                destinations@.len(),
            ));
            ghost! {
                initialized_mut_slices_range_split(
                    before_destinations,
                    0usize,
                    index,
                    destinations.len(),
                );
            };
            let read = read_scatter_segment(self, &mut *destinations[index]);
            let after_destinations = snapshot!(destinations@);
            ghost! {
                initialized_mut_slices_range_split(
                    after_destinations,
                    0usize,
                    index,
                    destinations.len(),
                );
                initialized_mut_slices_range_congruent(
                    before_destinations,
                    after_destinations,
                    0usize,
                    index,
                );
                initialized_mut_slices_range_congruent(
                    before_destinations,
                    after_destinations,
                    suffix_start,
                    destinations.len(),
                );
                scatter_segment_splice(
                    input,
                    output,
                    prefix,
                    segment,
                    suffix,
                    count,
                    read,
                    input_len,
                );
            };
            count = input_len - self.remaining();
            index += 1;
        }
        count
    }
}

/// Appends a gathered source sequence, scatters its prefix to initialized output
/// slices, and explicitly closes the temporary owner on normal return.
#[ensures(result@ == if prefix@.concat(initialized_slices(sources@)).len()
        < initialized_mut_slices(destinations@).len() {
    prefix@.concat(initialized_slices(sources@)).len()
} else {
    initialized_mut_slices(destinations@).len()
})]
#[ensures(initialized_mut_slices((^destinations)@) ==
    prefix@.concat(initialized_slices(sources@))[0..result@]
        .concat(initialized_mut_slices(destinations@)[result@..]))]
pub fn append_slices_scatter_read_and_close(
    prefix: &[u8],
    sources: &[&[u8]],
    destinations: &mut [&mut [u8]],
) -> usize {
    let mut owner = ExclusiveBytes::copy_from_slice(prefix);
    owner.extend_from_slices(sources);
    let count = {
        let mut cursor = Cursor::new(owner.as_slice());
        cursor.read_scatter_prefix(destinations)
    };
    owner.close();
    count
}

#[cfg(all(test, not(creusot), feature = "alloc"))]
mod tests {
    use super::{
        append_slices_scatter_read_and_close, copy_from_owner,
        copy_owner_read_prefix_and_close, ReadableOwner,
    };
    use crate::verified::{cursor::Cursor, exclusive::ExclusiveBytes};
    use alloc::{boxed::Box, string::String, vec, vec::Vec};

    #[cfg(feature = "std")]
    #[cfg(feature = "std")]
    #[test]
    fn cursor_bufread_fill_is_nonconsuming_and_consume_clamps() {
        use std::io::BufRead;

        let input = [10, 20, 30];
        let mut cursor = Cursor::new(&input);
        assert_eq!(cursor.fill_buf().unwrap(), &[10, 20, 30]);
        assert_eq!(cursor.remaining(), 3);
        cursor.consume(2);
        assert_eq!(cursor.fill_buf().unwrap(), &[30]);
        cursor.consume(usize::MAX);
        assert!(cursor.fill_buf().unwrap().is_empty());
        cursor.consume(0);
        assert!(cursor.fill_buf().unwrap().is_empty());
    }

    #[test]
    fn readable_owner_copies_return_original_values() {
        let bytes = vec![1, 2, 3];
        let (original, copied) = copy_from_owner(bytes);
        assert_eq!(original, vec![1, 2, 3]);
        assert_eq!(copied.as_slice(), &[1, 2, 3]);
        copied.close();

        let borrowed: &[u8] = b"borrowed";
        let (original, copied) = copy_from_owner(borrowed);
        assert_eq!(original, b"borrowed");
        assert_eq!(copied.as_slice(), b"borrowed");
        copied.close();

        let boxed: Box<[u8]> = Vec::from(&b"boxed"[..]).into_boxed_slice();
        let (original, copied) = copy_from_owner(boxed);
        assert_eq!(&*original, b"boxed");
        assert_eq!(copied.as_slice(), b"boxed");
        copied.close();

        let string = String::from("héllo");
        let (original, copied) = copy_from_owner(string);
        assert_eq!(original, "héllo");
        assert_eq!(copied.as_slice(), "héllo".as_bytes());
        copied.close();
    }

    #[test]
    fn copied_owner_read_returns_owner_and_closes_copy() {
        let source = String::from("héllo");
        let mut destination = [0xaa; 4];
        let (source, count) = copy_owner_read_prefix_and_close(source, &mut destination);
        assert_eq!(source, "héllo");
        assert_eq!(count, 4);
        assert_eq!(destination, [b'h', 0xc3, 0xa9, b'l']);

        let empty = Vec::new();
        let mut untouched = [0x55; 2];
        let (empty, count) = copy_owner_read_prefix_and_close(empty, &mut untouched);
        assert!(empty.is_empty());
        assert_eq!(count, 0);
        assert_eq!(untouched, [0x55; 2]);
    }

    #[test]
    fn explicit_scatter_gather_preserves_tail_bytes_and_empty_segments() {
        let source_segments: [&[u8]; 4] = [b"", b"ab", b"", b"cde"];
        let mut first = [0xaa; 2];
        let mut empty = [];
        let mut last = [0xbb; 5];
        let mut outputs: [&mut [u8]; 3] = [&mut first, &mut empty, &mut last];
        let count = append_slices_scatter_read_and_close(b"z", &source_segments, &mut outputs);
        assert_eq!(count, 6);
        assert_eq!(first, *b"za");
        assert_eq!(empty, []);
        assert_eq!(last, [b'b', b'c', b'd', b'e', 0xbb]);

        let short_segments: [&[u8]; 2] = [b"q", b""];
        let mut short_first = [0x11; 1];
        let mut short_last = [0x22; 3];
        let mut short_outputs: [&mut [u8]; 2] = [&mut short_first, &mut short_last];
        let count = append_slices_scatter_read_and_close(b"", &short_segments, &mut short_outputs);
        assert_eq!(count, 1);
        assert_eq!(short_first, *b"q");
        assert_eq!(short_last, [0x22; 3]);

        let no_sources: [&[u8]; 0] = [];
        let mut no_destinations: [&mut [u8]; 0] = [];
        assert_eq!(append_slices_scatter_read_and_close(b"", &no_sources, &mut no_destinations), 0);
    }

    #[test]
    fn scatter_reads_the_next_segment_after_consuming_half_the_input() {
        let mut cursor = Cursor::new(b"abcd");
        let mut left = [0; 2];
        let mut right = [0; 2];
        let mut destinations: [&mut [u8]; 2] = [&mut left, &mut right];

        let count = cursor.read_scatter_prefix(&mut destinations);

        assert_eq!(count, 4);
        assert_eq!(left, *b"ab");
        assert_eq!(right, *b"cd");
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn sealed_trait_method_is_callable_as_a_byte_law() {
        let owner = String::from("law");
        let bytes = owner.copy_bytes();
        assert_eq!(bytes, b"law");
    }

    #[test]
    fn explicit_readable_owner_cleanup_uses_the_verified_close_path() {
        let owner = ExclusiveBytes::copy_from_slice(b"cleanup");
        owner.close();
    }
}
