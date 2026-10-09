# AR frontend control archive audit

All three captures are immutable frontend-only diagnostics. For each archive, the outer SHA, unique member path set, and every member digest match its receipt. The captures contain no `.coma`, `proof.json`, or `probe/verif/` output; no solver result is asserted.

| Control | Archive SHA-256 | Result | Members |
|---|---|---|---:|
| `duplicate_value` | `0f9dc96d909c07a20628d94ad7efd7508ed60bf853f0b05b37aeb7f51b03a741` | `E0382` | 882 |
| `early_advance` | `331d6b7ef8aba4c355eb3b18a758bba145a052efe58c7cf89c65d8e034303e1b` | `E0502` | 882 |
| `snapshot_extract` | `91a53e3df3ebdd9642b7d45bb9436f9d4079c5c60c8cccd7e2aaf35c8b8a3a22` | `E0277` | 882 |

## `duplicate_value`

Two calls pass the same owned Bytes value to bytes_cursor_terminal_drop; the first consumes it and the second use is E0382.
Frontend log SHA-256: `c318b660d90bfb5011dc6918a0657c612406e1d3277b664078f11144f75fd954`. Archived generated source SHA-256: `b868bc36fbaa8f93af88135b124dda7b7dd8da548ef96b4435cb75bf709b8a30`.

## `early_advance`

An immutable chunk borrow remains live across advance_api(&mut value, rest), then is used by borrowed.to_vec(); captured compiler error is E0502. No E0505 appears in this archived log.
Frontend log SHA-256: `9d5eb4b927df282a44f718bd58c5fbbde078efa8bcd1bfbe9b7c108bc0300e5d`. Archived generated source SHA-256: `8e23d6de55d024ff4d7be5f914cfc17b2bb7cdfc08e7bae8a2427010858daebd`.

## `snapshot_extract`

The capture calls Snapshot<Bytes>::into_ghost; the captured private Std signature requires T: Plain, while rustc reports promotion::Bytes does not satisfy that bound (E0277).
Frontend log SHA-256: `0df2835b2cab3bcd62a43955978754d7aa2c1592e505908e528e2e02df4a25f2`. Archived generated source SHA-256: `12cedf5c6b6cf39df35a9267741ff6fd7cd2aa4be2908a68740b68baffedc0a8`.

## Limits

These are Rust frontend/translation diagnostics, not body VCs or semantic counterexamples. The early-advance archive records E0502 only; it contains no E0505. The snapshot-extract classification uses the captured private Std `Snapshot::into_ghost` signature requiring `T: Plain`.
