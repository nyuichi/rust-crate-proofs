# Actual shared-branch construction translation diagnostic

This follow-up keeps the original shared-branch construction statements from
`Bytes::from(Vec<u8>)` and substitutes only two already-approved interface
expressions: the retained alignment assertion obtains the numerical address via
the existing `crate::provenance_specs::pointer_addr`, and the vtable field calls
the true/true trusted getter. The assertion itself is unchanged. The production
helper is included by `#[path]`; its current source is snapshotted and checked.
In native cfg it uses the existing `as usize` branch; under Creusot it uses
`ptr.addr()` with the helper's numeric-address postcondition, which grants no
permission or provenance authority.

The probe copies the current `Bytes` field layout, exact `Vtable` declaration,
exact `Shared` field layout, `KIND_MASK`, and native table initializer. The
callback bodies are divergent stubs, and the production `Shared::drop` body is
omitted. The branch function accepts `ptr`, `len`, and `cap` as inputs to keep
the diagnostic inside the allocation/record branch; it omits Vec extraction,
`ManuallyDrop`, and the `len == cap` arm. It is not the full `From<Vec<u8>>`
method.

`check_source_fragments.py` verifies the probe layouts against current
production files and checks the original shared branch against the probe after
exactly the address-helper and vtable-getter substitutions. It also checks the
native table initializer and source snapshots. The source-fragment hashes are
captured in `logs/source-fragment-check.log` and `manifest.json`.

## Results

The direct-cast baseline is preserved in
`sources/lib-pointer-expose-cast.rs` and
`logs/proof-translation-pointer-expose-rejection.log`. With the original
`shared as usize` alignment assertion, Creusot rejected
`PointerExposeProvenance`; it also warned that `AtomicUsize::new(1)` has no
contract and would create an impossible caller precondition.

With the retained assertion using the generic `pointer_addr` helper,
translation-only Creusot exits 0 and emits
`artifacts/proof-pointer-addr-verif/bytes_vtable_shared_branch_probe_rlib/from_vec_shared_branch.coma`.
Native `cargo check --locked` exits 0. The generated `Vtable` type has all five
callback slots (`clone`, `into_vec`, `into_mut`, `is_unique`, `drop`) as
`Opaque.ptr`; `Shared` has the exact `buf`, `cap`, and `ref_cnt` fields.

The generated function now carries the requested postcondition
`result.ptr == ptr && result.len == len`. Its body still initializes the return
as `Any.any_l()` and the Box/atomic intermediate values as `Any.any_l()`. Thus
COMA emits the layout and contract, but does not retain a usable field relation
from this constructor body. The postcondition was not proved. Translation also
warns that both `AtomicUsize::new(1)` and `AtomicPtr::new(shared as _)` lack
contracts and would impose impossible preconditions on a caller. No VC or Why3
phase ran, so this experiment establishes frontend translation only; it does
not verify atomic initialization, Box authority/ownership, refcount behavior,
the static table, callback behavior, or any Bytes API theorem.

The proof cfg excludes the actual static table and uses a trusted local getter
with only `requires(true)` and `ensures(true)`, whose body is `unimplemented!()`.
Native cfg retains the static initializer and getter returning its address. The
proof abstraction states neither table identity nor callback values, and there
is no proof relating the native getter to the trusted abstraction. Native check
uses the probe's stub callbacks and is compile evidence only.
