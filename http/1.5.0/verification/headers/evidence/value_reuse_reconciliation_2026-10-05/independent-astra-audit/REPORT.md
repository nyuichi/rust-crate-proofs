# Value historical proof reuse audit

PASS for exact context reuse into the archived Value emission, source `197fe1071f054c5f018fe1a8cc21266d679164a77088b9459eeb2ee3dd34330e` (Name input `966107ba5e8df15aa82c4d34220ab3a7b8e4e7da7b4dfc4a050cff1232dcb064`).

| Prior evidence set | Selected COMAs | Initial tasks | Proved terminal leaves | Null | Nested nodes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Complete translation v7, Value `1ab32986…` | 67 | 183 | 184 | 0 | 1 |
| Bounded profile 5420, Value `5420fc76…` | 18 | 55 | 55 | 0 | 0 |

The selected target sets have an empty intersection and a union of 85 distinct current COMAs. Proof-leaf counts remain per evidence set; no aggregate proof count is asserted. Imported/support zero-child markers are listed and excluded.

For every target, the old proof JSON and old COMA match their snapshot hashes. Full unnormalized Why3 `split_vc` stdout from original old/current COMAs is byte-identical. Independent Why3 API extraction selects the exact owned goal and reproduces every ordered direct context, including unsuffixed child 0. The current 181-entry source ledger and 333-entry emission ledger match all files. All fifteen 5420 captured source files match their hashes. The v7 snapshot records seven source hashes but originally contained no source copies. Luna recovered the exact v7 Value source from Git commit `dfcdce85cbc4b6dbbbede626f3e5c811dbbee6ca`; its archived bytes independently match both Git and source SHA `1ab32986…`. The old Name and Creusot source bodies remain hash-only provenance in the v7 snapshot. This archive gap does not alter the independently checked old COMA/proof hashes or exact current proof-task identity.

The sole nested v7 proof is `are_visible_value_bytes`, direct child 8. The original-COMA API independently produces initial arity 9, then selected-parent arity 2 in both versions. This is a binary split: initial 9 becomes terminal 10. The nested parent is 31,808 bytes with SHA-256 `839444b87ae24da917b0762d7157a1fc41b789d8d416dbced5f974e2ee8978c6`; its two children are 31,827 bytes / `e372d32e7e31d73367e3bfbbba4abd65c5c8b0957c307e50c01af69f57122950` and 31,828 bytes / `e0effa852906f28f2216f5e2ec67b7ab8070ffc648d45ed1fbcfa54192594236`. Both terminal proof nodes are successful, and every root, parent, initial child, and selected nested child is byte-identical old/current.

All raw stdout, complete contexts, task ordering, proof hashes, source bindings, and zero-marker lists are preserved in report.json and compressed artifacts. The scripts use original COMAs, never reparsed printed `.why` contexts. No solver, frontend, or source edit was performed.

Reuse preserves each old contract and its imported assumptions. It does not prove those imported bodies, strengthen postconditions, or claim a joint fresh emission with the later Name `fa9742…` source. The separate formatter proof batch remains append-preservation scoped; this reconciliation does not turn that result into exact rendered-text correctness.
