# Scratch integration attempt against the actual bytes crate source

This is a disposable copy of the bytes 1.11.1 source at
`/tmp/bytes-buf-trait-integration-scratch`. No production `Buf` or handle
source was changed for this probe. The captured source files match the current
worktree files by the SHA-256 entries in `source-files.sha256`; the worktree
HEAD at the initial scratch copy was
`3ee5b240a5d82b7a263666851766ca9b843fe9c7`. The original `Buf` declaration
file hash was
`c1fe35b415615f39e2c1de487008231ec33d074875ed6ee992a4075453c15db6`.

The actual public `Buf` trait copy adds a proof-only logical unread sequence,
contracts on the actual required `remaining`, `chunk`, and `advance` methods,
and a result contract on the exact actual default `try_get_u8` body. The
initial candidate defines the model only for the exact existing `&[u8]` impl.
`trait-model.patch` records that first stage.

The first native `cargo check --locked --lib` passed. The first Creusot
translation then reported `E0046` for eight existing impls: `&mut T`, `Box<T>`,
`Cursor<T>`, `Chain<T,U>`, `Take<T>`, `VecDeque<u8>`, `Bytes`, and `BytesMut`.
That result is in `translation.log`; the initial proof gap was missing model
implementations, not a completed public-trait proof.

After Astra's review, the scratch added nontrusted models for the tractable
impls: forwarding through `&mut T` and `Box<T>`, concatenation for `Chain`, a
bounded prefix for `Take`, and the stock `VecDeque<u8>` view. The `Chain`
`remaining` contract uses `min(model length, usize::MAX)` because the existing
runtime implementation saturates and the mathematical concatenation may be
longer than `usize::MAX`. The translation then reduced the missing-model list
to `Cursor<T>`, `Bytes`, and `BytesMut`; see
`translation-after-easy-models.log`.

The next scratch stage attempted candidate models for all three. Generic
`Cursor<T>` cannot derive a model by calling `get_ref`, `position`, or
`AsRef::as_ref` in a logic body: the current Creusot surface exposes none of
those as logical operations, and `AsRef<[u8]>` itself does not promise a stable
sequence for arbitrary downstream implementations. `Bytes` similarly has no
logical byte-slot model for its default raw-pointer/vtable representation; the
candidate `as_slice()` call is a program operation in logic. Their exact
translation errors are in `translation-with-slot-bytesmut-candidate.log`.

For `BytesMut`, the initial `as_slice()` candidate had the same program-in-logic
problem. The final candidate instead defines a total sequence over the current
length from the existing `proof_view_slot`, mapping absent/unknown slots to
zero. This is only a pure logical projection: it does not assert that the
slots are initialized or that the fallback byte is the runtime byte. The
method uses scoped `#[logic(open(self))]` to keep the model's private-field
access local. This removes the `BytesMut` translation/frontend error, but does
not prove the public `chunk` law: that law still needs a proof that every
runtime-visible byte is initialized and corresponds to its slot. Astra
identified this as the concrete `proof_initialized` bridge frontier from the
field-only invariant. No BytesMut ownership, refcount, or slot authority was
made trusted.

The preceding `translation-with-handle-cursor-candidates.log` records the
first `BytesMut::as_slice()` model attempt and its program-in-logic plus
visibility errors. It was replaced by the total slot projection above. The
final attempted actual-source translation still exits with status 1 and 12
frontend errors: ten program-in-logic calls in the generic Cursor candidate,
and two in the Bytes `as_slice()` candidate. Because translation stops there,
no implementation VCs for the actual `Buf` laws or the default body were
proved in this actual-source scratch. The standalone feasibility probe is
separate evidence: its exact default body and `&[u8]` impl proved 13 obligations,
and its three native tests passed, but this does not discharge the current
public `Buf` implementations.

`final-models.patch` records the later scratch models and candidates atop the
first trait patch. It contains no trusted law or runtime change. `native-check.log`
records the successful native check before the follow-up model attempts. The
scratch-only attempt to disable `Bytes` and `Cursor` impls to isolate
`BytesMut` also stopped at `Bytes`'s existing `IntoIterator` associated-type
bound (`IntoIter<Bytes>: Iterator` requires `Bytes: Buf`); that diagnostic is
not used as proof evidence and did not change the above frontier.

The complete incremental scratch history is bundled in
`actual-source-stages.tar.gz` (SHA-256 recorded in
`actual-source-stages.archive.sha256`; 408 members are listed in
`actual-source-stages.membermanifest.txt`). `stages-index.md` describes each
stage. Every archived source snapshot has a source SHA-256 list and an evidence
SHA-256 list; all five transition patches replayed and reproduced their target
source snapshots byte-for-byte after extraction.
