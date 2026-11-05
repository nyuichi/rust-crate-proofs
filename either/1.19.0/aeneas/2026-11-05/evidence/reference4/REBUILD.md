# Supplemental regeneration and integration

The producer manifest remains an immutable inventory, including the excluded source/target cache paths. integration.json enumerates the omitted cache files. No compiled target cache is part of this public source subset.

From this directory, regenerate the original reference4 LLBC with the pinned official Charon command in regenerate.py. Generate a separate namespace using:

```sh
aeneas -backend lean -emit-json -filter-trait-methods -abort-on-error -namespace ReferenceSupplementGenerated -dest output either.llbc
```

The output Either.lean is installed as proofs/ReferenceSupplement/Either.lean without generated edits. ReferenceSupplementProofs changes only the import/opened namespace relative to the producer six theorem bodies and appends six axiom audits. The root Lake package has distinct default library targets for the old40 package and supplemental six-theorem package. Their generated models cannot be imported simultaneously because an automatically named discriminant instance is globally duplicated, even with the official namespace option. This is preserved as a failed integration attempt, without modifying generated output or introducing a custom translator. Both libraries are independently compiled by the formal Lake build; no simultaneous model identity bridge is claimed. Local dependency links are caches excluded from publication.
