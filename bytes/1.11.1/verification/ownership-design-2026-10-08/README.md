# Strong original ownership contract design — stages 1–2

Read [STRONG_SPEC_JA.md](STRONG_SPEC_JA.md) for the final contract target,
representation refinement, reuse map and architecture admission result.

This increment is design/evidence maintenance only. No production code changes,
new API body verification, trusted protocol or repeated frozen proof experiment.
Astra independently reviewed the requirements and reuse classification; Luna
reviewed the original representation/control-flow obligations.

Run the read-only source/evidence audit from bytes/1.11.1:

```sh
python3 verification/ownership-design-2026-10-08/audit.py
```

The active pinned tool source lookup in audit.py is environment-specific and must
be updated when rebuilding the environment. Archived source hashes and actual
required tool facts are checked before recording a result. This command does not
translate or prove Rust. The audited old positive61 remains a restricted
physical/content proof, not a full ownership/refcount invariant.

Admission: NOT ESTABLISHED for the full original shared API under current tool
constraints. Existing physical/content lemmas remain useful. Exact sidecar and
entire proof trees cannot be promised unchanged reuse. Original automatic Drop
is blocked by known destructor-effect semantics; explicit cleanup does not
silently close that original API obligation.
