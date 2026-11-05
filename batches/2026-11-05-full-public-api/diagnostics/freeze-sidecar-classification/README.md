# Append-only sidecar metadata correction and future freeze check

The immutable write_str103 manifest said compiled_sidecars=0 but included16 Lean
compiler sidecars: each generated module's .ir, .ir.sig, .olean.private and
.olean.server. Its new4/whole17 source proofs and audits are unaffected. Exact
paths/hashes and raw manifest/tree fingerprints are in WRITE-STR103-ERRATUM.json.
No raw file or old manifest was modified or deleted. The analogous generic122
raw correction is retained separately. Publisher owns any source-only subset.

The old exact-suffix check (.olean/.ilean/.c) was incomplete. classify.py is a
separate minimal helper for FUTURE Lean source-only freeze lists: it recognizes
.olean* and .ir* families, .ilean and .o, plus target/.lake/__pycache__ cache paths.
It reports exclusion candidates; it never deletes or changes a file. Native C
source, LLVM .ll, MIR .mir and Charon .llbc evidence stay present. This is not a
universal semantic file classifier: an intentionally authoritative input with a
matching compiler-sidecar name must be reviewed and archived separately instead
of silently relabeled a Lean cache. Original source and mathematical input
pinsets must remain independently checked.

SELF-CHECK.json records19 positive/negative filename/path cases, exact16 raw
members and87 retained source/evidence files, and reconfirms all103 raw hashes.
No build, solver, API, mathematical proof or published metadata was changed.
The helper does not retroactively repair raw manifests; a future freeze must
assert that its final source-only member set contains zero classified sidecars,
record the actual count, and preserve any raw evidence manifest independently.
