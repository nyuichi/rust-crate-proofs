use creusot_std::prelude::{ensures, logic};
#[cfg(creusot)]
use creusot_std::prelude::{pearlite, Int};

pub(crate) const CLASS_VALID: u8 = 0;
pub(crate) const CLASS_QUERY: u8 = 1;
pub(crate) const CLASS_FRAGMENT: u8 = 2;
pub(crate) const CLASS_HIGH: u8 = 3;
pub(crate) const CLASS_INVALID: u8 = 4;

#[logic(open(crate))]
pub(crate) fn path_byte_class_model(byte: Int) -> Int {
    pearlite! {
        if byte == 63 { 1 }
        else if byte == 35 { 2 }
        else if byte == 33 || (36 <= byte && byte <= 59)
            || byte == 61 || (64 <= byte && byte <= 95)
            || (97 <= byte && byte <= 122) || byte == 124 || byte == 126
            || byte == 34 || byte == 123 || byte == 125 { 0 }
        else if 128 <= byte && byte <= 255 { 3 }
        else { 4 }
    }
}

#[logic(open(crate))]
pub(crate) fn query_byte_class_model(byte: Int) -> Int {
    pearlite! {
        if byte == 35 { 2 }
        else if byte == 33 || (36 <= byte && byte <= 59)
            || byte == 61 || (63 <= byte && byte <= 126) { 0 }
        else if 128 <= byte && byte <= 255 { 3 }
        else { 4 }
    }
}

#[logic(open(crate))]
pub(crate) fn path_or_high(byte: Int) -> bool {
    let class = path_byte_class_model(byte);
    class == 0 || class == 3
}

#[logic(open(crate))]
pub(crate) fn query_or_high(byte: Int) -> bool {
    let class = query_byte_class_model(byte);
    class == 0 || class == 3
}

/// Exact successful scan result. Delimiter exclusion from the preceding
/// classes makes the recorded `?` and `#` the first delimiters of each kind.
#[logic(open(crate))]
pub(crate) fn path_scan_ok(
    bytes: creusot_std::logic::seq::Seq<u8>,
    query: Option<Int>,
    fragment: Option<Int>,
    high: bool,
) -> bool {
    pearlite! {
        let end = match fragment {
            None => bytes.len(),
            Some(index) => index,
        };
        let query_prefix = match query {
            None => forall<i: Int> 0 <= i && i < end
                ==> path_or_high(bytes[i]@),
            Some(index) => 0 <= index && index < end
                && bytes[index]@ == 63
                && (forall<i: Int> 0 <= i && i < index
                    ==> path_or_high(bytes[i]@))
                && (forall<i: Int> index < i && i < end
                    ==> query_or_high(bytes[i]@)),
        };
        let fragment_matches = match fragment {
            None => true,
            Some(index) => 0 <= index && index < bytes.len()
                && bytes[index]@ == 35,
        };
        let has_high = exists<i: Int> 0 <= i && i < end && bytes[i]@ >= 128;
        query_prefix && fragment_matches && (high == has_high)
    }
}

/// Delimiter facts consumed by the `PathAndQuery` constructor. A successful
/// scan has no fragment delimiter before the retained prefix, and its cached
/// query offset is exactly the first question mark in that prefix.
#[logic(open(crate))]
pub(crate) fn path_scan_delimiters(
    bytes: creusot_std::logic::seq::Seq<u8>,
    query: Option<Int>,
    fragment: Option<Int>,
) -> bool {
    pearlite! {
        let end = match fragment {
            None => bytes.len(),
            Some(index) => index,
        };
        (forall<i: Int> 0 <= i && i < end ==> bytes[i]@ != 35)
        && match query {
            None => forall<i: Int> 0 <= i && i < end ==> bytes[i]@ != 63,
            Some(index) => 0 <= index && index < end
                && bytes[index]@ == 63
                && (forall<i: Int> 0 <= i && i < index ==> bytes[i]@ != 63),
        }
        && match fragment {
            None => true,
            Some(index) => 0 <= index && index < bytes.len()
                && bytes[index]@ == 35,
        }
    }
}

/// A witness for a byte rejected before the first fragment delimiter.
#[logic(open(crate))]
pub(crate) fn path_scan_has_invalid(bytes: creusot_std::logic::seq::Seq<u8>) -> bool {
    pearlite! {
        (exists<i: Int> 0 <= i && i < bytes.len()
            && path_byte_class_model(bytes[i]@) == 4
            && (forall<j: Int> 0 <= j && j < i
                ==> path_or_high(bytes[j]@)))
        || (exists<q: Int> 0 <= q && q < bytes.len()
            && bytes[q]@ == 63
            && (forall<j: Int> 0 <= j && j < q
                ==> path_or_high(bytes[j]@))
            && exists<i: Int> q < i && i < bytes.len()
                && query_byte_class_model(bytes[i]@) == 4
                && (forall<j: Int> q < j && j < i
                    ==> query_or_high(bytes[j]@)))
    }
}

#[ensures(result@ == path_byte_class_model(byte@))]
pub(crate) const fn path_byte_class(byte: u8) -> u8 {
    match byte {
        b'?' => CLASS_QUERY,
        b'#' => CLASS_FRAGMENT,
        0x21 | 0x24..=0x3B | 0x3D | 0x40..=0x5F | 0x61..=0x7A | 0x7C | 0x7E
        | b'"' | b'{' | b'}' => CLASS_VALID,
        0x80..=0xFF => CLASS_HIGH,
        _ => CLASS_INVALID,
    }
}

#[ensures(result@ == query_byte_class_model(byte@))]
pub(crate) const fn query_byte_class(byte: u8) -> u8 {
    match byte {
        b'#' => CLASS_FRAGMENT,
        0x21 | 0x24..=0x3B | 0x3D | 0x3F..=0x7E => CLASS_VALID,
        0x80..=0xFF => CLASS_HIGH,
        _ => CLASS_INVALID,
    }
}
