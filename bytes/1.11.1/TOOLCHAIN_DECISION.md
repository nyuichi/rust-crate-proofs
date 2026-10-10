# Bytes proof toolchain

Use the dedicated `/workspace/bytes-proof-tools` installation, prepared by
`bash scripts/setup-bytes-toolchain.sh`; do not substitute the repository's older
Creusot installation. `scripts/prepare-proof-std.py` prepares the private Std
contracts. Each proof receipt identifies its exact tool/Std inputs.

- Rust: nightly-2026-06-22, rustc `91fe22da8`.
- Creusot/creusot-std: 0.13.0, `318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`.
- Why3: `54c92f96bb0711d6e991c18f10bfbc08d90d028b`.
- Why3find: `eab37557d3e24e1913a3c4f44bc5528ef497c6c9`.
- Alt-Ergo 2.6.2, Z3 4.15.3, CVC4 1.8, CVC5 1.3.1.

Run the affected probe's `run-proof.sh` with elevated execution for Why3 sockets.
The wrappers serialize proofs, use one prover and 1024 MiB, preserve native
Ordering and reject `sc-drf`. Default `std` is the working configuration.

`./verify-all.bash` attempts the full runtime and remains an integration
blocker diagnostic. Rerun it only after a relevant integration change.
For retained results, use the exact [evidence and scope](README.md).
