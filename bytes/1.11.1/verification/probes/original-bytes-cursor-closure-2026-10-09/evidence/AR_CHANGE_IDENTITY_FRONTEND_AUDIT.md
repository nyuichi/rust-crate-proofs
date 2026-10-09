# AR change_identity frontend capture audit

The immutable `ar-negative-change-identity-v1.tar.gz` archive matches SHA-256 `9638d3802254570b4335cb4e78fde3891b0a8114725838d82601d49393b050bc`. All 883 regular-file members match the receipt; the member paths are unique.

The archived compiler log (SHA-256 `1a86b3bf6d5a81d96b64a026171dff181d9f38358c0f023f2ab7e2ec8849100c`) reports `E0308`: the generated expression `p.core.ticket.id += 1` at `generated/active.rs:1785:35` uses an integer literal where the field expects `Int`. The captured generated source hash is `df34f04fdeab5b8338a3f2920a547511e93cf13cbd0e1b5e31631cca1bd81657`.

The receipt labels this `frontend_diagnostic_no_proof_claim` and explicitly says proof artifacts were excluded deliberately. I found no `.coma`, `proof.json`, or `probe/verif/` members, so this is a frontend construction failure only, not a proof result or semantic theorem. This v1 capture is immutable and is not an admission claim.

A distinct corrected v2 capture (`ar-negative-change-identity-v2`, archive SHA-256 `c8b9b8174484eedab89efa007deb4e7374848f48c34418f0164c5b3a7d9bff02`) is audited in `AR_SEMANTIC_CONTROL_AUDIT.json`: 150 targets and six null leaves. That later proof diagnostic does not change the v1 capture's frontend-only classification.
