# AP frontend-control archive audit

Archive-only inspection of the four AP type-control captures. No build, compiler, or solver was run for this audit. Each tar has 753 regular-file members; all paths and SHA-256 hashes match its receipt. The receipt marks each `frontend_diagnostic_no_proof_claim` and excludes proof artifacts. The captured frontend logs contain the expected errors:

| Capture | Archive SHA-256 | Frontend error | Perturbation in archived source |
|---|---|---|---|
| `ap-type-duplicate-peer-v1` | `5e646c2af4e3f772e13a4da8a51c216bc37887dd9e3e39e2b66ea4d8e34b6028` | E0382 | Second terminal-drop call repeats a consuming call with `peer`. |
| `ap-type-duplicate-survivor-v1` | `a7be2222e0d02d3a4941fedb457c0f6f54f7de404111588394dd38d4263cc486` | E0382 | Second terminal-drop call repeats a consuming call with `survivor`. |
| `ap-type-early-survivor-v1` | `b29985c2d2a6628f40e5c76a2d93775ae060286e647647d89ae305fad63c43eb` | E0505, E0502 | Terminal-drop moves `survivor` and mutably borrows `detached` before the outstanding borrowed child view is converted with `to_vec()`. |
| `ap-type-snapshot-extract-v1` | `227f92c20ba51b887d2c1c58c678d7cf1abec35038c41e41f80e77996b8bab8c` | E0277 | Attempts `snapshot!(survivor).into_ghost()` to extract `Bytes` from a Snapshot; `Bytes` does not satisfy the `Plain` bound. |

## Proof and cache artifacts

None of the four archives contains any `.coma` or `proof.json` file. The probe has no `target` output, proof-config, or Why3 cache directory. Its `probe/why3find.json` and 14 similarly named files from captured dependency inputs are configuration files, not solver outputs. These controls are compiler rejections, not proof-body nulls.

## Snapshot extraction diagnostic

The `snapshot-extract` capture adds this source line:

```rust
let _extracted: Ghost<Bytes> = snapshot!(survivor).into_ghost();
```

The archived frontend log reports E0277: `promotion::Bytes: creusot_std::ghost::Plain` is not satisfied. The same archive includes the private Std source that defines the restriction:

- `inputs/private-std/src/snapshot.rs:111-115`: `Snapshot<T>::into_ghost(self) -> Ghost<T>` requires `T: Plain`.
- `inputs/private-std/src/ghost.rs:266-270`: `pub trait Plain: Copy`.
- `probe/generated/compiled-inputs/public_records.rs:82`: the generated `Bytes` record contains an `AtomicPtr` field and has no `Plain` implementation in this file; the compiler diagnostic also reports the missing implementation.

Thus extraction fails at the frontend’s `Plain` bound; this capture establishes no body proof result. Exact archive, standard-library, compiled record, and log hashes are in `AP_FRONTEND_CONTROL_AUDIT.json`.
