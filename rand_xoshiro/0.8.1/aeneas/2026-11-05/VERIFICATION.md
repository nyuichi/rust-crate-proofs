# Exact next-word output and state update in Xoshiro128Plus

rand_xoshiro 0.8.1. For the unchanged Xoshiro128Plus::try_next_u32 body, every choice of four u32 words (a,b,c,d), including all zero, returns Ok(a+d modulo 2^32). Put c1=c XOR a and d1=d XOR b. The new state is (a XOR d1, b XOR c1, c1 XOR (b << 9), rotate_left(d1,11)), with 32-bit operations. The call terminates without model failure.

Only Xoshiro128Plus::try_next_u32 is verified, for the default feature configuration on x86_64-unknown-linux-gnu. Seeds, other generators, jumps, u64 output, buffer filling, serde, and inherited wrappers are not verified. No period, distribution, independence, nonzero-state invariant or cryptographic property is established.

The Rust compiler, Charon/Aeneas translation, official Aeneas scalar/array models and Lean kernel are trusted. Standard-library operator implementations are modeled by the official Aeneas library, rather than individually body-proved. The public theorem has no extra local axioms; transitive dependencies allow only propext, Classical.choice and Quot.sound.

Run `./verify-all.bash`. Original archive and source hashes: `evidence/`. Theorems and translation: `proofs/`. The input constructor state4 uses four independent arbitrary words, with no finite input-size bound. The supplementary state4_surjective theorem proves that every valid four-element Rust state array has this representation.
