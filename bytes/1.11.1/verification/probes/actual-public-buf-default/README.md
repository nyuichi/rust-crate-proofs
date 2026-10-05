# Actual public `Buf::try_get_u8` default probe

This probe extracts the actual public `Buf` declarations for `remaining`,
`chunk`, `advance`, and the default `try_get_u8` from bytes 1.11.1. It also
extracts the exact `remaining`, `chunk`, and `advance` bodies for the in-repo
`impl Buf for &[u8]`. A round-trip guard fails the build if any extracted body
changes without this probe being regenerated. The default method body is not
rewritten as a local predicate or helper.

The generated proof surface is deliberately a source-sliced copy of the public
trait. It adds only a Creusot logical `unread` sequence and candidate contracts
for the public required methods. The contracts capture the minimum cursor laws:
remaining is the unread model length, chunk is a nonempty prefix when unread
data exists, and advance removes the requested prefix. The selected slice
implementation discharges these laws using exact extracted runtime bodies and
the existing verified `slice_ops::advance_slice` helper. Generic and concrete
callers then exercise the actual default `try_get_u8` through trait dispatch.

This is an isolated feasibility probe, not an integrated change to
`src/buf/buf_impl.rs`. It does not prove every in-repository implementation or
downstream implementation, and it does not establish the contracts on the
actual source trait until those proof-only contracts are added there and every
admitted implementation is checked. No public law is trusted in this probe.

The positive run used the pinned bytes toolchain and proved 13 files with no
unproved VCs. In particular, the actual default and its generic caller proved,
and the selected slice implementation discharged the `remaining`, `chunk`, and
`advance` contracts. `cargo test --locked` also passed three native checks for
successful reads, the empty error case, and concrete wrapper dispatch. The
translation output, exact source fragments, proof JSON, and proof log are
preserved under `extraction/` and `evidence/positive/`.

To reproduce, source `/workspace/bytes-proof-tools/activate.sh`, enable offline
Cargo mode, and run the locked Coma translation followed by the serialized
Why3 proof command from this directory under `/tmp/itoa-creusot-proof.lock`.
The repository wrapper should include this probe before it becomes a
maintained verification gate.
