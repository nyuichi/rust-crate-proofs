# ASCII code and classification guarantees

Twelve original methods are proved over all 128 valid AsciiChar values: as_byte and eleven ASCII classification methods. Proofs.asciiCode is the explicit complete code table. The classification theorems characterize exact code ranges/sets, including whitespace rejecting vertical tab. The original four classifications are included in the same single generated module and proof module as the added eight APIs.

The source and Cargo.toml are unchanged and archive-compared. Compiler lint suppression is used for upstream binding names; doctest compilation uses the corresponding official rustdoc argument. The full original test run records one current-Rust incompatibility in the separate is_digit(radix 0/1) test, then checks the remaining 36 unit, 11 integration and 49 documentation tests without changing their source.

Run verify-all.bash to check provenance, those tests, the union extraction/hash comparison and twelve theorem audits. Other APIs and feature combinations remain unproved; see proofs.toml and the separate full API inventory for exact limits. The serde-without-std configurations fail to compile unchanged and have retained investigation logs, not proofs.
