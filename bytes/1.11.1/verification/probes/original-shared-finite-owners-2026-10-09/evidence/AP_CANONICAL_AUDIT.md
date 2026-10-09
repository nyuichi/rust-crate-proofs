# AP canonical independent audit

**Result: pass for this scoped gate.** This audit does not establish full-crate admission.

## Archive and proof results

- Archive: `ap-positive-canonical-v1.tar.gz`; SHA-256 `a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1`.
- Verified all 1065 unique regular archive members against their hashes; no member was missing or extra.
- The target list contains exactly 130 Coma files and all proof-tree hashes match: **1040 prover leaves, 0 null leaves, 0 structural leaves**. Prover distribution: alt-ergo 978, cvc5 7, z3 55.
- The canonical policy selects every archived Coma target and records no exclusions, features, terminal feature, or source-control mode; correspondence exit status is zero.
- Captured run log SHA-256 `43fa49d43895ff1b6c16db6f723df5018053ff4a531187954b5ff76209547a64`. It records one prover, 1024 MiB, native weak orderings, sc-drf disabled, and `Proved (130 files)`.

## Independent source and checker replay

- Reconstructed the archived sources under `/tmp/ap-canonical-repo`. All 63 files in `reviewed-production-inputs.json` match the captured hashes and current bytes worktree (base anchor `361c7cd261507ac0a705b3b836f73240070891c6`). All 110 private Std files match the installed tree and the captured Std source archive hash.
- The main correspondence checker passed and independently rederived a 51-block native CFG. Its 35 mutation controls all rejected as expected. The native checker passed over 20 MIR bodies (19 production bodies plus the client); its 76 controls all rejected as expected. These control runs invoked no solver, Cargo, or Rust build.
- All 8 captured tool binary hashes and 6 configuration-file hashes match their current installation paths. The four Cargo artifacts (fingerprint, build output, root output, and generated `public_records.rs`) match both archived captures and the receipt by SHA-256. No build was run.
- Four frontend controls are independently audited: duplicate peer (E0382), duplicate survivor (E0382), early survivor borrow/use (E0505/E0502), and snapshot extraction (E0277). The captured Std API requires `Plain` for `Snapshot::into_ghost`, and `Plain: Copy`; `Bytes` is not `Plain`. These controls contain no Coma or proof artifacts.

## Location and scope limits

The first all-scratch main-checker replay rejected because Cargo `root-output` embeds its original absolute OUT_DIR. I replayed the same archived checker and source against `/workspace/bytes-proof-tools/targets/bytes` after read-only verification that all four current files exactly match archive captures and the receipt; it passed. The gate retains an external, location-dependent artifact requirement and is not a self-contained toolchain reproduction.

The correspondence claim covers the closed finite runtime-count default-native client and normal completion. Concurrency/CAS-loser behavior, unwind, alternate configuration or representation, and full API refinement remain unproven. Native `Vec` and allocator storage/drop behavior remains generic Rust/Std/compiler/allocator TCB; no receipt for Vec storage is claimed.
