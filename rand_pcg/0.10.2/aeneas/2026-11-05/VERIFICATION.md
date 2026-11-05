# Exact next-word output and state update in Lcg64Xsh32

rand_pcg 0.10.2. For the unchanged Lcg64Xsh32::try_next_u32 body, every u64 state s and increment i returns the PCG-XSH-RR output rotate_right(low32(((s >> 18) XOR s) >> 27), low32(s >> 59)), and updates the state to (6364136223846793005*s+i) modulo 2^64 while preserving the increment. Right shifts are logical. The call terminates without model failure and returns Ok.

Only Lcg64Xsh32::try_next_u32 is verified, for the default feature configuration on x86_64-unknown-linux-gnu. Seed constructors, state access/restoration, other generators, signed/floating/ranged output, advancement, buffer filling, serialization and inherited wrappers are outside this proof. No period, distribution, independence, nonzero-state invariant or cryptographic property is established.

The Rust compiler, Charon/Aeneas translation, official Aeneas scalar/array models and Lean kernel are trusted. Standard-library operator implementations are modeled by the official Aeneas library, rather than individually body-proved. The public theorem has no extra local axioms; transitive dependencies allow only propext, Classical.choice and Quot.sound.

Run `./verify-all.bash`. Original archive and source hashes: `evidence/`. Theorems and translation: `proofs/`. Both state and increment range over all 64-bit values; no seed/odd-increment precondition is imposed on the one-step formula.
