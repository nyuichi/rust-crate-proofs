use http::HeaderValue;

#[test]
fn all_byte_values_follow_the_header_value_classifiers() {
    for value in 0u8..=u8::MAX {
        let parsed = HeaderValue::from_bytes(&[value]);
        let valid = value == b'\t' || value >= 32 && value != 127;
        assert_eq!(parsed.is_ok(), valid, "header value validity for {value:#04x}");

        if let Ok(parsed) = parsed {
            let visible = value == b'\t' || (32..127).contains(&value);
            assert_eq!(parsed.to_str().is_ok(), visible, "text conversion for {value:#04x}");
        }
    }
}
