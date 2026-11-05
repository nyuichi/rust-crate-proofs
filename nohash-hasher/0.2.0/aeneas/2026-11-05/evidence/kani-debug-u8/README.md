# Original NoHashHasher Debug supplement: u8 finite domain

This supplement checks the original hand-written `Debug::fmt` only for
`NoHashHasher<u8>` constructed by `Default`, with zero writes or one `write_u8`
of an arbitrary u8, using default `{:?}` formatting into a concrete fixed
64-byte writer. It does not establish general Formatter behavior, all stored
u64 values, other integer write methods, or the remaining inherited APIs.

The release source and Cargo.toml in `source/` match the retained crates.io
archive byte for byte. No adapted implementation or Aeneas Formatter model is
used. The pinned Aeneas Formatter model ignores output and therefore cannot
supply this contract; Kani reaches the original Rust `fmt` through `write!`.

## Useful checked behavior

The independent output specification requires the name/tuple delimiters,
canonical unsigned decimal digits, and equality of the parsed decimal number
to the expected hasher state. Parsing is checked by multiplication/addition,
not by calling Rust formatting to generate an expected string. Leading zeroes
are forbidden except for zero itself. The expected state is zero before any
write and the supplied u8 after one write. Thus initial state and all 256
single-write values are included, including writing zero (which differs from
initial state in debug mode).

With debug_assertions enabled, the tuple has a second boolean field: false
before writing and true afterwards. With debug_assertions disabled that field
is absent. The wrapper compile-time feature guards reject an unexpected mode;
Cargo's debug-assertions profile setting applies to the dependency as well.
This is a mode distinction, not a claim about checking panicking second writes.

The writer either accepts every call or returns `fmt::Error` on its first
write attempt. Accepting mode must return Ok, emit exactly the checked tuple,
and leave unused bytes at zero. Error mode must make a nonempty write attempt,
return Err, and leave its length and all 64 bytes unchanged at zero. This checks
real error propagation through `debug_tuple`, field formatting, and `finish`.
It does not quantify over arbitrary writers, failure after a partial prefix,
capacity exhaustion, pretty/alternate formatting, width/precision, or other
Formatter flags. The writer has sufficient capacity for the bounded contract.

## Tools and reproduction

`./verify.bash` archive-checks source, verifies the Kani harness separately with
debug_assertions true and false, then runs native exhaustive tests in debug
and release. Both Kani configurations keep unwinding assertions enabled with
explicit bound 80. The test harness has no assumptions, stubs or disabled
checks. Fixed three-digit and 64-byte loops avoid unbounded postcondition
loops while retaining the entire u8 contract.

The native tests enumerate 256 values × 2 write states × 2 writer modes = 1024
input valuations per configuration. Initial-state valuations repeat the same
state because the unused candidate value varies; they are not 1024 distinct
stored states. Native tests are recorded separately from Kani proofs.

Kani reports unsupported panic/caller-location and unallocated-pointer constructs
in the reachable-code inventory; all three corresponding reachability guards
succeed in each completed run. These paths are unreachable in this checked
domain; no unsupported behavior was stubbed or silently accepted.

Early probes with dynamic postcondition loops were stopped due solver cost;
their source/logs remain under evidence. A `--stop-on-fail` diagnostic is not a
valid verification result: it stops on Kani's intentionally false assertion
reachability marker, and the structured parser also panicked for that output.
`cbmc-properties.txt` identifies the marker and its associated overflow check.
Only the ordinary completed final Kani runs count as successful evidence.

No live publication or inventory promotion is performed by this packet.
`api-binding.json` marks this as a partial-domain supplementary result.
