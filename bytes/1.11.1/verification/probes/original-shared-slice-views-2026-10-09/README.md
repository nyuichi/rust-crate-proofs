# AQ actual nested public Range slicing

This development probe extends the audited AP positive through exactly one
proof-only enum declaration replacement: Root and Child payloads are unchanged,
View adds a real child ticket plus bounded offset metadata, and Empty has only
unbound provenance-free metadata and a readonly null-field binding. Published
ancestors and production source remain unchanged. All inherited targets are
reproved; the transformed prefix is explicitly pinned, not called byte-exact.

The actual native witness constructs from a nonempty Box, promotes a child,
drops the original, slices the surviving owner with a built-in Range, slices
that result again, then drops first and owner before reading selected. It saves
the Vec return before selected Drop. For valid variable ranges a<=b<=input.len
and c<=d<=b-a, the formal result is exactly input[a+c..a+d]. Both empty and
nonempty endpoints are included, including one-past empty. Native smoke
execution covers 100 cases; tests are corroboration, not proof.

SharedCore keeps original complete allocation base, capacity, initialized
content, physical/control authority and lifetime ticket. Offset/length views
never replace recovery metadata. The actual Shared-view clone uses the native
Relaxed load and guarded increment. Empty takes actual wrapping_add followed
by new_empty_with_ptr/without_provenance, issues no ticket and uses Static Drop.
The allocation is reclaimed exactly once by owner Drop when selected is empty,
or by selected Drop when it is nonempty; the empty result can then be read
without allocation liveness. Both payload/control recovery receipts stay strong.

Initial Rust/Creusot syntax and logical/program pointer classification failures
are archived separately. A Why3 reserved-keyword collision is resolved by the
explicit proof-local alpha rename begin -> view_begin, preserving native calls,
assertions and messages. The first semantic run leaves three slice_view goals
(139/1261/3). Exposing the pure shifted definition, exporting exact generic
pointer address facts and proving Clone's precise View return variant closes
those goals without weakening content/core or assuming Resolve. The next
diagnostic run proves 139/1202/0 with zero structural leaves; its correspondence
receipt is deliberately not_run and is not the final publication gate.

Native correspondence checks 25 MIR bodies (24 production plus client), exact
Range branch topology, pointer operations, static/shared dispatch and all four
normal Drop places. All 47 native mutations reject in agent and root replay.
The main checker joins the transformed source, copied/imported modules, exact
public source extraction, Cargo routes and four actual compiled artifacts.
Extracted native_view_bindings.rs is a review artifact, not a compiled artifact.
Astra found and fixed a fresh-hash detached-OUT_DIR capture gap; see evidence.

Twelve semantic defects leave 1/3/2/2/1/2/1/1/1/2/2/2 nulls, with 20 exact
durable task sidecars. They distinguish wrong offset/length/base reads, missing
registration, view capacity substitution, phantom empty registration, omitted
first/owner/selected effects, final Acquire and two frees. Generic feature
defects span retained callback paths and are counted once per feature. Free
controls conjure receipts; their failures are proof sensitivity evidence, not
direct native deallocation-absence theorems.

Three frontend controls reject duplicate selected use with E0382, retirement
across a live borrowed view with E0502/E0505, and Snapshot-to-Ghost owner
extraction with E0277 because actual Bytes does not satisfy Plain. Captures
exclude proof outputs and solver caches.

The restored full main correspondence passes and structural controls reject
76/76 after a full actual-input baseline audit. Two stale extractor/control
dispatch integration issues are corrected in AQ only. The canonical all-target
proof passes 139 targets / 1202 prover leaves / zero null or structural leaves,
with correspondence zero and no features, exclusions, diagnostic or source
controls. Immutable independent final audit passes; see evidence/AQ_CANONICAL_AUDIT.md. See TCB.md for the exact generic assumptions and replacement
paths. The selected theorem concerns valid built-in Range normal returns; other
RangeBounds implementations, panic/unwind, concurrency, escaping topology,
other representations/APIs/configurations and complete original architecture
remain open. Full original architecture is NOT ADMITTED.

Canonical archive SHA256 41e8e9c164a1110c7f611bb1726f490e111c6b10a77af6b57bfc707ce49ceba5, 1178 members.
See evidence/aq-positive-canonical-v1.json for exact target/member receipts.

Final audit replays all main76/native47 controls and all four compiled artifacts.
Absolute external OUT_DIR and toolchain paths remain location-bound; no archive
bytes were rewritten and no portable self-contained build is claimed.
