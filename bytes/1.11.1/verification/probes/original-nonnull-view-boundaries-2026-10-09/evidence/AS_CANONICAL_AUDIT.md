# AS canonical archive audit

**Result:** the immutable archive reconstruction and selected-gate checker
replay passed. This is not admission of the complete bytes crate.

I verified the archive and replayed its checkers from captured inputs under
`/workspace/work/as-canonical-replay`. `CARGO_TARGET_DIR` was unset. No Cargo
or Rust build, proof run, or solver was invoked. For full ancestry replay, the
omitted `verif/` working-tree directories were restored from the embedded
AR, AQ, and AP canonical proof archives. This restored captured outputs only;
all source, COMA, and proof inputs came from the AS archive or its embedded
canonical ancestors.

For the full main-checker replay and its controls, I patched
`C.assert_live_build` in memory to call `C.assert_compiled_capture_only`.
This checked the archived Cargo artifacts and their path joins without
accessing the live target. Absolute paths in the receipt were preserved as
recorded; no target files were statted or rebuilt.

## Archive and proof results

Archive SHA-256:
`969a1a0406e10677c1849200284881e043aae9c9bdf29c9b15609783c91a9f74`.
All 1,421 unique regular members match the receipt. The archive has 405 probe
members, 898 repository inputs, 110 private Creusot-Std source files, 7
tool/config metadata files, and one run log. I rehashed all 150 COMA/proof
pairs; the target list exactly matches all archived COMA files. The proof
trees recount to:

| Files | Prover leaves | Null leaves | Structural leaves |
| ---: | ---: | ---: | ---: |
| 150 | 1,328 | 0 | 0 |

The policy records no exclusions, no features, checker status 0,
`diagnostic: false`, and empty terminal/source-control settings. The log ends
with `Proved (150 files)`. It records one prover, a 1,024 MiB memory limit,
weak native orderings, and sc-drf disabled.

## Source and ancestry

The AS checker SHA-256 is
`e5ba544d0a3cdc4bbb0ef2e27a033ad4a0507a8bcedb1382c6910ad22417ce68`. Its
archive-only replay returned `pass`; the baseline inside the main-control
suite also passed. It reconstructs exactly these three changes to the AR
positive source: add the global nonnull conjunct to `view_valid`, add a
nonnull precondition to `borrow_empty`, and add a nonnull precondition to
`wrapping_bounded`. The remaining AR modules stay byte-identical, and the
native operations are unchanged. Active source SHA-256:
`9fbe1698c4a1a338c7bc30eeb60af8b69bc4077304b63acab9b3be34a7650744`.

The AS capture contains 61 production Rust files plus `Cargo.toml` and
`Cargo.lock` (63 selected package inputs); its additional `Cargo.toml.orig`
is also captured. The 61 source files and two selected manifests match the
bytes embedded in the AR canonical archive. All 110 private-Std files also
match AR byte-for-byte.

| Ancestor | Archive SHA-256 | Members | Targets / prover leaves |
| --- | --- | ---: | ---: |
| AR | `b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7` | 1,312 | 150 / 1,328 |
| AQ | `41e8e9c164a1110c7f611bb1726f490e111c6b10a77af6b57bfc707ce49ceba5` | 1,178 | 139 / 1,202 |
| AP | `a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1` | 1,065 | 130 / 1,040 |

The full main checker controls replay rejected 36/36 mutations with no
errors. Checker-control script SHA-256 is
`3ca3db45d9c561468cea2bd5c65d64371b02f3adb664cc4ee0c4a2771894ca3e`; fixture
SHA-256 is
`793bfcffa3c51f1e640127f0f772b2e62f40dfad959be0c498ff2be83cac4405`, and
receipt SHA-256 is
`80a5a039aea06663d3357382189cc2c3a5bb3bd7f82eae5a5f749957f4db379b`.

Native evidence is inherited unchanged from AR: its 30 selected MIR bodies
and capture metadata match byte-for-byte. Native checker SHA-256 is
`a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f`. I
replayed its in-memory mutation suite: 76/76 rejected, zero accepted, zero
errors. The replayed fixture and receipt match the archived hashes
`e79368d7c01de089c510ec0c3bc1afb2dec846335fd9bb87fe9d050b52ccd906` and
`ed703c3bc26743f73d8aef9f5b9fc42335a25ade73d0b8a2fb74c2bf1023ed37`. The
captured native harness log records its test passing; it was not run again.

## Cargo capture and tools

The archived checker reconstructed `public_records.rs` and validated the
four captured Cargo artifacts, exact source map, build directives, and
fingerprint/output/OUT_DIR/root-output joins:

| Artifact | SHA-256 |
| --- | --- |
| `public_records.rs` | `41eeb72bd7a4e3b59f042311508a436033c1a01e227035beb955bec749a8169b` |
| Cargo run-build fingerprint | `679d8454d0d7b5064046f4fb1479d2967720bf17599ead1b2b8dc1f0af851294` |
| Cargo build output | `0cab1e3220b72008729103a994de3b490da5f9fd8e7148efd9f8dd5f9f508344` |
| Cargo root-output | `8889bfe7499e3ba5c46c412a367c0e4880dad302ebdcff60a07fd5a36b853d79` |

The separate Cargo receipt SHA-256 is
`7f7320650851ae9f47d3aacde93a7f516b6a72a2943afd42143dbd480b0c540a`. The
captured target path is `/workspace/bytes-proof-tools/targets/bytes`; the
audit preserved and checked the original recorded absolute paths using the
capture only.

The captured installation manifest SHA-256 is
`008da7aa021c330a7d2b9f376957ffdcc4a5f807aba5a4d6f6e07a18c4f053b5`. It
records 8 binary tools, two Why3 configurations, one prover, a 1,024 MiB
limit, and a five-second limit. The binaries themselves are not in the
canonical archive. Root separately hashed all eight external installed
binaries against the captured manifest; the report is
`AS_EXTERNAL_TOOL_HASH_CHECK.json`, SHA-256
`12eedb18802334e1c0e8a354498de17e89672078e3dcbce702d3035c6b45664e`, and
records that all eight match. Four tools were marked different from the
manifest's *prior* values; that is historical and does not indicate current
binary drift. The external hash report supplements the archive metadata; it
does not make the executable payloads self-contained in the archive.

## Scope limit

This audit supports the selected AS nonnull-view source/proof/native gate and
its archived build-input capture. It does not admit the full crate, all-domain
`From` refinement, concurrency, or unwind behavior. Tool and Cargo paths are
location-bound.
