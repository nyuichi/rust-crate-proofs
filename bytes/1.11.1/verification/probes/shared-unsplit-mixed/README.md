# Mixed and Unique `BytesMut::unsplit`

This gate extracts the actual public `unsplit` method and its cfg bodies from
`src/bytes_mut.rs`. The default configuration selects
`bytes_proof_unsplit_mixed`, with one Unique and one Shared handle in either
order and one exclusive external coordinator. `--features unique_pair` selects
`bytes_proof_unsplit_unique`, with two Unique handles and no coordinator.

Both configurations cover empty-self adoption, preserving self when the other
capacity is zero, and concatenation through a fresh allocation. The copy helper
borrows both sources before replacing self and explicitly retires the consumed
owners. Contracts preserve every visible byte and establish the concatenated
length. Mixed ownership additionally accounts for the Shared registration and
frames unrelated registrations in the coordinator.

Unique inputs include canonical empty handles and advanced allocation views.
Cleanup uses the actual `get_vec_pos` and `release_unique_storage` bodies. A
zero-capacity view with a nonzero allocation offset still releases its original
allocation; only zero total capacity skips deallocation. The constant metadata
bit facts and the visible/absolute slot conversions are body-proved lemmas.

The kernel run proved 123 files. Its archived initial failure records seven
retirement goals caused by uninterpreted constant bit operations; the final
kernel archive includes the small bitwise lemma that resolves them. No trusted
protocol, ownership transfer, or cleanup contract was added.

The two public native matrices pass (384 cases each), covering ownership order,
zero/nonzero lengths, spare capacity, advanced offsets, and canonical empties.
The mixed public body/caller gate proves 121 files with zero unproved leaves.
Its first caller failure and the corrected positive run are archived separately.
Fresh extraction differs only by removal of a duplicate active Shared-reserve
postcondition; the remaining identical contract and all mixed bodies are
unchanged. The reextraction receipt records this explicitly. The both-Unique
public gate also proves 121 files with zero unproved leaves; its source and
proof evidence are archived independently.

Run from this directory:

```sh
bash run-proof.sh
bash run-proof.sh --features unique_pair
cargo test --locked
cargo test --locked --features unique_pair
```

These are sequential cfg bodies using explicit resource cleanup. They do not
establish default native dispatch, automatic `Drop`, or concurrent ownership.
The existing physical pointer/allocation bridges remain the trusted boundary.
The public Shared/Shared cases have separate adjacent, same-control fallback,
and independent-control gates.
