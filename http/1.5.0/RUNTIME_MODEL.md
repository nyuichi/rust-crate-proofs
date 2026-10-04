# Runtime/model map for `http` 1.5.0

This is a proof design map for `HeaderMap`, `Extensions`, `Request`, `Response`,
conversion helpers, and `Error`. It records the representation facts a body
proof would need; it does not assert any of them as established contracts. The
public API inventory is in [`API_INVENTORY.json`](API_INVENTORY.json). The
capacity arithmetic leaves are the only bodies proved in these areas; see
[`verification/map-capacity/README.md`](verification/map-capacity/README.md).

## `HeaderMap<T>`

The intended abstract value is a finite mapping from normalized
`HeaderName` keys to non-empty sequences of values. Values for each key are
ordered by insertion and append order. The order of distinct keys in map-wide
iterators is unspecified. Each `entries` bucket stores one key and the first
value; additional values occupy `extra_values`.

The implementation-to-model relation must establish:

- Every primary `Bucket<T>` represents one distinct normalized key. Its
  `value` is the first value in that key's abstract sequence.
- A bucket with `links: None` has no extra values. A bucket with links starts a
  doubly linked chain at `Links::next`, ends at `Links::tail`, and each
  `ExtraValue` in the chain contributes its value once, in sequence order.
  Every extra slot is in exactly one chain. Link indices stay within live
  allocations and `prev`/`next` relations are reciprocal.
- `indices` has one occupied slot for each primary entry and no duplicate
  entry index. Every `Pos::index` points to a live entry and its hash agrees
  with that entry under the current `Danger` strategy. `mask` is zero for the
  empty allocation; otherwise `indices.len()` is a power of two and `mask` is
  exactly `indices.len() - 1`.
- The primary entry count is at most `MAX_SIZE = 2^15`, so conversion to
  `Pos.index: u16` and the `!0` empty sentinel are sound. `len()` is
  `entries.len() + extra_values.len()`, while `keys_len()` is
  `entries.len()`. The implementation checks `MAX_SIZE` against primary
  entries; appending to an existing key adds an extra node and is not bounded
  by that constant.
- Green uses the local FNV fast hash. Repeated displacement moves through
  Yellow; Yellow either grows then returns to Green, or switches to Red with a
  `RandomState` and rehashes every entry. Hash identity and probe behavior
  support lookup but do not alter the abstract mapping.

The API transition relations should use that single mapping:

- `insert` replaces a key's sequence with the singleton new value and returns
  the previous first value if the key existed. Existing extra values are
  removed.
- `append` adds to the end of a present key's sequence. For a missing key, it
  creates a singleton sequence and reports that the key was absent.
- `remove` removes the key and returns its first value; remaining values are
  removed. `OccupiedEntry::remove_entry_mult` and `insert_mult` expose removed
  values through `ValueDrain`.
- `clear` empties both vectors and clears all index slots while retaining
  allocated capacity. `try_reserve`, growth, rehash, and rebuild preserve the
  abstract mapping.
- `get`, `get_mut`, `get_all`, `contains_key`, entry APIs, equality, indexing,
  and all iterators must agree with this mapping.

Iterator and ownership obligations are separate from this extensional model:

- `Iter` yields each key's primary value followed by its extra chain. `Keys`
  yields each key once. `Values` and `GetAll::iter` yield the values in each
  key's sequence order.
- `IterMut` yields a shared key reference and a mutable value reference for
  every value. Previously returned mutable values remain live while a later
  `next` reads bucket keys and links. Permissions therefore have to be split by
  fields; borrowing a whole bucket per yield does not justify later key/link
  access.
- `ValueIter` and `ValueIterMut` support alternating front/back traversal.
  Cursor state must prove the two ends never yield the same node, and mutable
  references must remain pairwise disjoint.
- `Drain` resets table state and transfers vector ownership to a raw-pointer
  iterator. Its `Drop` must consume each unyielded `T` exactly once.
  `ValueDrain::drop` has the same obligation for a key's remaining chain.
  `IntoIter::next` uses `ptr::read` on extras; its guarded `Drop` must avoid
  dropping moved values twice while still dropping all unconsumed values.

These mapping, iterator, and drop obligations have no verified contracts yet.
Raw pointer permission is currently blocked at `RawLinks` indexing; details are
in [`TOOL_BLOCKERS.md`](TOOL_BLOCKERS.md).

## `Extensions`

The intended abstract value is a partial map from a concrete Rust type identity
to one owned value of exactly that type. A lookup or removal at
`TypeId::of::<T>()` must project the corresponding `T`; aliases have the
identity of their underlying type and distinct nominal types must remain
distinct. The erased box's payload type must agree with its key at insertion,
clone, lookup, mutation, removal, and `extend`.

`insert` replaces the value for one type key and returns the old value.
`get_or_insert*` inserts once and returns a mutable reference to that value.
`remove` extracts just that type key. `extend` transfers all right-hand entries
with right-hand values replacing matching keys. `clone` invokes each concrete
value's `Clone` implementation; a relation between source and cloned payload
needs a specified `Clone` contract and cannot assume arbitrary user
implementations preserve logical equality.

The runtime relies on `TypeId`, `HashMap<TypeId, _>`,
`Box<dyn AnyClone + Send + Sync>`, and shared/mutable `dyn Any` downcasts.
Creusot currently rejects those dynamic types before VC generation, and
`TypeId` has no model. A future model needs a typed existential carrying a tag,
payload, permission, and valid projection witness, plus contracts for dynamic
methods and mutable borrow framing. No such model is present.

## Generic `Request<T>` and `Response<T>`

The concrete representation is a head `Parts` record and an owned generic body
`T`. The public `Parts` fields are the stored method, URI, status, version,
headers, and extensions returned by accessors. The head components retain
their own invariants. `T` may be any type, so the body has no HTTP-specific
invariant.

Accessors should return references to their corresponding stored fields;
mutable accessors should borrow just those fields. `from_parts` and
`into_parts` should be inverses at the field/value level. `into_body` extracts
the body. `map` calls its `FnOnce` exactly once on the old body, preserves the
head, and installs the returned body. `Default` uses the default head and
`T::default()`.

Builders contain `Result<Parts>`. Each mutator updates the parts only while the
state is `Ok`; after a conversion error, later mutators retain the same error.
`body` packages successful parts with the supplied body or returns the stored
error. Request and response builders share that state relation, with
request-specific method/URI parsing and response status conversion.

The body remains generic and opaque. A content-equality postcondition requires
a logical model for `T`, but these public methods do not add a `DeepModel`
bound. Contracts must describe movement and preservation parametrically
without strengthening public trait bounds. No such contract is established.

## Conversion macro and errors

`if_downcast_into!(T, Bytes, value, body)` compares
`TypeId::of::<T>()` with `TypeId::of::<Bytes>()`. On equality it moves the
value through `&mut dyn Any`, downcasts to `Option<Bytes>`, takes the payload,
and runs the specialized body. On inequality it skips the specialized body and
leaves the value for the generic path. The type-identity branch must match
Rust's concrete type identity and preserve the same observable bytes/URI/header
value as the generic path. The macro expands in URI, path/authority, and header
value production code, so those bodies inherit the dynamic-type translation
blocker.

`Error` wraps one `ErrorKind` variant containing the source error. Each `From`
implementation selects its matching variant; `get_ref` borrows its payload;
`Display`, `Debug`, and `source` delegate to it; `is::<T>()` reflects the
underlying concrete error type. Conversion from `Infallible` has no runtime
return path. The `dyn std::error::Error` object is also rejected by the current
Creusot translator, so these bodies remain unproved.
