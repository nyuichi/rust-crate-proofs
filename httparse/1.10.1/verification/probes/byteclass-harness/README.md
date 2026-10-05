# Byte-class runtime refinement harness

This target-local package imports the actual `src/byteclass.rs` implementation,
its original `byte_map!` macro, and the independent `src/verification/model.rs`
specification. It is isolated from the parser crate's unrelated upstream dev
dependencies. `verify.sh translate` checks contract/type translation;
`verify.sh prove` invokes the shared proof runner for this exact harness.
