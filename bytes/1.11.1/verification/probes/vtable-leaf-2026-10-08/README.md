# Vtable leaf materialization diagnostic

This probe records bounded frontend and proof boundaries for the shared-vtable
field in `Bytes::from(Vec<u8>)`. All runs use the pinned activated toolchain
and shared `/tmp/itoa-creusot-proof.lock`. The original comparison and first
two follow-ups were translation-only. The newest actual shared-branch variant
in `shared-branch-atomic-aligned/` also ran a limited Why3 attempt.

The first source-sliced comparison preserved the production `Bytes` fields,
the exact `Vtable` field layout, and the native static table/callback items.
`BytesMut` and callback bodies were stand-ins. Both direct `&SHARED_VTABLE` and
a local true/true trusted accessor failed before the record body because
Creusot rejected the `SHARED_VTABLE` static definition itself. This is a
frontend error, not a failed VC; see the complete baseline/helper logs and
exit-code files.

The follow-up `extern-getter/` excludes the actual static and callbacks from
proof cfg. Its trusted true/true getter supplies an arbitrary `&'static
Vtable`; native cfg retains the table and getter returning its address. The
record translation succeeds and emits all five Vtable slots as `Opaque.ptr`,
but the generated `Bytes` result remains `Any.any_l()`. The emitted translation
does not show a field relation or prove identity with the native table.

The follow-up `shared-branch/` retains the source-sliced current allocation and
record statements, exact `Shared` field shape, full Vtable layout and native
table initializer. A source checker hashes and compares the original branch;
the probe substitutes only the existing address-only `pointer_addr` helper in
the retained alignment assertion and the true/true table getter in the vtable
field. The direct `shared as usize` baseline fails with
`PointerExposeProvenance`; using the generic helper gets through COMA
translation. The generated function includes a `result.ptr`/`result.len`
postcondition, but the returned value and Box/atomic intermediates are still
`Any.any_l()`. Creusot warns that `AtomicUsize::new(1)` and `AtomicPtr::new`
have no contracts and would give impossible caller preconditions. This case
therefore identifies a translation shape and its limits, not a usable field
model or a Bytes proof.

`extern-getter/README.md` and `shared-branch/README.md` state the cfg
correspondence and exact exclusions for each follow-up. All native checks use
stub callbacks; native compilation is not runtime-equivalence evidence. The
probe did not edit production files. No commit was created.

The `shared-branch-atomic-aligned/` follow-up keeps the source-sliced shared
branch and uses true/true atomic constructor bridges with exact native
constructors, plus the existing generic aligned Box permission helper. It
returns the actual Bytes record and Ghost Box permission for field-level
checks. An initial Why3 run left the retained alignment assertion's panic path
unproved: the generic Box helper supplied logical alignment, but that fact had
not been connected to the numerical low bit. Calling the existing
body-proved `aligned_address_has_clear_low_bit` helper resolves that VC while
leaving the assertion unchanged. The final translation, Why3 run, and native
compile all pass. Bytes pointer/length, Box permission identity, and Shared
buffer/capacity postconditions are proved. Atomic constructors remain true/true
trusted boundaries, so no atomic value or `Bytes.data` link is claimed. The
exact initial task and final proof record are saved in its `artifacts/`.
