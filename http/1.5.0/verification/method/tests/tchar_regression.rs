use http_method_proofs::method::Method;

// RFC 9110's token characters, written as an independent allowlist rather
// than as the production range-and-exclusion predicate.
const RFC_TCHARS: &[u8] =
    b"!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[test]
fn every_byte_matches_the_rfc_method_character_allowlist() {
    for byte in 0u8..=u8::MAX {
        let accepted = Method::from_bytes(&[byte]).is_ok();
        assert_eq!(accepted, RFC_TCHARS.contains(&byte), "byte {byte:#04x}");
    }
}

#[test]
fn inline_and_allocated_extensions_preserve_the_full_method() {
    let inline = [b'A'; 15];
    let allocated = [b'A'; 16];

    assert_eq!(Method::from_bytes(&inline).unwrap().as_str().as_bytes(), inline);
    assert_eq!(
        Method::from_bytes(&allocated).unwrap().as_str().as_bytes(),
        allocated
    );
}

#[test]
fn from_ref_preserves_known_inline_and_allocated_methods() {
    let methods = [
        Method::GET,
        Method::from_bytes(b"CUSTOM").unwrap(),
        Method::from_bytes(&[b'A'; 16]).unwrap(),
    ];

    for method in &methods {
        let copy = Method::from(method);
        assert_eq!(copy.as_str().as_bytes(), method.as_str().as_bytes());
        assert_eq!(copy, *method);
    }
}

#[test]
fn empty_and_invalid_extensions_are_rejected() {
    assert!(Method::from_bytes(b"").is_err());
    assert!(Method::from_bytes(b"GET ").is_err());
    assert!(Method::from_bytes(&[0xff]).is_err());
}
