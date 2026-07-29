# heapless 0.9.2 provenance and Deque verification scope

**Verification status: substantial Creusot public subset plus a proved Verus
slot-transition leaf (partial).**

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
value, and leaves the slot uninitialized. A representative roundtrip caller is
also body-proved and establishes that writing and then reading returns the
original value. These three Verus bodies are proved without `assume` or
`external_body`.

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

The table above records the existing Creusot proof. The additional Verus leaf
status is:

| Verus component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| one-slot write transition | yes | yes | no | yes |
| one-slot read transition | yes | yes | no | yes |
| write/read roundtrip caller | yes | yes | no | yes |
| generic storage indexing and occupancy-to-initialization relation | no | no | absent | no |

## Explicit trusted boundaries and exclusions

Creusot does not currently model the initialized subset and ownership moves of a
generic `MaybeUninit<T>` ring. Reading and writing one in-range slot therefore
remain trusted to Creusot, with contracts preserving storage capacity. Verus
proves the isolated element-move bodies, but does not yet prove that
`VecStorage::borrow_mut` selects the intended physical slot or that the deque's
front/back/full state makes that slot initialized before every read and
uninitialized before every write. Connecting those facts is the next removal
condition for the Creusot trusted boundary. Construction and `clear`
are trusted because they respectively create uninitialized storage and drop the
initialized elements; their contracts fix the resulting empty state and
invariant.

APIs whose essential result is element identity or initialized-memory exposure
remain trusted or untranslated: slice and reference access, `make_contiguous`,
`get`, swapping, truncation, retention, conversion from arrays, formatting,
cloning, equality, and iterator bodies. Iterator protocol models deliberately
claim no element-order correspondence. `DequeView` and the dynamically sized
storage path are excluded from Creusot translation. Other heapless collections
and optional adapters are outside this target's verification scope.

Run `./verify-all.bash` in this directory to reproduce the default-feature
Creusot Deque proof and the locked `verus`-feature proof. The current Creusot
integrated result is `Proved (57 files)`; the Verus run proves the two slot
transitions and their representative roundtrip caller. Verus is pinned to
`0.2026.07.27.31579f0`, with `vstd` pinned to commit
`31579f0b8542a8a9ae4ae5604c16107ccde23ef2` in `Cargo.toml` and `Cargo.lock`.
The ordinary Deque unit-test module passes all 34 tests. Generated Why3, Verus,
and Cargo build artifacts are intentionally not tracked.
