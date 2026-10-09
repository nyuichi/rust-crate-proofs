# Independent archive audit

Date: 2026-10-09

## Canonical capture

Audited `boxed-positive-final1-2026-10-09.tar.gz` (SHA-256
`04aa0ecfde6d47af4f9b250c39d6fbb74556958752a674d8552688612217acb5`). All
2,512 archive members are unique and their paths and SHA-256 digests match the
capture receipt. The archive contains 1,332 `probe/` members, 1,063
`inputs/repository/` members, 110 `inputs/private-std/` members, six
`inputs/tools/` members, and `run.log`.

The 97 selected `.coma` files and their proof summaries match the receipt. A
separate count over their proof trees yields 97 files, 510 prover leaves, zero
null leaves, and zero structural leaves. The receipt selects all targets, has
no exclusions or feature overrides, reports correspondence status 0, marks the
run non-diagnostic, and has no terminal-control feature. The archived log hash
is `d6f04c0727d610b0a2f7aec150cbfb511a08cbb0e301d4117b87ceaff20d81ee`; its
tail reports `Proved (97 files)`.

The archived native harness log hash is
`7b32b068c20d085abb379babe5ecbdd0870bb548ab6d06d121af72fca556cbe4`. It
reports five successful inputs (0, 1, 7, 63, and 1024). The captured native
MIR set contains 11 selected files. The capture pins rustc
1.98.0-nightly (`91fe22da8084a1c9e993d78d4a56f22ab8396236`) and
cargo 1.98.0-nightly.

## Reproduction and current inputs

I extracted the archive to a temporary directory, assembled the captured
`inputs/repository/` tree, and restored the probe at its documented repository
path. Against this reconstructed tree, the archived `check_correspondence.py`
returned `correspondence_pass`; it checked the source map/markers, native
source tables, 11 MIR artifacts, and module paths without consulting live
repository source. I also ran `check_checker_controls.py` on that reconstruction:
115 of 115 checker mutations were rejected, none were accepted, and the control
run did not invoke a solver. Both checks used captured inputs only.

Compared with current workspace inputs, all 1,063 captured repository files,
all 110 private-Std files, all six tool/config files, and `run.log` match. The
private archive is creusot-std 0.13.0; its 110 files match both its SHA256SUMS
and the installed archive fingerprint
`17ca7c53dcfac9b67abef67588f54286b153c1bc7e2d351a8722b9be48d22e8d`. The
captured Why3 configuration uses magic 14, one prover, a 1024 MiB memory limit,
and a five-minute time limit; Cargo patches `creusot-std` to the captured
private copy. The manifest pins tool versions and binary digests; the installed
tool binaries checked during this audit match those recorded digests.

Within `probe/`, the only current-workspace difference is `README.md`, updated
after capture with result/admission documentation. It is documentation-only;
the archived executable inputs and proof outputs match. No solver or native
harness was rerun for this audit.

## Negative controls

The seven archived semantic controls were checked against their receipts,
member hashes, and target counts. They are sensitivity runs, not canonical
acceptance: their checker receipts say `not_run`.

| Control | Files / prover leaves / nulls | Archive SHA-256 |
| --- | ---: | --- |
| missing free | 97 / 506 / 1 | `b3378a28c20f07be13d63eafc07ba0bd5f444e88b0bcbfbb1214ebb57dce3994` |
| missing untag | 97 / 527 / 1 | `faf5018e7e561b020114be456d824c81bb160443073e53f520788ddbdbc98f3b` |
| omitted Drop | 97 / 509 / 1 | `df037ada41d00c8d2ac1a3dd353c340ad2b37f8d249b0229d29bd46ebe4d8a1e` |
| raw as ARC | 97 / 515 / 1 | `9915965f9317e0a5030df1df790ddb5f00f4d8f416a568905f7b104b7cb9a193` |
| raw as static | 97 / 522 / 1 | `e68c73a220d913d3d7af697aec192d55c5fb6444877af9a6593271ba9d5136ac` |
| wrong pointer | 97 / 528 / 2 | `9f6250a54b50ae359ff001c90d28e30b1dbcacc2f15498c759a5e0d9ce7e740b` |
| wrong size | 97 / 525 / 6 | `e18b44b02203cb3e741bd005e3f1e8136da97e569fa9536b93bbf67d1e037722` |

The omitted-Drop printed null is the negated `to_vec` content-contract
condition (`not (returned Vec length/content matches original input)`), not a
standalone completion assertion. Treat it as a caller-gate sensitivity/resource
contract rejection; the separate structural omission mutation rejects the
missing terminal call directly. The missing-free and raw-as-ARC null goals
concern capability resolution/disposal, not a direct standalone theorem that
no receipt was issued. The missing-untag null checks exact allocation-pointer
premises; wrong-pointer and wrong-size failures reflect their respective
deallocation-base/layout-capacity conditions. These null counts are reported as
captured, not normalized to one.

Three type-only controls were also checked: duplicate physical free and
duplicate owner Drop fail with E0382; Drop before the borrowed read fails with
E0505. Each has zero `.coma` files and no prover invocation. They refer to the
diagnostic input archive SHA-256
`02c36bb7a947ae80e18a9b6426125f3c58ae592ee1bebbdc5fe4446f287acadb`.

## Scope and limitations

This is evidence for the boxed read and unpromoted normal automatic-Drop path
under the explicitly stated generic exposed-provenance, equal-distance,
terminal-place, read/free, and erasure assumptions. The captured proof maps the
empty input to a static representation and nonempty input to the even/odd raw
allocation cases, checks the pointer/capacity/content/tag/vtable bindings, and
retains the native ARC branch while showing it unreachable from the selected
read-only binding. The capability consumed at deallocation carries the physical
read/free authority.

The source/MIR/shadow terminal-place relationship and ghost erasure remain a
generic TCB assumption, not a mechanized compiler-correctness result. The
exposed-provenance tag round trip, equal-pointer `offset_from`, physical
allocation read/free, and read-only `AtomicPtr` interpretation are also
assumptions in scope. The native harness checks runtime examples, not parity
proof. This capture does not establish unwind cleanup, clone/promotion,
arbitrary moves or concurrent closures, other representations, full-crate
`From` refinement, or complete-target admission.
