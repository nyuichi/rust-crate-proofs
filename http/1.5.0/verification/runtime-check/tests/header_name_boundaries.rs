use http::header::HeaderName;

const MAX_HEADER_NAME_LEN: usize = (1 << 16) - 1;

#[test]
fn parsing_preserves_normalization_across_scratch_and_length_boundaries() {
    for len in [63, 64, 65, MAX_HEADER_NAME_LEN] {
        let input = vec![b'A'; len];
        let expected = "a".repeat(len);

        let normalized = HeaderName::from_bytes(&input).unwrap();
        assert_eq!(normalized.as_str(), expected);

        let lowercase_input = vec![b'a'; len];
        let lowercase = HeaderName::from_lowercase(&lowercase_input).unwrap();
        assert_eq!(lowercase.as_str(), expected);
    }

    assert!(HeaderName::from_bytes(&vec![b'a'; MAX_HEADER_NAME_LEN + 1]).is_err());
    assert!(HeaderName::from_lowercase(&vec![b'a'; MAX_HEADER_NAME_LEN + 1]).is_err());
}
