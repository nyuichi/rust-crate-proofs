use http::header::HeaderName;

/// Records the 1.5.0 parser discrepancy without treating it as RFC-valid input.
/// RFC 9110's `token` grammar excludes DQUOTE. The case-insensitive parser
/// rejects it, while the lowercase parser currently accepts it because its
/// lookup table maps byte 34 to itself.
#[test]
fn lowercase_parser_currently_accepts_quote_despite_token_grammar() {
    let input = b"foo\"bar";

    assert!(HeaderName::from_bytes(input).is_err());
    let lowercase = HeaderName::from_lowercase(input).unwrap();
    assert_eq!(lowercase.as_str(), "foo\"bar");
}
