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
