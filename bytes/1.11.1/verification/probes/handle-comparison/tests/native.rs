use bytes_handle_comparison::compare_unique;

#[test]
fn comparisons_cover_empty_prefix_mismatch_and_high_bytes() {
    let values: &[&[u8]] = &[b"", b"a", b"ab", b"ac", &[0, 255], &[255]];
    for left in values {
        for right in values {
            assert_eq!(compare_unique(left.to_vec(), right.to_vec()),
                (left == right, left.cmp(right)));
        }
    }
}

#[cfg(all(feature = "actual-traits", feature = "readonly-deref"))]
#[test]
fn vec_partial_order_matches_the_exact_bytes_mut_source_impl() {
    use bytes_handle_comparison::compare_vec_to_unique;
    use std::cmp::Ordering;

    let cases: &[(&[u8], &[u8], Ordering)] = &[
        (b"", b"", Ordering::Equal),
        (&[0], &[1], Ordering::Less),
        (&[1], &[0], Ordering::Greater),
        (&[0], &[0, 1], Ordering::Less),
        (&[0, 1], &[0], Ordering::Greater),
        (&[0, 255], &[0, 128], Ordering::Greater),
    ];

    for &(left, right, expected) in cases {
        assert_eq!(compare_vec_to_unique(left.to_vec(), right.to_vec()), (expected, expected));
    }
}

#[cfg(feature = "str-adapters")]
#[test]
fn strings_compare_their_exact_utf8_bytes() {
    use bytes_handle_comparison::compare_text_unique;
    for text in ["", "a", "é", "\u{1f980}"] {
        for bytes in [text.as_bytes(), &[255][..], &[0][..]] {
            assert_eq!(compare_text_unique(bytes.to_vec(), text),
                (bytes == text.as_bytes(), bytes.cmp(text.as_bytes())));
        }
    }
}

#[cfg(feature = "concrete-iterator")]
#[test]
fn concrete_iterator_preserves_suffix_and_exact_size() {
    use bytes_handle_comparison::concrete_iterator::IntoIter;
    for input in [&[][..], &[0][..], &[0, 128, 255][..]] {
        let mut iter = IntoIter::new(input);
        for (index, value) in input.iter().enumerate() {
            assert_eq!(iter.size_hint(), (input.len() - index, Some(input.len() - index)));
            assert_eq!(iter.len(), input.len() - index);
            assert_eq!(iter.next(), Some(*value));
            assert_eq!(*iter.get_ref(), &input[index + 1..]);
        }
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.into_inner(), &[]);
    }
}
