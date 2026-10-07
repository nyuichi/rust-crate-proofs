# Original `BytesMut` constructor path diagnostic

This bounded probe captures the original `BytesMut` representation and the
current `with_capacity -> from_vec` constructor path. It extracts the actual
`BytesMut` and `Shared` declarations, the exact `with_capacity`, `from_vec`,
`len`, and `capacity` methods, the original capacity encoder, pointer helpers,
and the source constants needed by those items. The generator is guarded by
the source snapshot SHA-256 and writes a fragment-level extraction receipt.

At creation, `src/bytes_mut.rs` was captured byte-for-byte at Git HEAD
`675c73673ebf87360cae63341a74e265f97112f3`, SHA-256
`cd4dd691c3ecfedfa7cbf8db095bb1d6eebefbcdc71c67f0304e467a2d45e29d`.
See `evidence/source-snapshot.json`. The generated receipt also records the
live production file hash when Cargo runs the generator. If the production
file changes meanwhile, this remains evidence for the saved initial snapshot;
it must not be described as a gate on the later source.

The probe includes no replacement representation, ghost ownership model,
protocol/snapshot module, or bytes-specific trusted contract. It omits the
original `Drop` implementation and all mutation/sharing methods. Native tests
manually recover the exact Vec allocation from the extracted fields before
freeing it; they do not establish automatic drop behavior. The test path checks
only the native constructor results: zero length and requested capacity for
`with_capacity`, and pointer/length/capacity/bytes preservation for `from_vec`.

Run `./run-diagnostic.sh` from this directory. It holds the shared lock, uses
the pinned tool activation and a task-specific target directory, runs only the
two local native constructor tests, cleans the probe target before Creusot
translation, then runs translation-only. It never launches Why3 proof. The
translation result is a diagnostic for the real extracted bodies in this
captured source context, not integrated crate verification.

The durable diagnostic inputs/results are under `evidence/`: source snapshot,
per-item extraction manifest, generated exact fragments, native/translation
logs, exit status, and before/after source hash receipt. The next frontend
diagnostic in `translation.log` identifies the blocker for stage 2/3 planning.

## Captured result

The two native tests pass. Translation-only stops in the exact original
`invalid_ptr` body at `debug_assert_eq!(ptr as usize, addr)` with:
`Unsupported pointer cast: move _11 as usize (PointerExposeProvenance)`.
No Coma files were emitted and Why3 was not launched. Creusot also reports that
the actual `ManuallyDrop::new` and `null_mut().wrapping_add(addr)` calls have no
contracts, so they would be additional impossible-precondition obligations if
translation proceeds past the pointer-cast frontend error.

The source hash matched the saved snapshot both when fragments were generated
and after translation. The probe resolved `creusot-std` to the existing
`/workspace/bytes-proof-tools/bytes-proof-std` path package; its 110-file tree
hash is recorded in `evidence/diagnostic-result.json`. This uses that package's
generic Vec contracts but adds no BytesMut ghost model, trusted contract, or
ownership theorem. The first harness attempt is separately retained as
`evidence/translation-attempt-01-*`; it failed before translation because the
probe initially omitted the Creusot prelude import, so it is not a source
counterexample.

Production `src/bytes_mut.rs` was edited after this diagnostic completed. Its
later working-tree hash (`1e91531b067d3af58897ff7db0e16f0f9214b66e843b7fd59cfdd0579b43cdd4`)
is recorded in `evidence/production-after-diagnostic.json`; the frontend result
above applies only to the initial captured hash, not to that later source.
