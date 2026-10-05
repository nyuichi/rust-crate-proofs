# HeaderName runtime behavior

Header character validation and normalization now use numeric matchers in the
parser paths. The H1 matcher lowercases `A` through `Z` and otherwise accepts
the existing token-byte set. The H2 matcher preserves the original lookup
table's behavior, including acceptance of `"` (byte 34) by
`HeaderName::from_lowercase`. That quote behavior is covered separately from
the RFC token grammar in `tests/header_name_anomalies.rs`.

The former H1 table remains in `MaybeLower::hash`. The H2 table is compiled only
for tests as a reference. A runtime regression test compares both numeric
matchers with their original tables for all 256 byte values, including the
quote case.

Standard header equality now compares per-variant ranks. The ranks follow the
enum's existing variant order, and a runtime test checks all 81 by 81 pairs
against `std::mem::discriminant`. Standard spellings still return the same
static lowercase strings; parsing tests cover all standard headers and their
uppercase forms.

These changes alter the implementation shape: byte normalization uses range
and value matches, and standard-header lookup checks the fixed spelling list
with byte equality. No performance benchmark was run, so this change makes no
throughput claim.

The focused runtime results and source fingerprint are recorded in
[`evidence/header-name-runtime.json`](evidence/header-name-runtime.json).
