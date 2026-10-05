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
| `HeaderValue` v7 archive | The archived v7 snapshot records 67 targets and 184 own leaves, including `HeaderValue::to_str` / 5 leaves. | This is historical for `value.rs` SHA `1ab329864e57e80bd494fcc610ee0da3c7ac77c95c13fcf5a23647166a452e0c`; current `value.rs` is a different source snapshot. Do not aggregate it with current-source evidence. |
| `HeaderValue::hex_digit` | The focused current-source proof closes 4 own leaves on `value.rs` SHA `1009a163a8f3dc853d228d8fc25de07cc176c974aa278361b274c4e72c8702e9`. A separate runtime parity check compares the old formatter output across all 256 byte values. | This is one helper and a runtime regression check, not a fresh proof of the v7 API group. |
| `HeaderName` | The sealed snapshot at `verification/headers/evidence/archived_snapshots/name_hdrname_invariant_v1/snapshot.json` records 34 accepted targets and 280 own leaves on source SHA `fbb82430831eee11bbb5d6487edfe6975f89d211e0757566efd4843b9baaa63f`. Its 34 proof JSON artifacts and all 195 recorded Comas passed the snapshot hash check. | `parse_hdr` remains a separate partial attempt at 35/37 own leaves; its two unresolved leaves are excluded. The failed `HdrName` Debug refinement is also excluded. This does not establish complete parser behavior. |
| `ByteStr` / bytes consumers | The ByteStr and bytes manifests record the actual included HTTP `ByteStr` bodies and 36 actual bytes consumers (79 consumer VCs), with exact byte-sequence contracts. | The pinned `bytes` implementation and the documented Creusot UTF-8 boundary remain premises/limitations; these are not proofs of the dependency crate. `ByteStr`'s unsafe-constructor panic text differs from the published implementation. |
| HeaderMap capacity/index helpers | The map-capacity manifest records 15 source targets and 30 successful leaves, including `desired_pos` and `probe_distance`. | The private callers' power-of-two, mask, and range invariants are not proved at all HeaderMap call sites. This does not prove insertion, lookup, growth, removal, or the map as a whole. |
| HeaderMap capacity constructors | The curated [two-target archive](header-map-api/evidence/run-2026-10-05-capacity-constructor-lemma/manifest.json) proves `max_size_is_power_of_two` (1 own leaf) and actual-source `HeaderMap::try_with_capacity` (13 own + 16 call/spec leaves). Its named-profile frontend and normal HTTP library `cargo check` both pass. | This proves the selected constructor under the named leaf profile; the `usize::checked_next_power_of_two` standard-library contract is a TCB premise. It does not prove the integrated HeaderMap invariant or all capacity/reservation APIs. |
| HeaderMap actual-source leaves | The named `http_map_api_leaf` snapshot records `new`, `default`, `len`, `keys_len`, `is_empty`, `clear`, and `capacity`, plus the allocation-bound/Vec probes: 47/49 total leaves pass. In `len`, 6 own leaves pass and 2 remain open; its supporting calls pass. The conditional `IterMut::next_unsafe` run has 79 own leaves and 33 supporting leaves (112 total); the 70 first-level tasks and nine selected two-child splits were checked against the archived tree. | `next_unsafe` assumes storage/cursor-ready preconditions; map graph validity, cross-entry non-revisiting, and public Iterator refinement are not established. Drain and raw removal remain open. |
| Request/Response composition | The fresh sealed archive records 49 targets with 107 terminal leaves: 65 own and 42 call/spec support leaves. All 49 proof JSON trees match the run-log arities. It includes 30 structural API bodies, three Builder version bodies, Parts Clone, generic Request/Response Clone bodies and refinements, and four Debug bodies/refinements. | The manifest is bound to source-map SHA `54d174cd6a830a50e101d376ab95209a2aad258d1b7f19d78d2e26a8e5bb1efd` and records source stability during proof. Parts Clone relies on imported field Clone contracts, especially HeaderMap/Extensions. Debug proves append preservation, not exact rendered bytes. Builder header/extension methods and `Error::is`, `get_ref`, and `source` remain outside the named profile because their respective dependencies are unresolved or trait-object translation is unsupported. |
| URI | The URI path API checkpoint records selected actual-source groups. `PathAndQuery` Display closes the body, refinement, and transitive formatter helper: 31 own leaves (25 + 3 + 3). Its comparison checkpoint has 28 targets and 125 terminal leaves, with first-level and nested task arities checked. The Authority comparison archive records 119 own leaves (44 helper, 41 PartialEq, 34 PartialOrd) and a native comparison regression check of 1/1. | These are named URI leaf-harness results, not a full URI API proof. The comparison run is bound to archived Authority source SHA `5b5698cb74cfc68c84f234481e6291c1112b1e80cbcf7fdd56d573b750b5ac5d`; the current file is SHA `fd3f129ab0683ba62f747a18f6e43106888bdb6224d4f7e4477e389ef913828f`. A post-emission runtime regression for the helper-based `Authority::hash` passes 1/1, but proof of that changed Hash body is pending. `Parts::default` in `uri/mod.rs` is also a post-emission source delta. Other URI constructors and API closures remain profile-scoped. |

## Historical or partial results

- The sealed `HeaderName` snapshot records 34 accepted current-source targets
  and 280 own leaves. Its separate `parse_hdr` attempt has two unresolved
  leaves and is excluded from that accepted count.
- Earlier HeaderName archives include useful constructor, equality, standard
  spelling, and refinement work. The 8 hash-related targets/51 leaves and
  other pre-invariant batches remain historical; only the exact targets listed
  in the sealed 34-target snapshot count as accepted current-source results.
- The Builder known-field/default relay has 11 accepted own goals in its
  archived run, but its manifest marks that run historical after dependent
  source changes. It gives conditional evidence for GET, `/`, HTTP/1.1, and
  response status 200; it does not establish exact HeaderMap or Extensions
  state.
- The composition evidence is a named-profile leaf result. Request shortcuts
  and Builder callbacks retain their separate conditional contracts; this is
  not a proof of every Builder setter or the integrated crate.
- `Builder::and_then` has accepted own-body results under its conditional
  `FnOnce` callback contracts. Those results do not establish arbitrary
  `Extensions` contents or prove every setter call.
- `Extensions` retains the real `dyn AnyClone + Send + Sync` representation.
  Current frontend diagnostics reject its dynamic trait-object model before
  useful body VCs are generated. No substitute field model is accepted here.

## Pending work and current non-claims

- The broader HeaderMap reserve/growth/lookup family remains open. The
  successful capacity-constructor pilot above does not prove those operations
  or the map's global table invariant. The standard-library
  `checked_next_power_of_two` declaration is a TCB contract, not a proof of the
  Rust standard-library body.
- `HeaderMap::len` still has the two generic layout obligations above. The
  existing permission helper proves a `Vec` allocation bound only when called
  from a real borrowed Vec; it cannot be called from pure logic. A live
  positive generic element does not currently imply a nonzero generic ADT
  layout in the model.
- The conditional `IterMut::next_unsafe` body proof preserves its storage
  invariant for one call, with all 79 own leaves independently arity-checked.
  The public Iterator's repeated-call graph invariant and Drain ownership
  transition remain open. No full raw-pointer or map-safety claim is made.
- The separate `HeaderName::parse_hdr` attempt remains at 35/37 own leaves;
  its two open leaves are outside the accepted Name target count. Status
  constants remain excluded. These are not hidden by aggregate counts.
- `verify-all.bash` is a crate-level pipeline check and may stop at the first
  frontend/proof failure. Its outcome is recorded separately from the accepted
  leaf results above; it is not a substitute for the exact target manifests.
- An earlier integrated check stopped before VC generation with 27 frontend
  diagnostics: 8 missing root exports, 17 unused-parentheses diagnostics, and
  2 unused Pearlite imports. Its logs and unchanged-source hash are retained
  under `verification/evidence/`. This checkpoint does not claim a fresh
  integrated production check; in-progress frontend results are recorded only
  after the corresponding source snapshot is frozen.

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
