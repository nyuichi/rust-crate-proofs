# Diagnostic controls

These immutable captures are development controls, not canonical acceptance.
All six semantic controls include all 77 Coma targets and exactly one null leaf.
Their correspondence receipts deliberately say `not_run`; the separate final
62 structural mutations are rejected by the frozen checker.

| Capture | Actual proof statistics | Archive SHA-256 |
| --- | --- | --- |
| automatic-negative-omit-second-v1 | {'files': 77, 'prover': 448, 'null': 1, 'structural': 0} | `f9ade77bb486021619a95a65f4ba59e53f66fed4e1f22a8a2cb8c4f56eebfe0f` |
| automatic-negative-omit-first-v1 | {'files': 77, 'prover': 449, 'null': 1, 'structural': 0} | `6adc9f30fe5b82edcc1ea17bcf482280420cfb339abfb0a948d909f2f4969545` |
| automatic-negative-swap-places-v1 | {'files': 77, 'prover': 455, 'null': 1, 'structural': 0} | `2f945819b599a3000389bda3e89267b84b06a5a630ea31dc81d771c620d04230` |
| automatic-negative-missing-acquire-v1 | {'files': 77, 'prover': 472, 'null': 1, 'structural': 0} | `76d1e3aa9de6c57f357b07d81fc217d3d37fc6f6085a80687ef847b14f652af7` |
| automatic-negative-missing-payload-free-v1 | {'files': 77, 'prover': 451, 'null': 1, 'structural': 0} | `cd5b3568ad9badc364c0d54bb3142a0fe0bfbedce796ce30e6c14a8ae73859a0` |
| automatic-negative-missing-control-free-v1 | {'files': 77, 'prover': 453, 'null': 1, 'structural': 0} | `40e99a5caaeea7f9ed0bf6180e0ec796976c4d8faf6cbb53ae1ee1f8406e525f` |

Omitting second or first Drop fails the caller's completion assertion. Swapping
places fails the owner-specific map removal assertion, even though the first
release could otherwise say KeptAlive. Missing Acquire fails the checked release
body. Missing either free fails free_recovered. As in AI, the missing-payload
null normalizes to an AtomicUsize invariant precondition: this demonstrates
sensitivity/rejection, not a direct theorem that the buffer receipt is absent.
Standalone printed caller tasks are retained alongside these archives.

Three type-only captures record rejection before VC generation: duplicate
second and wrong place yield E0382; early second while its slice is still
borrowed yields E0505. Each has zero Coma files and no prover invocation. Their
manifests reference the complete diagnostic input archive for external pinned
inputs and do not reuse a stale solver target manifest.

The first positive diagnostic (77/445/0) predates the finished correspondence
checker. Its status remains diagnostic. The canonical positive replay is recorded
separately after positive source restoration and the frozen 62-control suite.
