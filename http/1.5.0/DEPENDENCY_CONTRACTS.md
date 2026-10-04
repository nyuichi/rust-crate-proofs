# HTTP dependency contract ledger

## Status and scope

This file records the external dependency contracts HTTP may assume under the
user-authorized counterfactual that `bytes` **1.11.1 has completed full
verification**. A conditional HTTP proof may use these contracts without
rerunning `bytes`. This ledger is not itself an implementation of those
external contracts and does not claim that the current workspace copy of the
`bytes` proof has finished.

The current [`bytes/1.11.1/PROVENANCE.md`](../../bytes/1.11.1/PROVENANCE.md)
reports a structural length/capacity model and explicitly says that byte
contents are not modeled. Its runtime implementation is excluded from its
Creusot translation. Therefore none of the byte-content or allocation
ownership facts below may be presented as results from the current workspace
`bytes` proof. For this task, they are permissible external assumptions only
because the user explicitly authorized the completed-`bytes` counterfactual.
HTTP must still implement and list the narrow external contracts that expose
that premise to its proof harnesses. Any unconditional claim about the
dependency itself would require separate evidence from a completed `bytes`
proof.

The dependency is pinned to registry release `bytes = 1.11.1` in HTTP's
verification graph. The intended byte observer is a named HTTP-local opaque
logic function, conceptually:

```rust,ignore
#[logic(opaque)]
pub fn bytes_seq(value: bytes::Bytes) -> Seq<u8> { dead }

#[logic(opaque)]
pub fn bytes_mut_seq(value: bytes::BytesMut) -> Seq<u8> { dead }
```

These are abstract observers backed by the explicitly assumed dependency
contracts. Do not implement `View` for the foreign `bytes::Bytes` type, and do
not infer contents from its length. Tie each observer contract to the ordinary
pinned API; a successful HTTP helper proof that merely assumes an unconstrained
`Seq<u8>` is not sufficient evidence.

## Required `Bytes` content contracts

`bytes_seq(b)` denotes exactly the initialized bytes visible through `b`.
Every operation below must preserve that meaning:

| API used by HTTP or required to connect its source conversions | Required contract |
|---|---|
| `Bytes::new()` | `bytes_seq(result) == Seq::empty()` |
| `Bytes::from_static(s)` | `bytes_seq(result) == s@` |
| `Bytes::copy_from_slice(s)` | `bytes_seq(result) == s@`; the input slice is unchanged |
| `From<Vec<u8>> for Bytes` | The result sequence is the consumed vector's sequence, byte for byte |
| `From<String> for Bytes` | The result sequence is the consumed string's UTF-8 byte sequence |
| `Clone::clone(&Bytes)` | Result sequence equals the source sequence; the source remains unchanged. Shared immutable storage is allowed |
| `AsRef<[u8]>` and `Deref<Target = [u8]>` | The returned slice view is exactly `bytes_seq(self)` |
| `len` / `is_empty` | `len()@ == bytes_seq(self).len()` and `is_empty() == (bytes_seq(self).len() == 0)` |
| `split_to(at)` | Requires `at <= old.len`; result is `old[0..at]`, and the new receiver is `old[at..old.len]` |
| `split_off(at)` | Requires `at <= old.len`; new receiver is `old[0..at]`, and result is `old[at..old.len]` |
| `truncate(n)` | New sequence is the old prefix of length `min(n, old.len)`; a larger `n` leaves it unchanged |
| `clear()` | New sequence is empty |

The split postconditions are a sequence partition: prefix concatenated with
suffix equals the old sequence, with no logical overlap or missing byte. They
must describe both the returned value and the updated receiver. The observer
tracks contents; the dependency proof remains responsible for allocation
validity, aliasing, reference counts, and safe ownership transfer.

## Required `BytesMut` content and ownership contracts

`bytes_mut_seq(b)` denotes exactly the initialized prefix of a mutable buffer.
The invariant is `bytes_mut_seq(b).len() <= b.capacity()`, and the observer's
length equals `b.len()`. Contents and frame conditions must cover:

| API | Required contract |
|---|---|
| `BytesMut::new()` / `with_capacity(n)` | Start with an empty sequence; capacity is at least the requested amount |
| `From<Bytes>`, `From<&[u8]>`, and `From<&str>` | Result contains exactly the source sequence |
| `Clone::clone(&BytesMut)` | Result initially contains the same sequence and owns a separately mutable value; later writes to either value do not change the other |
| `AsRef<[u8]>`, `Deref<Target = [u8]>` | The borrowed slice is exactly `bytes_mut_seq(self)` |
| `AsMut<[u8]>`, `DerefMut<Target = [u8]>` | The mutable view covers exactly the initialized sequence and its writes update that sequence while preserving the capacity invariant |
| `len`, `is_empty`, `capacity` | Report the exact observer length, emptiness, and capacity |
| `put_slice(s)` | Requires `old.len + s.len <= isize::MAX`; appends exactly `s@`, so the new sequence is `old.concat(s@)` and capacity covers the new length |
| `put_u8(byte)` | Requires `old.len < isize::MAX`; appends exactly `byte`, so the new sequence is `old.push_back(byte)` and capacity covers the new length |
| `fmt::Write::write_str(s)` | If `remaining_mut() >= s.len()`, returns `Ok` and appends `s`'s UTF-8 bytes; otherwise returns `Err` and leaves the sequence unchanged |
| `split_to(at)` | Requires `at <= old.len`; result is `old[0..at]`, and new receiver is `old[at..old.len]` |
| `split_off(at)` | Requires `at <= old.capacity`; if `at <= old.len`, receiver is `old[0..at]` and result is `old[at..old.len]`; if `old.len < at`, receiver stays `old` and result is empty |
| `truncate(n)` / `clear()` | Keep the old prefix of length `min(n, old.len)` / empty the sequence, respectively |
| `freeze(self)` / `From<BytesMut> for Bytes` | Consume the mutable buffer and produce `Bytes` whose sequence is exactly the old initialized sequence |

HTTP calls `BytesMut::put_u8` through `BufMut` while normalizing a header name.
Do not add a generic axiom that every `BufMut::put_u8` appends to an arbitrary
implementation. Creusot's generic trait/specialization boundary makes such an
axiom broader than the HTTP dependency requires. State and justify the exact
append contract for the concrete `BytesMut` call path (or a verified concrete
adapter), backed by the `BytesMut` implementation's completed proof. Likewise,
the integer-to-`HeaderValue` path calls `fmt::Write::write_str` on a concrete
`BytesMut`; its success and failure frame must be exact.

The concrete `BytesMut::put_slice` implementation may call `reserve` and
`extend_from_slice`; their contracts must preserve old contents and establish
the capacity/initialized-length bound required by the append postcondition.
`reserve(additional)` leaves the sequence unchanged and, when it returns,
provides capacity for `len + additional` subject to the buffer's maximum
allocatable size. `remaining_mut()` is the maximum size still allocatable, not
necessarily spare capacity, so it must not be modeled as `capacity() - len()`.

## HTTP call sites covered by this boundary

The exact-sequence contracts are consumed by these published implementation
paths:

| HTTP path | Needed dependency facts |
|---|---|
| `ByteStr` | Empty/static/copy/string constructors, `AsRef`, `From<ByteStr>`, clone, and exact `Bytes` content |
| `HeaderName` | `copy_from_slice`, `from_static`, conversions to `Bytes`, and concrete mutable append/freeze while lowercasing |
| `HeaderValue` | Static/shared/copy/string/vector construction, borrowed byte view, exact length, clone, `BytesMut` string append/freeze for integer conversions |
| `Uri`, `Authority`, `PathAndQuery`, `Scheme` | Static/copy/string/vector conversions, borrowed input scans, split partitioning, and path fragment truncation |

`from_maybe_shared` and `if_downcast_into!` add a separate ownership/type-erasure
obligation: after a successful downcast to `Bytes`, the original buffer must be
preserved exactly; on other types the copied `AsRef<[u8]>` sequence must be
preserved. A correct `bytes_seq` observer does not prove Rust `Any` downcasts,
`AnyClone`, or their move/frame behavior. Those remain separate HTTP/toolchain
gates.

## Other dependencies and standard-library models

### `itoa` 1.0.18

`HeaderValue::from` for integer types writes `itoa::Buffer::format` into a
`BytesMut`. The existing [`itoa` proof record](../../itoa/1.0.18/PROVENANCE.md)
contains a proved decimal-sequence model and a proved formatter result
contract, including ASCII range facts. Its integrated runtime proof retains the
narrow trusted `assume_init_slice` boundary for the `MaybeUninit<u8>` view
conversion, as recorded there. This is separate evidence for `itoa` and must
be cited as such; it does not prove the `BytesMut::write_str` append contract,
the `HeaderValue` conversion body, or an HTTP integrated proof.
`http/Cargo.toml` aliases the local verified package as `itoa-proof` while the
Rust library name remains `itoa`.

### UTF-8

`creusot_std::std::string::valid_utf8(Seq<u8>)` is the existing logical model:
a sequence is valid exactly when it is the UTF-8 encoding of a character
sequence. The standard-library contract for checked and unchecked UTF-8
conversion is a separate dependency. A `ByteStr` invariant should be stated
using this model; the bytes observer only says which bytes are present and
cannot establish UTF-8 validity by itself.

### `NonZeroU16`

`creusot-libs/creusot-std/src/std/num.rs` now defines the shared opaque
`nonzero_value` observer, a local primitive model implemented for `u16`, and
generic external specifications for the generic standard-library methods
`NonZero<T>::new`, `get`, and `new_unchecked`. The `NonZeroU16` `DeepModel`
implementation unfolds to this same observer; `http::StatusCode` uses that
model directly rather than defining a second observer.

This is an explicit standard-library TCB boundary: the contracts state exact
construction, extraction, and unchecked-construction behavior, but the actual
`core::num::NonZero` method bodies are not proved. The scalar consumer harness
proves `new`/`get` round-trip plus equality and ordering refinements (10 VCs)
against these contracts; it does not turn them into proof of `libcore`.
`StatusCode::from_u16`, `from_bytes`, `as_u16`, and its five classification
bodies are independently proved against the same observer. See
[`verification/scalars/README.md`](verification/scalars/README.md) for exact
commands and the partial harness exclusions.

## Proofs that do not use the Bytes assumption

The existing successful isolated proofs for `header::value::is_valid`,
`header::value::is_visible_ascii`, HeaderMap capacity leaves, URI `Port`, and
`Version` do not import or consume `bytes_seq`/`bytes_mut_seq`. Their results
are independent of this ledger. They also do not prove `ByteStr`, public
`HeaderName` or `HeaderValue` constructors, full `HeaderMap`, URI parsers, or
the complete HTTP crate. In particular, a successful header byte predicate
proof is not evidence that the current `bytes` content contracts exist.

## Proof status and remaining HTTP work

The session premise allows a **conditional HTTP proof** to consume these exact
sequence and ownership contracts as external assumptions. Such a result must
name the premise and must not be reported as a new proof of `bytes` itself. The
current HTTP tree still lacks the centralized `bytes_seq` / `bytes_mut_seq`
observer and narrow external specifications, so these contracts are a planned
interface rather than an already usable model.

To consume the premise in HTTP, implement one named exact-sequence observer
and narrow external specifications tied to the pinned methods above. Do not
model a substitute implementation or mark an HTTP runtime body trusted to
stand in for a missing dependency interface. Contract the concrete
`BytesMut` append path narrowly; do not introduce a universal `BufMut` append
axiom. Prove HTTP's `ByteStr` UTF-8 invariant, caller preconditions, and
constructor/accessor postconditions against the existing UTF-8 model.

If the project later removes the counterfactual and claims unconditional
verification of all dependencies, then the actual `bytes` 1.11.1 runtime proof
must substantiate these contracts. The current length-only surrogate does not
meet that stronger claim.
