# Tokio broadcast mutation audit

Date: 2026-08-01

Mutations were applied independently to a disposable copy.

| Mutation | Detecting test | Result |
|---|---|---|
| advance the lagged Receiver one position past the oldest retained value | `sync_broadcast::lagging_rx` | rejected: the next value is `three` instead of `two` |
| suppress `RecvGuard`'s remaining-reader decrement and final value release | `sync_broadcast::unconsumed_messages_are_dropped` | rejected: the payload Arc remains retained after Receiver drop |

These failures exercise the two channel-specific invariants completed here:
exact lag recovery and remaining-reader ownership conservation.
