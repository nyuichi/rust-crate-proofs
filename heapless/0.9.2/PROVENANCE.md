# heapless 0.9.2 provenance and Deque verification scope

**Verification status: substantial Creusot public subset plus Verus-verified
generic `DequeInner<T, S>` push/pop orchestration (partial).**

This source tree is copied from the `heapless` 0.9.2 package published on
crates.io. The published archive has SHA-256 checksum
`2af2455f757db2b292a9b1768c4b70186d443bcb3b316252d6b540aec1cd89ed`.
Its `.cargo_vcs_info.json` records upstream revision
`75192be01a2487bdd0c8ab7adcd4031a44c700bc`.

The ordinary Rust build retains the upstream public API and field visibility.
Creusot translates only `heapless::Deque` and a verification-only projection of
its fixed-size storage; unrelated heapless collections are excluded from the
verification build. The overflow-safe rewrites of wrapped-length calculation
and logical-to-physical index calculation are used by both ordinary and
verification builds and preserve the upstream ring-buffer behavior.

## Established contracts

The logical view of a deque is its length. Its invariant states that capacity is
positive and representable as `usize`, both cursors are within capacity, and a
full deque has equal front and back cursors. The integrated proof establishes:

- exact fixed and storage capacity, exact `len`/`storage_len`, and equivalence of
  `is_empty` and `is_full` with the corresponding logical-length conditions;
- in-range wraparound for increment, decrement, and logical-to-physical index
  conversion without arithmetic overflow;
- preservation of the representation invariant by checked and unchecked
  `push_front`, `push_back`, `pop_front`, and `pop_back`;
- an exact length increase or decrease for successful pushes and pops, with no
  length change when a checked operation encounters a full or empty deque.

The bodies of all four unchecked push/pop primitives are proved by Creusot.
Their remaining Creusot trusted callees are the two slot-level helpers that move
a `T` into or out of a `MaybeUninit<T>` slot while preserving storage capacity.

With the `verus` feature, those helpers delegate their element move to two
Verus-aware leaf functions in the same `src/deque.rs` source file. Verus proves
that writing requires an uninitialized slot and leaves it initialized with the
exact input value; reading requires an initialized slot, returns its exact
value, and leaves the slot uninitialized. The proof now lifts those leaves to a
fixed-size array, with frame conditions stating that every non-selected slot is
unchanged.

The Verus model defines the occupied physical slots from `front`, logical
length, and capacity, and relates occupation exactly to each slot's
`MemContents::Init` state. Its logical `Seq<T>` reads initialized slots in
front-to-back order. Body-proved executable array callers cover all four fixed
array transitions. The proof also projects generic `VecStorage` through
`borrow_mut`, proves selected-slot and all-other-slot frame conditions, and
lifts them to the actual production `DequeInner<T, S>` fields. Both the four
unchecked public methods and their four checked wrappers are body-proved with
exact front/back sequence effects and `no_unwind`. Separate small lemmas prove
cursor/full-state preservation, wraparound, and checked space/nonempty
preconditions.

The ordinary read implementation now performs the same explicit transition by
swapping the slot with `MaybeUninit::uninit()` before `assume_init`, rather than
leaving the moved-from slot bytes in place through a raw-pointer `read`. This
preserves the deque's runtime element-move behavior while making the resulting
uninitialized state explicit.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| cursor arithmetic and length observers | yes | yes | no | yes |
| checked front/back push and pop | yes | yes | no | yes |
| unchecked front/back push and pop | yes | yes | no | yes |
| slot read/write through `MaybeUninit` | yes | no | yes | yes |

The table above records the existing Creusot proof. The additional Verus status
is:

| Verus component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| one-slot write transition | yes | yes | no | yes |
| one-slot read transition | yes | yes | no | yes |
| physical-index and occupied-set lemmas | yes | yes | no | yes |
| initialized-slot and logical-content relation | yes | spec/lemmas | no | yes |
| fixed-array read/write with frame conditions | yes | yes | no | yes |
| fixed-array front/back push transitions | yes | yes | no | yes |
| fixed-array front/back pop transitions | yes | yes | no | yes |
| generic `VecStorage` read/write with frame conditions | yes | yes | projection contract | yes |
| generic production cursor/storage transitions | yes | yes | no | yes |
| public unchecked `DequeInner<T, S>` push/pop | yes | yes | no | yes |
| public checked `DequeInner<T, S>` push/pop | yes | yes | no | yes |
| cursor/full-state transition lemmas | yes | yes | no | yes |
| checked push/pop precondition lemmas | yes | yes | no | yes |
| write/read roundtrip caller | yes | yes | no | yes |
| `Deque::new` | yes | no | assume specification | yes |
| `clear` normal-return state | yes | no | external body/drop glue | yes |
| `Drop` | partial | no | arbitrary `T::drop` | runtime-tested |

## Explicit trusted boundaries and exclusions

Creusot does not currently model the initialized subset and ownership moves of a
generic `MaybeUninit<T>` ring. Reading and writing one in-range slot therefore
remain trusted to Creusot, with contracts preserving storage capacity. Verus
proves these slot moves, their generic frame conditions, all four
value-preserving transitions, and the public checked and unchecked orchestration.

Verus still trusts the external `VecSealedStorage::borrow`/`borrow_mut`
specification that connects an abstract storage projection to the returned
slice. This contract states exact before/final sequence equality and is the only
trusted step between a generic storage object and the body-proved slot update.
Its removal condition is native Verus support/specifications for heapless's
sealed storage trait. A tiny `verus_slice_len` external body is also retained
only because the pinned vstd slice `len` contract omits `no_unwind`.

Construction and `clear` remain trusted to Creusot. A direct Verus proof of the
production constructor is currently blocked by Verus's unsupported non-`Copy`
const array-fill expression (`[const { MaybeUninit::uninit() }; N]`). It has a
reviewed Verus assume specification establishing empty cursors, all slots
uninitialized, both fixed and generic invariants, and `no_unwind`. The removal
condition is a reusable verified non-`Copy` array initializer or an upstream
vstd specification for the equivalent construction.

`clear` and `Drop` now remove each item from the proved deque state before
invoking its destructor. This is a runtime change from the former bulk slice
drop and preserves a valid remaining deque if `T::drop` unwinds; a regression
test checks that a panicking first destructor is not invoked twice. The
one-element drain transition is the body-proved public `pop_front`. The loop and
arbitrary Rust drop glue remain a Verus external/trusted boundary. `clear` has a
normal-return contract establishing the empty invariant, but deliberately no
`no_unwind`: unconditional panic freedom is impossible for arbitrary `T`.
`Drop` is likewise not claimed panic-free. Its next proof step needs a Verus
model for destructor effects and unwinding, or a separately stated
non-panicking-destructor precondition.

Array conversion now normalizes the full-ring `back` cursor to zero. The prior
`back = capacity` state violated the representation invariant and prevented a
cursor-based drain from terminating, even though ordinary element order often
appeared correct.

APIs whose essential result is element identity or initialized-memory exposure
remain trusted or untranslated: slice and reference access, `make_contiguous`,
`get`, swapping, truncation, retention, conversion from arrays, formatting,
cloning, equality, and iterator bodies. Iterator protocol models deliberately
claim no element-order correspondence. `DequeView` and the dynamically sized
storage path are excluded from Creusot translation. Other heapless collections
and optional adapters are outside this target's verification scope.

Run `./verify-all.bash` in this directory to reproduce the default-feature
Creusot Deque proof and the locked `verus`-feature proof. The current Creusot
integrated result is `Proved (57 files)`; the Verus run includes the slot and
fixed-array layers, generic storage framing, actual `DequeInner<T, S>`
transitions, and all eight public push/pop methods, with the current result
`54 verified, 0 errors`. Verus is pinned to
`0.2026.07.27.31579f0`, with `vstd` pinned to commit
`31579f0b8542a8a9ae4ae5604c16107ccde23ef2` in `Cargo.toml` and `Cargo.lock`.
The ordinary and `verus`-feature Deque unit-test modules each pass all 35 tests.
Generated Why3, Verus,
and Cargo build artifacts are intentionally not tracked.
