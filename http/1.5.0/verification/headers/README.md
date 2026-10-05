# Header validation proof checkpoint

> **Checkpoint note (2026-10-05):** The HTTP checkpoint records broader
> historical results. This page records the later focused HeaderName and
> HeaderValue runs below. Read every result against its exact frozen source
> hash; none is a blanket proof of the header API.

## Resumed verification snapshot (2026-10-05)

The accepted focused HeaderName result is bound to the frozen
`src/header/name.rs` SHA `8c4633906a8ecd6e30e7eb75f8ace00c9b9f89d5e5f4c325f7e6b10a4eee94d6`:
17 scoped targets, 58/58 terminal leaves, with direct arities reconciled.
`StandardHeader::from_bytes` on that freeze remains partial at 81/82; its
universal `None` child is open, so this is not a parser proof. The exact
records are
[`name_remaining17_current_wip_20261005.json`](evidence/name_value_wip_current_2026-10-05/proof_batches/name_remaining17_current_wip_20261005.json)
and
[`name_from_bytes_current_wip_20261005.json`](evidence/name_value_wip_current_2026-10-05/proof_batches/name_from_bytes_current_wip_20261005.json).

The later Name parser-absence experiment is separate: its explicit 81-case
helper passed 81/81 children, while the caller passed 82/83 and retains one
open child. This later parser experiment remains unpublished worktree evidence
and is not counted as a closed parser proof. Name source SHA `966107ba…` used
by the fresh Value snapshot below is an archived input; the live Name file has
since changed and must be checked against its own source-bound evidence.

Native parity checks pass 27 Name tests and 38 HeaderValue tests for the
recorded source snapshots; raw logs, exact commands, and source hashes are in
[`native_checks_2026-10-05.json`](evidence/name_value_wip_current_2026-10-05/native_checks_2026-10-05.json).
These tests validate runtime behavior, not proof obligations. The 38-test
Value run covers `value.rs` SHA
`197fe1071f054c5f018fe1a8cc21266d679164a77088b9459eeb2ee3dd34330e`.

The current focused HeaderValue source SHA
`197fe1071f054c5f018fe1a8cc21266d679164a77088b9459eeb2ee3dd34330e` has a
fresh 333-COMA Creusot emission. Four error-formatter bodies and their four
trait refinements are proved: 25 body children plus 8 refinement children,
33/33 total, with no null children. These results establish the recorded
`formatter_extends` postconditions and refinements using the imported
formatter contracts; they do not prove the standard library formatter
implementations or every concrete output string. The proof inputs freeze Name
SHA `966107ba…`, not the later live Name source. The focused manifest, commands,
proof trees, and hashes are in
[`value_current_emission_2026-10-05`](evidence/value_current_emission_2026-10-05/manifest.json).
The independent Why3 task extraction matched all 33 archived task contexts
byte-for-byte and reconciled the proof JSON tree; see its arity audit and task
hash ledger in that directory. Astra's read-only independent audit, including
the reviewed formatter boundary, is recorded in
[`independent_audit.md`](evidence/value_current_emission_2026-10-05/independent_audit.md).

The same emission includes the indexed `HeaderValue` Debug body/refinement and
writer/transitivity helpers. Their exact direct arities are 54, 3, 7, 29, and
3 respectively (96 tasks total), and none is proved in the 33-child batch.
They remain open work. No claim is made that HeaderValue Debug or the full
formatter path is proved.

## Earlier API and representation plan

The table below is the original scope plan, not a current status report. Later
focused results are recorded above and in the HTTP checkpoint; use their
source-bound manifests when assessing what is proved.

This is the checkpoint for the assigned `method.rs`, `header/name.rs`,
`header/value.rs`, and `byte_str.rs` files. The public contract shapes and
representations that still need review and specification are:

| Type / API | Intended contract and representation | Status |
|---|---|---|
| `Method` | Built-in variants denote their exact uppercase token. Extension variants preserve the input token exactly; empty input and non-`tchar` bytes are rejected. Lengths through 15 use the inline representation; longer extensions allocate. `as_str`, equality, ordering, `is_safe`, and `is_idempotent` must agree with the represented method. `QUERY` is currently classified safe and idempotent. | Contract and body unverified |
| `HeaderName` | Standard variants denote their lowercase standard spelling. Custom names store the normalized lowercase ASCII bytes. Inputs must be nonempty and within `MAX_HEADER_NAME_LEN`; ordinary parsing case-folds ASCII, while `from_lowercase` must preserve already-lowercase names. Equality, hashing, and byte/string views must use the same normalized name. | Contract and body unverified; quote discrepancy is recorded in `HEADER_ANOMALIES.md` |
| `HeaderValue` | Stores opaque bytes. Validity is HTAB or `0x20..=0xff` except DEL; `to_str` succeeds only for HTAB or visible ASCII `0x20..=0x7e`. The sensitive flag affects debug masking but not equality, ordering, or hashing. | An earlier pinned source group proves `From<u16>` body/refinement plus `hex_digit` (3 targets / 12 own leaves). Other constructor/accessor paths require their own source-bound results; the current formatter status is above. |
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

## Historical representation plan

This proposed gate predates the later Bytes and consumer verification records;
it is retained as design context and is not a current blocker/status summary.

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
