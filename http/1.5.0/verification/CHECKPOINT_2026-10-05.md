# HTTP 1.5.0 verification checkpoint — 2026-10-05

This is a source and evidence checkpoint for ongoing API verification. It is
not a claim that the complete HTTP crate or every public API has been proved.
The crate's normal runtime implementation is still the published HTTP 1.5.0
implementation, with the source changes listed in this checkpoint; proof-only
profiles select real source bodies and do not replace production types with
stand-ins.

## How to read the results

- **Own-body goal** means a VC generated for the selected function's body or
  refinement. Supporting call/specification goals are reported separately.
- **Fresh** means the proof JSON and Coma are retained with a manifest that
  binds them to the source snapshot and named profile. **Historical** means the
  result applies to the recorded earlier snapshot and is not silently carried
  to the current source.
- An external library contract is a premise, not a proof of that library's
  implementation. An open VC, frontend rejection, interrupted run, or
  zero-obligation target is not a body proof.
- A named leaf harness is not an integrated proof of all callers in the normal
  crate profile. Current results must be read with the area README and manifest.

## Accepted source-body results

| Area | Accepted result | Boundary or open work |
|---|---|---|
| `Method` | The final Method snapshot records 103 targets and 877 successful leaves (737 own, 140 supporting), including all nine exact fixed-method constructors. The HTTP Request shortcut batch separately proves 9 own bodies, one per shortcut, with 108 supporting goals. | The Method harness is a leaf source proof. Request/Response default constructors and the full builder chain still depend on opaque published component contracts. |
| `StatusCode` / `Version` | The scalar snapshot records 80 supported targets and 381 successful leaves (133 own, 248 supporting). | The 62 numeric associated constants remain excluded after a recorded constant-translation failure. |
| `HeaderValue` | The durable v7 snapshot records 54 targets and 156 own leaves: 42 comparison targets (40 `PartialEq`/`PartialOrd`, 2 `Ord::cmp`) / 116 leaves, 9 getter/sensitivity/`AsRef`/`Clone` targets / 16 leaves, 2 byte-validation targets / 19 leaves, and `from_static` / 5 leaves. | This is a selected comparison/accessor/validation slice. Other constructors, conversions, formatting, and the full value API are not included in that count. The `HeaderValue` source fingerprint matches the current source; read the manifest for supporting premises. |
| `ByteStr` / bytes consumers | The ByteStr and bytes manifests record the actual included HTTP `ByteStr` bodies and 36 actual bytes consumers (79 consumer VCs), with exact byte-sequence contracts. | The pinned `bytes` implementation and the documented Creusot UTF-8 boundary remain premises/limitations; these are not proofs of the dependency crate. `ByteStr`'s unsafe-constructor panic text differs from the published implementation. |
| HeaderMap capacity/index helpers | The map-capacity manifest records 15 source targets and 30 successful leaves, including `desired_pos` and `probe_distance`. | The private callers' power-of-two, mask, and range invariants are not proved at all HeaderMap call sites. This does not prove insertion, lookup, growth, removal, or the map as a whole. |
| HeaderMap actual-source leaves | The named `http_map_api_leaf` snapshot records `new`, `default`, `len`, `keys_len`, `is_empty`, `clear`, and `capacity`, plus the allocation-bound/Vec probes: 47/49 total leaves pass. In `len`, 6 own leaves pass and 2 remain open; its supporting calls pass. `IterMut::next_unsafe` also has a fresh conditional result: 79 own leaves and 33 supporting leaves pass (112 total). | The two `len` obligations are generic nonzero-layout facts for `Bucket<T>` and `ExtraValue<T>`. `next_unsafe` assumes its storage/cursor-ready preconditions; map graph validity, cross-entry non-revisiting, and the public Iterator sequence refinement are not established. Drain and raw removal remain open. The current source also contains a capacity-contract pilot that has not yet been translated or proved. |
| Request/Response composition | The current request-shortcut archive proves the 9 fixed Request methods under explicit generic conversion callback premises. `Builder::and_then` has separate conditional callback-relay evidence. The Error formatter archive proves both outer formatter bodies and both refinements (4 own leaves). | Formatter contracts establish append-only formatter-model behavior, not exact output bytes. `Error::is`, `get_ref`, and `source` are excluded from that named profile because trait-object translation is unsupported. Dynamic Extensions and full default field state remain outside these results. |
| URI | The URI path API checkpoint and authority UTF-8 manifest record selected actual-source URI groups and exact artifacts for their recorded profiles. | The URI owner's `PathAndQuery` comparison group has 28 targets and 125 terminal leaves, with all own goals closed; `Display::fmt` remains open at 1/2 own goals. Other constructors and API closures are profile-scoped; see the URI checkpoint manifest. |

## Historical or partial results

- `HeaderName` source at checkpoint SHA
  `fbb82430831eee11bbb5d6487edfe6975f89d211e0757566efd4843b9baaa63f` passes
  its current named frontend check. The archived translation/proof snapshots
  predate the current source; callback-contract changes affect the
  `from_bytes` and `from_static` proof obligations. Do not describe the full
  current HeaderName proof chain as fresh until those caller artifacts are
  regenerated and checked.
- Archived HeaderName results include useful constructor, equality, standard
  spelling, and refinement work, but the current callback contracts changed
  the `from_bytes` and `from_static` tasks. The latest native check does not
  upgrade those artifacts into current proof results. The previous evidence
  reported 8 hash-related targets/51 leaves and several constructor/equality
  batches; they remain historical until the current callers are replayed.
- The Builder known-field/default relay has 11 accepted own goals in its
  archived run, but its manifest marks that run historical after dependent
  source changes. It gives conditional evidence for GET, `/`, HTTP/1.1, and
  response status 200; it does not establish exact HeaderMap or Extensions
  state.
- The composition field-accessor, mutator, parts, and `map` evidence is a
  named-profile leaf result. The current Request shortcut result is separate;
  older broad builder manifests are not proof of an integrated builder.
- `Builder::and_then` has accepted own-body results under its conditional
  `FnOnce` callback contracts. Those results do not establish arbitrary
  `Extensions` contents or prove every setter call.
- `Extensions` retains the real `dyn AnyClone + Send + Sync` representation.
  Current frontend diagnostics reject its dynamic trait-object model before
  useful body VCs are generated. No substitute field model is accepted here.

## Pending work and current non-claims

- The HeaderMap capacity API pilot in `src/header/map.rs` and the matching
  `usize::checked_next_power_of_two` contract in `creusot-libs` are source
  changes only at this checkpoint. They have no accepted frontend or proof
  result yet; the standard-library declaration is a TCB contract if retained.
- `HeaderMap::len` still has the two generic layout obligations above. The
  existing permission helper proves a `Vec` allocation bound only when called
  from a real borrowed Vec; it cannot be called from pure logic. A live
  positive generic element does not currently imply a nonzero generic ADT
  layout in the model.
- The conditional `IterMut::next_unsafe` body proof preserves the storage
  invariant for one call. The public Iterator's repeated-call graph invariant
  and Drain ownership transition remain open. No full raw-pointer or map
  safety claim is made.
- The URI `Display::fmt` body has one unresolved own leaf. HeaderName's current
  callback callers need fresh proof artifacts. Status constants remain
  excluded. These are not hidden by aggregate counts.
- `verify-all.bash` is a crate-level pipeline check and may stop at the first
  frontend/proof failure. Its outcome is recorded separately from the accepted
  leaf results above; it is not a substitute for the exact target manifests.
- The current default integrated source check is still open: the first
  frontend attempt reports unresolved `ErrorModelRef`/`error_model_ref`
  imports and deny-level unused parentheses/Pearlite imports. These are source
  integration errors, not an accepted external limitation; all-features native
  regressions passed in the separate runtime snapshot.

## Native regression snapshot

The post-rebase native checkpoint records 416 passing unit/integration tests
across the default and all-features profiles, plus the default source check and
the named HeaderName/hash-trace boundary checks. See
[`runtime-check/evidence/http-native-checkpoint-results-2026-10-05.json`](runtime-check/evidence/http-native-checkpoint-results-2026-10-05.json)
and its all-features summary. The test evidence records the source snapshot; it
does not turn leaf proofs into an integrated functional proof.

## Evidence index

The stage selection for this checkpoint is enumerated in
[`CHECKPOINT_STAGE_LIST_2026-10-05.md`](CHECKPOINT_STAGE_LIST_2026-10-05.md).
Area-specific manifests remain authoritative for source hashes, command lines,
target names, own-versus-support counts, and artifact hashes. Generated
`verif/` trees, proof-server state, interrupted outputs, ICE dumps, and
`_creusot_erasure` are not evidence unless a curated archive is explicitly
listed there.
