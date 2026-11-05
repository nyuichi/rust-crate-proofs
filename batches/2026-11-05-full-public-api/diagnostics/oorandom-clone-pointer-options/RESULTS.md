# Pointer-model option followup: full Layout domain still unverified

The documented Kani pass-through option `-Z unstable-options --cbmc-args
--object-bits 8` ran the identical harness/domain with8objectbits/56offsetbits.
It failed the same one of105 properties (4 unreachable) in9.59s. No checks were
suppressed; no assumptions/domain/source/model changed. Prior16/48 result is
preserved separately.

Official Kani0.68.0 models.rs125–145 explicitly documents that CBMC pointer
addition can wrap at2^(pointer_width-object_bits), rather than pointer width,
and explains this exact address-consistency guard. Reducing objectbits improves
representable offset range, but leaves a finite split and does not establish
coverage of all Rust Layout-valid isize-sized regions. No sound standard option
was found that eliminates this split for the unchanged full target domain.
Disabling the safety guard or silently bounding N would change the claim and
was not done. This is no new machine theorem or original-crate defect finding.
A smaller backend-representable fresh-region domain could be a separately named
future contract; it must not be called arbitrary-valid-heap completion.
