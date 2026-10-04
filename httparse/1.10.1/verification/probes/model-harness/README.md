# Scalar model translation harness

This small package imports the crate's `src/verification/model.rs` directly so
Creusot can typecheck its contracts without resolving the upstream runtime
crate's unrelated benchmark and test dependencies. It is a development probe,
not an integrated runtime proof.

From this directory run `./verify.sh translate` for Creusot translation or
`./verify.sh prove` to run the translated model proof. Proof runs use the
repository's shared Why3 queue and need the usual elevated execution profile.
