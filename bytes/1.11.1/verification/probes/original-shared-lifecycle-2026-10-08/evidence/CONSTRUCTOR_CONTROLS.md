# Original Shared constructor misuse controls

These isolated clones test the exact committed constructor at 3ba052a6, without
changing the evolving positive source. Each mutation and full tested source,
Coma/proof tree, configuration and log are preserved in its archive.

- constructor-negative-wrong-refcnt-field.tar.gz constructs two different
  actual atomics, both initialized to 1 through field_event::new with explicit
  histories. Shared.ref_cnt uses the second, while the result retains the first
  permission. The selected constructor VC proves 16/17 leaves and rejects one;
  all other proof files pass (34 Coma/34 JSON, one null leaf total).
- constructor-negative-wrong-buf-pointer.tar.gz sets Shared.buf to null while
  retaining the original Vec pointer and capabilities. The selected constructor
  VC proves 15/16 leaves and rejects one; all other files pass (34/34, one null).
- constructor-negative-contractless-wrong-refcnt-field-diagnostic.tar.gz is only
  an authoring diagnostic. A direct core AtomicUsize::new without a Creusot
  contract cannot establish the required external-call precondition. That failure
  is not evidence that the semantic wrong-field case is rejected; the revised
  contracted case above supplies that evidence.

Root independently checked every declared member hash (414, 449 and 346
respectively), inspected the exact mutations, and counted actual JSON null nodes,
not occurrences of the word null in VC names. Archive hashes and counts are in
root-constructor-negative-audit.json. These controls establish rejection of the
selected constructor's field/payload mismatch; they do not prove Clone, final
release, native Drop or the public crate.
