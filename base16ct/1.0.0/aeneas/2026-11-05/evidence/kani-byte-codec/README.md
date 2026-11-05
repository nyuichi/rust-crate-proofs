# Original base16ct byte codec: fixed-size Kani supplement

Five original public APIs now have independent **bounded-size** contracts:
lower/upper/mixed decode for every2-byte input, and lower/upper encode for
every1-byte input. This does not close these APIs for arbitrary lengths or
capacities and adds zero Aeneas mappings. All original crate/archive files
are preserved without adaptation in source/; final evidence executes that copy.

## Decode contract

Each of three harnesses passes the entire input &[u8] of length2 and the entire
initialized destination &mut[u8] of length3 to the real API. Every byte of both
input and initial destination is symbolic. The independent specification uses
ordinary ASCII ranges/differences: digits0-9 and lowera-f, upperA-F, or both
according to the API; it does not reuse the implementation's signed-mask helper.

Ok occurs exactly when both input bytes belong to that alphabet. Its returned
slice has length1 and contains16*high+low. Otherwise the result is exactly
InvalidEncoding. InvalidLength is absent in this even-input/sufficient-capacity
domain. After the returned borrow ends, destination indices1 and2 equal their
initial arbitrary values, on both Ok and Err. **Destination index0 on Err is
unspecified here**: no unchanged-prefix claim is made. The real implementation
writes before reporting accumulated character errors. Passing full dst length3
and checking a returned length1 is distinct from passing only dst[..1].

## Encode and composition contract

Each encoder harness passes source length1 and full destination length4. Every
input byte and all four initial destination bytes are symbolic. The result is
Ok of length2, equal to an independent ordinary quotient/remainder nibble-to-ASCII
specification for that case. Destination indices2 and3 remain unchanged. Decoding
the returned output through the corresponding real API into a separate length1
buffer returns exactly the original byte. The returned-slice and mutation/frame
observations are all through safe public API references, not private observers.

## Evidence, trust and remaining limits

Five separately selected sequential harnesses pass233/233/242/388/388 checks,
1484 total with zero failures. Each uses explicit unwind8 with unwinding checks
retained and a60-second timeout; each completed below3 seconds. No assumptions,
stubs, disabled checks, Formatter model, source/LLBC edits or private-state
injection are used. All typed u8 patterns are valid API byte inputs, including
non-ASCII that decoders reject normally. The saved per-harness metadata records
show no stubs; check-results validates every successful separate log.

Configuration is x86_64/default-features/debug, Kani0.68.0/CBMC6.11.0 and Rust
nightly2026-08-21. Compiler and supported Rust/std semantics are trusted. These
results do not establish arbitrary lengths/capacities, empty/odd input, short
destinations, allocation APIs, encode_str/unchecked UTF8, constant-time timing,
release or other targets/features. They do not alter the existing encoded_len
precondition or prove overflow behavior above usize::MAX/2. Existing three
Aeneas claims and the separate restricted Display supplement remain distinct.

The preliminary shared-source-path logs are retained separately under
initial-shared-path and lower-pair.log. Final five logs reran the exact same
harness against the complete self-contained original source copy. No preliminary
result is substituted for that final source/configuration evidence.

`./verify.bash` archive-checks every source/manifest/archive file, records metadata,
runs one harness at a time and checks results. Source hashes, original archive,
compiler version, Cargo.lock, actual public calls and configuration are retained.
Publication and any report integration are handled by the coordinating publisher.
