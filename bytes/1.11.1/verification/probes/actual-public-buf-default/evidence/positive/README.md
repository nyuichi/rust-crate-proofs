# Positive evidence snapshot

The proof run translated the extracted default and model-law contracts, then
proved 13 generated files with no null/unproved obligations. The core trait
default `try_get_u8`, its generic caller, and the `&[u8]` implementation's
`remaining`, `chunk`, and `advance` methods are in the preserved Why3 inputs and
proof JSON. All successful VCs used Alt-Ergo.

`artifacts.tar.gz` contains the translated Coma inputs and proof JSON under
`verif/`, plus the exact generated Rust and source fragments under
`extraction/`. `membermanifest.txt` records the member hashes and
`archiveSHA256.txt` records the archive hash. `../../logs/positive.log` is the
complete runner output. The native test output and pinned tool manifest digest
are recorded beside this snapshot.

Scope: this is a feasibility result for a source-sliced copy of public `Buf`
with proof-only model/laws. It is not evidence that the laws are present on the
actual bytes source trait, or that all current/downstream implementations
discharge them.
