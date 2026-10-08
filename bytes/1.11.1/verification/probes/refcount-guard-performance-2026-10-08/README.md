# Guarded increment: native compatibility and cost

The production algorithm now uses Relaxed/Relaxed `fetch_update` with the pure
`next_ref_count` guard. All three increment paths (`owned_clone`,
`shallow_clone_arc`, `BytesMut::increment_shared`) call the same helper.
The abort happens on refusal without an increment by that operation; retries
execute only the pure guard. Release decrements and final Acquire remain.
This changes the native algorithm, not the public API or native representation.

`MAX_REF_COUNT = usize::MAX / 2`. A successful increment may reach
`MAX_REF_COUNT + 1`, so the Owned decrement debug assertion accepts that value.
The prior algorithm already allowed this value on a successful return.
Initial counts 1 and 2 satisfy the same bound. No increment `fetch_add` remains
in either production bytes module. This audit is not an ownership proof.

## Native checks

Commands run from bytes/1.11.1 with the pinned activation script:

```sh
cargo test --locked
cargo build --locked --no-default-features
cargo test --locked --features extra-platforms --lib
RUSTFLAGS='--cfg loom' cargo test --locked --lib
```

Default: 1,013 non-doc tests and 246 doc tests pass. Portable atomic: four
library tests pass. Loom: four library tests pass, including both original
concurrent cloning models. No-default library build succeeds. Two new tests
exercise the actual last-allowed increment and refusal at the machine maximum;
the refused operation leaves its value unchanged. These are bounded runtime
observations, not formal concurrent adequacy or complete crate verification.

## Microbenchmark

`bench.rs` includes the exact production guard/helper via relative paths.
Build with `rustc -O .../bench.rs -o /tmp/bytes-guard-bench`, then run the binary.
Each thread alternates an increment with a Release decrement of one shared
counter; seven trials alternate baseline/guarded order. Baseline executes the
previous post-`fetch_add` guard. The final counter is checked after each trial.

Observed medians (ns per increment/decrement pair):

| Threads | Previous algorithm | Guarded algorithm | Ratio |
|---|---:|---:|---:|
| 1 | 5.824 | 6.538 | 1.123 |
| 4 | 14.598 | 22.354 | 1.531 |

This measures only atomics on this host, not public Bytes throughput. Four-thread
results vary considerably; do not infer platform-independent regressions from
these numbers. Guarded CAS costs additional work and may retry/starve under
contention. No termination/fairness claim is made. Compiler/platform and every
trial are retained. Performance optimization must preserve the pre-store guard.

## Formal status

Pure guard arithmetic is included from production by
`original-public-shared-gate-2026-10-08`. Its successful proof receipt must be
reviewed separately. Generic atomic/event and release-sequence semantics remain
explicit TCB; bytes registration, retirement and lastness must be body proved.
Earlier C44 evidence corresponds to the previous increment algorithm and is
historical until a new source bridge proves the guarded implementation.

The canonical formal capture is `positive-final-4.tar.gz`: four proof files,
29 actual prover leaves, zero nulls. Root independently checked all 23 archive
members and current proof inputs. This proves the guard and model CAS bodies;
it does not prove the native operation/invariant TCB or public Bytes clone.

After the native measurements, the same fetch_update operation was extracted
into `try_increment(&AtomicUsize) -> Result<usize, usize>` so a later typed field
bridge can call the actual production operation. `increment` only aborts Err.
The two boundary tests were repeated on this final helper (`native-helper-refactor.log`).
`native-guard-v1` retains the pre-extraction exact source; `native-guard-v2`
retains the extracted helper and original measurement plus repeated-test logs.
