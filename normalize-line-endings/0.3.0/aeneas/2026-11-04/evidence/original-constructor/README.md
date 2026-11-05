# Importable original constructor supplement

Copy only `proofs/OriginalConstructorGenerated/NormalizeLineEndings.lean` and
`proofs/OriginalConstructorSupplement.lean` into the corresponding existing
proof project, declare the generated library as required by its Lake layout,
and import `OriginalConstructorSupplement` from the main fresh-built Proofs.

The first file is wholly official Aeneas output using `-namespace
OriginalConstructorGenerated -subdir OriginalConstructorGenerated`. Neither
its filename nor its contents are rewritten. This preserves a distinct module
path and namespace alongside the existing adapted `Generated.NormalizeLineEndings`
(namespace `normalize_line_endings`). `OriginalConstructorSupplement.initializes`
proves that original `normalized` returns the same arbitrary iterator and sets
its previous-CR flag to false, for every official Iterator dictionary.

The unchanged original source/manifest are archive-checked. Official Charon
`--start-from crate::normalized` reaches only this original constructor. This
is not a proof of original next or arbitrary-length original iterator behavior.
The existing adapted mapped claim remains separate; adding this supplement
must not change the machine-claim count or reinterpret its implementation.

`./verify.bash` reproduces source-only extraction, official namespace/subdir
output, exact generated-byte comparison, and Lean theorem/audit. Existing
`normalize-original` is not modified by this packet. Its original finite0..4
Kani437 result remains a separate bounded guarantee.

`evidence/Compatibility.lean` was checked in an environment whose LEAN_PATH
additionally contained the existing adapted project's compiled library directory.
It imports both existing `Proofs` and `OriginalConstructorSupplement`, then
checks both separately named constructor definitions and the new theorem.
`compatibility.log` records success; this is an import-compatibility check,
while the publisher will still fresh-build the actual integrated project.
