# Identity and publication requirements

This supplements `../inventory-schema.md`; it does not narrow the user request.

## Inventory identity

An entry represents one Rust declaration or implementation/specialization under an explicit configuration. Keep these fields separate:

- `local_id`: stable digest of crate/version + declaration/source identity + full self/trait/type/const arguments + configuration key. Source span and compiler rustdoc ID remain evidence; a rustdoc ID alone is not stable across runs.
- `rust_identity`: item kind, defining crate, canonical public path(s), owner/self type, complete trait reference, impl parameters/where clauses, method parameters/where clauses, full signature, source span/hash, and compiler item/impl IDs.
- `configurations`: target triple, target cfg, Cargo default/no-default/features, build profile/debug assertions, and any unsupported/conflicting cfg combinations. Do not represent all-features as all combinations.
- `origin`: source provenance (handwritten, macro-authored, derived, inherited default, external blanket, auto trait); defining body crate; `is_standard_trait`; derive attribute/provenance. Exclusion requires both actual derive provenance and a standard trait. A no-body trait declaration is an obligation/interface, not a proved body.
- `catalogue_links`: zero or more server canonical keys/API IDs, with exact source revision or observed server lookup; note collisions and aliases. Never use this lossy identity as the inventory primary key.
- `obligations`: meaningful behavioral and safety contracts, including panic/error/termination cases, callback/dependency contracts, invariants and composition requirements.
- `evidence`: theorem/run/source/toolchain, original versus adapted source, quantified type/value domain, configuration domain, trusted or modeled boundaries, body status and remaining obligations.
- `coverage`: independently track body proof (`original_proved`, `adapted_proved`, `unproved`, `no_body`), domain (`complete_for_entry_domain`, `partial`, `none`), dependency coverage and integration success. `excluded_derived_standard` is not a proof. `tool_blocked` is not an observed crate defect.

Types, constants, public fields/variants and macros need separate invariant or expansion obligations. External blanket implementations and arbitrary downstream implementations are an open boundary; record it explicitly, not as an arbitrary finite list or as silently excluded coverage.

A crate may be called complete only after every eligible identity, configuration and stated domain obligation is discharged, with no unresolved modeled/dependency gap hidden by the count. A passing finite list of methods cannot establish that condition.

## proofs.rs representation limits

Reviewed server source: `proofs-rs/proofs-rs` commit `0a39304e702b196e6dff917bbf17c63dc01abcc6`; deployed revision is not established.

1. The importer stores callable keys. Its type-driven route uses public nominal owner paths and intentionally erases generic/specialization arguments. Multiple `Either<...>::cloned` implementations can consequently share a key; `output.set(name,...)` replaces the prior entry. Full signatures/metadata do not repair this identity collision. See `src/imports.ts:285-365,450-505`.
2. Structural implementations of local public traits use a later self-type-aware route, but only for implementations not visited by the nominal route. This can preserve primitive identities such as `<u8 as bit_field::BitField>`, without guaranteeing full coverage for all type-driven specializations (`src/imports.ts:515-555`).
3. Synthetic impls are skipped. Local default trait methods can be added, but foreign-trait inherited defaults are not expanded by `implMethods`. Public fields/constants/macros are not individual callable API catalogue entries (`src/imports.ts:375-399,430-480`). A catalogue listing is therefore not the complete Rust public surface.
4. `/resolve-api` matches an exact display path or a crate-prefixed version. It does not solve generic equivalence or distinguish feature configurations (`src/api.ts:1673-1700`).
5. Server claims can carry separate preconditions, explanations, limitations and evidence references; claim targets are API IDs. The inspected validation rejects duplicate claim IDs, but does not reject repeated API targets, so the server can represent multiple scoped claims for the same erased API ID (`src/api.ts:704-765`). The current server caps a report at 200 claims (`src/schemas.ts:1`). The local cargo-proofs 0.4.1 adapter instead caps Aeneas mappings at 100 and requires distinct API IDs/theorems (`src/aeneas.rs`, `src/publish.rs`).

Use a generic theorem only when it actually quantifies all relevant type parameters and reviewed trait assumptions. For an erased API ID with several distinct implementation obligations, retain the full inventory and publish an accurately scoped conjunction or separate explicitly scoped claims only if the interface supports them. Never treat a u8 proof or one specialized impl as proof of the erased generic item. Do not invent API IDs or aliases to bypass these limits. If publication cannot represent a verified obligation, document that limitation and link its exact evidence; do not erase the obligation from the inventory.

No new publication or import was performed for this audit. Existing report updates do not require a new catalogue import when the version is already ready. New-version imports remain subject to the documented daily quota; old records must not be republished as evidence for newly extended guarantees.
