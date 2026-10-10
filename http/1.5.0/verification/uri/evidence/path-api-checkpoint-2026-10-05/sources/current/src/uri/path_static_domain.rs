//! Public, verifier-only description of the inputs accepted by
//! `PathAndQuery::from_static`.

use creusot_std::prelude::{logic, pearlite, Int, Seq};

#[doc(hidden)]
#[logic(open)]
pub fn path_static_path_byte_valid(byte: Int) -> bool {
    pearlite! {
        byte == 33 || (36 <= byte && byte <= 59) || byte == 61
            || (64 <= byte && byte <= 95) || (97 <= byte && byte <= 122)
            || byte == 124 || byte == 126 || byte == 34 || byte == 123 || byte == 125
    }
}

#[doc(hidden)]
#[logic(open)]
pub fn path_static_query_byte_valid(byte: Int) -> bool {
    pearlite! {
        byte == 33 || (36 <= byte && byte <= 59) || byte == 61
            || (63 <= byte && byte <= 126)
    }
}

/// Exact source-byte domain on which `PathAndQuery::from_static` returns.
#[doc(hidden)]
#[logic(open)]
pub fn path_static_input_is_valid(bytes: Seq<u8>) -> bool {
    pearlite! {
        bytes.len() > 0 && bytes.len() <= 65534
            && (
                (bytes.len() == 1 && bytes[0]@ == 42)
                || (
                    (bytes[0]@ == 47 || bytes[0]@ == 63)
                    && (
                        (forall<i: Int> 0 <= i && i < bytes.len()
                            ==> path_static_path_byte_valid(bytes[i]@))
                        || (exists<query: Int>
                            0 <= query && query < bytes.len()
                                && bytes[query]@ == 63
                                && (forall<i: Int> 0 <= i && i < query
                                    ==> path_static_path_byte_valid(bytes[i]@))
                                && (forall<i: Int> query < i && i < bytes.len()
                                    ==> path_static_query_byte_valid(bytes[i]@))
                        )
                    )
                )
            )
    }
}
