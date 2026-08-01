# Tokio mpsc mutation audit

Date: 2026-08-01

Mutations were applied independently to a disposable copy.

| Mutation | Detecting test | Result |
|---|---|---|
| remove the second queue pop after AtomicWaker registration | bounded `loom_mpsc::closing_tx` | rejected: loom reports a lost-wakeup deadlock |
| omit the semaphore permit returned by successful `try_recv` | `sync_mpsc::try_recv_bounded` | rejected: a later `try_send` incorrectly reports Full |
| allow weak upgrade after the final strong Sender is gone | `sync_mpsc_weak::downgrade_upgrade_sender_failure` | rejected: the channel is resurrected |

The mutations cover the receive recheck, bounded capacity conservation, and
strong/weak no-resurrection invariants proved by the scoped mpsc models.
