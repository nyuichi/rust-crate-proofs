# `http` 1.5.0 API and verification-support inventory

Generated from rustdoc JSON for the modified verification tree and compared line-by-line with the official crates.io archive. The origin field distinguishes upstream declarations from verification-support additions. Proof statuses default to `false`; the small set of accepted actual-source leaf results comes from `API_EVIDENCE.json`, and none sets `integrated_run`.

Declarations in the current modified tree: **917**.

| Declaration origin | Count |
|---|---:|
| upstream release declaration | 911 |
| verification-support addition | 6 |

| Kind | Count |
|---|---:|
| `assoc_const` | 79 |
| `assoc_type` | 75 |
| `constant` | 81 |
| `enum` | 1 |
| `field` | 12 |
| `function` | 494 |
| `module` | 7 |
| `reexport` | 121 |
| `struct` | 42 |
| `trait` | 2 |
| `type_alias` | 1 |
| `variant` | 2 |

The machine-readable per-entry ledger is [`API_INVENTORY.json`](API_INVENTORY.json). It records source location, mapped official-archive line when unchanged, declaration origin, exported path or paths, implementation identity for trait items, proof evidence, and `contract_reviewed`, `body_proved`, `trusted`, and `integrated_run` status. Evidence overrides are maintained in [`API_EVIDENCE.json`](API_EVIDENCE.json); leaf proofs never set `integrated_run`.

The inventory follows public module trees and re-exports, and includes source-defined methods and associated items from public types and traits. Each item identifies whether its declaration line matches the official release archive or was added as verification support; added View/DeepModel/Invariant/model helpers are not counted as upstream API. Trait implementation entries retain their rustdoc impl id, trait arguments, and `for` type so overloads remain distinct. It excludes inherited standard-library defaults and compiler-generated blanket/auto impls without an `http` source body.
