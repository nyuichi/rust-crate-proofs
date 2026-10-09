# AW-PACK independent audit

**Result: GO for evidence-closure transport and reconstruction, and replay of the captured AV audit/checker.** This audit does not enlarge the AV proof scope or admit full `bytes` behavior.

## Integrity and reconstruction

The raw manifest SHA-256 matches the externally supplied root pin `31d82fc5061a6112ee8031aed021c325f114001c93510ec4777d4c6acdf56288`; the frozen validator checks this digest before parsing or resolving the manifest. I independently checked the complete hydrated namespace: all 12,497 paths, sizes, and SHA-256 values match the manifest, with no duplicate paths, symlinks, or non-regular files.

The closure has 23 objects and 23 mounts. Its 12,497 logical files expand to 940,024,573 bytes. The accounting layers are distinct: 793,163,181 bytes of unique object payloads, 598,032,311 bytes of referenced published transport, and 1,519,092 bytes of newly published CAS payloads. The AV canonical archive has 2,164 members; every member’s path, size, and digest matches the mounted inventory. The origin archive has 2,128 members. All 167 proof targets (334 COMA/proof files) are byte-identical between origin and canonical archives.

All 550 critical-input identity rows are unique and match both archives. The reviewed production input set has 63 files (61 Rust files plus `Cargo.toml` and `Cargo.lock`) at base commit `361c7cd261507ac0a705b3b836f73240070891c6`. The six nested published ancestor archives (AP through AU) also match their declared digests and complete member inventories.

## Checks and replay

The frozen package consists of validator `ef7e19053b8c0db0495b83328a5901ec45578fac92f8eea7f576155bc8356e1c`, bootstrap `b1fa3420c02898671165b62905b2168a01cf22becc3ca9a6eeda089820414fd6`, builder `6e429b30c2ae2edc81048736f09f34341fb6c8236a4148e3fbbe28d9d06ea081`, parser test `0fe2eb82757d2459bcdf68d36af99f1047a78b63b8b9b6409bc770bb64ab7183`, and fixture `c24c28c9c7109e94974bc5a9370ed1f4d6fe162f92c77244796158cc189504be`.

The captured preflight reports 22 malformed-manifest cases rejected, a two-file round trip, rejection of a corrupted required object before checker execution, and rejection of a pre-existing hydration destination before checker execution. The fresh-destination offline replay reports exit 0 for both the evidence audit and archived correspondence checker. The checker ran in `--audit-compiled-capture-only` mode. No Cargo build, solver, prover, or source edit was performed for this audit.

The report index is verified (8 entries; index SHA-256 `6479055c3bebe32879604c0ddbfa100234a1524b18c9079101e07dc99f13e943`). The corresponding machine-readable details and per-report hashes are in [AW_PACK_AUDIT.json](AW_PACK_AUDIT.json).

## Scope limits

This validates evidence transport, archive reconstruction, and replay of captured AV audit inputs. The evidence reuses the previously captured AV proof results: 167 targets, 1,683 prover leaves, zero null leaves, and zero structural leaves; it does not claim a new proof run. The selected AV result concerns one bounded promote-then-first-`Clone` client. Full-crate `bytes` behavior remains not admitted. Tool executables and the original proof environment remain external dependencies and are not bundled in this closure.
