# Trusted base and unresolved obligations

## Fixed tool foundation

Rust `nightly-2026-06-22` (rustc `91fe22da8`), vanilla Creusot 0.13.0
`318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`, matching registry creusot-std/proc/
pearlite-syn 0.13.0 with Cargo locks, pinned Why3/Why3find and fixed solver versions.
Their translation, primitive specifications, solver logic, Rust/LLVM and Global
allocator semantics are foundational assumptions, not proved by these probes.

## Additional reviewed standard primitive contracts

- `STD-CONVERT-01`: u64 -> usize fallible integer conversion, in `src/std_specs.rs`.
- Existing `Perm::from_box`, `as_ref`, borrowed `split_at`, `to_box`, `live`, and
  `PtrAddExt::add_live`: actual allocation/borrow permission foundations.
- `STD-PTRDIFF-01`: u8 native offset_from, requiring both pointers in the same
  live-allocation range with matching provenance. No ZST case.
- `STD-LAYOUT-01`: from_size_align restricted to positive size <= isize::MAX and
  alignment one. It is not a general complete Layout specification.
- `STD-DEALLOC-01`: native dealloc requiring the base and matching (length,1)
  layout of a full owned Box slice permission. It consumes that unique token and
  is not callable in ghost code. A borrowed/partial slice token is insufficient.

The last three are in the isolated deallocation probe and were reviewed by Astra.
Owned slice permission splitting, if later added, requires re-auditing the
full-allocation token interpretation used by deallocation.

## Not admitted as bytes TCB

Clone/drop/refcount/unique/freeze/promotion/vtable correctness, initialized-capacity
ownership, Send/Sync resource transfer, and unwind cleanup remain proof targets.
No bytes-specific contract for them has been trusted to pass the crate.

## Experimental tool patch

The impure specification-free trait self-bound patch is an experiment, not part
of the standard toolchain used by helper/storage/deallocation proof results.
Astra reviewed its restrictions and identified the external-spec guard. Dedicated
positive and negative regressions are recorded; broader verifier soundness and
invariant/resolve/spec dependency cycles remain review concerns.
