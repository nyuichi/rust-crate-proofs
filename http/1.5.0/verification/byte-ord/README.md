# Generic sequence-order proof

The central `creusot-std` model now defines lexicographic `OrdLogic` for every
`Seq<T>` whose element type implements `OrdLogic`. This supplies the exact
sequence order used by `Bytes` and `ByteStr`, and also makes sequence order
available for `Seq<char>` consumers such as HTTP method text.

The comparator is recursive on the finite sequence length. Its independent
lemmas establish equality, reversal, and transitivity using only the element
type's `OrdLogic` laws. They do not use `Seq` ordering or assume the new
`OrdLogic for Seq<T>` laws. The generic trait implementation delegates each
law to those closed lemmas. The generated Coma and Why3 results are retained
under `evidence/`; the helper VCs use element `OrdLogic` laws only, so there is
no circular assumption of the sequence-order laws being proved.

The local harness includes the exact helper source and proves its four bodies:

```text
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove -- --locked --offline
```

The owning `creusot-std` crate was also proved with filtered targets:

```text
CARGO_NET_OFFLINE=false ../http/1.5.0/scripts/run-proof.sh cargo creusot -p creusot-std prove 'std::seq_ord::*' -- --features bytes-model --locked
CARGO_NET_OFFLINE=false ../http/1.5.0/scripts/run-proof.sh cargo creusot -p creusot-std prove 'std::seq_ord_impl::*' -- --features bytes-model --locked
```

The exact locked crates were fetched before those runs; the lockfile retains its
original versions plus only the optional `bytes 1.11.1` entry required by the
new feature.

Results: the consumer harness proved all 4 lemma bodies, the standard-library
helper target proved the same 4 bodies, and the standard-library trait target
proved 18 law/refinement VCs. There are no outstanding order VCs. The
`OrdLogic` laws required of element type `T` remain ordinary generic trait
premises; for `char`, the existing primitive `OrdLogic` implementation
provides them.
