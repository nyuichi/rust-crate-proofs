# HTTP 1.5.0 verification checkpoint — 2026-10-05

This is a source and evidence checkpoint for ongoing API verification. It is
not a claim that the complete HTTP crate or every public API has been proved.
The crate's normal runtime implementation is still the published HTTP 1.5.0
implementation, with the source changes listed in this checkpoint; proof-only
profiles select real source bodies and do not replace production types with
stand-ins.
Every accepted result below is bound to its recorded source snapshot. Current
uncommitted Map, Name, Value, and URI work is not automatically covered by an
older source fingerprint. The archived whole-production frontend and native
runtime checkpoints predate some current WIP and are not checks of the current
full worktree.

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
| `Method` | The current-source reconciliation matches all 103 full, untransformed Why3 stdout streams byte-for-byte and reuses all complete trees: 877 leaves (737 own, 140 support). Independent audits cover 28 root tactic nodes / 682 child tasks and 7 nested nodes / 15 selected child tasks. It includes all nine exact fixed-method constructors. | This is isolated Method/ASCII leaf evidence. Request/Response shortcut proofs are a separate overlapping batch; neither is an integrated HTTP run. Hasher callback semantics remain an explicit std boundary. |
| `StatusCode` / `Version` | The current scalar-source reconciliation matches and reuses all 80 complete tasks/trees: 381 leaves (133 own, 248 support). Status scope includes conversions, predicates, exact reason/text, Default, equality/order/hash/formatting; selected Version and private Http bodies are covered too. | The 62 numeric StatusCode constants remain excluded after a recorded translation failure at the `NonZeroU16` niche. Counts overlap component targets and are not additive to other batches. |
| `HeaderValue` v7 archive | The archived v7 snapshot records 67 targets and 184 own leaves, including `HeaderValue::to_str` / 5 leaves. | Historical for `value.rs` SHA `1ab329864e57e80bd494fcc610ee0da3c7ac77c95c13fcf5a23647166a452e0c`; current-source evidence is listed separately. |
| `HeaderValue` numeric/TryFrom group | The latest accepted numeric snapshot at `value.rs` SHA `5420fc7682706cb56adc2516a94cd29182ffad5997186cf59593742549c9f8f6` proves all 18 scoped targets / 55 own leaves; all 106 profile COMAs are archived. Earlier `value_numeric_hex_contract_v1` and `value_numeric_conversions_v1` records are overlapping subsets, not additional totals. | The exact bytes premise remains explicit. Later formatter WIP at SHA `197fe107…` passes 38 focused native tests, but its manifest records no fresh Creusot emission/proof; formatter closure and other HeaderValue paths remain open. |
| Earlier `HeaderValue::hex_digit` snapshot | A separate prior-source proof closed 4 own leaves on `value.rs` SHA `1009a163a8f3dc853d228d8fc25de07cc176c974aa278361b274c4e72c8702e9`; the runtime parity check compares old formatter output for all 256 byte values. | Historical after the `5420fc…` source snapshot; its body result is superseded by the fresh 4-leaf `hex_digit` target in the numeric/hex group. |
| `HeaderName` | Accepted scopes include the base snapshot (34 targets / 280 leaves), the source-650 exact task reuse plus manual Debug/rank increment, and the independent Astra audit of 28 targets / 336 own leaves. On frozen source SHA `8c463390…`, a fresh 17-target proof closes 58/58 own leaves; its COMAs were freshly emitted and independently root-arity checked. | Counts refer to distinct, overlapping source snapshots and are not additive. On that same freeze, `StandardHeader::from_bytes` is partial at 81/82 leaves; its helper split and caller are not proved. Later Name structure WIP at recorded SHA `966107ba…` has no accepted proof. The 195 emitted Comas are not a 195-target closure. Exact accepted proof records are under `headers/evidence/name_value_wip_current_2026-10-05/`. |
| `ByteStr` / bytes consumers | The ByteStr and bytes manifests record the actual included HTTP `ByteStr` bodies and 36 actual bytes consumers (79 consumer VCs), with exact byte-sequence contracts. | The pinned `bytes` implementation and the documented Creusot UTF-8 boundary remain premises/limitations; these are not proofs of the dependency crate. `ByteStr`'s unsafe-constructor panic text differs from the published implementation. |
| HeaderMap capacity/index helpers | The map-capacity manifest records 15 source targets and 30 successful leaves, including `desired_pos` and `probe_distance`. | The private callers' power-of-two, mask, and range invariants are not proved at all HeaderMap call sites. This does not prove insertion, lookup, growth, removal, or the map as a whole. |
| HeaderMap capacity constructors | The curated [two-target archive](header-map-api/evidence/run-2026-10-05-capacity-constructor-lemma/manifest.json) proves `max_size_is_power_of_two` (1 own leaf) and actual-source `HeaderMap::try_with_capacity` (13 own + 16 call/spec leaves). Its named-profile frontend and normal HTTP library `cargo check` both pass. | This proves the selected constructor under the named leaf profile; the `usize::checked_next_power_of_two` standard-library contract is a TCB premise. It does not prove the integrated HeaderMap invariant or all capacity/reservation APIs. |
| HeaderMap actual-source leaves | The named `http_map_api_leaf` snapshot records `new`, `default`, `len`, `keys_len`, `is_empty`, `clear`, and `capacity`, plus allocation-bound/Vec probes: 47/49 total leaves pass. The [Entry accessor archive](header-map-api/evidence/run-2026-10-05-entry-accessors/manifest.json) proves 6 bodies / 6 own leaves plus 4 Vec-index support leaves. The [try-insert helper archive](header-map-api/evidence/run-2026-10-05-try-insert-entry/manifest.json) proves the actual private `try_insert_entry` body: 1 own leaf and 3 Vec/error-constructor support leaves; the no-preprocess printer roots match all four proof JSON roots. The conditional `IterMut::next_unsafe` run has 79 own leaves and 33 supporting leaves (112 total); its 70 first-level tasks and nine selected two-child splits were checked against the archived tree. | Entry proofs require only `index < entries.len`; the insertion helper proves its bounded append/error branch and explicit field frame, not occupancy, lookup reachability, caller index updates, or the global map invariant. `len` still has two open generic layout leaves. `next_unsafe` assumes storage/cursor-ready preconditions; map graph validity, cross-entry non-revisiting, and public Iterator refinement are not established. Drain and raw removal remain open. |
| HeaderMap lookup helper | The fresh [`find_with_hash` archive](header-map-api/evidence/run-2026-10-05-find-with-hash/manifest.json) proves 1 own body leaf and 8 call/specification support leaves. A solver-free no-preprocess Why3 print independently matches all 9 proof JSON roots. | The actual helper requires `header_map_find_ready(*self)` and a nonempty key table. Its `Some` postcondition establishes probe/index bounds and matching input, bucket, and table hashes plus key equality under the existing model relation. `None` is unconstrained. Public lookup callers, premise establishment, absence completeness, and the whole Robin Hood invariant remain open. |
| HeaderMap lookup/getter resume | The accepted [13-target archive](header-map-api/evidence/run-2026-10-05-map-lookup-getters-resume/manifest.json) at map source freeze SHA `ca22bfb5…` has 10 HTTP body roots, 3 refinement roots, and 30 literal-`true` imported support roots; all 43 direct roots pass and an independent direct-root arity audit is archived. | These are conditional modular body/refinement results, not a full map proof. `find` and `find_with_hash` have meaningful `Some` posts but unconstrained `None`; getters have no functional result post. The `header_map_find_ready` predicate is not established by constructors/mutations; there is no absence completeness, key/hash coherence, or full map invariant proof. The unpublished functional-lookup WIP's first frozen attempt failed the frontend before COMA generation; a later frontend check passed but has no proof result. Follow-on WIP is not accepted evidence. |
| Request/Response composition | The fresh sealed archive records 49 targets with 107 terminal leaves: 65 own and 42 call/spec support leaves. All 49 proof JSON trees match the run-log arities. It includes 30 structural API bodies, three Builder version bodies, Parts Clone, generic Request/Response Clone bodies and refinements, and four Debug bodies/refinements. | The manifest is bound to source-map SHA `54d174cd6a830a50e101d376ab95209a2aad258d1b7f19d78d2e26a8e5bb1efd` and records source stability during proof. Parts Clone relies on imported field Clone contracts, especially HeaderMap/Extensions. Debug proves append preservation, not exact rendered bytes. Builder header/extension methods and `Error::is`, `get_ref`, and `source` remain outside the named profile because their respective dependencies are unresolved or trait-object translation is unsupported. |
| Request/Response Builder relays and fixed-method shortcuts | The [`current-builder-relays-and-shortcuts`](composition/evidence/run-2026-10-05-current-builder-relays-and-shortcuts/manifest.json) batch contains 35 targets / 280 terminal leaves: 116 own leaves from 35 HTTP body roots and 164 support leaves. All 35 target roots match proof JSON; nested nodes were audited. It covers selected Builder method/body/URI relays, fixed-method Request shortcuts, and actual `Method::builder_*` constructors. | The independent v2 re-emission audit at [`report.json`](composition/evidence/run-2026-10-05-current-builder-relays-and-shortcuts/current-source-reemit-after-error-core-v2/independent-astra-task-streams/report.json) (SHA `9d6f8721…`) records exact full stdout identity for 9 changed targets and raw COMA byte identity for the other 26 targets; it does not claim 35 full stdout matches. Runtime URI/Status/Version imports and abstract callback/specification leaves remain support premises; header/extension mutation setters remain open. This batch is not additive to the 49-target or 15-target runs. |
| Request/Response constructors and defaults | The later [`current-default-constructors-fixed-models`](composition/evidence/run-2026-10-05-current-default-constructors-fixed-models/manifest.json) snapshot proves 15 targets / 41 leaves (15 own, 26 support), with all trees matching the run-log arity and direct Why3 goal counts. It covers selected component defaults, `Parts::new`, `Builder::{new,default}`, `Request/Response::builder`, and generic request/response constructors/defaults. | Source-map SHA `9291932624e16e6a3a4c2a938c26c1e5b7c157fb47d60fd23c2b5f59675454ee`; the exact target/source snapshot is distinct from the 49-target run, so these counts are not additive. `Parts::new` support calls use imported `HeaderMap` and `Extensions` defaults; this does not prove those dependency bodies or every field's exact default state. |
| URI Builder API | The accepted [URI Builder batch](uri/evidence/uri-builder-api-2026-10-05/manifest.json) covers 15 targets / 46 own + 36 support leaves; all 15 old/fresh full printed task streams match byte-for-byte. The native Builder regression passes 8/8. | This is bounded leaf evidence for selected Builder, `Uri::from_parts`, and `TryFrom<Parts>` contracts. It is not additive to other URI batches or an integrated/parser proof. |
| URI Authority/PathAndQuery conversions | The fresh [conversion batch](uri/evidence/uri-conversion-proof-2026-10-05/manifest.json) proves both `From` bodies and their two refinement posts: 6 own leaves. The four archived COMAs match fresh clean emission and independent solver-free split printing yields exactly six owned tasks. | Only the named conversions and component-preservation posts are covered. `Uri::Display`, same-type/`str` equality, parser and other conversions remain separate local work. |
| `Debug for Uri::fmt` | The accepted [Debug proof](uri/evidence/uri-debug-proof-2026-10-05/manifest.json) closes the body and refinement with 7 own leaves; an independent solver-free arity audit prints all 7 tasks. | Debug uses an opaque imported `Display::fmt` support contract. The separate `Uri::Display::fmt` body remains unproved (48 body + 3 refinement tasks emitted). |
| `Hash for Uri` | The accepted [Hash proof](uri/evidence/uri-hash-proof-2026-10-05/manifest.json) closes the body and refinement with 25 own leaves (22 + 3), all independently arity-audited. | Proves only `Hasher` invariant preservation on the recorded pre-Eq-fix freeze, under modular getter, component Hash, and library Hash contracts. It does not specify digest contents, establish Eq/Hash coherence, or prove the separate `Hash<Authority>` / `Hash<Scheme>` bodies. |
| URI | The URI path API checkpoint records selected actual-source groups. `PathAndQuery` Display closes the body, refinement, and transitive formatter helper: 31 own leaves (25 + 3 + 3). Its comparison checkpoint has 28 targets and 125 terminal leaves, with first-level and nested task arities checked. The fresh [Parts and Authority checkpoint](uri/evidence/parts-authority-comparison-2026-10-05/manifest.json) records 45 targets / 194 own leaves, all passed and independently arity-audited: Parts/default/conversions 55, Authority fold/case helpers 42, Hash 22, PartialEq 41, and PartialOrd 34. | These are named URI leaf-harness results, not a full URI API proof. The earlier 119-leaf Authority and 48/55 Parts records refer to older source/model snapshots; the 45-target/194-leaf checkpoint is the current bounded result and subsumes overlapping targets. URI parsing and other API closures remain profile-scoped; no `Ord` implementation is claimed. |

## Whole-production frontend snapshot

The archived default-feature and all-features production `cargo creusot --check`
attempts at `composition/evidence/current-production-check-2026-10-05/attempt-12-default-final/`
and `attempt-13-all-features-final/` each used the same stable 159-path source
map (`cc2009b2192b237b893e0ecae63eed6395e89aba13ab28321e002cfbaefce433`).
Both stopped with 19 unsupported dyn/experimental diagnostics and zero other
errors. These commands emitted no COMA and ran no Why3 proof; they do not count
as integrated proof passes. The Map `find` contract/profile work recorded
after this snapshot is not included in its source map.

## Historical or partial results

- The HeaderName base snapshot records 34 targets / 280 own leaves, followed by
  distinct source-650 task reuse and a fresh 17-target / 58-leaf proof on
  frozen source SHA `8c463390…`. `StandardHeader::from_bytes` on that same
  freeze remains partial at 81/82 leaves; its helper split and caller are not
  proved. Later Name structure WIP at recorded SHA `966107ba…` has no accepted
  proof. Keep snapshots separate; none proves the complete Name surface, and
  no blanket 195-target closure is claimed.
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

- HeaderMap remains a broad local proof task. The accepted lookup/getter resume
  at source freeze `ca22bfb5…` proves its selected modular bodies and
  refinements, but every one depends on an unestablished `header_map_find_ready`
  premise; `None` is not an absence theorem, getters have no functional result
  post, and 30 imported true support roots do not prove their callees. The
  later functional lookup attempt is preserved as unpublished worktree WIP;
  its first frozen attempt failed the frontend before COMA generation, while
  a later frontend check passed without producing a proof result. Follow-on
  mutation/invariant WIP has no accepted proof yet. Continue with
  actual append/link/index and table-invariant transitions, then reserve,
  growth, and removal. The capacity constructor and `try_insert_entry` pilots
  remain local body evidence, not a global map invariant.
  `checked_next_power_of_two` remains a std TCB contract.
- `HeaderMap::len` still has the two generic layout obligations above. The
  existing permission helper proves a `Vec` allocation bound only when called
  from a real borrowed Vec; it cannot be called from pure logic. A live
  positive generic element does not currently imply a nonzero generic ADT
  layout in the model.
- The conditional `IterMut::next_unsafe` body proof preserves its storage
  invariant for one call, with all 79 own leaves independently arity-checked.
  The public Iterator's repeated-call graph invariant and Drain ownership
  transition remain open. No full raw-pointer or map-safety claim is made.
- Status constants remain excluded. The Name snapshots record only their
  enumerated successful own goals; historical reuse candidates with unknown
  run-exit provenance and targets without complete matching trees are not
  counted as accepted proofs.
- HeaderValue numeric/TryFrom scope is closed at its accepted 18-target /
  55-own-leaf source snapshot. Current formatter repairs in `value.rs` are
  local WIP: the prior Error formatter attempt closed four bodies (15 own
  leaves total) but each formatter refinement retained one open leaf, and the
  HeaderValue Debug body attempt did not close. Formatter completion is not
  waiting on a dependency or Creusot change.
- `verify-all.bash` is a crate-level pipeline check and may stop at the first
  frontend/proof failure. Its outcome is recorded separately from the accepted
  leaf results above; it is not a substitute for the exact target manifests.
- The earlier URI `Parts`/conversion run is preserved at
  `verification/uri/evidence/parts-uri-constructors-2026-10-05/`. Its manifest
  records an older 12-root attempt with 48/55 leaves passed and 7 unresolved.
  The later `parts-authority-comparison-2026-10-05` archive freshly closes
  those 55 selected Parts/default/conversion leaves and the listed Authority
  group. The earlier partial result remains historical and is not added to
  the newer 194-leaf total.
- URI has accepted Builder API evidence (15 targets / 46 own + 36 support
  leaves) and current-source `From<Authority>` / `From<PathAndQuery>` for `Uri`
  evidence (6 own leaves). Other URI component getters, conversions, formatting,
  equality/hash and parser contracts remain local WIP; they are not broadly
  blocked on the dependency premise or toolchain.
- Same-type `Uri` equality has no complete proof: its [published partial
  manifest](uri/evidence/uri-eq-proof-2026-10-05/manifest.json) records 16/19
  body leaves and 4/4 refinement leaves passed, with three body leaves open
  (20/23 successful leaves total). `Uri`-to-`str` equality remains open.
  `Hash for Uri` is separately proved for invariant preservation only; its
  modular `Hash<Authority>` and `Hash<Scheme>` callees remain unproved, and no
  digest value or Eq/Hash coherence result is established.
- The earlier 27-error integrated frontend attempt is historical and
  superseded by the two stable 19-diagnostic snapshots above. The newer
  snapshots are still frontend-only and remain blocked by dyn/experimental
  translation; no blanket lint suppression or replacement model is counted.

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
