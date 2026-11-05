# Exact integer and floating bit serialization, with explicit failure conditions

byteorder 1.5.0: fixed-width unsigned and signed integer writes, plus BE/LE write_f32/write_f64, write their exact bit representations in the selected endian order and preserve trailing bytes when the output is long enough; shorter buffers panic before writing. Float proofs quantify over every 32/64-bit representation, including NaN payloads and signed zeros, under explicit bit-pattern and std to_bits models. No floating arithmetic is modeled. The two handwritten Default methods always panic. 

F32/F64 are explicit 32/64-bit logical patterns; two executable std to_bits models preserve those patterns as unsigned words, with checked Lean value-preservation lemmas. Guards require original writer bodies and allow no other float operation. Kani checks std bit identity for every pattern and actual released writers with 6/10-byte buffers. Connecting these Rust checks to the Lean representation is reviewed, not mechanized; Rust nightlies differ. Official translation/standard integer and slice models, Lean, and Kani/CBMC are trusted. Mapped audits permit only propext, Classical.choice and Quot.sound.

This covers 20 selected writer methods, six unsigned readers, four native-target unsigned identity conversions, and two handwritten Default methods, not all public APIs. Fixed 16/32/64/128-bit reads, signed readers, variable-width writers, other bulk and in-place conversions, I/O extensions, other targets and release extraction remain unproved. The float result establishes bit serialization only: floating arithmetic, comparisons, rounding, numeric accuracy, NaN arithmetic and signaling behavior are not modeled. Kani public-writer buffer lengths are fixed at 6 and 10; arbitrary valid slice lengths are quantified separately by the Lean theorems. Panic messages and unwinding side effects are not specified.

Archive-identical Rust/Cargo.toml; x86_64 default features, debug extraction. Rust nightly-2026-09-17, Charon08d7c9d50144bd84848212c24701253585e90d1f, Aeneas be86ab0f1989d04944292b345b24c435be3ad10a, Lean4.35.0. Kani0.68.0/CBMC6.11.0/Rust nightly-2026-08-21: six harnesses, 276 checks, no failures per crate.

Evidence: proofs/FloatProofs.lean; unchanged raw FunsExternal_Template and filled executable FunsExternal; model/provider/use guards; evidence/kani-float source, results, full log, before-inventory and after source fingerprints. Original Rust source and manifest are archive-compared. Kani covers all float bit patterns but only fixed 6/10-byte public buffers; Lean separately quantifies over every valid slice length under the stated models.

## Unsigned variable-width reads

BE/LE read_uint/read_uint128 quantify over every byte slice and count: valid counts are 1 through 8/16 with enough input. Normal results exactly decode the selected prefix with high zero padding; invalid guards produce assertion failure. The proofs invoke the extracted original bodies directly and add no standard model. Fixed-width unwrap-based reads remain Formatter-axiom-blocked. All mapped proof modules are also imported and audited together.

## Unsigned24 prefix reads and exact body reuse

BE/LE read_u24 returns the exact selected-endian three-byte unsigned prefix decode, bounded below2^24, and asserts on shorter input. The selected original dependency read_uint callee and the existing root-extracted original read_uint body are both present and source-traced to archive-identical src/lib.rs. Their generated executable bodies are text-identical and their full function applications are definitionally equal, proved by U24Bodies. The read_u24 theorem targets the actual selected original default body and uses this checked body equality to apply the already proved root contract. No opaque alias, arbitrary callback assumption or reduced axiom audit is used. Both extraction paths are regenerated and hash-checked.

## Native unsigned identities: version-specific scope

On x86_64 little-endian/default features/debug extraction, the original provided LittleEndian from_slice_u16/u32/u64/u128 methods leave every mutable slice unchanged and return normally, for all lengths and all unsigned values. Official nonmono split-file generation retains the exact identity bodies. Five unused declarations from discarded target branches remain in the untouched unimported external template; the provider supplies only imports, zero definitions. Exact body/use guards require zero external references and no fallback model.

This scope was independently checked for byteorder1.5.0. The signed four from_slice methods are inherited defaults that always reinterpret signed slices through unsafe from_raw_parts_mut; they are not proved here. Floating conversion methods and opposite-endian/other-target paths remain unproved. The byteorder-lite eight-identity result is not assumed applicable to this version.
