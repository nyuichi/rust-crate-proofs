# Exact compiler-commit source followup (no new solver run)

rustc1.100.0-nightly, commit8925ea358a0f265ca61026aadc7ecc506c545cbe,
2026-08-20, x86_64-unknown-linux-gnu, LLVM23.1.0. Queried actual Kani-selected
nightly2026-08-21 rustc --version --verbose. Public GitHub connector retrieved
rust-lang/rust raw files pinned to this commit: library/core/src/any.rs,
clone.rs, clone/uninit.rs. Direct shell network failed; no outgoing write used.

Any blanket (any.rs140–145) returns TypeId::of<T>() without reading self.
TypeId::of (767–769) is const intrinsics::type_id<T>(). Therefore the state-
independence bridge follows from the exact body, while global TypeId uniqueness
and compiler intrinsic correctness remain compiler trust, not established by
two concrete Kani comparisons.

CloneToUninit blanket (clone.rs647–654) calls CopySpec::clone_one. The default
clone_one (clone-uninit.rs12–22) ptr::write(dst,src.clone()); the specialization
for TrivialClone (54–63) uses ptr::copy_nonoverlapping(src,dst,1). The original
Rand32/64 derive(Clone,Copy) participates in TrivialClone (clone.rs258–300).
Thus describing execution as necessarily calling derived Clone was too strong:
the Copy specialization is relevant, consistent with logged copy UB checks.
No allocation is present in either sized body. This manual source observation
is not a machine proof for all heap provenance or arbitrary valid destinations.

Safety docs (clone.rs625–631): valid writes for size_of_val(self) bytes and
proper alignment. The actual specialized copy additionally requires nonoverlap,
satisfied by the tested distinct source and MaybeUninit stack slots. Pointer
validity, allocation extent, alias/provenance and caller obligations remain
explicit; invalid-pointer experiments are neither needed nor performed.
Only the fixed-slot all-state Kani result is claimed. Reviewed exact08 source
now resolves the earlier missing-source review gap without changing prior logs,
proof runs, original crate or harness. Prior RESULTS records that earlier state.
