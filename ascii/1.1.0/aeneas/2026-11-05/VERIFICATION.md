# ASCII code and classification guarantees

Twelve original methods are proved over all 128 valid AsciiChar values: as_byte and eleven ASCII classification methods. Proofs.asciiCode is the explicit complete code table. The classification theorems characterize exact code ranges/sets, including whitespace rejecting vertical tab. The original four classifications are included in the same single generated module and proof module as the added eight APIs.

The source and Cargo.toml are unchanged and archive-compared. Compiler lint suppression is used for upstream binding names; doctest compilation uses the corresponding official rustdoc argument. The full original test run records one current-Rust incompatibility in the separate is_digit(radix 0/1) test, then checks the remaining 36 unit, 11 integration and 49 documentation tests without changing their source.

Run verify-all.bash to check provenance, those tests, the union extraction/hash comparison and twelve theorem audits. Other APIs and feature combinations remain unproved; see proofs.toml and the separate full API inventory for exact limits. The serde-without-std configurations fail to compile unchanged and have retained investigation logs, not proofs.

## Additional original API proofs

The same union extraction now includes eq_ignore_ascii_case and the checked/unchecked ToAsciiChar implementations for AsciiChar itself. Equality is specified using an independent mathematical A-Z-to-a-z code fold and proved for all 128×128 valid pairs using kernel reduction; this does not prove the separate case-conversion APIs. The two conversion implementations are identities on valid AsciiChar values. No unchecked integer/char conversion or invalid enum representation is covered. verify-all checks fifteen API theorems plus the independent casefold helper.

## Separate Kani body checks of checked integer/char conversions

The unchanged public checked ToAsciiChar implementations for u8, i8, u16, u32 and char were also directly verified by Kani over their complete type domains. Ok is returned exactly for ASCII-range values and preserves the codepoint; all other inputs return ToAsciiCharError. See evidence/kani-conversions/README.md and provenance.json for the exact domains, direct observations, source relation and different toolchain. These five body checks and one valid-discriminant intrinsic check are distinct from the twenty-one Aeneas API mappings. No invalid enum state is constructed or claimed verified.

## Separate full-domain Kani case and character checks

Seven additional direct original-body checks establish uppercase/lowercase mapping, exact mutable writeback, as_char code preservation, mathematical case-insensitive equality and printable control pictures over all 128 valid values (all 128² pairs for equality). The safe original input constructor and its public observation check ensure complete enum-domain coverage without creating invalid enum states. See evidence/kani-case-operations; non-ASCII constructor behavior remains outside the proof. The Aeneas cast blocker remains recorded even though these original Rust bodies pass Kani.

## Six additional public predicate/radix APIs

The union now includes is_alphabetic/is_alphanumeric/is_lowercase/is_uppercase, is_whitespace (including VT) and is_digit for all 128 characters and all u32 radix values. The digit theorem partitions the whole input domain: radix 0–36 returns exact digit-value comparison; radix >36 is a model failure corresponding to the original Rust panic. It does not claim a panic message payload. Radix 0 always returns false and radix 1 accepts only the character '0'. DigitProofs normalizes scalar representations through proved unsigned-value lemmas; it introduces no axiom or Rust adaptation. The current machine mappings are twenty-one Aeneas APIs; separate Kani evidence remains separately labeled.

## Additional original API bodies

All 24 original roots are extracted together. is_ascii_punctuation returns exactly the four standard punctuation intervals; handwritten Default returns NUL. valid_up_to returns the stored usize field for every representable index; the theorem is a getter specification, not a proof that error constructors identify the first bad byte. Existing separate Kani checks remain distinct.

## Four original AsciiString APIs

The union extracts 28 original roots without Rust adaptations. The official sequence-based Vec model establishes new has no elements, len is exact, is_empty iff no elements, and push appends exactly one supplied character with complete receiver writeback. The push theorem covers all model lengths: below usizeMAX it appends; at usizeMAX it fails maximumSizeExceeded. Capacity, the real isizeMAX allocation bound, OOM and arbitrary allocator failure are not represented or verified. No real Rust panic partition is inferred from the model partition. Unsupported original pop/clear/truncate external models remain unproved and are not included.

## Four original AsciiStr slice APIs

Singleunion32 roots now includes as_slice/as_mut_slice/len/is_empty without unsafe casts. All representable slice lengths are covered: initial slice elements are preserved, mutable backward writeback installs each valid same-length replacement exactly, len is exact and is_empty iff empty. The replacement length premise matches mutable-slice operations and does not claim Rust slice resizing. These are value/backward-function reference contracts; pointer identity, compiler borrow checking and lifetime/aliasing soundness are not separately proved.

## Three already-ASCII identity implementations

The receiver is already AsciiStr. Its checked AsAsciiStr conversion returns Ok with the same view; its unchecked immutable conversion preserves the view; the AsMutAsciiStr unchecked identity preserves the initial view and same-length valid replacement writeback. This is not byte validation or a guarantee for other implementations. The reproduction builds original proof source once, then simultaneously imports and audits all machine-claim modules exactly as the CLI does; it avoids re-elaborating identical source a second time.
