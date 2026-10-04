# Native RawBuffer probe

This isolated native-only probe includes `src/ownership_proof/raw_buffer.rs`
by path. `RawBuffer` is a move-only pointer/capacity holder for a detached
global-allocator `Vec<u8>` allocation. Its unsafe recovery and deallocation
methods rely on caller-held native ownership; they introduce no Creusot
contracts or memory permissions and prove no Bytes/BytesMut integration.

From this directory, using the repository's pinned
`nightly-2026-06-22` toolchain:

```sh
source /workspace/bytes-proof-tools/activate.sh
cargo test --locked --offline
cargo fmt -- --check
```

The final test output is saved in `logs/native-test.log` (5 passed); the format
check succeeded with no stdout, saved as `logs/fmt-check.log`.

The tests check initialized-prefix round trips with pointer/capacity identity,
disjoint raw-region mutation, zero-capacity Vec handling, deallocation after
writes only to spare slots, and deallocation after a former initialized byte
is overwritten with `MaybeUninit::uninit()`. The latter allocation is never
recovered as a Vec.
