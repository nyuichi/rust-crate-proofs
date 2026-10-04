# bytes runtime proof checkpoint

Full verification is **not complete**. The 0.13 migration and initial soundness
repairs produced genuine isolated implementation proofs. Runtime integration is
blocked on verifier support/model obligations; it does not pass the old length
model as a substitute.

| Component | Contract reviewed | Body proved | Trusted | Runtime integration |
|---|---|---|---|---|
| saturating_sub_usize_u64 | yes; fixed | yes, exact runtime source | no | blocked |
| min_u64_usize | yes | yes, exact runtime source | no | blocked |
| general callers and helper boundary examples | yes | yes | no | isolated |
| Box allocation/read/recovery | yes | yes | standard pointer primitives | foundation only |
| borrowed slice permission split | yes | yes | standard pointer primitives | foundation only |
| free_boxed_slice extracted runtime body | yes, Astra primitive review | yes | standard pointer/layout/allocator primitives | Bytes callers not connected |
| real Box -> suffix -> explicit deallocation caller | yes | yes | same primitives | isolated |
| restricted impure trait self-bound patch | candidate reviewed | positive body/caller proved | experimental compiler | not adopted |
| Bytes/BytesMut/API/refcount/vtable/Drop | incomplete | no | not assumed | blocked |
| legacy length/capacity model | historical | not rerun | excluded from runtime coverage | not counted |

Negative proof VCs fail for wrong helper postcondition, reachable assert(false),
byte+1, ownership-free Box recovery, incorrect deallocation layout/capacity, and
false bodies under the experimental patch. Translation restrictions remain for
logic/associated-type/ghost/termination/mutual/contracted trait cycles and stronger
recursive implementations. See logs and manifest for exact executed outcomes.

Default-std ordinary upstream tests/doctests pass on the baseline and migrated
runtime. No repository-wide test/proof run, optional-feature run, 32-bit proof,
Miri, Loom, Verus proof, push or external publication has been performed.

Reproduction: `scripts/verify-bytes.sh helpers`, `storage`, `deallocation`, and
`./verify-all.bash` (the last currently reproduces a runtime translation blocker).
Proof commands require elevated execution because Why3 uses Unix-domain sockets.
Artifacts: `verification/artifacts/`; installation/full logs additionally under
`/workspace/bytes-proof-artifacts/`.

## Subsequent component progress

- Bounded adapter helpers: five Coma files prove on vanilla 0.13 (minimum,
  arbitrary-byte prefix, budget decrement and representative callers).
  The wrong-prefix postcondition translates and fails its intended VC.
- Take, Limit, Reader and Writer now call those shared helpers in their ordinary
  runtime. Their generic trait-level composition is not yet proved.
- Default-std ordinary tests and doctests pass after these refactorings; see
  `verification/artifacts/logs/component-runtime-tests.log`.
- API inventory is a compiler snapshot from before these new extractions; it
  must be regenerated before the final component audit.
