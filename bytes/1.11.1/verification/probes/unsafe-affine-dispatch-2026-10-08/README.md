# Unsafe indirect dispatch with checked affine forwarding

This isolated probe addresses unsafe function-pointer invocation and reification.
It does not implement `Bytes::clone`, vtable selection, a resource-transforming
callback, or automatic Drop. Production source and installed Std are unchanged.

The private `declare_registered!` macro accepts the native body once and emits:

1. An unsafe native function with a checked pre/post contract.
2. A safe shim with the same contract and a fixed body calling that exact item.
3. A trusted registration getter returning the native pointer and a ghost-only
   safe function pointer representing the checked shim's contract.

Both native functions (identity and successor) and both shims are body proved.
Only the getter and generic `invoke` assume dispatch correspondence. The opaque
`registered(native, spec)` predicate means invocation of the native target is
safe under the spec's precondition and satisfies its normal-return contract.
There is no termination claim. Registration ties the safe pointer's FnExt
pre/post to the checked shim, not to an arbitrary user-supplied specification.
The native unsafe call occurs in `invoke`; its Ghost spec is never executed.

The relation is semantic realization, not code-address identity. It requires
neither uniqueness of function addresses nor an injective code-pointer model.
No arbitrary pointer/spec registration factory is exposed. The macro is private
and uses the same target identifier in the actual getter and shim. Correctness
of this generation/reification rule is explicit generic TCB. Editing a trusted
getter's native body is a TCB change requiring audit; the negative tests do not
claim to detect arbitrary corruption inside trusted code.

`invoke_forward` and `affine_identity` are ordinary body-checked functions.
They move the incoming `Ghost<Resource<Option<Excl<()>>>>` unchanged and return
that exact value. The native callback does not consume, mutate or create the
resource. Thus this is not yet erasure-aware transport of a callback's own ghost
input/output, and cannot supply C's missing public-Clone quota.

The Std analogue is `std::ops::FnExt`/`FnOnceExt` with pre/post contracts.
Unsafe pointers do not implement those traits; the safe shim supplies the
contract vocabulary. The getter bypasses unsupported FnDef-to-FnPtr reification
inside an explicit trusted definition. A future generic unsafe-function contract
and verified reification primitive can replace these two boundaries without
changing the checked forwarding clients.

## Validation

Default: `./run-proof.sh` — six generated files, complete engine success, zero
null leaves. Native: `cargo test --locked` — one test, identity/successor on four
inputs each (including u32::MAX - 1).

Wrong certificate: `./run-proof.sh --features negative_wrong_certificate` —
seven files, exactly one null in `vc_wrong_certificate`, 2/3 subgoals proved.
The exact null is `registered(identity_native_pointer, successor_spec_pointer)`;
the spec precondition and final result relation separately close. It is a call
admission failure, not a type error or an unrelated final equality.

Duplication: `./run-proof.sh --features negative_duplicate_resource` — frontend
E0382 on using the same non-Copy Ghost<Resource> twice; zero Coma/VCs generated.
The final default gate is restored and passes after both controls.

`evidence/` preserves exact source, Coma/JSON, logs, selected pinned Std analogues,
configuration and SHA manifests. Early authoring failures (native import,
non-erased Ghost::new, macro result hygiene, CLI argument placement) are not
semantic negatives. No timeout increase or solver auto-detection was used.
