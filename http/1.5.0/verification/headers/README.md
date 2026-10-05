# Header validation proof checkpoint

> **Checkpoint note (2026-10-05):** This page is an earlier proof plan and
> does not reflect later HeaderName/HeaderValue evidence. The current status,
> including the historical/current distinction for HeaderName and the v7
> HeaderValue snapshot, is in
> [the HTTP checkpoint](../CHECKPOINT_2026-10-05.md). Use the archived
> manifests for exact target/source status.

## Resumed verification snapshot (2026-10-05)

The current focused HeaderName emission uses `src/header/name.rs` SHA
`8c4633906a8ecd6e30e7eb75f8ace00c9b9f89d5e5f4c325f7e6b10a4eee94d6`.
Seventeen scoped Name targets are proved with 58/58 terminal leaves and exact
direct-arity matches. `StandardHeader::from_bytes` is still partial: 81/82
children passed, while the universal `None` child remains open. The archived
proof reports the exact failed task and excludes any claim that the parser is
proved. See
[`name_remaining17_current_wip_20261005.json`](evidence/name_value_wip_current_2026-10-05/proof_batches/name_remaining17_current_wip_20261005.json),
[`name_from_bytes_current_wip_20261005.json`](evidence/name_value_wip_current_2026-10-05/proof_batches/name_from_bytes_current_wip_20261005.json),
and the fresh emission manifest in that evidence directory.

Native parity checks pass 27 Name tests and 38 tests in the focused profile
that includes the current HeaderValue formatter; raw logs, exact commands, and
source hashes are in
[`native_checks_2026-10-05.json`](evidence/name_value_wip_current_2026-10-05/native_checks_2026-10-05.json).
These tests validate runtime behavior, not proof obligations.

The live `src/header/value.rs` has since changed to add an opaque formatter
append-transitivity lemma and chain formatter writes across loop iterations
and early returns. It currently has SHA
`197fe1071f054c5f018fe1a8cc21266d679164a77088b9459eeb2ee3dd34330e` and has
only passed the focused native tests. This later Value source was not part of
the archived Name emission and has no fresh Creusot emission or proof yet.
The exact unproved WIP source is preserved in
[`value_formatter_wip_current_2026-10-05/manifest.json`](evidence/value_formatter_wip_current_2026-10-05/manifest.json).

## API and representation map

This is the checkpoint for the assigned `method.rs`, `header/name.rs`,
`header/value.rs`, and `byte_str.rs` files. The public contract shapes and
representations that still need review and specification are:

| Type / API | Intended contract and representation | Status |
|---|---|---|
| `Method` | Built-in variants denote their exact uppercase token. Extension variants preserve the input token exactly; empty input and non-`tchar` bytes are rejected. Lengths through 15 use the inline representation; longer extensions allocate. `as_str`, equality, ordering, `is_safe`, and `is_idempotent` must agree with the represented method. `QUERY` is currently classified safe and idempotent. | Contract and body unverified |
| `HeaderName` | Standard variants denote their lowercase standard spelling. Custom names store the normalized lowercase ASCII bytes. Inputs must be nonempty and within `MAX_HEADER_NAME_LEN`; ordinary parsing case-folds ASCII, while `from_lowercase` must preserve already-lowercase names. Equality, hashing, and byte/string views must use the same normalized name. | Contract and body unverified; quote discrepancy is recorded in `HEADER_ANOMALIES.md` |
| `HeaderValue` | Stores opaque bytes. Validity is HTAB or `0x20..=0xff` except DEL; `to_str` succeeds only for HTAB or visible ASCII `0x20..=0x7e`. The sensitive flag affects debug masking but not equality, ordering, or hashing. | The current source-linked group proves `From<u16>` body/refinement plus `hex_digit` (3 targets / 12 own leaves). Other integer conversions and the remaining constructor/accessor/debug paths are open; see the current checkpoint for the exact source hash and dependency boundary. |
| `ByteStr` | Its `Bytes` contents are valid UTF-8. Checked construction establishes this; string-based construction preserves the exact UTF-8 bytes; the unsafe constructor assumes the UTF-8 condition; dereference may rely on the invariant. | Invariant contract and body unverified |

The following two small byte predicates are the completed leaf proof targets.
Their intended contracts mirror the production classifiers:

| Function | Accepted byte set | Use |
|---|---|---|
| `is_visible_ascii` | HTAB or `0x20..=0x7e` | `from_static` and `to_str` |
| `is_valid` | HTAB or `0x20..=0xff`, excluding `0x7f` | header value constructors |

The production functions live in `src/header/value_validation.rs`. The
`#[path]` module in `src/header/value.rs` calls these functions, and the
focused proof harness includes this same source file. The
runtime regression in `verification/runtime-check/tests/header_value_bytes.rs`
checks all 256 inputs through the public constructors and text conversion.
Both production bodies are body-proved through that source include, with no
trusted boundary:

```text
cd verification/headers
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove -- --locked --offline

Library verif.http_headers_proof_rlib.value_validation.is_valid: ✔ (1)
Library verif.http_headers_proof_rlib.value_validation.is_visible_ascii: ✔ (1)
```

This is a focused leaf result, not an integrated proof of `HeaderValue` or the
full http crate.

## Remaining representation gate

The larger method and header constructor proofs have not started. `ByteStr`,
`HeaderName`, and `HeaderValue` depend on byte-sequence and copy-preservation
contracts for the pinned `bytes::Bytes` dependency, plus UTF-8 validity for
`ByteStr`. A length-only model is insufficient. The leaf proof above does not
use or prove any Bytes contract. Record the exact Bytes dependency assumption
and prove any http-owned bridge before reporting those constructors or
representations as verified.

The suggested `ByteStr` proof interface is a private-field `View` over the
HTTP-local exact-byte observer for `Bytes`, with invariant
`creusot_std::std::string::valid_utf8(self@)`. The unsafe constructor should
require that predicate, and dereference should preserve the observer's exact
byte sequence. This is a proposed contract shape, not an implemented or proved
specification.

No additional trusted function or cfg substitute model has been introduced in
these four files. Public conversions, comparisons, formatting, parsing,
extension storage, and unsafe UTF-8 reliance remain outside the proved scope.
