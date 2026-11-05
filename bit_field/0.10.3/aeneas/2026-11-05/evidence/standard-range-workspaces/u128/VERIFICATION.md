# u128 standard-range verification supplement

Original bit_field 0.10.3 u128 get_bits/set_bits are invoked through two external UFCS callers and compiler-inlined by the official nonmonomorphizing route. Only core::ops::Range<usize> is selected. Source/Cargo/model changes are zero. This does not complete arbitrary RangeBounds, signed types or BitArray.

For every word and usize endpoints, valid means start<128 and start<=end<=128. The getter returns (word/2^start)%2^(end-start), including zero for empty valid ranges; invalid endpoints assert. Setter invalid endpoints assert. Empty valid ranges with zero replacement preserve the word and permit arbitrary returned-reference writeback; nonzero replacement fails with integerOverflow. Nonempty valid ranges accept exactly replacement<2^(end-start), replace the selected bits, preserve outside bits and permit arbitrary writeback; nonfit values assert. CompleteSpec states this exhaustive all-input Result semantics.

The configuration is x86_64 little-endian default/debug overflow-checking extraction. No release-profile, Rust panic payload/unwinding, runtime allocation/address or generic RangeBounds guarantee is claimed. The existing PrimitiveRustLayout provider is copied unchanged; actual size_of U128 uses existing layout_u128 (sixteen bytes), not a new instance or arbitrary Rust type model. Its class remains open and the frozen ground-use closure is the boundary.

33 audited theorem names include helper arithmetic/layout proofs, not 33 APIs. All use only the standard three permitted Lean axioms. No new Kani run is claimed for this width. The kernel theorems cover all u128 values and usize endpoint inputs directly; u8/u16 Kani evidence is not transferred to u128.

Old artifacts are unchanged. An exact u128 coimport has not been run; prior u8/u16 probes encounter duplicate global instDiscriminantBoundIsize. This remains a separate verification environment, without claiming u128 coimport success. This is a separate library environment with this collision unresolved. Primary count addition zero and publication held; source binding/integration/fresh and independent review are separate gates.
