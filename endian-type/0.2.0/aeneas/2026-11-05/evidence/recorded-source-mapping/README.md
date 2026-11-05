# Exact source bindings for endian-type 0.2.0

The table covers all 66 Aeneas API contracts in the reviewed union candidate. `bindings-66.json` preserves exact Rust identities, source-scanner-compatible recorded keys, macro definition/invocation spans, original-body origin, proof targets and hashes. `bindings-66.tsv` is the compact view. `proofs-source-keys.toml` changes only proof metadata API spellings: the six generic owners retain `<T>`, and all sixty concrete From identities retain both self and trait-input type arguments. The standard prelude `From` spelling resolves to `core::convert::From`; this equivalence changes no type argument.

The updated metadata was checked through the actual compiler-expanded Aeneas discovery path using the diagnostic scanner probe: all 66 configured exact keys match exactly one public original callable. Source/Cargo.toml copies match the original; see exact-key-probe-result.json and probe-source-hashes.json. The audit made no server requests, performed no publication, and changed no shared CLI file.

## Original-body origin

- Six generic bitwise methods are direct original method roots. Their LLBC embeds the original src/lib.rs content, which matches the release source hash.
- Sixty From methods are the actual original dependency functions reached through caller stubs. Each is public, transparent, non-FFI, non-opaque, and has a translated body; the theorem targets its generated function. The caller functions are not API targets.
- Compiler path remapping gives the dependency Virtual source path `endian-type-0.2.0/src/lib.rs`. Its LLBC does not embed source contents. The separate archive/source hash guard, exact path dependency and compiler command identify the original source; the table does not claim embedded-source equality for those sixty methods.
- Twenty From implementations have a foreign primitive self type (for example `<u16 as From<endian_type::BigEndian<u16>>>::from`). The local wrapper occurs in the trait argument and must never replace the actual primitive owner.

## Official recording path

Use only the publisher-managed fresh recorded-run workflow for this already imported crate/version. Existing ready evidence is in `/workspace/batch-ten/service-limit-details.json`; this audit does not independently requery current server state.

1. Capture each validated configured exact key as the sole `contract.api_paths` entry. Do not emit erased aliases alongside it.
2. [POST /api/v1/runs/prepare](https://github.com/proofs-rs/proofs-rs/blob/0a39304e702b196e6dff917bbf17c63dc01abcc6/src/runs.ts#L322-L363) validates immutable SARIF and previews bindings without storing evidence, creating API rows, importing rustdoc or publishing. The publisher should verify every returned key and source identity against this table.
3. [Official run upload](https://github.com/proofs-rs/proofs-rs/blob/0a39304e702b196e6dff917bbf17c63dc01abcc6/src/runs.ts#L366-L432) records the actual evidence and its API observations. It uses the verification-run quota, while this work makes no new crate/version catalogue import request. This is the service's source-observation path for genuine verified public source APIs, not an independent/manual API-insertion mechanism.
4. [recordBindings](https://github.com/proofs-rs/proofs-rs/blob/0a39304e702b196e6dff917bbf17c63dc01abcc6/src/recorded-apis.ts#L12-L55) prefers an existing matching key from any supplied path over a missing first path. Supplying typed and erased aliases together can therefore bind to an old incorrect nominal-owner row. A sole exact source key avoids that fallback.
5. Existing run bindings are immutable. Use a fresh genuine recorded run when API metadata changes; do not patch an old uploaded SARIF or pretend an old binding changed.

Missing exact keys receive author-observed `recorded` provenance through [observationStatements](https://github.com/proofs-rs/proofs-rs/blob/0a39304e702b196e6dff917bbf17c63dc01abcc6/src/recorded-apis.ts#L57-L105), rather than an independent rustdoc signature/safety assertion. A matching correct existing exact key may reuse its ID. Each JSON row includes the deterministic recorded ID *if that exact key is absent*; these are predictions, not live server observations. The deployed server revision was not independently identified; the cited public source revision is `0a39304e702b196e6dff917bbf17c63dc01abcc6`.

Publication and any CLI integration remain under the publisher's control. No import-quota workaround, arbitrary API declaration, erased owner alias, or replacement of a foreign primitive owner is proposed.
