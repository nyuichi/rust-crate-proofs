# Universal UTF-8 body checkpoint

This probe verifies the two pinned standard-library bodies that construct
UTF-8 bytes for a Rust `char`: `utf8_byte` and `CharExt::to_utf8`. The primary
translation compiles the exact `creusot-std` package; these targets are not
local copies of the helpers. No scalar-specific examples are used.

The successful translation and proof bundle is in
[`evidence/stdlib-body-translation-20261005T162325Z-22970/`](evidence/stdlib-body-translation-20261005T162325Z-22970/).
Its `selected-targets.txt` lists the two `.coma` modules relative to the
translation output directory. `REPORT.md` records their source identity,
dependencies, proof results, and the remaining universal UTF-8 work.

The body results close only these two standard-library contracts. They do not
yet establish RFC 3629 validity for every encoded character or byte string.
