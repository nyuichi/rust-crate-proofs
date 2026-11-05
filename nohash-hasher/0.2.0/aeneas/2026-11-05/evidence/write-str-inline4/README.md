# Original generic Hasher::write_str call: all-string panic contract

For nohash-hasher0.2.0, the original nightly Hasher::write_str default first calls
self.write(s.as_bytes()), then write_u8(0xff). The original NoHashHasher.write
unconditionally panics, so the second call is unreachable. Four kernel-checked
contracts prove the actual compiler-inlined generic public call returns modeled
Result.fail panic for every represented T under the original IsEnabled bound,
every receiver state and every represented Str. This includes all valid Rust
strings, empty or nonempty, without a fixed buffer/length/content restriction.
No extra string-type axiom, dictionary, standard model or private observer is used.

The four executions separately cover dependency std/default and no-default-
features, each with debug assertions enabled/disabled, x86_64/nightly2026-09-17.
The drivers compile-assert their debug configuration. Actual dependency feature
resolution is saved. Receiver representations remain correctly distinct between
debug and release. The complete write_text declaration is byte-identical in all
four outputs; source/module namespaces are outside that compared declaration.

Official rustc passes used are Inline, RemoveStorageMarkers,
ReferencePropagation, DeadStoreElimination-final and SimplifyLocals-final, through
official Charon optimized MIR and unchanged official Aeneas. Initial Inline/DCE
attempts retained a dead TwoPhaseMut copy and failed with “Can't copy a mutable
borrow”. Those raw diagnostic logs are preserved; intermediate failing LLBC was
not separately frozen before subsequent official re-extraction. The exact pinned
compiler pass pipeline source is saved. No LLVM/LLBC/source/generated body was
hand-edited. The successful final LLBC contains no remaining function calls or
reads of its string argument and retains original default/write panic source
spans; SOURCE-IR-GUARD records and checks this. The original first write does not
read byte contents. The proof relies on compiler inlining/dead-code cleanup and
official extraction, not a proof of compiler correctness or a panic stub.

Each new Lean process had30second/4500MiB caps; all passed. The four new theorem
axiom audits use only propext/Quot.sound. Simultaneous import with old public
Proofs and the prior13 generic-family declarations audits all17 using standard3
only. This Result contract does not specify Rust panic hooks, unwinding,
messages/locations, physical memory/aliasing/lifetime behavior, const evaluation,
other targets or compiler settings. It is an inlined-caller guarantee, not a direct
original default-method source-card mapping. Primary Aeneas count increase0.

The complete original crate archive is included and extracted byte-identically.
Executed Cargo.producer.toml files retain their original dependency locations;
portable Cargo.toml files change only that path to ../original. This relocation
is packaging, not another execution. Frozen generated metadata/comments retain
the executed absolute source locations. The prior3API generic family and old
public packet are unchanged and remain the base for the simultaneous-import
check. No install, shared CLI, source-card publication or server operation occurs.
