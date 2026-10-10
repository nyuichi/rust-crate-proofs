# Independent Builder proof-tree audit

Auditor: Astra (read-only JSON and split-task audit; no solver run).

- 15 selected target roots, all closed with no null or missing leaf.
- 46 proved own terminal JSON leaves.
- 36 additional support leaves and 14 empty support trees excluded from the own-leaf total.
- Own leaf counts are recorded per target in `manifest.json`.
- `Uri::from_parts` nested parents `[4]`, `[11]`, `[16]`, `[22]`, `[23]` each have exactly two children; second-printer mappings are respectively `[4,5]`, `[12,13]`, `[18,19]`, `[27,28]`, `[29,30]`.
- Nested branch contexts were reviewed for equivalence; no omitted child was found.
