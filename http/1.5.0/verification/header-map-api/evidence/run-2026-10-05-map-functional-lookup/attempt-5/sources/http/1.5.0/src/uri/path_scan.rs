use super::{ErrorKind, MAX_LEN};

#[path = "path_class.rs"]
mod path_class;
use self::path_class::{
    path_byte_class, query_byte_class, CLASS_FRAGMENT, CLASS_HIGH, CLASS_QUERY,
    CLASS_VALID,
};

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, invariant, proof_assert, requires, variant, DeepModel, Int};

pub(super) const NONE: u16 = u16::MAX;

pub(super) struct Scanned {
    pub(super) query: u16,
    pub(super) fragment: Option<u16>,
    pub(super) is_maybe_not_utf8: bool,
}

/// Derive the explicit delimiter facts needed by the constructor from the
/// scanner's exact successful-prefix contract.
#[cfg(creusot)]
#[requires(path_class::path_scan_ok(
    *bytes,
    if query == NONE { None } else { Some(query@) },
    match fragment {
        None => None,
        Some(index) => Some(index@),
    },
    high,
))]
#[ensures(path_class::path_scan_delimiters(
    *bytes,
    if query == NONE { None } else { Some(query@) },
    match fragment {
        None => None,
        Some(index) => Some(index@),
    },
))]
pub(super) fn successful_scan_has_delimiters(
    bytes: creusot_std::snapshot::Snapshot<creusot_std::logic::seq::Seq<u8>>,
    query: u16,
    fragment: Option<u16>,
    high: bool,
) {
    proof_assert!(path_class::path_scan_delimiters(
        *bytes,
        if query == NONE { None } else { Some(query@) },
        match fragment {
            None => None,
            Some(index) => Some(index@),
        },
    ));
}

#[ensures(match result {
    Ok(scanned) => bytes@.len() > 0 && bytes@.len() <= MAX_LEN@
        && ((bytes@.len() == 1 && bytes@[0]@ == 42)
            || bytes@[0]@ == 47 || bytes@[0]@ == 63 || bytes@[0]@ == 35)
        && path_class::path_scan_ok(bytes@,
            if scanned.query == NONE { None } else { Some(scanned.query@) },
            match scanned.fragment {
                None => None,
                Some(index) => Some(index@),
            },
            scanned.is_maybe_not_utf8),
    Err(error) => (error.deep_model() == 10 && bytes@.len() == 0)
        || (error.deep_model() == 9 && bytes@.len() > MAX_LEN@)
        || (error.deep_model() == 8 && bytes@.len() > 0
            && bytes@.len() <= MAX_LEN@
            && !(bytes@.len() == 1 && bytes@[0]@ == 42)
            && bytes@[0]@ != 47 && bytes@[0]@ != 63 && bytes@[0]@ != 35)
        || (error.deep_model() == 0 && bytes@.len() > 0
            && bytes@.len() <= MAX_LEN@
            && (bytes@.len() == 1 && bytes@[0]@ == 42
                || bytes@[0]@ == 47 || bytes@[0]@ == 63 || bytes@[0]@ == 35)
            && path_class::path_scan_has_invalid(bytes@)),
})]
pub(super) const fn scan_path_and_query(bytes: &[u8]) -> Result<Scanned, ErrorKind> {
    let mut i = 0;
    let mut query = NONE;
    let mut fragment = None;
    let mut is_maybe_not_utf8 = false;

    if bytes.is_empty() {
        return Err(ErrorKind::Empty);
    }

    if bytes.len() > MAX_LEN {
        return Err(ErrorKind::TooLong);
    }

    if bytes.len() == 1 && bytes[0] == b'*' {
        return Ok(Scanned {
            query,
            fragment,
            is_maybe_not_utf8: false,
        });
    }

    if !matches!(bytes[0], b'/' | b'?' | b'#') {
        return Err(ErrorKind::PathDoesNotStartWithSlash);
    }

    #[invariant(bytes@.len() > 0)]
    #[invariant(bytes@.len() <= MAX_LEN@)]
    #[invariant(bytes@[0]@ == 47 || bytes@[0]@ == 63 || bytes@[0]@ == 35)]
    #[invariant(i@ <= bytes@.len())]
    #[invariant(query == NONE)]
    #[invariant(fragment == None)]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==>
        path_class::path_or_high(bytes@[j]@))]
    #[invariant(is_maybe_not_utf8 == (exists<j: Int>
        0 <= j && j < i@ && bytes@[j]@ >= 128))]
    #[variant(bytes@.len() - i@)]
    while i < bytes.len() {
        match path_byte_class(bytes[i]) {
            CLASS_VALID => {}
            CLASS_QUERY => {
                debug_assert!(query == NONE);
                query = i as u16;
                i += 1;
                break;
            }
            CLASS_FRAGMENT => {
                fragment = Some(i as u16);
                break;
            }
            CLASS_HIGH => {
                is_maybe_not_utf8 = true;
            }
            _ => return Err(ErrorKind::InvalidUriChar),
        }
        i += 1;
    }

    // query ...
    if query != NONE {
        #[invariant(bytes@.len() > 0)]
        #[invariant(bytes@.len() <= MAX_LEN@)]
        #[invariant(bytes@[0]@ == 47 || bytes@[0]@ == 63 || bytes@[0]@ == 35)]
        #[invariant(query@ < i@ && i@ <= bytes@.len())]
        #[invariant(bytes@[query@]@ == 63)]
        #[invariant(fragment == None)]
        #[invariant(forall<j: Int> 0 <= j && j < query@ ==>
            path_class::path_or_high(bytes@[j]@))]
        #[invariant(forall<j: Int> query@ < j && j < i@ ==>
            path_class::query_or_high(bytes@[j]@))]
        #[invariant(is_maybe_not_utf8 == (exists<j: Int>
            0 <= j && j < i@ && bytes@[j]@ >= 128))]
        #[variant(bytes@.len() - i@)]
        while i < bytes.len() {
            match query_byte_class(bytes[i]) {
                CLASS_VALID => {}
                CLASS_HIGH => {
                    is_maybe_not_utf8 = true;
                }
                CLASS_FRAGMENT => {
                    fragment = Some(i as u16);
                    break;
                }
                _ => return Err(ErrorKind::InvalidUriChar),
            }
            i += 1;
        }
    }

    Ok(Scanned {
        query,
        fragment,
        is_maybe_not_utf8,
    })
}
