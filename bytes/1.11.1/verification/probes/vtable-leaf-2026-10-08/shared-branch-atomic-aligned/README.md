# Shared-branch atomic and allocation-interface diagnostic

This source-sliced experiment tests whether the actual `Bytes::from(Vec<u8>)`
shared-branch allocation and record construction can establish three narrow
postconditions when it uses existing generic permission infrastructure:

- the returned `Bytes.ptr` and `.len` equal the branch inputs;
- the returned `Box<Perm<*const Shared>>` points at the allocated `Shared`;
- its value has the input `buf` and `cap` fields.

The body retains the current production `Shared` fields, the allocation field
values, the low-bit alignment assertion, and the `Bytes` record fields. It
routes `Box<Shared>` through the existing
`src/ownership_proof/boxed_alignment.rs::into_raw_aligned` helper. The probe
returns the actual `Bytes` value together with the raw `Shared` pointer and
the helper's ghost permission so the narrow field postconditions can be
checked. This tuple is an interface experiment, not a replacement executable
constructor.

The proof cfg uses two local trusted atomic constructor bridges with only
`requires(true)` / `ensures(true)` and divergent bodies. Native cfg calls the
exact `AtomicUsize::new(value)` and `AtomicPtr::new(value)` constructors. No
atomic value, initialization ordering, pointer-content, refcount, or ownership
fact is stated by those bridges. The actual generic aligned Box helper carries
its existing contracts for pointer alignment, permission ward, and pointee
value. The address-only `pointer_addr` helper is included from production and
the retained debug assertion is not weakened. The vtable proof abstraction
remains a trusted true/true getter; native cfg retains the exact static table
and its stub callbacks.

`check_source_fragments.py` verifies the copied layouts, native table,
authorized source substitutions, and included production helpers against the
current source. `sources/` preserves this probe's source, exact production
source snapshots, and the prior direct-pointer-cast negative. Logs record the
translation, Why3 attempt, native check, and source hashes.

## Results

The source guard passes, COMA translation exits 0, Why3 proves all generated
VCs (`Proved (7 files)`), and native `cargo check --locked` exits 0. The
proved postconditions establish `Bytes.ptr == ptr`, `Bytes.len == len`, the
returned Box permission's ward equals the returned raw `Shared` pointer, and
the permission value carries the input `buf` and `cap` fields.

The initial proof attempt without the low-bit bridge proved 3/4 split goals.
The exported failed task shows the remaining branch came from the retained
alignment `debug_assert`: it had the `is_aligned_logic()` fact from the Box
helper but could not derive the numerical low bit from the address mask. The
final body calls the existing `boxed_alignment::aligned_address_has_clear_low_bit`
lemma using `pointer_addr(shared)` and `align_of::<Shared>()`, then keeps the
same assertion unchanged. This helper is body-proved in production; the probe
adds no trusted alignment, pointer, permission, or atomic fact.

The atomic constructors remain true/true trusted bridges, so this proves no
atomic value, ordering, refcount, or `Bytes.data` to `Shared` pointer link.
COMA still contains `Any.any_l()` locals, while the concrete field relations
above are established by the generated VCs. The pre-lemma failed task,
successful proof record, and both solver caches are preserved in `artifacts/`.

This experiment omits Vec extraction, `ManuallyDrop`, the `len == cap` arm,
`Shared::drop`, callback behavior, the native getter-to-proof-getter
correspondence, and any relation between `Bytes.data` and the returned shared
pointer. It does not prove `Bytes::from(Vec<u8>)` or a Bytes API theorem.
