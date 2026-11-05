# Ten original-source Aeneas verifications — 2026-11-05

All ten selected crates have unchanged crates.io Rust sources and Cargo manifests. Verification targets individual APIs, not complete crates. The ledger records exact source commits, mapped APIs, theorem preconditions, guarantees, limitations, run IDs, and publication state. Forty API mappings are covered; derived standard trait implementations are excluded.

| Crate | Version | Established scope | proofs.rs |
|---|---|---|---|
| oorandom | 11.1.5 | Exact Rand32 next-word output and state update | [25](https://proofs.rs/report/25) |
| endian-type | 0.2.0 | Six generic bitwise wrappers preserve underlying operator results | [26](https://proofs.rs/report/26) |
| either | 1.19.0 | Seven branch tests, projections, defaults and flip; two-flip composition | [27](https://proofs.rs/report/27) |
| rand_pcg | 0.10.2 | Exact Lcg64Xsh32 next-word output and state update | Awaiting catalogue quota |
| rand_xoshiro | 0.8.1 | Exact Xoshiro128Plus next-word output and state update | Awaiting catalogue quota |
| base16ct | 1.0.0 | Encoded length under no-overflow condition; decoded length parity | Awaiting catalogue quota |
| ascii | 1.1.0 | Exact digit, blank, whitespace and control classification | Awaiting catalogue quota |
| byteorder | 1.5.0 | Eight fixed-width unsigned endian writes, sufficiently long slices | Awaiting catalogue quota |
| byteorder-lite | 0.1.0 | Eight fixed-width unsigned endian writes, sufficiently long slices | Awaiting catalogue quota |
| bit_field | 0.10.3 | u8 get_bit/set_bit at indices 0–7, including returned-reference behavior | Awaiting catalogue quota |

All fresh recorded runs compile target proof modules from source and audit theorem dependencies. Only pinned official dependency caches are restored. No local theorem axiom is added; audited dependencies are limited to `propext`, `Classical.choice`, and `Quot.sound` (some proofs use fewer).

The endian wrappers assume arbitrary underlying operator dictionaries; they do not prove those operators. PRNG proofs do not establish seeds, periods, distributions or cryptographic properties. Byte writes do not cover reads, I/O or short buffers. The bit_field proof uses one guarded executable `size_of::<u8>() = 1` model with supplementary Kani checks and an explicitly reviewed Rust-to-Lean bridge. The ascii release has one unrelated `is_digit` radix-0/1 test incompatible with the pinned modern Rust; its failure and the remaining successful tests are retained. See each crate's `proofs.toml` and `VERIFICATION.md` for the full applicable domain.

## Why seven reports remain unpublished

A new catalogue preparation returned HTTP 429 `daily_limit`. The inspected public server source limits new import jobs to five per user and fifty per IP hash per UTC calendar date. Existing ready catalogues continue to work, permitting the three reports above. The seven remaining names are absent from the public imported-crate listing; their checked exact API spellings return 404.

The next bucket is expected at 2026-11-06 00:00 UTC. Under the inspected limits, five remaining new imports can be prepared on that date and two on 2026-11-07. The production deployment revision was not identified, so this reset expectation is source-based, not a guarantee from a server reset header. Raw 429 headers were not archived. [Quota evidence](service-quota.md) includes exact source links and limitations. No alternate identity or quota bypass was used.

## Resume publication without re-running completed proofs

`runs/<crate>/run.sarif.json` contains each unchanged recorded SARIF. Its SHA-256, original source commit and run ID are in `publication-ledger.json`. No authentication configuration or token is included. `resume.py` restores these exact files to cargo-proofs' Linux local-data run directories, preserving run IDs and source references. The CLI checks recorded crate/version, tool configuration and proof mappings before publication.

In the existing environment:

```bash
source /workspace/aeneas-tools/activate.sh
export PATH=/workspace/proof-tools/bin:$PATH
export PROOFS_CONFIG_DIR=/workspace/proof-tools/config/cargo-proofs
cd /workspace/publish-batch-ten
python3 batches/2026-11-05-aeneas-ten/resume.py
# At or after the quota reset, explicitly execute at most five pending entries:
python3 batches/2026-11-05-aeneas-ten/resume.py --publish --max-new-imports 5
```

The first command only prints the plan and sends no requests. The explicit publication command performs a dry run, publishes, reads back the result, saves logs, and updates the ledger after each success. It stops on service failure without retry. It does not schedule future work. Commit/push the updated ledger after successful publication. Then use the Google Sheets connector to update the source spreadsheet: for each newly published crate, replace only its H-cell quota-hold text with the verified scope and report URL and set its B cell to `nyuichi`; retain all other cells and formatting and read both cells back. Sheet row numbers and previous updates are archived in `sheet-status.json`. The resume script does not access Google Sheets and does not perform this remaining step. The source spreadsheet is https://docs.google.com/spreadsheets/d/1AlA2DtRJt28UipNFOhTdFpv6szVUgPQcjAew8L8hE6Q/edit .

For another Linux checkout, retain this batch directory and all referenced source commits. Install the same patched cargo-proofs 0.4.1 and authenticate normally; use that environment's ordinary trusted CA configuration. Obtain the official `cargo-proofs-0.4.1.crate` archive from crates.io (SHA-256 `38e4254b7640667c8e226bc8f1b889ade4167c2a6c9ca40673bbf6c239ba8591`), extract it and apply `cli/source-discovery-and-audit.patch` with `patch -p1`. Build `cargo build --release --bin cargo-proofs` and place that binary on PATH. `cli/environment-ca.patch` is only for the managed environment's documented proxy CA path; it retains TLS verification and is unnecessary on an environment with normal CA trust. The exact installed binary used here had SHA-256 `9073457b02b79b2650f98fa54b17b9c0ada1e6b349656aa174b22d9938147a66`.

The source-discovery fixes recognize known optional cfgs, preserve the exact rand_core identity reexport, discover generic inherent methods, check only explicitly selected Aeneas API mappings for uniqueness, and use official rustc expansion for macro-generated APIs. Known standard bit-operator catalogue aliases still require a distinct self type/method and exactly one resolved API ID. The upstream-compatible Lean audit variable fix does not change the allowed-axiom policy. Correspondence remains author-supplied, as with the official Aeneas integration; these compatibility fixes do not expand proof guarantees.

Individual immutable run uploads, if publication is resumed manually, are performed by the supported CLI command from the corresponding `proofs/` directory:

```bash
cargo proofs publish --manifest-path ../Cargo.toml --run <ledger-run-id> --dry-run
cargo proofs publish --manifest-path ../Cargo.toml --run <ledger-run-id>
```

The final command uploads the recorded SARIF through the normal authenticated API; do not edit its contents or fabricate a new run ID. If configuration is changed or record validation fails, produce a fresh target-scoped run instead.
