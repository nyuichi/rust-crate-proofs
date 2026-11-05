# Independent source-only legacy compatibility results

This supplements the preserved phase1 review; its pending statements describe
that earlier checkpoint rather than the current results. No reviewer CLI edits,
builds, install, server calls or publication were performed.

The old frozen probe and new candidate probe were executed on the same isolated
frozen37 fixture. Their raw JSONL API-path outputs are byte-identical,37 rows.
An unrelated original nohash14 fixture likewise produces byte-identical14 rows.
Positive41 retains those first37 rows and adds four distinct singleton paths.
Exact binary/output hashes and comparison results are in phase2-comparison.json.

The new Rust configuration output for the absent/None option is exactly the
same29960 bytes as the canonical configuration extracted from the immutable
public37 SARIF. The optional ascii_vec_source_bindings field is absent. This is
an actual old-baseline/new-output comparison, beyond the earlier same-binary
state-isolation check. The baseline canonicalization provenance remains explicit
in PHASE1-REVIEW.md and baseline-provenance.json.

Read-only phase2 source inspection also confirms explicit edition2015 (including
the manifest default), a closed new-four request set, old37 legacy discovery and
separate merging of the typed four. No unconditional resolver change was found.
The dedicated scope validator checks fixed source/proof/evidence hashes, exact
41 mappings and pinned compiler/configuration conditions. These checks do not
constitute an independent final review of captured-publish integration or its
negative tests. Those publication gates remain with enum, PRNG reviewer and the
sole publisher; these regression results alone do not authorize publication.
