# AX outer reuse closure audit

**Result: pass for the captured bounded proof-output reuse closure. Full-original admission is not claimed.**

## Pinned closure

The outer manifest SHA-256 matches its external pin `05cca18b78627f28340e01f86abc52c7e75669734f1143389b832d7a1db5e8a5` for root ID `bytes-original-raw-suffix-drop-ax-admitted-reuse`. The capture report SHA-256 is `97752c2d8cd809367fe80c1bec092d3d74a7fc4f206e2e718e32bcc512e7684c`.

I validated the captured manifest using `verification/tools/evidence_closure.py` SHA-256 `ef7e19053b8c0db0495b83328a5901ec45578fac92f8eea7f576155bc8356e1c`, the external root pin, repository root `/workspace/bytes-work`, the closure's `objects` directory, and a fresh scratch cache. Validation passed: 12,938 logical files, 464 mounts, 431 objects, and 973,987,691 logical expanded bytes. The object graph contains 417 published-part references, 6 archive-member references, and 8 new CAS objects. All eight new CAS filenames, sizes, and content hashes verify; they total 683,486 bytes.

The closure imports the exact pinned origin manifest `1f9a614d5c49802dc8d5145f05dbb09f0ef2c72f483f66cae19ba6c037ec9db7`, with all 455 selected origin mounts preserved exactly once. I compared source and output mount prefixes, formats, object identities, and provenance roles for all 455 entries. The archived import selection contains all 455 rows and names the same origin manifest hash. The raw capture recipe is preserved at `lineage/capture-recipe.json` with SHA-256 `22a10b0466065cbcd4c39fe5448f7b5ed519c9b089afae286902c6e841f53754`.

## Reused proof and inputs

The frozen profile binds 354 proof output rows. Each row’s path, SHA-256, and size matches the origin manifest, the outer imported path, and the current probe output. The canonical row digest is `0b69930787d098b8e0bd4d40533a357cdb710b6730578976318a16598d0db699`.

The full input binding file has 791 rows and digest `baad43c9452a7e1539a0fc1723e6354560f28288a2bdc3c7b93c5cba58d00cbe`: 364 ancestor references and 427 local inputs. Every row’s path, SHA-256, and size resolves identically in the origin and outer closures. The six new local metadata sources are the reuse profile, builder, origin capture report, promotion receipt, and independent origin audit JSON/Markdown; their mounted bytes match the recipe and capture.

The profile binds a passing correspondence receipt (SHA-256 `3d8dddd75adb0e0f28e7e8570bbf523e487b85f0a225f62df78aeb46075071f5`), a passing four-artifact Cargo capture summary (`5ac1619e5d769591712c70acf8b6fba40c8fcd98ddb5304750da17175b7a8664`), a passing native audit (31 selected MIR bodies, 30 production), and the passing raw-free control (`276f6ff6ae05025f7e464a73c92b3011f20c849c83720cb7de1624c8dae5c761`). These are captured receipts; I did not rerun Cargo, the correspondence checker, or a prover.

## Scope status

The reused proof origin itself records 177 files, 1,798 prover leaves, 0 null, and 0 structural results, but marks the run diagnostic with policy correspondence exit 2. The later correspondence receipt passes for the bounded nonempty owned Box to raw suffix advance/read/ordinary physical Drop scope and explicitly says `full_original_admitted: false`. It makes no whole-crate or unwind claim.

The frozen profile’s `status: admitted_reuse` denotes reuse of those exact proof outputs with the separately recorded gates. It does not state full-original admission. The same profile records `full_original_admitted: false` and `outer_closure_state: ready_for_final_root_capture_and_review`; this audit preserves those recorded fields and verifies the captured outer closure separately.

The machine-readable audit is [AX_ADMISSION_AUDIT.json](AX_ADMISSION_AUDIT.json). The validator summary is at `/workspace/work/ax-admission-validation-audit.json`, SHA-256 `219a647f0cddf9e82852adb6d8716c27bc6d79e5ccd8ad31ea74413712f07833`.
