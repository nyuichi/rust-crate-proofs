# bytes 1.11.1 coverage ledger

`ledger.json` inventories local Rust function bodies from
`evidence/adapted-whole-final/crate.llbc` and maps claims only when their theorem
names appear in a kernel-check receipt. `build_coverage.py` only reads existing
artifacts and regenerates the ledger; it does not start proofs or extraction.

The translated file has 706 local function records: 469 transparent and 237
opaque, exactly matching the frozen opacity inventory. These records include
25 global/constant/vtable initializer slots and 6 compiler-generated `DropGlue`
records, which are listed separately and are not treated as source Rust
functions. Vtable initializer records are the function-shaped side of trait
dictionary metadata; trait implementation metadata is counted independently.
The source-function inventory is therefore 675 declarations (450
transparent, 225 opaque), including public APIs, private implementation
helpers, and macro-expanded methods. Charon marks 234 source-generated methods
through `generated_from_span`: 216 are the Buf/BufMut deref forwarding macros,
and 18 come from indexing/formatting macros. Opaque source function bodies stay
`blocked`; a transparent body without a mapped receipted theorem stays `not
yet specified`. Globals, promoted/anonymous constants, type declarations,
trait-implementation metadata, and generated drop glue remain separate from
the source-function denominator.

Proof status is per mapped property. `proved` means the checked claim has no
explicit preconditions or extra model axioms. `conditional proved` records
explicit callback/content hypotheses or nonstandard model-axiom dependencies.
It does not claim the whole function is verified when the claim covers only one
property. Elicit audit entries marked full count as function-contract proofs.

## Extension manifest

For extension proofs, add `verification/extensions/<group>/manifest.json` using
this shape. The `receipt` path is relative to the group directory; the receipt is
authoritative for theorem compilation and axiom dependencies. Each Rust mapping
must name only a source body covered by the listed theorem. `source_span` is
optional and disambiguates same-named or macro-generated functions.

```json
{
  "schema": "bytes-coverage-extension-v1",
  "group": "readers",
  "receipt": "extensions/readers/receipt.json",
  "proofs": [
    {
      "theorem": "BytesProofs.Readers.example",
      "rust_functions": [
        { "target": "bytes::buf::buf_impl::Buf::try_get_u8", "source_span": "src/buf/buf_impl.rs:..." }
      ],
      "verified_behavior": "The claim proved about the translated body.",
      "assumptions": []
    }
  ],
  "blocked_roots": [
    { "rust_function": "bytes::...", "reason": "compiler or specification blocker" }
  ]
}
```

The receipt should list successfully kernel-checked theorem names, per-theorem
axiom dependencies, and SHA-256 hashes for the generated/proof/manifest/audit
files. The ledger checks these hashes against current files before accepting any
claim, so edited proofs cannot inherit an earlier successful receipt. Helper
lemmas without a Rust body mapping, uncompiled names, and self-equality
tautologies do not count as implementation coverage.
