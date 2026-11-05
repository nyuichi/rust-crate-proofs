# Original ASCII scalar API supplements

Thirty scalar conversion/comparison contracts passed 1646 Kani checks, and five unsafe scalar conversions passed 18 checks over precisely their documented ASCII safety domains. These are separate original-Rust Kani results; the existing 51 Lean mappings and their preconditions are unchanged. No solver was rerun during packaging.

The frozen producer manifests and all listed files are copied byte for byte. Their dependencies contain the nine runtime src files and Cargo.toml matching the original archive; archive-root tests.rs is absent and crate tests were not run for these checks. Complete source-package reconstruction is not claimed. Original archives and executed-path provenance remain included. Kani nightly August 21 and the September inventory/compiler are distinct; no cross-compiler equivalence is claimed.

The scalar conversions return exact codes. Bidirectional u8/valid-Unicode-char comparisons agree with integer equality/order, including inherited ne/lt/le/gt/ge on manual implementations. Unsafe u8/i8/u16/u32/char conversion is covered only for codes 0..127; larger values with ASCII low bytes are excluded. Invalid representations, other configurations, general memory safety and all-API completion are not established.
