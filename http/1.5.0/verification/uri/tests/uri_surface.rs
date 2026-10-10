use http_uri_proof::uri::{Authority, PathAndQuery, Uri};

#[test]
fn uri_display_and_debug_use_the_component_text() {
    for (source, rendered) in [
        ("*", "*"),
        ("/", "/"),
        ("/alpha?x=1", "/alpha?x=1"),
        ("http://Example.COM:80/alpha?x=1", "http://Example.COM:80/alpha?x=1"),
        ("https://example.com", "https://example.com/"),
    ] {
        let uri: Uri = source.parse().unwrap();
        assert_eq!(uri.to_string(), rendered);
        assert_eq!(format!("{:?}", uri), rendered);
    }
}

#[test]
fn component_conversions_preserve_the_component_and_default_fields() {
    let authority = Authority::try_from("Example.COM:8080").unwrap();
    let authority_uri = Uri::from(authority);
    assert_eq!(authority_uri.authority().unwrap().as_str(), "Example.COM:8080");
    assert_eq!(authority_uri.scheme(), None);
    assert_eq!(authority_uri.path(), "");
    assert_eq!(authority_uri.to_string(), "Example.COM:8080");

    let path = PathAndQuery::try_from("/alpha?x=1").unwrap();
    let path_uri = Uri::from(path);
    assert_eq!(path_uri.authority(), None);
    assert_eq!(path_uri.scheme(), None);
    assert_eq!(path_uri.path(), "/alpha");
    assert_eq!(path_uri.query(), Some("x=1"));
    assert_eq!(path_uri.to_string(), "/alpha?x=1");
}

#[test]
fn uri_equality_uses_component_rules() {
    let lower: Uri = "http://example.com/Path?query".parse().unwrap();
    let upper: Uri = "HTTP://EXAMPLE.COM/Path?query".parse().unwrap();
    assert_eq!(lower, upper);

    let different_path: Uri = "http://example.com/path?query".parse().unwrap();
    assert_ne!(lower, different_path);

    let no_query: Uri = "/path".parse().unwrap();
    let empty_query: Uri = "/path?".parse().unwrap();
    assert_ne!(no_query, empty_query);

    let absolute_no_path: Uri = "http://example.com".parse().unwrap();
    let absolute_slash: Uri = "http://example.com/".parse().unwrap();
    assert_eq!(absolute_no_path, absolute_slash);
}

#[test]
fn uri_text_comparison_keeps_prefix_and_fragment_behavior() {
    let uri: Uri = "/path?query".parse().unwrap();
    assert_eq!(uri, "/path?query");
    assert_eq!(uri, "/path?query#fragment");
    assert_ne!(uri, "/path?query-extra");

    let absolute: Uri = "http://example.com/".parse().unwrap();
    assert_eq!(absolute, "http://example.com");
    assert_eq!(absolute, "http://example.com/#fragment");
}
