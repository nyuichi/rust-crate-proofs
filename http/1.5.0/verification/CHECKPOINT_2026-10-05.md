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
| `HeaderValue` v7 archive | The archived v7 snapshot records 67 targets and 184 own leaves, including `HeaderValue::to_str` / 5 leaves. | Historical for `value.rs` SHA `1ab329864e57e80bd494fcc610ee0da3c7ac77c95c13fcf5a23647166a452e0c`; current-source evidence is listed separately. |
| `HeaderValue` numeric/hex group | The source-linked snapshot [`value_numeric_hex_contract_v1`](headers/evidence/archived_snapshots/value_numeric_hex_contract_v1/snapshot.json) proves 3 targets / 12 own leaves on `value.rs` SHA `5420fc7682706cb56adc2516a94cd29182ffad5997186cf59593742549c9f8f6`: `From<u16>` body (7), its refinement (1), and `hex_digit` (4). All 12 direct own tasks passed the independent arity check. | Other integer `From` implementations remain unproved. The `bytes` 1.11.1 model remains an explicitly assumed dependency premise; this is not a full HeaderValue/API proof. |
| Earlier `HeaderValue::hex_digit` snapshot | A separate prior-source proof closed 4 own leaves on `value.rs` SHA `1009a163a8f3dc853d228d8fc25de07cc176c974aa278361b274c4e72c8702e9`; the runtime parity check compares old formatter output for all 256 byte values. | Historical after the `5420fc…` source snapshot; its body result is superseded by the fresh 4-leaf `hex_digit` target in the numeric/hex group. |
| `HeaderName` | The sealed base snapshot at `verification/headers/evidence/archived_snapshots/name_hdrname_invariant_v1/snapshot.json` records 34 targets / 280 own leaves on source SHA `fbb82430831eee11bbb5d6487edfe6975f89d211e0757566efd4843b9baaa63f`. A later exact-source increment in `name_hdrname_manual_debug_parser_v1/snapshot.json` proves 3 targets / 52 own leaves on source SHA `3f68bc7cafe0d96d6fb237cfa7f5931521a72818e550bf50863f13fae44bc48b`: `parse_hdr` (39), `HdrName` Debug body (10), and Debug refinement (3). Its only post-proof source delta is cfg(test) parity code; the native parity check passes 1/1 at source SHA `b807544438a60c3213677e112f335f3cd1e05cb16784cfb42d9ac4d408d02a80`. | These are two source-linked checkpoints, not a 37-target proof on one identical source snapshot. The second snapshot's 195 emitted Comas are comparison inputs; only its three named proof targets are fresh proofs. It does not establish all Name APIs or a full 195-target closure. |
| `ByteStr` / bytes consumers | The ByteStr and bytes manifests record the actual included HTTP `ByteStr` bodies and 36 actual bytes consumers (79 consumer VCs), with exact byte-sequence contracts. | The pinned `bytes` implementation and the documented Creusot UTF-8 boundary remain premises/limitations; these are not proofs of the dependency crate. `ByteStr`'s unsafe-constructor panic text differs from the published implementation. |
| HeaderMap capacity/index helpers | The map-capacity manifest records 15 source targets and 30 successful leaves, including `desired_pos` and `probe_distance`. | The private callers' power-of-two, mask, and range invariants are not proved at all HeaderMap call sites. This does not prove insertion, lookup, growth, removal, or the map as a whole. |
| HeaderMap capacity constructors | The curated [two-target archive](header-map-api/evidence/run-2026-10-05-capacity-constructor-lemma/manifest.json) proves `max_size_is_power_of_two` (1 own leaf) and actual-source `HeaderMap::try_with_capacity` (13 own + 16 call/spec leaves). Its named-profile frontend and normal HTTP library `cargo check` both pass. | This proves the selected constructor under the named leaf profile; the `usize::checked_next_power_of_two` standard-library contract is a TCB premise. It does not prove the integrated HeaderMap invariant or all capacity/reservation APIs. |
| HeaderMap actual-source leaves | The named `http_map_api_leaf` snapshot records `new`, `default`, `len`, `keys_len`, `is_empty`, `clear`, and `capacity`, plus allocation-bound/Vec probes: 47/49 total leaves pass. The separate [Entry accessor archive](header-map-api/evidence/run-2026-10-05-entry-accessors/manifest.json) proves 6 actual method bodies / 6 own leaves, with 4 successful `Vec<Bucket<T>>` index support leaves; its printer task names/counts match all six proof JSON files. The conditional `IterMut::next_unsafe` run has 79 own leaves and 33 supporting leaves (112 total); its 70 first-level tasks and nine selected two-child splits were checked against the archived tree. | Entry proofs require only `index < entries.len`; this proves selected-slot access, not that the slot is occupied, matches a lookup key, or is reachable from a valid `Entry`. `len` still has two open generic layout leaves. `next_unsafe` assumes storage/cursor-ready preconditions; map graph validity, cross-entry non-revisiting, and public Iterator refinement are not established. Drain and raw removal remain open. |
| Request/Response composition | The fresh sealed archive records 49 targets with 107 terminal leaves: 65 own and 42 call/spec support leaves. All 49 proof JSON trees match the run-log arities. It includes 30 structural API bodies, three Builder version bodies, Parts Clone, generic Request/Response Clone bodies and refinements, and four Debug bodies/refinements. | The manifest is bound to source-map SHA `54d174cd6a830a50e101d376ab95209a2aad258d1b7f19d78d2e26a8e5bb1efd` and records source stability during proof. Parts Clone relies on imported field Clone contracts, especially HeaderMap/Extensions. Debug proves append preservation, not exact rendered bytes. Builder header/extension methods and `Error::is`, `get_ref`, and `source` remain outside the named profile because their respective dependencies are unresolved or trait-object translation is unsupported. |
| URI | The URI path API checkpoint records selected actual-source groups. `PathAndQuery` Display closes the body, refinement, and transitive formatter helper: 31 own leaves (25 + 3 + 3). Its comparison checkpoint has 28 targets and 125 terminal leaves, with first-level and nested task arities checked. The Authority comparison archive records 119 own leaves (44 helper, 41 PartialEq, 34 PartialOrd) and a native comparison regression check of 1/1. The Parts/constructor archive contains 12 roots and 55 expected own terminal leaves; its independent COMA-arity audit confirms all 55 are present. | These are named URI leaf-harness results, not a full URI API proof. The Parts/constructor batch has 48 passed leaves and 7 unresolved leaves; those 7 are not counted as proof. The comparison run is bound to archived Authority source SHA `5b5698cb74cfc68c84f234481e6291c1112b1e80cbcf7fdd56d573b750b5ac5d`; the current file is SHA `fd3f129ab0683ba62f747a18f6e43106888bdb6224d4f7e4477e389ef913828f`. A post-emission runtime regression for the helper-based `Authority::hash` passes 1/1, but proof of that changed Hash body is pending. `Parts::default` in `uri/mod.rs` is also a post-emission source delta. Other URI constructors and API closures remain profile-scoped. |

## Historical or partial results

- The HeaderName base snapshot records 34 targets / 280 own leaves, followed by
  a separate 3-target / 52-own-leaf exact-source increment for `parse_hdr` and
  `HdrName` Debug. The second increment's cfg(test)-only parity addition passes
  its native regression 1/1. Keep the two source fingerprints separate; neither
  proves the complete Name surface, and no blanket 195-target closure is
  claimed.
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
- Status constants remain excluded. The Name snapshots record only their
  enumerated successful own goals; historical reuse candidates with unknown
  run-exit provenance and targets without complete matching trees are not
  counted as accepted proofs.
- `verify-all.bash` is a crate-level pipeline check and may stop at the first
  frontend/proof failure. Its outcome is recorded separately from the accepted
  leaf results above; it is not a substitute for the exact target manifests.
- A separate URI `Parts`/conversion run is preserved at
  `verification/uri/evidence/parts-uri-constructors-2026-10-05/`. Its manifest
  records 12 roots and 55 expected own terminal leaves; the independent
  COMA-arity audit confirms all 55, with 48 passed and 7 unresolved. The
  unresolved leaves are in `From<Uri> for Parts` and `Uri::from_parts` as
  listed in the manifest. This remains a partial attempt, not a completed
  constructor/API proof. Its exact source snapshot and post-emission source
  changes are listed in that manifest.
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
