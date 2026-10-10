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
| `HeaderValue` numeric/TryFrom group | The accepted numeric snapshot at `value.rs` SHA `5420fc7682706cb56adc2516a94cd29182ffad5997186cf59593742549c9f8f6` proves 18 scoped targets / 55 own leaves; the full profile emitted and archived 106 COMAs. Earlier numeric archives overlap and are not additive. | This remains a distinct older source freeze from the later formatter batch; the exact bytes premise remains explicit. |
| `HeaderValue` formatter batch | The fresh [current emission manifest](headers/evidence/value_current_emission_2026-10-05/manifest.json) binds to `value.rs` SHA `197fe107…` and 333 COMAs. Thirteen focused targets close 129/129 own tasks: Error bodies/refinements 33/33, formatter helper targets 39/39, and HeaderValue Debug body/refinement 57/57; the independent task/context audit passes with no nulls. The exact-source native parity profile passes 38/38. | A separate current-source [is_visible_ascii proof](headers/evidence/value_is_visible_ascii_current_2026-10-05/manifest.json) closes its 3/3 body leaves on the exact accepted Value source input, with Astra context audit PASS. It remains outside the 129-task count. Formatter contracts establish append preservation, not exact rendered text; public callers and imported byte-view models remain separate. |
| Earlier `HeaderValue::hex_digit` snapshot | A separate prior-source proof closed 4 own leaves on `value.rs` SHA `1009a163a8f3dc853d228d8fc25de07cc176c974aa278361b274c4e72c8702e9`; the runtime parity check compares old formatter output for all 256 byte values. | Historical after the `5420fc…` source snapshot; its body result is superseded by the fresh 4-leaf `hex_digit` target in the numeric/hex group. |
| `HeaderName` | Accepted scopes include the base snapshot (34 targets / 280 leaves), the source-650 exact task reuse plus manual Debug/rank increment, and the independent Astra audit of 28 targets / 336 own leaves. On frozen source SHA `8c463390…`, a fresh 17-target proof closes 58/58 own leaves; its COMAs were freshly emitted and independently root-arity checked. A separate published `ed50c0e4…` comparator prototype closes 19 own tasks and passes 27 native tests. | Counts refer to distinct, overlapping source snapshots and are not additive. On the earlier `8c463390…` freeze, `StandardHeader::from_bytes` is partial at 81/82 leaves; its helper split and caller are not proved. The prototype [manifest](headers/evidence/name_standard_comparator_prototype_2026-10-05/manifest.json), SHA-256 `6673efe22e8bea8c558e120e18ecceb3a0955f8abd1b245074400defe5c270b8`, limits scope to two of 81 parser variants plus a selected consumer. Astra reprinted all 19 task contexts; raw frontend stdout was not retained. It establishes no variant completeness. The 195 emitted Comas are not a 195-target closure. Exact prior accepted proof records are under `headers/evidence/name_value_wip_current_2026-10-05/`. |
| `ByteStr` / bytes consumers | The ByteStr and bytes manifests record the actual included HTTP `ByteStr` bodies and 36 actual bytes consumers (79 consumer VCs), with exact byte-sequence contracts. | The pinned `bytes` implementation and the documented Creusot UTF-8 boundary remain premises/limitations; these are not proofs of the dependency crate. `ByteStr`'s unsafe-constructor panic text differs from the published implementation. |
| HeaderMap capacity/index helpers | The map-capacity manifest records 15 source targets and 30 successful leaves, including `desired_pos` and `probe_distance`. | The private callers' power-of-two, mask, and range invariants are not proved at all HeaderMap call sites. This does not prove insertion, lookup, growth, removal, or the map as a whole. |
| HeaderMap capacity constructors | The curated [two-target archive](header-map-api/evidence/run-2026-10-05-capacity-constructor-lemma/manifest.json) proves `max_size_is_power_of_two` (1 own leaf) and actual-source `HeaderMap::try_with_capacity` (13 own + 16 call/spec leaves). Its named-profile frontend and normal HTTP library `cargo check` both pass. | This proves the selected constructor under the named leaf profile; the `usize::checked_next_power_of_two` standard-library contract is a TCB premise. It does not prove the integrated HeaderMap invariant or all capacity/reservation APIs. |
| HeaderMap actual-source leaves | The named `http_map_api_leaf` snapshot records `new`, `default`, `len`, `keys_len`, `is_empty`, `clear`, and `capacity`, plus allocation-bound/Vec probes: 47/49 total leaves pass. The [Entry accessor archive](header-map-api/evidence/run-2026-10-05-entry-accessors/manifest.json) proves 6 bodies / 6 own leaves plus 4 Vec-index support leaves. The [try-insert helper archive](header-map-api/evidence/run-2026-10-05-try-insert-entry/manifest.json) proves the actual private `try_insert_entry` body: 1 own leaf and 3 Vec/error-constructor support leaves; the no-preprocess printer roots match all four proof JSON roots. The conditional `IterMut::next_unsafe` run has 79 own leaves and 33 supporting leaves (112 total); its 70 first-level tasks and nine selected two-child splits were checked against the archived tree. | Entry proofs require only `index < entries.len`; the insertion helper proves its bounded append/error branch and explicit field frame, not occupancy, lookup reachability, caller index updates, or the global map invariant. `len` still has two open generic layout leaves. `next_unsafe` assumes storage/cursor-ready preconditions; map graph validity, cross-entry non-revisiting, and public Iterator refinement are not established. Drain and raw removal remain open. |
| HeaderMap lookup helper | The fresh [`find_with_hash` archive](header-map-api/evidence/run-2026-10-05-find-with-hash/manifest.json) proves 1 own body leaf and 8 call/specification support leaves. A solver-free no-preprocess Why3 print independently matches all 9 proof JSON roots. | The actual helper requires `header_map_find_ready(*self)` and a nonempty key table. Its `Some` postcondition establishes probe/index bounds and matching input, bucket, and table hashes plus key equality under the existing model relation. `None` is unconstrained. Public lookup callers, premise establishment, absence completeness, and the whole Robin Hood invariant remain open. |
| `HeaderMap::append_value` | The accepted [append batch](header-map-api/evidence/run-2026-10-05-map-append-value/manifest.json) proves 1 actual body root + 4 imported literal-`true` support roots from the exact attempt-6 COMA; all five proof JSON roots match. | Astra's solver-free audit confirms that full raw Why3 stdout/stderr match without normalization. This proves the named body only; imported support bodies and other map mutations/global invariants remain outside scope. It overlaps the attempt-6 snapshot. |
| HeaderMap lookup/getter resume | The accepted [attempt-6 archive](header-map-api/evidence/run-2026-10-05-map-functional-lookup/attempt-6-name-eq-open/manifest.json) freezes Map SHA `9e2dbdc3…` with Name SHA `bc3be7d9…`: 15 targets, 37 direct roots (10 bodies, 5 refinements, 22 literal-`true` support stubs), and 40/40 proved terminal leaves. The independent direct-root arity audit is archived. | These are conditional modular body/refinement results, not a full map proof. `find` and `find_with_hash` have meaningful `Some` posts but unconstrained `None`; getters have no functional result post. The `header_map_find_ready` predicate is not established by constructors/mutations; there is no absence completeness, key/hash coherence, or full map invariant proof. The Map SHA `91dd661a…` captured by production attempts 22/23 adds a `get` result post outside this accepted freeze; `get_all` and `contains_key` remain in scope. |
| Private `IdHasher` emission | The stable [source-bound emission](id-hasher/evidence/current-source-emission-2026-10-05/manifest.json) freezes 146 inputs and emits five COMAs with five solver-free Why3 tasks. | Four normal-return body results are from separately accepted prior proofs; this capture did not rerun a solver. `Hasher::write(&[u8])` deliberately panics, leaving its non-returning/panic-free obligation open. |
| Request/Response composition | The fresh sealed archive records 49 targets with 107 terminal leaves: 65 own and 42 call/spec support leaves. All 49 proof JSON trees match the run-log arities. It includes 30 structural API bodies, three Builder version bodies, Parts Clone, generic Request/Response Clone bodies and refinements, and four Debug bodies/refinements. | The manifest is bound to source-map SHA `54d174cd6a830a50e101d376ab95209a2aad258d1b7f19d78d2e26a8e5bb1efd` and records source stability during proof. Parts Clone relies on imported field Clone contracts, especially HeaderMap/Extensions. Debug proves append preservation, not exact rendered bytes. Builder header/extension methods and `Error::is`, `get_ref`, and `source` remain outside the named profile because their respective dependencies are unresolved or trait-object translation is unsupported. |
| Request/Response Builder relays and fixed-method shortcuts | The [`current-builder-relays-and-shortcuts`](composition/evidence/run-2026-10-05-current-builder-relays-and-shortcuts/manifest.json) batch contains 35 targets / 280 terminal leaves: 116 own leaves from 35 HTTP body roots and 164 support leaves. All 35 target roots match proof JSON; nested nodes were audited. It covers selected Builder method/body/URI relays, fixed-method Request shortcuts, and actual `Method::builder_*` constructors. | The independent v2 re-emission audit at [`report.json`](composition/evidence/run-2026-10-05-current-builder-relays-and-shortcuts/current-source-reemit-after-error-core-v2/independent-astra-task-streams/report.json) (SHA `9d6f8721…`) records exact full stdout identity for 9 changed targets and raw COMA byte identity for the other 26 targets; it does not claim 35 full stdout matches. Runtime URI/Status/Version imports and abstract callback/specification leaves remain support premises; header/extension mutation setters remain open. This batch is not additive to the 49-target or 15-target runs. |
| Request/Response constructors and defaults | The later [`current-default-constructors-fixed-models`](composition/evidence/run-2026-10-05-current-default-constructors-fixed-models/manifest.json) snapshot proves 15 targets / 41 leaves (15 own, 26 support), with all trees matching the run-log arity and direct Why3 goal counts. It covers selected component defaults, `Parts::new`, `Builder::{new,default}`, `Request/Response::builder`, and generic request/response constructors/defaults. | Source-map SHA `9291932624e16e6a3a4c2a938c26c1e5b7c157fb47d60fd23c2b5f59675454ee`; the exact target/source snapshot is distinct from the 49-target run, so these counts are not additive. `Parts::new` support calls use imported `HeaderMap` and `Extensions` defaults; this does not prove those dependency bodies or every field's exact default state. |
| URI Builder API | The accepted [URI Builder batch](uri/evidence/uri-builder-api-2026-10-05/manifest.json) covers 15 targets / 46 own + 36 support leaves; all 15 old/fresh full printed task streams match byte-for-byte. The native Builder regression passes 8/8. | This is bounded leaf evidence for selected Builder, `Uri::from_parts`, and `TryFrom<Parts>` contracts. It is not additive to other URI batches or an integrated/parser proof. |
| URI Authority/PathAndQuery conversions | The fresh [conversion batch](uri/evidence/uri-conversion-proof-2026-10-05/manifest.json) proves both `From` bodies and their two refinement posts: 6 own leaves. The four archived COMAs match fresh clean emission and independent solver-free split printing yields exactly six owned tasks. | Only the named conversions and component-preservation posts are covered. Same-type `Uri` equality closes in the separate field-conjunction batch. `Uri::Display`, `Uri`-to-`str` equality, parser and other conversions remain separate local work. |
| `Debug for Uri::fmt` | The accepted [Debug proof](uri/evidence/uri-debug-proof-2026-10-05/manifest.json) closes the body and refinement with 7 own leaves; an independent solver-free arity audit prints all 7 tasks. | Debug uses an opaque imported `Display::fmt` support contract. The separate `Uri::Display::fmt` body remains unproved (48 body + 3 refinement tasks emitted). |
| `Hash for Uri` | The accepted [Hash proof](uri/evidence/uri-hash-proof-2026-10-05/manifest.json) closes the body and refinement with 25 own leaves (22 + 3), all independently arity-audited. | Proves only `Hasher` invariant preservation on the recorded pre-Eq-fix freeze, under modular getter, component Hash, and library Hash contracts. It does not specify digest contents, establish Eq/Hash coherence, or prove the separate `Hash<Authority>` / `Hash<Scheme>` bodies. |
| Same-type `Uri` equality | The [field-conjunction run](uri/evidence/uri-eq-field-conjunction-proof-2026-10-05/manifest.json) on `src/uri/mod.rs` SHA `7224a7fa…` closes all 30 owned leaves (26 body + 4 refinement), with zero nulls. Astra's read-only audit independently confirms all 30 full Why3 task contexts byte-identical and all 295 evidence artifact hashes matching. | The [audit report](uri/evidence/uri-eq-field-conjunction-proof-2026-10-05/analysis/astra-field-conjunction-audit.md) has SHA-256 `d42875d62642b78a2602811f3119ec398c25bceac31dc4651c652b1ec9d97c12`. Earlier published/transport/branching partial results (20/23, 26/27, 28/30) are historical and non-additive. Six exact carried-over obligations—From Authority (3 leaves), From PathAndQuery (3), and Uri Debug (7)—were separately checked by byte-identical full stdout streams. `Uri`-to-`str` equality remains separate. |
| URI | The URI path API checkpoint records selected actual-source groups. `PathAndQuery` Display closes the body, refinement, and transitive formatter helper: 31 own leaves (25 + 3 + 3). Its comparison checkpoint has 28 targets and 125 terminal leaves, with first-level and nested task arities checked. The fresh [Parts and Authority checkpoint](uri/evidence/parts-authority-comparison-2026-10-05/manifest.json) records 45 targets / 194 own leaves, all passed and independently arity-audited: Parts/default/conversions 55, Authority fold/case helpers 42, Hash 22, PartialEq 41, and PartialOrd 34. | These are named URI leaf-harness results, not a full URI API proof. The earlier 119-leaf Authority and 48/55 Parts records refer to older source/model snapshots; the 45-target/194-leaf checkpoint is the current bounded result and subsumes overlapping targets. URI parsing and other API closures remain profile-scoped; no `Ord` implementation is claimed. |

## Whole-production frontend snapshot

The latest stable default and all-features production frontend captures are
[`attempt-22`](composition/evidence/current-production-check-2026-10-05/attempt-22-default-name-ed50-freeze/manifest.json)
and
[`attempt-23`](composition/evidence/current-production-check-2026-10-05/attempt-23-all-features-name-ed50-freeze/manifest.json).
Both hash the exact same 159-path production HTTP plus `creusot-std` input set
before and after the check; source-map SHA is
`663962f0597f15c34b3d02ffa97bbc9180d6af938652dcb7f6e534a706d25632`. Each
check stops at 19 unsupported dyn/experimental diagnostics and has no other
errors. These are frontend-only `cargo creusot --check` runs; they emit no
COMA and run no Why3 solver, so they are not integrated proof passes.

Attempts 14–23 preserve the production frontend progression. These are
diagnostic captures only: no attempt emitted COMA or ran Why3.

| Attempt | Profile | Captured result |
|---|---|---|
| [14](composition/evidence/current-production-check-2026-10-05/attempt-14-default-fresh/manifest.json) | Default | Stable source map. Six unresolved `proof_assert!` macro errors in URI and one denied unused-import error in Map were local frontend failures. |
| [15](composition/evidence/current-production-check-2026-10-05/attempt-15-all-features-fresh/manifest.json) | All features | Unstable: URI source changed during the check. The observed Map unused-import and URI `Seq<u8>::eq_model` errors are not attributed to a stable snapshot. |
| [16](composition/evidence/current-production-check-2026-10-05/attempt-16-default-post-import-fix/manifest.json) | Default | Stable source map with three local HeaderMap errors: qualified `pearlite!` use and unused proof imports. |
| [17](composition/evidence/current-production-check-2026-10-05/attempt-17-default-map-settled/manifest.json) | Default | Stable source map; 19 dyn/experimental errors plus one local denied dead-code warning. |
| [18](composition/evidence/current-production-check-2026-10-05/attempt-18-all-features-map-settled/manifest.json) | All features | Stable source map; 19 dyn/experimental errors plus one local denied dead-code warning. |
| [19](composition/evidence/current-production-check-2026-10-05/attempt-19-default-post-uri-fix/manifest.json) | Default | The log has 19 dyn/experimental diagnostics, but the contemporaneous after-map was overwritten; source stability is unverifiable, so this is not an accepted stable run. |
| [20](composition/evidence/current-production-check-2026-10-05/attempt-20-all-features-post-uri-fix/manifest.json) | All features | Stable source map; 19 dyn/experimental errors plus one local Name dead-code error. |
| [21](composition/evidence/current-production-check-2026-10-05/attempt-21-default-stable-post-uri-fix/manifest.json) | Default | Unstable: Name changed during the check. The captured log has 19 dyn/experimental diagnostics and no local error, but is not a stable-source result. |
| [22](composition/evidence/current-production-check-2026-10-05/attempt-22-default-name-ed50-freeze/manifest.json) | Default | Stable 159-input source map; 19 dyn/experimental diagnostics, zero other errors. |
| [23](composition/evidence/current-production-check-2026-10-05/attempt-23-all-features-name-ed50-freeze/manifest.json) | All features | Stable 159-input source map identical to attempt 22; 19 dyn/experimental diagnostics, zero other errors. |

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

- HeaderMap remains a broad local proof task. The accepted attempt-6 lookup/getter batch at Map SHA `9e2dbdc3…` and Name SHA `bc3be7d9…` proves 40/40 terminal leaves across 15 targets, but selected contracts remain conditional on `header_map_find_ready`; lookup `None` is not an absence theorem and getters have no functional result post. The batch contains 22 literal-`true` support roots and does not prove their callees. The Map SHA `91dd661a…` captured by production attempts 22/23 has a `get` result post outside this proof freeze; `get_all` and `contains_key` remain covered by the accepted freeze. Constructor/mutation readiness, absence completeness, key/hash coherence, and the table invariant remain open. Continue with actual append/link/index and table-invariant transitions, then reserve, growth, and removal. The capacity constructor and `try_insert_entry` pilots remain local body evidence, not a global map invariant. `checked_next_power_of_two` remains a std TCB contract.
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
- HeaderValue numeric/TryFrom scope is closed at its accepted 18-target / 55-own-leaf snapshot. At the later source SHA `197fe107…`, a separate fresh formatter batch proves 13 targets / 129 own tasks with an independent task/context audit and no nulls. It covers Error formatter bodies/refinements, formatter helpers, and HeaderValue Debug body/refinement. A separate exact-source batch proves `is_visible_ascii` in 3/3 leaves; it is not included in the 129 count. Formatter contracts establish append preservation, not exact rendered text; public callers and imported byte-view models remain separate. Other HeaderValue APIs remain open.
- `verify-all.bash` is a crate-level pipeline check and may stop at the first
  frontend/proof failure. Its outcome is recorded separately from the accepted
  leaf results above; it is not a substitute for the exact target manifests.
- The earlier URI `Parts`/conversion run is preserved at
  `verification/uri/evidence/parts-uri-constructors-2026-10-05/`. Its manifest
  records an older 12-root attempt with 48/55 leaves passed and 7 unresolved.
  The later `parts-authority-comparison-2026-10-05` archive freshly closes
  those 55 selected Parts/default/conversion leaves and the listed Authority
  group. The earlier partial result remains historical and is not added to
  the newer 194-leaf total. An older aggregate also reported 181 leaves; that
  historical count has not been reconciled with the primary manifest, which
  records 194, so only the manifest-backed 194 is used as the current bounded
  result and the 181 is not added or treated as equivalent.
- URI has accepted Builder API evidence (15 targets / 46 own + 36 support
  leaves), current-source `From<Authority>` / `From<PathAndQuery>` for `Uri`
  evidence (6 own leaves), same-type `Uri` equality (30 own leaves), and
  invariant-preserving `Hash for Uri` (25 own leaves on its recorded freeze).
  `Debug for Uri` is proved under an opaque imported Display contract, while
  `Uri::Display`, `Uri`-to-`str` equality, parsing, remaining component
  getters/conversions, and `Hash` bodies for Authority/Scheme remain separate
  local work; these are not broadly blocked on the dependency premise or
  toolchain.
- Same-type `Uri` equality has a published 20/23 partial result on an earlier source. The later field-conjunction run on `src/uri/mod.rs` SHA `7224a7fa…` closes 30/30 owned leaves (26 body + 4 refinement), with no null leaves. Astra's independent audit confirms all 30 complete Why3 contexts byte-identical and all 295 evidence artifact hashes matching; see the report SHA above. The separate transport and branching attempts reached 26/27 and 28/30; counts are historical and non-additive. Six carried-over exact obligations (From Authority 3, From PathAndQuery 3, Uri Debug 7) were independently checked by byte-identical full stdout streams. `Uri`-to-`str` equality remains separate. `Hash for Uri` proves invariant preservation only; no digest value or Eq/Hash coherence result is established.
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
