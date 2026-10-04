# Sequential native counter bridge

This probe exercises the exact native `AtomicUsize` operations needed by a
sequential Shared protocol. `scalar_transitions` and `two_handle_countdown` are
body-proved callers. The four primitive methods in
`src/ownership_proof/sequential_counter.rs` are an explicit additional trusted
boundary, not proved atomic implementations.

The private native cell is associated with one sealed identity. `CounterOwn`
contains a private exclusive resource with that identity and current scalar;
it cannot be cloned, sent, or shared across threads. Creating a new counter
creates its only ownership token. Operations require matching ownership,
return the previous scalar for updates, and preserve identity. Addition and
subtraction require that their mathematical results fit in `usize`.

Native orderings remain Relaxed for addition, Release for subtraction, and
Acquire for load. No ordering is strengthened. The contracts provide no
synchronization, release-sequence, byte visibility, allocation recovery, or
BytesMut protocol claim. There is no standard-library or compiler change.

Commands, from the bytes/1.11.1 crate directory:

```
./scripts/verify-bytes.sh sequential-native-counter
./scripts/verify-bytes.sh sequential-native-counter --features negative_wrong_identity
./scripts/verify-bytes.sh sequential-native-counter --features negative_underflow
cargo test --locked --offline --manifest-path verification/probes/sequential-native-counter/Cargo.toml --test native
```

The negative configurations deliberately fail: a same-valued counter's token
cannot authorize another counter, and subtracting one from zero violates the
nonunderflow precondition. The native test exercises the positive paths,
including `usize::MAX` boundaries. Proof evidence with source/configuration
hashes is in `../../artifacts/evidence/sequential-native-counter/`.

Removal condition for the local atomic trust: verified native-core contracts
must expose exclusive cell ownership and these exact scalar operations. A
concurrent protocol additionally needs a separate synchronization model; the
sequential contract does not establish it.
