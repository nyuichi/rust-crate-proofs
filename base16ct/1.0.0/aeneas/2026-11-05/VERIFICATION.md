# Base16 lengths and formatting-error conversion

The original decoded_len returns half the length when even and InvalidLength when odd, without examining bytes. encoded_len returns twice the length provided length ≤ usize::MAX/2. From<Error> for core::fmt::Error maps either source error to the formatting error value, terminating normally and discarding the original variant.

All three original roots are extracted together into one Generated.Base16ct module and checked in one Proofs module. verify-all.bash checks archive provenance, original tests, regeneration hashes and all three transitive theorem audits. Default features on x86_64 are covered; codec bodies, actual formatting, allocation, unchecked UTF-8 conversion and other features remain unproved.

A separate targeted extraction attempt of the codec APIs fails in the official backend with Arrow types are not supported yet and further translation errors. That is recorded as a proof-tool obstacle, not as proof of a defect or as permission to exclude those APIs from the full inventory.
