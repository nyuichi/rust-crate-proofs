# Focused Creusot translation probes

These bins isolate two suspected translation boundaries in `http` 1.5.0:

- `dyn-any` reproduces the `AnyClone` trait object and `dyn Any` downcast view
  used by `Extensions`.
- `dyn-any-views` isolates shared/mutable `Any` erasure, downcasts, and
  `TypeId` equality.
- `raw-slice-deref` reproduces `RawLinks`' raw slice dereference used by
  `HeaderMap` iterators and drains.
- `type-id` isolates the external `TypeId::of` model requirement.
- `dyn-error` reproduces `Error::{get_ref, is}` and
  `std::error::Error::source` dynamic dispatch.

They are diagnostic inputs, not successful proofs. Run them independently from
this directory with the configured Creusot toolchain and the shared HTTP proof
lock, for example:

```sh
../../scripts/run-proof.sh cargo creusot prove --no-cache -- \
  --locked --offline --manifest-path Cargo.toml --bin dyn-any-views
```

The version check remains enabled: `cargo creusot version` and the pinned
`creusot-std` path dependency both report `0.11.0-dev`. The expected nonzero
translation results and exact diagnostics are preserved under `logs/`; these
commands fail before Why3 generates any proof obligations.
