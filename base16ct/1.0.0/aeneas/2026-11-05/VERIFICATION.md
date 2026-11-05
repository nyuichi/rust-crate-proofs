# Base16 length calculations

The original `decoded_len` body is proved to terminate normally on every slice, returning half its length when even and InvalidLength when odd. This checks no character contents. The original `encoded_len` body returns twice the length when length ≤ usize::MAX/2; no claim covers larger inputs.

The model quantifies arbitrary representable slice lengths, without an input-size enumeration bound. Compiler/translation/library models and Lean's kernel are trusted. No custom models or local axioms are introduced. Both theorem transitive axiom audits permit only propext, Classical.choice and Quot.sound.

Only these two helpers are covered; encoding/decoding, destination writes, UTF-8 conversion and allocation remain unverified. Sources and Cargo.toml match the release archive byte for byte. Run `./verify-all.bash` to check provenance, original tests, regeneration hashes, the Lean build and axiom audits. Full statements are in `proofs/Proofs.lean`.
