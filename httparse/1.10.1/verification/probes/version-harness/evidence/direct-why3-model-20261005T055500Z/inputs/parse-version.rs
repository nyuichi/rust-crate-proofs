// The exact `parse_version` implementation included at crate root.

#[requires(crate::verification_model::valid_cursor(bytes@))]
#[ensures((^bytes)@.input == bytes@.input
    && (^bytes)@.mark == bytes@.mark
    && (^bytes)@.end == bytes@.end)]
#[ensures((^bytes)@.cursor == crate::verification_version::parse_version_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).cursor)]
#[ensures(match result {
    Ok(Status::Complete(version)) =>
        crate::verification_version::version_result_is_complete(
            crate::verification_version::parse_version_model(
                bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end),
            version@),
    Ok(Status::Partial) =>
        crate::verification_version::version_result_is_partial(
            crate::verification_version::parse_version_model(
                bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(Error::Version) =>
        crate::verification_version::version_result_is_error(
            crate::verification_version::parse_version_model(
                bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(_) => false,
})]
#[inline]
#[doc(hidden)]
#[allow(missing_docs)]
// WARNING: Exported for internal benchmarks, not fit for public consumption
pub fn parse_version(bytes: &mut Bytes) -> Result<u8> {
    if let Some(eight) = bytes.peek_array8() {
        // NOTE: should be const once MSRV >= 1.44
        let h10_bytes: [u8; 8] = [b'H', b'T', b'T', b'P', b'/', b'1', b'.', b'0'];
        let h11_bytes: [u8; 8] = [b'H', b'T', b'T', b'P', b'/', b'1', b'.', b'1'];
        let h10: u64 = u64::from_ne_bytes(h10_bytes);
        let h11: u64 = u64::from_ne_bytes(h11_bytes);
        // SAFETY: peek_array8 returned Some, so at least 8 bytes are available.
        unsafe {
            bytes.advance(8);
        }
        let block = u64::from_ne_bytes(eight);
        // NOTE: should be match once h10 & h11 are consts
        return if block == h10 {
            #[cfg(creusot)]
            ghost! {
                crate::verification_version::native_pack8_injective(
                    snapshot!(eight@),
                    snapshot!(h10_bytes@),
                );
            };
            Ok(Status::Complete(0))
        } else if block == h11 {
            #[cfg(creusot)]
            ghost! {
                crate::verification_version::native_pack8_injective(
                    snapshot!(eight@),
                    snapshot!(h11_bytes@),
                );
            };
            Ok(Status::Complete(1))
        } else {
            Err(Error::Version)
        };
    }

    // else (but not in `else` because of borrow checker)

    // If there aren't at least 8 bytes, we still want to detect early
    // if this is a valid version or not. If it is, we'll return Partial.
    expect!(bytes.next() == b'H' => Err(Error::Version));
    expect!(bytes.next() == b'T' => Err(Error::Version));
    expect!(bytes.next() == b'T' => Err(Error::Version));
    expect!(bytes.next() == b'P' => Err(Error::Version));
    expect!(bytes.next() == b'/' => Err(Error::Version));
    expect!(bytes.next() == b'1' => Err(Error::Version));
    expect!(bytes.next() == b'.' => Err(Error::Version));
    Ok(Status::Partial)
}
