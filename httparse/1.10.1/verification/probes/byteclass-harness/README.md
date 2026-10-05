# Byte-class runtime refinement harness

This target-local package imports the actual `src/byteclass.rs` implementation,
its original `byte_map!` macro, and the independent `src/verification/model.rs`
specification. It is isolated from the parser crate's unrelated upstream dev
dependencies. `verify.sh translate` checks contract/type translation;
`verify.sh prove` retranslates, requires the expected nonempty Coma target, then
uses the shared proof runner to invoke `why3find prove` directly. The proof
phase selects an explicit Why3 config and Creusot package path; the runner
validates the config profile and passes those values to `why3find` after its
own environment setup. Cargo-creusot's frontend setup is not part of the
solver phase.

Each translation cleans only `httparse-byteclass-harness` from this harness's
explicit `target/` directory, removes prior `.coma` outputs from this
harness's `verif` target, and checks that the caller, imported runtime helper
and table-constructor, and model-body targets in the mandatory manifest were
regenerated as nonempty files. Additional Coma outputs are allowed.
