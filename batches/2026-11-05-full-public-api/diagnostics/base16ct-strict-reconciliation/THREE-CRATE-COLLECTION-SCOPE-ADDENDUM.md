# Superseding collection-scope clarification for strict3crate counts

The existing strict-api-reconciliation preserves its original ledgers; its
189(nohashstd)/50(nostd)/80(normalize)/128(endian) are conditional applicable
signature counts within those ledgers after alias/bound/derive corrections.
They must not be read as an independently re-collected exhaustive catalogue of
all hidden and nightly-standard-library APIs.

Evidence: the base16 inventory marks document_hidden_items=true. The preserved
three-crate per-configuration provenance rustdoc commands have no
--document-hidden-items option. Both inputs use private=false, but hidden and
private are different filters. This is a collection-option difference, not
source/body verification failure. Full hidden catalogue completeness is not
established by this reconciliation; no further collection/build was run here.

Suggested append-only note to the older strict README/Git handoff:

> Counts189/50/80/128 reconcile the preserved input ledgers. These inputs were
> collected without explicit hidden-item documentation and are not proven to
> exhaust the hidden/nightly catalogue represented by later document-hidden-items
> inventories. Keep ordinary/gated/conditional distinctions and this collection
> scope when comparing counts across crates. Existing14/66/1 mapped source
> bindings and their theorem domains are unchanged.

Do not revise proof counts, delete claims, or infer missing/verified APIs from
the collection-filter difference. A later complete catalogue would be a new
separately pinned source snapshot, not a silent replacement of these raw inputs.
