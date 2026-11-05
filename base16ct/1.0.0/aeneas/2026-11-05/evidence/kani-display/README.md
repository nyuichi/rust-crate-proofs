# Restricted base16ct Error Display verification

Run `./verify.bash`; `tools/check-evidence.py` validates the selected scope, original archive, metadata/unwind setting and complete successful result. Original files are in source/, with no source or Cargo.toml edits. The test crate depends directly on that copy.

See VERIFICATION.md for the narrowly stated guarantee. src/lib.rs uses two symbolic booleans and a fixed [u8;64] fmt::Write implementation. Default write!(sink,"{}",error) calls the actual crate implementation. The expected length is selected independently from the variant, and every fixed output position is checked, including unused zero bytes. The writer's nonempty-attempt length prevents an early empty-piece failure from making the error case vacuous.

The first attempt used dynamic slice equality and was stopped when builtin memcmp expansion grew without a solver verdict. evidence/initial-dynamic-slice.log, its source and initial-checkpoint-result.json preserve that checkpoint. One authorized structural retry replaced the postcondition with fixed-index checks and used #[kani::unwind(80)] with default unwinding checks enabled. It succeeded without weakening the four-case input domain or substituting formatting behavior.

This is unpublished partial-domain Kani evidence for one public Display API. It is not a general arbitrary-Formatter/writer proof, an Aeneas formatting proof, or evidence from the simplistic Formatter model.

Publisher-ready audit: literal byte lengths are23 and21. The historical results
JSON mistakenly reported22 and20, while original source, unchanged harness and
successful1910check run already use the correct strings/lengths. The original
metadata is retained in evidence/historical-results-before-length-correction.json
and the editorial correction is explicit in evidence/metadata-erratum.json.
No new Kani execution is claimed by this packaging step. This supplementary
result adds no Aeneas mapping and does not change the three existing claims or
the unproved encoded_len overflow domain.
