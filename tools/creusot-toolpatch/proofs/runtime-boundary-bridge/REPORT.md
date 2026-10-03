# Runtime formatting proof report

The current x86_64 source candidate proves the production `Buffer::format`
path through the same concrete sealed writers used by ordinary builds. The
full integrated run exited successfully in both configured feature modes:

| Configuration | Proved libraries | VCs |
| --- | ---: | ---: |
| Default | 254 | 2,083 |
| All features | 269 | 2,145 |

The exact combined output is in `integrated/fullsuite.log.gz`. The command was run from
`itoa/1.0.18` through the required wrapper:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh ./verify-all.bash
```

That crate-scoped script runs default and `--all-features` Creusot proofs. In
the default run, the actual runtime `Buffer::format` body passed 3 VCs, the
initialized suffix to string conversion passed 4, and the recursive ASCII
mapping passed 30. In the all-features run those targets passed 5, 6, and 30
VCs respectively. Each of the 12 concrete sealed writer bodies and its
separate refinement goal also passed in both configurations. The focused
writer proof recorded 120 VCs: 20 for `u8::write`, 8 for each other concrete
writer, and one for each refinement goal. Focused current-source sessions for
`Buffer::format` and the slice conversion are saved in
`integrated/focused-runtime/`; the final ASCII proof is saved in
`ascii/focused-final/`.

Boundary A uses a checked conversion from the known prefix of the physical
`[MaybeUninit<u8>; 40]` buffer to each associated fixed array. Every concrete
writer body proves the prefix bounds, successful `.try_into().unwrap()`,
canonical output bytes, and maximum output length. Creusot could not normalize
the associated constant while the concrete method redundantly restated
`where Self: Integer`; removing that redundant clause lets the concrete
`MAX_STR_LEN` values reduce to their declared array lengths. The ordinary
formatting algorithms remain the production algorithms.

Boundary B is the only locally trusted runtime representation conversion:
`assume_init_slice` requires every slot initialized, preserves the length, and
maps each output byte to the corresponding initialized payload. Its caller
proves the selected suffix bounds, initialization, ASCII range, UTF-8 model,
and byte relation. The bodies of the sealed writers, `slice_buffer_to_str`,
and runtime `Buffer::format` are all included in the integrated proof.

The three private digit, pair, and quad write helpers no longer carry the
standalone `no_panic` annotation: their contracts require valid indices and
ranges, so those helpers are only panic-free under their proved call-site
preconditions. Their Rust bodies and contracts remain unchanged, and the
public writers and `Buffer::format` retain their `no_panic` annotations.

The proof target is the current x86_64 build. The 16- and 32-bit
`usize`/`isize` implementations are outside this run. Native default and
release/all-features checks, including all 12 integration tests and two
doctests each, are recorded in `native/README.md`.

The proof input source blob hashes are:

| Source | Git blob SHA-1 |
| --- | --- |
| `itoa/1.0.18/src/runtime.rs` | `42061b9ea1b147cc71c83e75303f257a53705dbc` |
| `itoa/1.0.18/src/ascii.rs` | `f956ee77a0de6e50211fcc3466e840143788a09b` |
| `itoa/1.0.18/src/verification.rs` | `5ec989eec6ae9fe4664ed813a9302d437dce763c` |
| `itoa/1.0.18/src/lib.rs` | `03afac9bb2c690c0d3de4fed3b8392b078a60af3` |
| `itoa/1.0.18/tests/test.rs` | `fa6415cbeb25e19a6893e793c2149d024c2265f1` |
| `creusot-libs/creusot-std/src/std/char.rs` | `3b0fa7b66c471dfa97a2c5de61982b856e451397` |
