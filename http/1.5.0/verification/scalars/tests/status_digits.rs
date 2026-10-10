use http_scalar_proofs::status::StatusCode;

#[test]
fn as_str_matches_the_decimal_value_for_the_entire_supported_range() {
    for code in 100u16..=999 {
        let expected = [
            b'0' + (code / 100) as u8,
            b'0' + ((code / 10) % 10) as u8,
            b'0' + (code % 10) as u8,
        ];
        assert_eq!(
            StatusCode::from_u16(code).unwrap().as_str().as_bytes(),
            &expected,
            "status code {code}"
        );
    }
}
