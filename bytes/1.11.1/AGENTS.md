# bytes 1.11.1

Only this crate is in scope. Preserve the original API, runtime representation,
actual implementation verification and Vec reverse-comparison fix. Small source
changes needed for verification are allowed. Commit/push validated increments to
origin bytes-runtime-verification; never main or force-push.

Read README.md. Work on default std, x86_64 and native atomic orderings; sc-drf
stays disabled. Use verify-all.bash (one prover, shared lock, elevated Why3).
Use --check for unchanged-source evidence maintenance; it runs no solver.

Retain actual contracts, proofs, necessary lemmas and explicit TCB assumptions.
Bytes-specific ownership/refcount/last-owner/destructor laws need body proofs.
Reviewed generic physical/library/tool boundaries may be trusted with strong
contracts, native interpretation and a replacement path. Consult pinned Std and
upstream examples when blocked. A separate model is not implementation coverage.

Current source correspondence reuses a reviewed relation for exact source bytes.
Do not update expected hashes alone after changing implementation or proof inputs.
Do not claim complete API coverage from this selected sequential normal-return
witness. Important rejected reasoning remains in verification/ARCHITECTURE_DECISIONS.md.

Past failed experiments and audit chains belong in Git history, not the normal
verification path. No mandatory negative suite, ancestor audit, multi-document
status update, model routing or separate Astra consultation. For a new trusted
boundary/checker or suspected gap, use one relevant diagnostic when needed.
Prove the affected positive target; rerun only after relevant changes or failure.
