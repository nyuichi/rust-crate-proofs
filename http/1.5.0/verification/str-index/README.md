# UTF-8 byte-boundary and `str` indexing proofs

This target proves std helpers that connect character sequences to UTF-8 byte
offsets, then exercises them through actual callers of the `str` range-index and
`split_at` contracts. The helpers are mathematical logic functions; they add no
runtime implementation.

The std proof contains eight independently proved helpers (70 leaf VCs):

- generic sequence concatenation, head/tail, and exact-half lemmas;
- recursive UTF-8 encoding concatenation and byte-length preservation;
- the UTF-8 layout fact that continuation bytes are non-ASCII;
- `prefix_at_ascii_byte`, which returns character and byte subsequences split
  immediately before a selected ASCII byte;
- `prefix_through_ascii_byte`, which returns the prefix through that byte.

The prefix lemmas allow arbitrary modeled Unicode before and after the selected
ASCII byte. The HTTP callers prove the exact `[a, é, b]` range result, the
`RangeFull` identity, and byte offsets 2 and 3 around `?` in `é?🍕` (23 leaf
VCs). The combined UTF-8 helper and caller evidence is recorded in
[`evidence/manifest.json`](evidence/manifest.json) and [`outcomes.json`](outcomes.json).

The `Range<usize>` indexing contract requires ordered endpoints that each equal
the UTF-8 byte length of a character prefix. `str::split_at` also requires an
explicit character-prefix witness. These are external std contracts; this
target proves representative callers meet their preconditions, but it does not
prove the core-library implementations. Invalid boundaries remain in Rust's
panic domain.

All claims inherit the bundled Creusot character-model limitation: its integer
model is bounded by `< 0x10FFFF`, excluding valid Rust `char` U+10FFFF.

Run the HTTP caller proof from this directory with:

```text
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache -- --locked --offline
```

The helper bodies were proved one target at a time from `creusot-libs` using
`../http/1.5.0/scripts/run-proof.sh cargo creusot -p creusot-std prove
'std::str_index::utf8::<helper>' -- --features bytes-model --locked`.
