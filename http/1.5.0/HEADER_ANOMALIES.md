# Observed HeaderName anomaly

The pinned `http` 1.5.0 implementation has an RFC token validation discrepancy:

```text
HeaderName::from_bytes(b"foo\"bar")       -> Err
HeaderName::from_lowercase(b"foo\"bar") -> Ok("foo\"bar")
```

RFC 9110 defines a header field name as a token; DQUOTE (`0x22`) is not in the
`tchar` set. The `HEADER_CHARS_H2` table in `src/header/name.rs` instead maps
index 34 to DQUOTE itself, so `parse_hdr` does not reject it. The independent
runtime check in `verification/runtime-check/tests/header_name_anomalies.rs`
reproduces this behavior against the ordinary runtime build.

This is recorded as behavior of the pinned upstream source, not as a validity
claim in the verification contract. The upstream implementation remains
unchanged here; the discrepancy must be resolved explicitly before claiming
RFC-conforming `HeaderName` validation.
