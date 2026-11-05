# Proposed exact external-blanket source observation

This is a read-only design and evidence inventory. No CLI/server code or public record was changed. Existing 25 Rng call proofs are successful in the frozen supplemental54 packet. Their coexistence with the newer local44 packet has not been compiled, simultaneously imported, or audited. The potential union is 69 distinct APIs, not a completed or published 69-API result.

## Smallest honest scope

Opt in only to rand_xoshiro 0.8.1, external rand_core 0.10.1 with archive checksum `63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69`, x86_64, default and serde as separate checked configurations, and exactly the existing 25 concrete `Rng::next_u32`/`next_u64` calls. This does not discover all external APIs, prove arbitrary blanket implementations, or add the five missing 32-bit engines' Rng::next_u64 compositions. Keep the exact existing keys `<rand_xoshiro::Type as rand_core::Rng>::next_uXX` and the existing caller-root theorem mappings. These are callable concrete external blanket specializations, not methods declared in rand_xoshiro. The blanket methods have their own implementation bodies, not trait default bodies.

The meaningful extension is a new scanner observation category `external_blanket_specialization`. It must not reuse endian's current exact-key selection hook as if keys alone supplied missing callables. Current aeneas::discover falls back from local source scanning to expanded local source, neither of which observes external blanket impls. The narrow endian hook deliberately only selects already observed source keys and is hardcoded to endian-type 0.2.0. That mechanism cannot honestly create these 25 observations.

## Evidence available now

`existing-observations.json` records 50 existing compiler-resolved observations (25 APIs, default/serde): exact rustdoc JSON/hash, concrete owner, external trait path and crate ID, the actual external blanket impl item, method signature/item, local TryRng impl and associated Error item, exact selection caller and original Inline MIR/hash. The original target's entire Rust file set/Cargo/lock and all files of the external archive are hashed. The original supplemental packet already compares all 19 rand_core archive files against the source compiled from Cargo's registry, pins the lock checksum, verifies exact forwarding caller bodies/receiver/return, and checks that generated selected programs have no unknown external declarations and are identical across default/serde.

Rustdoc format61 shows the actual concrete blanket applicability, not merely a textual generic impl: for example default item675 has `for Xoshiro512PlusPlus`, `trait rand_core::Rng` (external crate_id2), `blanket_impl R`, and source rand_core/src/lib.rs65..67. Its original where predicate is `R: TryRng<Error = Infallible> + ?Sized`; the local concrete implementation explicitly writes Error=Infallible. Rustdoc normalizes this associated equality to `primitive never` on this compiler. Preserve that raw normalization plus the original source binding and compiler-resolved application; do not generalize every never-looking alias to Infallible without compiler/source evidence.

## Proposed observation schema

Each entry should retain the exact runtime identity and exact recorded key separately, with no erasure or alias creation:

- `kind`: external_blanket_specialization; `local_declaration`: false.
- `runtime_identity`: concrete owner path/type arguments, external trait path/type arguments, method, mutable receiver, exact return type, feature configuration, target, dependency resolution.
- `record_api_paths_exactly`: the one existing fully qualified external trait key. No TryRng alias or wrapper/local API key.
- `source_chain`: original public concrete type/reexport, local actual TryRng impl, associated Error equality, external public Rng trait method declaration (lib.rs49..63), actual blanket impl/body (lib.rs65..89), applied substitution R=concreteType and complete predicates. Include source spans and hashes, not unstable rustdoc item IDs as identity.
- `compiler_observation`: rustc/Charon versions and command, raw rustdoc artifact/hash, resolved concrete impl and method signatures, raw associated-type normalization, exact selector source/hash, original trait and native-callee Inline MIR scopes/hash, target/features.
- `proof_binding`: existing caller-root module/theorem and exact generated symbol/signature plus generated body hash, original callee closure, no-unknown-external check, theorem domain and standard-three audit. This proves a call to this specialization, not the universal external trait/blanket implementation.
- `source_integrity`: target/dependency archive hashes, all source file hashes, Cargo manifest/lock checksum, exact feature/dependency versions. Reject mismatch rather than weakening resolution.

The trusted production scanner should construct/validate these observations from compiler output for the captured immutable source snapshot. A user-supplied manifest of names/hashes alone cannot establish applicability; either regenerate the compiler observation in the record workflow or authenticate it through the same captured compiler/source pipeline. A read-only JSON adapter is useful evidence but is not itself a verified production scanner.

## Record/upload boundary

The checked server snapshot's recorded-apis.ts already permits exact recorded API spellings when a rustdoc catalogue entry is absent; the immutable binding retains that exact path. This suggests a server protocol extension may be unnecessary if its current deployed behavior is identical. This is a proposal, not a server compatibility result: confirm the deployed schema and exact binding origin before implementation. The narrow client change would supply a genuine source observation Contract for each validated external specialization, and route the same immutable-source observation through both discovery and validate_aeneas_source_bindings. Do not bypass the publish-time exactly-one source observation check or claim server origin `rustdoc` when registration used `recorded`.

## Required negative cases

Reject wrong dependency version/checksum or lock, stale/mutated original source or manifest, undeclared/private owner, a same-name local trait or method, wrong trait identity/crate ID, missing/ambiguous blanket impl, negative/synthetic/nonapplicable impl, Error unequal to compiler-resolved Infallible, unsatisfied generic predicates, erased owner/type arguments, mutability/return-width mismatch, wrong native callee, wrapper bodies containing any extra work, missing original Inline scope, unknown external callee, theorem for another selector/state representation, unproved feature or target, duplicate keys, cross-run/source/config binding substitution, path escape, missing coimport, or an attempt to include the five unproved Rng next_u64 compositions. Ordinary all-zero internal states remain within the proved output/state formulas; this does not establish seed reachability or statistical claims.

Before a 69-API candidate exists, retain old44 and supplemental54 unchanged; combine in a new candidate, prove nominal namespace compatibility or re-extract officially if necessary, regenerate both configurations, preserve the old44 claims and model boundary, run all claim modules in one import, and audit every transitive theorem. No such integration is claimed here.
