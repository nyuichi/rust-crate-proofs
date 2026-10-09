# AT duplicate-value frontend control audit

The immutable frontend archive `at-frontend-duplicate-value-v1.tar.gz` has SHA-256 `0832496804638e3ae8b8adca8e10cd1adbe72687cd8014c0a98733a8a57edef8`. All 1056 members match the captured manifest hashes. The archive deliberately contains no COMA or proof-result files.

The archived generated `active.rs` differs from `positive.rs` by one inserted line: a second `bytes_cursor_terminal_drop(value, …)` call after the first call consumed `value`. The captured compiler output reports **E0382**, “use of moved value: `value`,” at that second use. This is a Rust ownership/frontend diagnostic, not a proof result or a native execution counterexample.
