# Original inherited length-prefix behavior: Kani supplement

All900 checks passed in four independent debug-assertion/feature configurations.
For each of the ten original enabled integer T instantiations, every usize length
written via the actual std Hasher::write_length_prefix from Default initial
NoHashHasher is observed by finish() as length as u64. This passed260 checks each
with dependency default/std and with default-features=false.

Two debug-assertions=false configurations additionally quantify every u64 prior
value and a bool selecting either Default or a prior public write_u64, then
check prefix overwrites finish() with the length. Each passed190 checks. The
harness remains an ordinary std binary when the dependency is no_std. Per-variant
metadata records actual dependency feature resolution; release-style variants
have compile-time !cfg(debug_assertions) guards and explicit profile.dev setting.
Kani does not accept --release; that initial CLI rejection is preserved separately
and is not counted as a verification run.

No observer, private-state injection, model or original Rust change is used.
This is all-input length/prior-value evidence for the specified ten concrete T,
not an arbitrary user-defined IsEnabled implementation theorem. The debug already-
written panic case and generic arbitraryT closure are not proved here. These
limitations prevent calling the entire inherited generic API fully verified.
The actual Rust signature is nightly hasher_prefixfree_extras, not a stable API.

Official Charon extracted the original core default body, which calls write_usize
once. Official Aeneas emitted it, but Lean rejected the generated field access:
its existing Hasher dictionary has only finish/write, not write_usize. The exact
invalidField error and unmodified generated output are retained. No new model was
added to hide this obstacle. Aeneas machine API count increases by0.

The run used the previously published dependency's archive-identical Rust and
normalized Cargo.toml. That dependency copy inherited proof-report documentation
and omitted some archive dotfiles/Cargo.toml.orig. Packaging restored the complete
original archive byte-for-byte afterwards; this was a documentation/packaging
repair, not another proof run. The exact run/portable distinction is in RESULTS.
The pinned std08 core hash source was independently fetched by read-only official
GitHub connector; std09 extraction remains a separate tool configuration.
