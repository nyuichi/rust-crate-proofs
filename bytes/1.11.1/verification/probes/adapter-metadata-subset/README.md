# Source-sliced adapter metadata proof

This probe extracts the exact `Take<T>`, `Limit<T>`, and `Chain<T, U>` field
layouts and the selected method bodies from the pinned bytes 1.11.1 sources:

- `Take`: constructor, `limit`, `set_limit`, `get_ref`, `get_mut`, and
  `into_inner`;
- `Limit`: constructor, `limit`, `set_limit`, `get_ref`, `get_mut`, and
  `into_inner`;
- `Chain`: constructor and `into_inner`;
- `Reader` and `Writer`: constructors, `get_ref`, `get_mut`, and `into_inner`.

The extracted bodies retain their source statements. Proof-only contracts state
field projection, budget replacement, the unchanged inner/limit field across
mutable accessors and budget updates, and the two fields returned by consuming
the adapters. The callers compose those contracts for metadata round trips.
The build writes full source snapshots plus per-fragment source offsets and
hashes under Cargo's `OUT_DIR`.

The proof does not use `Buf` or `BufMut` methods, assign a byte-content model,
or make claims about generic byte transfers, ownership, reference counts,
allocation, or `Drop`. It proves only the extracted adapters' field and limit
metadata behavior for arbitrary stored type parameters. For `Reader` and
`Writer`, their `Read`/`Write` transfer implementations are excluded.

The probe places the exact `Reader`/`Writer` accessor bodies in unconstrained
generic impl blocks because these selected bodies do not call the traits. The
runtime keeps its original `B: Buf` and `B: BufMut` impl bounds; omitting those
bounds here proves no trait law and says nothing about I/O transfer behavior.

Run the native source-slice checks with `cargo test --locked --offline`. Run
the serialized, pinned Creusot proof with `bash run.sh`. The resulting compact
archive and SHA-256 receipt are written under `evidence/` after both gates pass.
