# Four PRNG crates: source-origin, configured claims and supplemental contracts

The four crates are oorandom11.1.5, rand_pcg0.10.2, rand_xoshiro0.8.1 and rand_xorshift0.5.0. This is a read-only reconciliation of existing source/compiler inventories and proof packets, not a new verification run or server publication. Full-API verification remains incomplete. The107 configured machineAPI keys (12+25+69+1) each match exactly one default-configuration runtime owner/trait/method identity. No count correction below changes those107keys or the source/proof artifacts.

## Count ledger

These are entries observed in each original rustdoc graph, not a claim that every external trait a downstream crate could define is enumerable. Conditional generic families stay in the API inventory, with bounds and Self substitutions retained; they are not declared usable for arbitrary type arguments. Configurations are not summed as distinct APIs. No-default uses the same empty localfeatures as default here; an equivalentfeatures interpretation is not an additional verification run.

| Crate | Default function entries | Manual original/macro | Actualstdderive entries excluded | Inherited defaults | Externalblanket entries | Nonexcluded function entries | Configuredmachine keys | Catalog gap to those keys |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| oorandom |46|16+0|14|0|16|32|12|20|
| rand_pcg |126|29+9|24|20|44|102|25|77|
| rand_xoshiro |459|105+14|107|60|173|352|69|283|
| rand_xorshift |27|7+0|6|3|11|21|1|20|
| Total |658|157+23|151|83|244|507|107|400|

Xoshiro additionally has one public Seed512([u8;64]) data-constructor operation in every configuration. It remains in full-API scope and coverage.json, separately from function-callable entries; it is not excluded. Prior460/537 totals mixed this operation into the function count. Correct function totals are459/default and536/serde, with+1data operation each.

| Crate, serde/all-features graph | Function entries | Manual original/macro | Stdderiveexcluded | Nonstdderive pending | Inheriteddefault pending/catalog entries | Externalblanket entries | Nonexcluded function entries | Machine keys within stated featuredomain |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| oorandom (no localfeatures) |46|16+0|14|0|0|16|32|12|
| rand_pcg |146|29+9|24|8|24|52|122|25|
| rand_xoshiro |536|105+14|107|30|75|205|429|69|
| rand_xorshift |30|7+0|6|2|4|11|24|0|

The Xorshift machine theorem is withoutserde. Default Kani packets are not silently promoted to serde proofs. Source configurations and callback bounds for every supplement are separate fields. The prior threecrate serde-derived list38=Xoshiro30+PCG8 is preserved; the fullfourcrate list is40 after addingXorshift2. All40 remain verification targets, not exclusions.

## Scope of existing evidence

| Crate | Machineclaim scope | Independent supplementary column; overlaps do not add registrations |
|---|---|---|
| oorandom |12exact integerword/constructor/state API bodies | Rand32/Rand64 rand_float range-only Kani under allpublicfrom_state state/inc, output0<=f<1. ExactIEEEbits/state/distribution not established. rand_range remains unproved. |
| rand_pcg |25original API bodies, including4advance contracts for every delta; default/serde | MCGfrom_seed all16bytes→publicstate=LE128|1 under core0.10.0; fourmanualDebug concreteconstructor/defaultfmt/64bytewriter domains. Remaining3LCGseed numeric/composition attempts and PCG32next64 attempts are timeout checkpoints, not successes. |
| rand_xoshiro |69keys=35direct+34compiler-inlined calls, default/serde. Local44 and externalRng25 identities remain distinct; serializer11 retain explicitU32/U64layout bridge. |15originalfrom_seed and15originalstate Kani, plus5TryRng32-engine next64 composition Kani, each with exactfixedword/seed domains and originalprefix instrumentation boundaries. State15 overlap machineclaims. No genericexternalRngblanket universalproof is inferred. |
| rand_xorshift |1modeled-boundary Aeneas try_next_u32 for all4wordstates; stdWrapping bridge explicit | Seed/nonzero327; publicnext64relation294; restrictedDebug2217; boundedfill0..8relation379; finitecallbackinit675. Callback675 proves2actual APIs under3concrete sourceprotocols, not allR or allrejectionhistories. These domains are detailed in xorshift-coverage-reconciliation and per-packetstatus. |

Each row in coverage.json contains machine_claim and supplementary_tool_contracts independently. A supplemental method-name match does not establish an unrestricted API contract. Timeouts in PCGnumeric/fromseed, Xorshiftseed_from_u64/fill64, oorandomarbitraryoffset/fixed10..18range and Seed512Debug are not promoted to completed evidence. The combined Seed512Debug accept/error probe timedout; neitherbranch has a completed proof.

## Trait defaults, source origin and bounds

The original SeedableRng default declarations are not additional callable bodies when an impl overrides them. shadowed-seedable-defaults51.json records17SeedableRngspecializations per configuration (51configuration records):15Xoshiro seed_from_u64 overrides, and Xorshift from_rng/try_from_rng overrides. Each carries the actualoverrideorigin/span, dependencydefaultspan, rustdocimplID and providedmethodset. The activeoverride is retained; the shadoweddefault is not added a secondtime. The alltraits overlap audit additionally finds20serdegraph ToOwned::clone_into blanketoverrides (PCG4/Xoshiro16), giving71configuration records in shadowed-defaults-all71.json. Exactdefault/override stdsource is in toowned-default-source.json. PCG seed_from_u64 is a real inheriteddefault, not an overriddenmanual API. Actualstdderive exclusion also retains compilerassertion/default metadata as excluded catalogentries; no independent correctness is asserted for those helpers.

There are109default conditional generic entries:6oorandom,20PCG,78Xoshiro,5Xorshift. These include Into<U>/TryFrom<U>/TryInto<U> and generic sourceinitializers. A predicate such as U:From<Self> or R:Rng is not discharged for an unspecifiedU/R. These are conditionalgeneric APIs, not proved-unusable APIs; no concrete unsatisfiability is claimed without a concrete type and compiler check. The complete impl/method signatures and predicates remain. Concrete mockR cases are only the stated supplemental domains.

Rustdoc blanketparameterT may already be specialized to the concreteSelf even while a trait displays Borrow<T>/From<T>. coverage.json records actualblanket_impl and Self substitution, preserving the raw observedidentity and a separate self-substituted display. It does not erase generic restrictions, synthesize a production alias, or treat From<T> forT as an implementation accepting arbitraryT into everygenerator.

The original inventory lacked signatures for20 inheritedserde Deserialize::deserialize_in_place specializations (PCG4+Xoshiro15+Xorshift1). serde-default-source.json and the exact sourceexcerpt supplement the genericD:Deserializer<'de> declaration/defaultbody from lockedserde_core1.0.228, with sourcehash/lock records. All20 remain unproved; source supplementation is not compiler regeneration or callback-behavior verification. The original rawsignature hole remains visible instead of being silently replaced by invented rustdocdata.

## Count/classification corrections and limits

1. Keep Seed512 tupleconstruction as one public dataoperation, separate from functioncallablecounts. No machinekey changes.
2. The40 ToOwned::{to_owned,clone_into} entries newlyvisible in serde graphs (PCG8/Xoshiro32) reflect alloc traitgraph visibility, not an established localcfg(feature=serde) gate. The types already implementClone in default. graph-visibility-corrections.json retains exactIDs and rawtrait/source metadata. Earlier membership-basedcfg labels were too strong.
3. Threecrate38serde-derived APIs become fourcrate40;19inheritedDeserializedefaults become20. All remain targets. No derivednonstdserde item is excluded.
4. The function/catalog gap400 is not400unconditionallyavailable independent concrete implementations;109 are conditionalgeneric families. The downstream externaltrait universe is open. The current compiled-source snapshots cover default/no-default/serde onx86_64, not allfutureplatform/dependencyfeature combinations.
5. The107machinekeymatch is an identity/count reconciliation, not a freshrecord/serverreadback or a proof that editorial claims exceed their theorem domains. Model/compiler/observer boundaries remain exactly those in existing packets.

Rawinventory/compilerinput hashes are in input-hashes.json, exact rows in coverage.json, publicconstructor entries in public-data-constructors.json, and nonstdderivedpending40 in serde-derived-pending40.json. All original Rust, Cargo manifests, proof sources and existing publicationrecords are untouched.
