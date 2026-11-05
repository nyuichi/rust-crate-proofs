# Additional feature coverage and default-model composition

The fourteen published mappings retain their API identities and now conjoin exact std/no-std debug/release contracts. Official targeted extraction uses the unchanged crate, with no-default-features for the two new translations. The official Aeneas namespace option keeps separate model definitions. After namespace normalization, each new generated body is byte-identical to its std counterpart; each new body is nevertheless imported and proved in separate Lean modules. No source, IR or verifier changes are used.

SupplementalProofs.clone_from_spec proves exact replacement of destination by source by composing the actual generated clone method with official Std.Clone.clone_from.default. This is a model-based composition, not an independent body proof of the Rust standard-library default; it is retained as supplementary evidence without a fifteenth public API mapping. It adds no local axioms and participates in the fresh default Lake build.

Original no-std unit tests pass: 11 debug and 1 release. The separate full no-std Cargo test logs retain one failed doctest: the IsEnabled example refers to IntMap, intentionally absent without std. The other two no-std doctests pass. This existing documentation feature-gating mismatch is not hidden or treated as a verified method assertion failure. verify-all.bash runs the no-std unit target explicitly and the complete std unit/doc targets.

Manual Debug formatting, four inherited Hasher defaults and collection/type-alias dependency bodies remain unproved. The formatting model currently ignores actual text output, so model-normal termination does not establish the Debug formatting contract.
