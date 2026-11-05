# Bitwise wrapper result preservation

For endian-type 0.2.0, the handwritten BitAnd, BitOr, and BitXor implementations of BigEndian<T> and LittleEndian<T> apply the corresponding underlying T operator and wrap its successful output. Each of the six proved contracts quantifies arbitrary operand and output types, operands, and operator dictionaries: the translated wrapper result equals the underlying operator result. This includes propagation of failure or divergence represented in the Aeneas result model. The wrappers add no independent failure in that model. These contracts prove the unchanged published wrapper bodies; they do not prove any user-supplied T operator implementation.

Only six handwritten generic bitwise operator method bodies are covered. No endian conversion, integer From implementation, byte representation, raw-pointer/slice access, transmute safety, derived standard trait implementation, or complete crate memory-safety claim is established. Concrete integer operator dictionaries are not independently proved here. Extraction/build used x86_64-unknown-linux-gnu; other targets were not checked.

The Rust compiler, Charon/Aeneas translation, their standard-library models, and the Lean kernel are trusted. Underlying operator behavior is represented by an arbitrary trait dictionary, rather than asserted to implement a particular integer operation. The theorems have only propext, Classical.choice, and Quot.sound as transitive axioms.

## Evidence

- `proofs/Proofs.lean`: six generic equality contracts and transitive axiom audits.
- `proofs/Generated/EndianType.lean`: official generated bodies.
- `evidence/upstream.crate`, `tools/check-source.py`: archive-wide byte identity.
- `evidence/verification.log`: integrated build and audits.
- Run `./verify-all.bash` to repeat source checks, original tests, regeneration, and Lean verification.

Original upstream revision: bed2afaf0dad0ce8214a23c5d1143b38fa1f4db9.

The official frontend rejects trait-generic From root selectors; attempting primitive self-type roots also fails name resolution. No alternative models or modified sources were introduced to include conversions. The covered operator roots avoid unrelated unsafe APIs.
