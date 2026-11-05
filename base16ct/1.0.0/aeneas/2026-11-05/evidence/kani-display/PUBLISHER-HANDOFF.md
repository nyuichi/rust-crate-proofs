# Supplement handoff: original base16ct Error Display, restricted Kani domain

Ready for publisher-managed integration, not yet published by this task.
Keep the three existing Aeneas mappings unchanged (encoded_len, decoded_len,
and From<Error>→fmt::Error). This supplement is not a fourth Aeneas claim.

Suggested public guarantee: For either base16ct1.0.0 Error variant, ordinary
default {} formatting into the specified initialized64-byte accepting writer
returns exactly `invalid Base16 encoding` (23bytes) or `invalid Base16 length`
(21bytes). If that writer rejects the attempted literal before storing bytes,
formatting propagates Err and leaves output length/bytes unchanged. Both error
variants and both writer modes are symbolic; all four cases are checked.

Evidence is the unchanged original-body Kani run: one harness/1910checks,
zero failures/125 internal unreachable checks,11 successful harness properties,
three successful unsupported-call reachability guards, explicit unwind80 with
unwinding checks retained. No assumptions/stubs/Formatter model are used.
Source/manifest/all archive files, metadata, actual literal lengths and successful
log were independently re-audited. Packaging reuses this completed run rather
than pretending a new fresh Kani execution; verify.bash can reproduce it.

Trust/limits: compiler/Kani0.68.0/CBMC6.11.0/Rustnightly2026-08-21 and supported
std semantics; x86_64 default features/debug; this concrete fixed writer and
default format only. Arbitrary Formatter options/writers, other feature/target
configurations, partial-prefix failure policies and general Display remain
unproved. The three existing Aeneas scopes and encoded_len overflow exclusion
remain unchanged. Decode/encode/UTF8/error-default APIs are not covered here.

An explicit editorial erratum corrects the old results JSON's22/20 lengths to
23/21. The successful source/harness always checked23/21, so theorem/evidence
behavior did not change. Original packet and old JSON remain separately retained.
