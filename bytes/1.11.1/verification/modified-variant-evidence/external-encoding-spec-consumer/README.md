# Downstream public-contract check

This is a separate crate depending on the actual modified bytes 1.11.1 library
with `verified,std`. Its six callers prove BE/LE writer bytes, unsigned variable
reads, signed variable reads, fixed signed reads, and native-endian signed wide
reads using the public contracts and model definitions. It does not copy the
implementation bodies into a local interface.

`evidence/external-six-positive` records the completed six-file proof, zero null
leaves, and three native tests. Its dependency sources match the production
`public-read-models-255` archive byte for byte. The standard-library TCB remains
the version captured there; this is a downstream composition result, not full
API/configuration coverage.

The initial arithmetic frontend error and missing local proof configuration are
preserved as harness diagnostics. `evidence/opaque-reader-weight` preserves the
real one-goal public-contract failure: the downstream reader could not unfold a
private byte-weight definition. Production subsequently exposes the unchanged
pure definitions through `verified::encoding_spec` and re-proves all 255 files.

Run `./verify.sh` elevated. It uses the shared lock, one prover and a 1024 MiB
limit. `capture.py` audits a completed result and refuses to overwrite evidence;
set `BYTES_PROVEN_RUN` to the exact matching production positive run. Existing
archives are immutable even when the current library later changes.
