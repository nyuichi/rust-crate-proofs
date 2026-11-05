# Original static AsciiStr default — all inputs

The original manual `<&'static AsciiStr as Default>::default` returns an empty ASCII string: len0, is_empty true, empty character slice and empty byte contents. Its direct Kani harness passed25 checks with no failures. The API has no parameters, so the empty output check covers the complete API input domain; it is not an arbitrary-slice API tested only on a fixed empty buffer.

This manual reference-type implementation is distinct from excluded derived AsciiString::default and the existing AsciiString::new proof. Its unique exact strict-inventory binding is retained. The original body calls From on an empty character slice, and original pointer/view operations are exercised by Kani without source adaptation, new models or suppressed checks. No Aeneas rawmetadata route was retried.

All runtime src9 and Cargo.toml match the original archive and remain unchanged. Kani0.68/CBMC6.11/nightly2026-08-21 compiler/std semantics and default x86_64 configuration are trusted; pointer address identity, other configurations/targets and const evaluation are not claimed. The returned static reference type is checked by the Rust compiler. Archive root tests.rs is not copied or run; full upstream archive is retained. Publication/source binding and primary counts remain separate gates.
