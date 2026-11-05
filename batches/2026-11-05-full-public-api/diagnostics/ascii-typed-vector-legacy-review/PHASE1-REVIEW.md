# Independent read-only review: scoped ASCII typed-four prototype

Reviewed source delta against ascii-vec-source-cli/source-before. Only config.rs,
aeneas.rs and scan.rs changed; main.rs, publish.rs and scanner_probe.rs are byte
unchanged. No CLI edit, build/install, publication, server call or credential
operation was performed by this reviewer.

## Confirmed phase1 boundaries

- Scanner carries a local bool defaulting false. Existing public entry paths
  call the new inner routine with false. No global/static mode was added.
- The only resolver delta is conditional: mode false preserves the previous
  import.target module-relative recursion exactly. Existing depth/cycle,
  ambiguous-import checks and rand_core identity exception are unchanged.
- Nested trait-argument public-path rendering is `!legacy || mode`; at false
  it is the old expression. The dedicated new entry enables true only for
  ascii1.1.0. No unconditional overall resolver change was found.
- Tool's optional ascii_vec_source_bindings has serde(default) and
  skip_serializing_if=Option::is_none. configuration() adds it only for Some.
  Thus existing missing/None configurations retain the old serialized fields
  by source reasoning. New serialization-focused byte checks remain a final gate.
- Production discover currently rejects Some with an explicit prototype-incomplete
  error. This is deliberate pending implementation, not a working publication path.
- The phase1 original-source fixture passed: exactly4 canonical source bodies,
  exactly-one matching each; old37 paths byte-identical before/after a scoped call.
  Both before/after scans use the same new binary's false-mode path. This tests
  mode state isolation, not independently an old-binary/new-binary comparison.

## Pending production requirements sent to enum/parent/publisher

1. Enforce original manifest/source/edition2015, including default-edition2015
   when unspecified. Current dedicated entry checks only package name/version;
   Project lacks an edition field. A differently edited2021 package with those
   names must not silently receive2015 use-resolution rules.
2. Restrict the scoped requested set to the closed new4, merge old37 from legacy
   discovery. Do not accidentally run all41 through the new-mode renderer.
3. Preserve None serialization/configuration bytes and unrelated-crate legacy
   behavior with focused regressions. Source-only mapping is not verification.
4. Complete final manifest/captured-source/commit/hash/theorem closure validation
   and negative tests before considering Some publication-ready. Publisher retains
   the install/fresh/publication decision; this phase1 is not approval for it.

## Independent actual baseline prepared

frozen37-fixture is an isolated copy of the specified frozen original41 packet.
Only the last4 proof mapping blocks are removed; original src and Cargo.toml
bytes are unchanged. Its exact37 proof configurations equal the37 entries in
immutable public run040721a1-0f98-46e5-8132-0d6295872dcf.

Old frozen scanner-probe SHA59109b6b953538cb7be503af6b9a560f3ae598823b53ba60f8e704d4851108b8
was run source-only on this fixture, emitting exactly37 JSONL path rows into
old37-api-paths.jsonl. Installed production CLI SHA032640e536faffa600f4345b14cc51b40d827c322745d286b225858d84612d68
was fingerprinted and not executed through mutating/publishing commands. Its
CLI has no equivalent source-only API-path output command; the frozen diagnostic
probe and immutable run are the appropriate separate evidence origins.

Old aeneas configuration was extracted only from the immutable SARIF's public
configuration field, not credentials/other machine state. The canonical JSON
is a compact UTF8 sorted-map re-encoding of that Value, not a literal substring
of pretty-printed SARIF. It contains no new optional field. The source SARIF,
fixture, old binary, installed binary and canonical config fingerprints are in
baseline-provenance.json. No secret fields were output.

New candidate probe/configuration comparison is **pending** receipt of the final
candidate binary from enum. Use exactly this same fixture for both path outputs;
compare raw JSONL bytes and canonical configuration bytes, retaining source and
binary fingerprints. Do not mislabel the source-reasoned omission or same-binary
isolation test as that old/new execution result.
