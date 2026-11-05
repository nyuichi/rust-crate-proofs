# nohash Debug no_std dependency variant — verified limited domain

The original dependency source/manifest and Kani harness are byte-identical to
the fixed std Debug packet. Only this wrapper manifest disables dependency
default features; the proof harness remains an ordinary std consumer.

Cargo metadata confirms empty nohash-hasher features in both configurations.
Original cfg_attr therefore selects no_std. Compile-time root guards enforce
expected debug_assertions true/false. The separate std-only IntMap fixture
fails with the expected E0432, rejecting accidental std-feature enablement.
The intentional compile failure is a configuration guard, not a failed proof.

Separate no_std Kani runs succeeded: debug2175/release2143 checks, each zero
failures and159 unreachable internal checks. Unwind80 and all unwinding checks
remain enabled. No stubs, assumptions or suppressed checks are used. All three
unsupported-construct reachability guards succeed in each completed run.
The prior resource hold was lifted before these runs; jobs ran sequentially.

The unchanged useful contract is u8-only: Default then zero/one write_u8 of any
u8, default {:?}, fixed64 writer accept-all or first-error. It checks independent
canonical decimal parsing/state, tuple structure, debug boolean presence/value,
error propagation and unchanged unused bytes. General Formatter, all stored
u64, other integer writes, other writer/options, or inherited APIs remain
outside the guarantee. See the fixed std packet README for the same detailed
contract. Its std results are separate, never substituted for these no_std logs.

`./verify.bash` reproduces metadata/configuration checks and both no_std proofs.
`results.json` preserves each result. The four-configuration candidate copies
both complete variants while retaining their distinct configurations/evidence.
