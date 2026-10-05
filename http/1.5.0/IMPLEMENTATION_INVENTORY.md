# `http` 1.5.0 implementation-body inventory

This checklist complements [`API_INVENTORY.json`](API_INVENTORY.json) and the
behavioral requirements in [`RUNTIME_MODEL.md`](RUNTIME_MODEL.md). The API
ledger covers public declarations; complete validation also requires every
reachable production body, private helper, unsafe block, iterator state step,
and destructor. It is a source-component inventory, not a claim that a whole
program call graph or any module has been completely proved. All rows remain
open until their reachable body and representation contracts are proved in an
integrated run.

| Source component | Reachable implementation work that must be covered | Current evidence or gate |
|---|---|---|
| [`src/lib.rs`](src/lib.rs) | Feature selection, compile-time public module/re-export graph, unit-test inclusion | Default and all-feature Rust runtime matrices pass; Creusot integration has no successful full run |
| [`src/byte_str.rs`](src/byte_str.rs) | Byte ownership, UTF-8 invariant, checked/unchecked construction, dereference and conversions | Pending exact `Bytes` model and UTF-8 contracts; unsafe unchecked constructor is in scope |
| [`src/convert.rs`](src/convert.rs) | `if_downcast_into!` expansion behavior at each call site | `dyn Any` and `TypeId` translation gate; see [`TOOL_BLOCKERS.md`](TOOL_BLOCKERS.md) |
| [`src/error.rs`](src/error.rs) | `ErrorKind` projections, source references, formatting, and all conversions | Dynamic `std::error::Error` projection and full conversion bodies pending |
| [`src/extensions.rs`](src/extensions.rs) | Insert/get/remove/clear/extend/clone, type tags, erased ownership and downcasts | `dyn Any` / `AnyClone` translation gate; sound typed-existential and clone/frame contracts required |
| [`src/header/mod.rs`](src/header/mod.rs) | Header exports, standard constants, and module-level helpers | API inventory is source-backed; full integration pending |
| [`src/header/name.rs`](src/header/name.rs) | Parsing, normalization, standard-name tables, hashing/comparison, case conversion, `MaybeUninit` initialization helpers | Two source-linked checkpoints cover 34 targets / 280 own leaves, then `parse_hdr` (39), `HdrName` Debug (10), and its refinement (3) on a later source snapshot. Native cfg(test) parity passes 1/1 after a test-only delta. The full source remains open; see the Name snapshot manifests and do not infer 195-target closure. |
| [`src/header/value.rs`](src/header/value.rs) | Constructors, byte/string views, formatting, equality/order/hash, integer conversion, unchecked constructors | Current source-linked `From<u16>` body/refinement and `hex_digit` group: 3 targets / 12 own leaves; other integer `From` bodies and the remaining constructors/accessors/formatting paths are open. The dependency byte model remains a stated premise. Historical v7 evidence is kept separate. |
| [`src/header/value_validation.rs`](src/header/value_validation.rs) | `is_valid` and `is_visible_ascii` predicates | Both actual-source bodies proved, one VC each; public callers and integrated use remain open |
| [`src/header/map_capacity.rs`](src/header/map_capacity.rs) | Checked capacity arithmetic and usable-capacity calculation | Both actual-source bodies proved, three VCs total; callers and map invariant remain open |
| [`src/header/map.rs`](src/header/map.rs) | Hashing, bucket insertion/rehash, duplicate-value links, entry APIs, reserve/remove/retain, iterators, drain, `IntoIter`, drop guards | Selected actual-source leaves now cover capacity arithmetic/constructors, seven `HeaderMap` methods with two open generic `len` layout leaves, six conditional Entry accessors (6 own + 4 Vec-index support leaves), and conditional `IterMut::next_unsafe` (79 own + 33 support leaves). Lookup/reserve/growth/removal, map graph validity, public Iterator refinement, Drain, raw-pointer field ownership, and exact-drop behavior remain open. See the named manifests; these are not an integrated HeaderMap proof. |
| [`src/method.rs`](src/method.rs) | Token parse, inline/heap representation, byte/string views, classifications, equality/order/hash, unchecked UTF-8 | No whole-body proof; exact bytes and UTF-8 facts required |
| [`src/request.rs`](src/request.rs) | Generic body construction/map/parts conversion, field access/mutation, builder state/error handling, trait impls | Pending field-wise model and ownership/frame proofs |
| [`src/response.rs`](src/response.rs) | Generic body construction/map/parts conversion, field access/mutation, builder state/error handling, trait impls | Pending field-wise model and ownership/frame proofs |
| [`src/status.rs`](src/status.rs) | `NonZeroU16` construction/access, byte parsing, numeric conversion/classification, constants, formatting, unsafe construction | `from_u16`, `from_bytes`, `as_u16`, and five classifications proved in the scalar leaf harness; constants, formatting, unsafe construction, and integrated use remain open. The standard-library model boundary is tracked in [`DEPENDENCY_CONTRACTS.md`](DEPENDENCY_CONTRACTS.md) |
| [`src/version.rs`](src/version.rs) | Closed enum view, default, equality/order/hash, debug formatting | `Default` body/refinement proved as a leaf; integrated run and remaining methods open |
| [`src/uri/mod.rs`](src/uri/mod.rs) | URI parse dispatch, accessors, conversions, formatting, equality/hash, parts/builders, unsafe UTF-8 sites | New scheme-prefix leaves proved; full parsing/conversion remains open; downcast macro gate applies |
| [`src/uri/authority.rs`](src/uri/authority.rs) | Authority validation, host/port scans, constructors, formatting/comparison/hash, UTF-8 conversion | Pending exact-byte, scanner, and downcast contracts |
| [`src/uri/builder.rs`](src/uri/builder.rs) | Builder state changes, error retention, build validation and parts conversion | Pending |
| [`src/uri/path.rs`](src/uri/path.rs) | Path/query scans, offsets/fragments, constructors, conversions, UTF-8 conversion | Pending exact-byte and downcast contracts |
| [`src/uri/port.rs`](src/uri/port.rs) | Port parse, numeric/text view, `From`, `AsRef`, equality and formatting | `as_u16`, numeric `From`, and three `PartialEq` bodies/refinements proved in actual-source harnesses; other methods and integrated use remain open |
| [`src/uri/scheme.rs`](src/uri/scheme.rs) | Scheme classification, validation, case-insensitive equality/hash, formatting and conversion | Two HTTP/HTTPS prefix helper bodies proved; scheme parsing and remaining behaviors open |
| [`src/uri/scheme/fixed.rs`](src/uri/scheme/fixed.rs) | ASCII lowercase and fixed-prefix scan loop | Both actual-source helper bodies proved, four VCs total; containing parser remains open |

The `cfg(test)` module [`src/uri/tests.rs`](src/uri/tests.rs) is covered by the
Rust runtime test matrix; it is not a substitute for proving production
methods. The integration and fuzz regression tests under `tests/` are also
runtime evidence only. The `unsafe`, raw-pointer, iterator, and `Drop` portions
of `HeaderMap`, all `ByteStr` unchecked UTF-8 paths, and all erased downcast
paths remain explicitly in scope.

The repository search used to seed the unsafe/iterator/drop checklist was:

```sh
rg -n '\bunsafe\b|impl<.*Drop|impl Drop|fn drop\b|Iterator|IntoIterator|if_downcast_into' src --glob '!**/tests.rs'
```

This source checklist must be reconciled again after any source change and
before any complete-verification claim.
