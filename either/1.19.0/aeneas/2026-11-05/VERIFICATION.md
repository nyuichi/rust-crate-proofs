# Either branch operations

The original crates.io source is unchanged. Seven method contracts cover arbitrary payload types and values: exact branch predicates, Option projections, default-value projections and variant swapping. `Proofs.flip_twice` additionally proves the actual two-call composition returns its original value.

The published scope excludes closures, iterators, pinning, formatting, serialization and other trait implementations. Destructor/resource behavior is outside Aeneas's pure value semantics. Compiler/translation/library models and the Lean kernel are trusted; axiom audits allow only propext, Classical.choice and Quot.sound.

Run `./verify-all.bash` with the pinned Aeneas tools. It checks every original Rust file against `evidence/upstream.crate`, runs original tests, regenerates and hash-compares the translation, builds the Lean proofs and audits transitive theorem axioms. See `proofs/Proofs.lean`, `proofs/Generated/Either.lean`, `proofs.toml` and the evidence logs.
