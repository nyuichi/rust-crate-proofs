# Bytes model consumer proof

This focused target checks real HTTP-side callers against the centralized
`creusot-std` model for the pinned `bytes = 1.11.1` dependency. Enable the
`bytes-model` feature to use `bytes_seq`, `bytes_mut_seq`, and
`bytes_mut_capacity`; the feature includes `std` because the model specifies
the dependency's `String` conversions and formatting implementation.

The model's external contracts describe exact byte contents for construction,
borrowing, cloning, equality/order, splitting, truncation, append/write, and
freeze. The mutable observer describes the initialized byte prefix, and the
capacity observer is separate. These are dependency premises under the
completed-Bytes-verification assumption; this target does not prove the bytes
crate itself. The HTTP module only re-exports those observers, so every caller
uses the same model identity.

Run from this directory:

```text
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove -- --locked --offline
```

The expanded run passed all 79 VCs across 36 actual consumer functions. It
covers `Bytes` construction from slices, strings, and vectors; `BytesMut`
construction/conversions, capacity and reserve; exact `AsRef` and `Deref`
views for both types; clone and equality/order; `Bytes` and `BytesMut`
split-to/split-off with both returned contents and remaining source contents;
truncate and clear; concrete `BufMut::put_slice`; `fmt::Write::write_str`; and
freeze after append/write. It also checks lengths, emptiness, capacity growth,
and the caller-visible exact sequence after each mutation. The `BytesMut`
`split_off` test includes the branch where the split point is within capacity
but beyond the initialized prefix. No general `BufMut` method is assigned an
append axiom.

Per-caller `.coma` and proof JSON are captured under `evidence/consumers/`.
`evidence/manifest.json` records each exact Why3 target/goal, source
fingerprints, and checksums for those captured artifacts.

The previous compact callers remain in the same target: `append_and_freeze`
(5 VCs), `borrow_bytes` (2), `clone_bytes` (2), `copy_from_slice` (2),
`copy_string` (3), `mutable_from_slice` (2), and `write_and_freeze` (5).

This is a leaf consumer proof, not an integrated proof of the full HTTP crate.
