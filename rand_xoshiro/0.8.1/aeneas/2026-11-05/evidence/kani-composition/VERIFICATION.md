# Exact two-word composition for five xoshiro-family APIs

For rand_xoshiro 0.8.1, Kani verifies the original `try_next_u64` bodies of Xoroshiro64Star, Xoroshiro64StarStar, Xoshiro128Plus, Xoshiro128PlusPlus and Xoshiro128StarStar. For every internal state, the result is `Ok(low | (high << 32))`, where low is the first native 32-bit output and high is the second; the resulting state is exactly the state after both specified native transitions. Each native transition and scrambler is independently expressed by scalar equations in `source/src/composition_harnesses.rs`.

The domain includes every two-u32 or four-u32 word state, including all zero. These are the complete fixed-size engine types, not an artificial bound on a variable-length input. The harnesses use arbitrary words without assumptions, check the entire returned 64-bit value and every resulting state word, and call the original method body. Default and serde configurations both pass five harnesses, 892 proof obligations each, with no failures. The checks are not concrete test cases.

Original Rust and Cargo manifest bytes are retained. A generated measurement copy preserves every original source file as a complete byte prefix and appends only constructors/readers of private scalar state; one separate harness module is added. No original algorithm body is replaced or annotated with contracts. `provenance.json`, `api-bindings.json` and `check-source.py` record and check this relationship.

The Rust compiler, Kani/CBMC and their standard-library models are trusted. Kani uses Rust nightly-2026-08-21, whereas the separate Aeneas work uses nightly-2026-09-17. The two tool results are independent guarantees; no formal cross-tool connection is claimed. Source observers only create/read valid scalar fields. The proof does not establish seed validity, period, statistical distribution, independence, cryptographic security, serde behavior, other methods or other targets.

Aeneas extraction of these five compositions remains blocked by copied mutable borrows or unresolved dictionaries; aggressive inlining introduces unsupported intrinsic externals and is not used as a proof. These results must remain separately labeled as Kani guarantees. cargo-proofs 0.4.1 only binds `proof_for_contract` harnesses to API catalogue entries, so these ordinary `kani::proof` results are ready as evidence but are not falsely registered as Aeneas API claims.

Reproduce with `./verify-kani.bash`. `prepare.py` recreates the measurement source from `upstream.crate`. Logs and `results.json` retain both feature runs.
