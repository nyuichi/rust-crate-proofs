Run ../../verify-kani.bash from this directory, or verify-kani.bash from the
verification root. KANI_HOME defaults to /workspace/kani-tools/kani-0.68.0.
The official prebuilt verifier bundle must have its toolchain symlink pointing
to Rust nightly-2026-08-21, including rustc-dev, and a cargo-kani executable
(symlink to kani-driver) in bin. The script adds the compiler shared-library
path and selects the exact toolchain. Set CARGO_NET_OFFLINE=true after all
Cargo dependencies are cached if running in the restricted shell sandbox.

std_wrapping_operator_models: all u32 x,y and all usize shifts on x86_64; checks
Rust Wrapping shifts/xor against the primitive formulas used by the supplied
Lean external models. Log: std-models.log (10 checks, zero failures).

original_next_u32_matches_bitvector_formula: all four u32 state words; calls
the actual original Rust next_u32 body and checks output and entire updated
state. Log: original-body.log (107 checks, zero failures).

build.rs embeds the entire original src/lib.rs byte-for-byte and adds only a
private-state observer in its module. The implementation body is not rewritten
or copied by hand. The source SHA is recorded in ../upstream.sha256.
