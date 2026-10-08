# Constructor experiment handoff (label 2026-10-09; executed 2026-10-08 UTC)

Work only on `bytes-runtime-verification`; never main or force-push. Scope is
bytes1.11.1 and the actual upstream API/representation. The retired alternate
buffer API stays retired. Follow AGENTS.md, architecture decisions AD–AF and
shared proof locking/elevated execution. Generic physical/sync/tool TCB is
explicit; bytes ownership/refcount/lastness/destructor laws must remain body proved.

## Proven increment

The constructor representation sum preserves native four-field Bytes and the
Vec len==capacity Box optimization. Both From trait refinements are unconditional.
Actual constructor/read extraction plus arbitrary-input callers passes 21 files,
153 actual prover leaves, zero nulls, with no constructor exclusions. Canonical:

- `probes/original-public-constructor-gate-2026-10-09/evidence/constructor-positive-final-2026-10-09.tar.gz`
- SHA-256 `c31aa5b4f53fdbeeccfbbcb63a54d650bea18f0ea8365a0de6882d2c3f80849e`.
- Both wrong-vtable and wrong-extent controls fail their validity/read VCs, two
  null leaves each. The first parity-model failure is archived separately.
- Existing selected public Shared regression passes17files/189leaves/0nulls,
  separate from historical final6's17/267/0; sparse core v24's37/207/0 stays historical
  matching-source evidence.
- Native:1,014 non-doc tests pass;246doc tests pass on the pinned rustdoc rerun;
  no-default library build and3new Box physical-boundary checks pass.

New B1-BOX generic physical TCB ties the actual consumed Box::into_raw pointer
and allocation to affine Recovery/PhysicalRegion resources. Equal logical Box
content must never determine its address: pinned Creusot lowers Box<T> to T.
Astra caught that observer before proving; it was removed. Namespace freshness
is not numerical address freshness. Static permission comes from Std as_ptr_perm;
common read projection borrows exact pointer/initialized authority for its result
lifetime. Native table/tag/null and source-erasure adequacy remain TCB.

## Complete target remains blocked

Do not count this bounded distinguishing experiment as whole-crate admission.
The representation sum does not support promotable Clone/promotion or branch-aware
cleanup yet. Automatic Drop is still erased to Goto by pinned Creusot; explicit
cleanup evidence does not prove scope-exit destructor effects.

AF reviewed actual Std auth/lifetime/atomic/Arc APIs and examples. No capability
connects every completed Clone(&self) to an exhaustive issuance history. Retiring
all caller-known handles therefore cannot establish that no unseen owner remains.
Do not add a trusted bytes-specific last-owner/close theorem, exact-ID/quota
requirements, a mutable parent registry, or rerun unchanged frozen probes.

Reopening AD requires a concrete generic scope/effect/access discipline with a
native interpretation, supported or separately justified capability, and controls
rejecting escaped clones, unfinished callbacks and forgotten live handles.
Only after that and a viable destructor-effect treatment should the representation
sum absorb promotable clone/cleanup and the remaining actual API. The target is
blocked, not completed or implicitly reduced.

## Reproduction

Restore tools from their exact archived manifest/private Std. Current activation:
`/workspace/bytes-proof-tools/activate.sh`. Rust nightly2026-06-22;
Creusot318615be3b8bbc60d1f6d52469ba5c0bdebed4f1;
Why354c92f96bb0711d6e991c18f10bfbc08d90d028b;
why3findeab37557d3e24e1913a3c4f44bc5528ef497c6c9.

Run constructor `bash run-proof.sh` elevated, one prover,1024MiB,no sc-drf.
It cleans only this package before translation: Cargo cached feature switches
otherwise leave external verif outputs behind. The stale-feature replay failed
and is not positive evidence. Audit immutable archives with `evidence.py audit`;
independent source reconstruction is recorded in INDEPENDENT_AUDIT.json.
