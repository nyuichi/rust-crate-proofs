# Partial-domain Kani evidence for integer slice API instantiations

Run `./verify.bash`. `source/` is a byte-identical copy of the original endian-type 0.2.0 archive. The separate verification crate uses the unchanged path dependency and ten integer-only harnesses. `tools/check-evidence.py` guards the selected instantiations, lack of assumptions/stubs/unwind limits, archive equality, successful specifications and unreachable unsupported calls.

See VERIFICATION.md and results.json for the exact domain. api-bindings.json records 60 concrete integer specializations of the six generic APIs; inventory-delta.json proposes partial-domain Kani status for those six original generic entries while retaining generic_case_unproved=true. This does not turn 60 instantiations into 60 independently defined generic APIs or claim the arbitrary Copy-T case.

The unsafe constructor receives only initialized, live, exact-size array fields at offset zero in repr(C,align(16)) storage. The primitive size/alignment are checked. Other inputs are never manufactured. Mutable replacements are ordinary valid integer representations. No invalid-value, short-buffer, misaligned-read or Miri experiment occurs in these harnesses.

Kani 0.68.0 / CBMC 6.11.0 / Rust nightly-2026-08-21; explicit x86_64 little-endian 64-bit target guards. verification.log contains the complete run, and evidence/u16-feasibility.log preserves the initial requested feasibility check. Evidence hashes and original source provenance are retained under evidence/.
