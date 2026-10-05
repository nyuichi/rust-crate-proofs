#[cfg(creusot)]
use creusot_std::std::string::valid_utf8;

/// Decode the exact byte span returned by method parsing.
///
/// The cursor-based parsers can be entered after a safe `Bytes::next` call,
/// while `slice_skip` returns a span from the retained mark. Validate that
/// whole span before exposing it as `&str`.
#[ensures(match result {
    Some(value) => value@.to_bytes() == raw@,
    None => !valid_utf8(raw@),
})]
pub(crate) fn method_from_bytes<'a>(raw: &'a [u8]) -> Option<&'a str> {
    match str::from_utf8(raw) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}
