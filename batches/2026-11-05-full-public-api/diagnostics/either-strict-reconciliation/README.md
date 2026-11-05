# Either 1.19.0: strict public API reconciliation

This is an independent read-only inventory reconciliation, not completion of every API. All 2,096 original rows across four compiler-observed configurations are retained. The 40 existing default/x86_64 model-contract mappings each resolve to one original row; their source, theorem, precondition, public report and machine-claim count are unchanged.

| Configuration | Original rows | Shadowed default rows | Constructor operations / public alias rows |
|---|---:|---:|---:|
| default (std) | 556 | 50 | 2 / 4 |
| all (std + serde) | 566 | 50 | 2 / 4 |
| no default | 476 | 38 | 2 / 4 |
| serde without std | 498 | 40 | 2 / 4 |

Counts describe observed identities, declarations and conditional generic candidates. They are not counts of unconditional concrete functions available for arbitrary type arguments. The source feature matrix distinguishes std and serde; use_std is a manifest alias for std and has no independent source cfg gate. These observations do not prove other-target behavior or extend the default-only proof domain to the other configurations.

## Overrides and defaults

On this compiler, provided_trait_methods includes names explicitly overridden by an impl. Exact full Rust identity matching finds 50 shadow pairs in default/all, including the previously identified Iterator/ExactSizeIterator subset of 28. The remaining pairs include DoubleEndedIterator, IO, Error, ToOwned and other traits. The explicit method body remains an obligation or existing proof; the shadowed trait-default declaration remains evidence rather than a second callable for that concrete dispatch identity. No row is erased and no unrelated specialization is merged because a server key omits generic arguments. shadowed-defaults.json records both original IDs, origins, source spans and complete runtime identity candidates. Only actual generated standard-trait bodies receive the user's derive exception; nonstandard serde derives are never excluded, and inherited origin is never relabeled as generated code.

## Scoped binders and applicability

rows.json contains diagnostic scoped binding IR for each callable/default candidate. Owner, implementation, trait, method and nested higher-ranked binders are distinguished. The source blanket parameter is replaced once with the complete applied owner; replacement does not recurse into an owner's coincidentally named R/T parameter. Self is the real owner, including foreign core::result::Result<R,L> in the reverse From implementation, rather than the local type mentioned in a trait argument. Trait arguments and method parameters are kept in separate scopes and retain their originating document context.

The IR is not emitted Rust and is not a trait-solver certificate. It preserves impl and method predicates, lifetime requirements, implicit/default Sized obligations and associated projections. Unknown applicability is conditional, never silently converted to not-callable or counted as universal availability. binder-examples.json illustrates From<Self> specialization, residual Into<U> with U:From<Self>, Any's 'static requirement, the foreign Result self and serde's trait lifetime versus method D. For example, a borrowed payload with a nonstatic lifetime cannot be assumed to satisfy Any's 'static premise; Read/Iterator/Clone/serde predicates on arbitrary L/R likewise remain requirements. HRTB parameters are scoped separately. Associated Item/Error projections remain symbolic pending their stated constraints.

The original labels may retain a source blanket T in a trait argument. The instantiated argument/substitution IR, not that uninstantiated label or an erased server key, determines what it means. No unresolved signature, projection, language-feature requirement or compiler-observed conditional blanket is discarded.

## Constructors and serde remain obligations

Left and Right are two actual tuple-constructor operations, exposed through four variant rows: enum-qualified and root reexports. constructors.json preserves all aliases and intended tag/payload contracts; counts.json records the two original rustdoc variant items. Generated datatype use in existing proofs does not by itself create an explicit standalone constructor source/machine binding. Constructor behavior remains an operation obligation, with no physical layout/ABI claim, rather than being excluded as merely structural.

Both serde configurations retain seven unproved operations: two nonstandard derived Serialize/Deserialize bodies, four handwritten serde_untagged/serde_untagged_optional wrappers, and the active deserialize_in_place default. Their two implementation relationships also remain. serde-obligations.json includes all rows. In particular, generic L/R serde bounds, the 'de lifetime and method Deserializer D are not automatically fulfilled by the existence of a derive declaration.

The default classification separately records the 40 model contracts, four nominal views specialized from the already proved actual IntoEither Sized blanket, and two conditional original-default body helpers. These are not six additional machine claims. There are still 132 unproved callable rows and 183 other active default candidates in the default inventory, plus constructor and structural/implementation obligations; these groups retain generic applicability conditions and are not a single concrete-API denominator. Other configurations do not inherit the default proof merely because a source name matches.

## Reproduction and remaining scope

Run `python3 reconcile.py`. It reads the original inventories/rustdoc and frozen Either40 metadata, performs no compiler build or server operation, and writes the separate reconciliation files. inputs.json identifies the exact hashed source documents. proof40-bindings.json is the one-to-one mapping check. remaining-exact-scope.json lists every unfinished row with its configuration, origin, signature/binder context and conditional obligations. No common inventory, publication, source, model or proof was edited. The earlier checkpoint is historical and superseded by FINAL.json. This packet can be published as Git evidence without a proof/report revision.
