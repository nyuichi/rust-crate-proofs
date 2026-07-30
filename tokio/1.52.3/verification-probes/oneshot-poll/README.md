# Tokio oneshot `Future::poll` Verus connection probe

This independent crate checks one narrow question: can the Tokio 1.52.3
`oneshot::Receiver<T>` `Future` implementation's production signature be
presented to the pinned Verus toolchain?

The corresponding production signature is:

```rust
fn poll(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
) -> Poll<Result<T, RecvError>>
```

Verus 0.2026.07.27.31579f0 does not currently provide usable specifications
for the hand-written polling surface used here. `Pin`, `Poll`, `Context`, and
`Waker` are therefore declared as opaque external type specifications, and the
`poll` body is an explicit `external_body` boundary.

Run the probe from this directory with:

```console
cargo verus verify --locked
```

An output such as `verification results:: 0 verified, 0 errors` means only
that this external boundary and the production-shaped trait signature connect
successfully. **Zero verified functions is not a proof of `Receiver::poll`,
`Inner::poll_recv`, wake-up correctness, ownership transfer, memory safety,
linearizability, liveness, or any Tokio production body.**

Removing the boundary will require specifications for at least:

- `Pin` projection and mutation, including the relevant `Unpin` reasoning;
- construction and inspection of `Poll::Pending` and `Poll::Ready`;
- `Context::waker` and Waker clone, identity, wake, and drop behavior;
- Tokio's cooperative scheduling calls;
- the relationship between Tokio's loom wrappers and Verus atomic invariants
  and cell permissions.
