# httparse verification thread handoff — 2026-10-05

## Current resumption — restored archive and method UTF-8 checkpoint

This section is the current durable state; the environment and “unfinished
method preparation” statements below describe the earlier handoff snapshot.
The user asked to continue full formal verification, restore the archived work
in a fresh worktree, rebuild the proof tools, and push each reviewed commit to
`origin/main`. The restored worktree is `/workspace/httparse-cloud-resume` on
`httparse-cloud-resume`. All 1,593 entries in the archive passed
`FILES.SHA256`; this worktree has integrated the published `origin/main`
checkpoint `7a038aa`.

The safe-entry `parse_method` UTF-8 gap was fixed in commit `267fb46`. The
shared checked helper is `src/parse_method_utf8.rs`; GET, POST, and generic
token completion paths use it after the existing `slice_skip(1)` commit. A
bad retained prefix returns `Error::Token`. `parse_uri` already checks the
whole retained span with `core::str::from_utf8`. Native default and
no-default suites passed (105/101 unit, 263 URI integration, and 6 docs each);
four focused method/URI regressions also passed after the shared helper was
qualified as `core::str::from_utf8`.

Commit `71253be` records the rebuilt string-model proof profile and the
method-helper proof checkpoint; it is part of the published `7a038aa` history.
The exact setup and identities are in
`verification/probes/method-utf8/evidence/tool-rebuild-20261005/`. Compiler
SHA-256 is `29fcf8914166db07a910c45e4c8462f5665cfe4eee482509b885e629d3ff826e`
and matching prelude SHA-256 is
`cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b`. Fresh
translation and type checking are retained in
`verification/probes/method-utf8/evidence/translation-20261005T145116Z-12312/`.
The actual included `method_from_bytes` helper passed all 3 VCs, and the
representative outcome mapper passed all 3 VCs. Raw solver runs and their
source/tool bundles are in the adjacent `proof-20261005T145600Z-*` and
`proof-20261005T145626Z-*` directories. `core::str::from_utf8`'s frozen
standard-library contract is trusted; its Rust body is not proved. The mapper
is a harness caller, not the production scanner, so `parse_method`'s scanner
body and full crate proof remain open.

The five-scalar replay under the rebuilt profile is complete in
`verification/probes/utf8-lemmas/evidence/replay-five-scalars-20261005/`.
Its five targets were copied byte-for-byte from the recovered archive at
`verification/probes/utf8-lemmas/evidence/direct-why3-five-scalars-20261005/`;
the new directory preserves its own input hashes, tool identity, type-only
logs, actual commands, and fresh solver output. Fresh replay results are
6+7+7+7+8 = **35 Valid**, with no counterexamples; all five corrected target
runs exited 0. The rejected initial `-j 1` command is retained separately and
did not start a solver. The old archived proof logs remain historical evidence
and are not used as results for the rebuilt profile. These targets assume the
existing `utf8_byte_spec` numeric and `to_utf8_char_spec` length contracts;
neither universal encoding nor those helper bodies is proved. The replay
directory's `SHA256SUMS` validates all of its files.

Continue using the single-worker proof lock and 30-second / 1000-MiB bound.
On a future proof run, stop at the first non-Valid solver result for Astra
review.

## Codex Cloud: start here in a new task

The next thread may have a fresh container. Do not assume that this thread's
`/workspace`, `/tmp`, tools, caches, environment variables, agent threads, or
uncommitted files survive. GitHub is the durable handoff location:
`https://github.com/nyuichi/rust-crate-proofs`, branch `main`.

Read `httparse/1.10.1/cloud-handoff/README.md` first. Its committed archive
preserves 1,593 work-in-progress source/evidence files from this task, with
per-file and archive SHA-256 checksums. Restore to a separate worktree at the
recorded source base; do not overlay an advancing main branch blindly. These
are preservation artifacts, not newly reviewed or accepted proofs.

Absolute paths below describe the old container and historical provenance.
Discover the new checkout path and rebuild the pinned proof environment.
Do not rely on the old local-file link or on ongoing agents. All old agents
were stopped/completed. Resume orchestration with newly created agents.

## User instructions and resume procedure

Continue complete formal verification of httparse 1.10.1, not merely tests.
The primary agent itself orchestrates. Overall planning and difficult proof
consultation use Astra (`gpt-6-astra`, high); implementation uses Luna
(`gpt-6-luna`, xhigh). User explicitly requested those delegations and models.
Commit concrete audited milestones and immediately push every commit to
`origin` (`https://github.com/nyuichi/rust-crate-proofs.git`), `HEAD:main`.
Do not force push. The user called it rust-crare-proofs; the actual repo is
rust-crate-proofs. Communicate in Japanese.

Workspace: `/workspace/rust-crate-proofs`. Read applicable AGENTS.md and
`.agents/playbooks/verification.md`, then `VERIFICATION_STATUS.md` and
`VERIFIER-SUPPORT-CONSULT.md`. Read the cloud-environment runtime skill if
available. Inspect git status before editing: much valuable evidence and
in-progress source is uncommitted. Do not clean/reset it. At handoff no child
agents remain live; create fresh agents as needed. No solver lock is held.

**Full crate verification remains OPEN.** Isolated helper closures, translation,
type-only results, and compilation never imply full Request/Response proof.
The last verification commit pushed before this handoff was `61840c8`.
This handoff is committed separately; check git log for its hash.

## Committed checkpoints

- `6263082`: selected Error derived traits/native parameter environments:
  21 targets, 28 named VCs Valid.
- `de80723`: exact skip_spaces and Bytes dependency closure, 69 split Valid;
  actual helper 26. `7a5ccb5` includes it in main library.
- `f9e4db4`: Gate B static atomic invariant translation/proofs, 3 Valid,
  two deliberately wrong cases Timeout, read-twice literal true/no task.
- `b3b8202`: exact little-endian parse_version: 263 main-closure Valid leaves
  (actual parser 62), plus external array8 caller 7; two model true/no tasks.
  u64::from_ne_bytes contract is explicit standard-library TCB; big endian open.
- `a2453b7`: exact newline helper 78 Valid (Bytes 56, actual helper 22).
  `1993ffe`: Request call site uses it, default/no-default compilation passed.
- `61840c8`: Gate C fixed, unconstrained shared CPU capability symbols,
  effective rustc feature/cfg checks, positive translation and six rejections,
  eight COMAs type-only. No solver, detector/dispatch proof still open.
- Earlier selected parse_code closure 33 Valid; Bytes earlier 139, SSE pure
  overlay 18 Valid with five intrinsic TCB contracts. Counts overlap and must
  not be added into a fictitious total.

## Highest-priority finding: public safe method entry

Source-level safety counterexample, checked by Astra; no UB program executed:

```rust
let mut b = httparse::_benchable::Bytes::new(b"\xffGET ");
let _ = b.next();
let _ = httparse::_benchable::parse_method(&mut b);
```

Safe Iterator::next advances cursor to 1 but mark stays 0. GET fast path checks
bytes at cursor, then slice_skip(1) returns mark-based `b"\xffGET"` to
from_utf8_unchecked. Public-hidden benchmark warnings impose no Rust unsafe
precondition. Even unsafe advance(1) satisfies its documented bounds-only
requirement, but the safe construction is sufficient. Normal Request impact
is not established. Do not silently add proof-only mark==cursor to a public
safe function. Enforce a runtime boundary condition or validate every returned
byte, preserving intended behavior; review public parse_uri similarly. Full
diagnosis and options are in the end of VERIFIER-SUPPORT-CONSULT.md. A Luna
parse_method preparation task was interrupted for consultation; at that point
no finished method harness was known. The current remediation and proof
checkpoint are recorded at the top of this file.

## Uncommitted ready/reviewable proof milestones

### UTF-8

`verification/probes/utf8-lemmas/`: five concrete scalar cases U+00E9, U+20AC,
U+D7FF, U+E000, U+10FFFF passed 6+7+7+7+8 = **35 Valid**, all empty CEX.
Root independently decoded raw logs and checked 30 bundle hashes.
Evidence: `evidence/direct-why3-five-scalars-20261005/`. stdout includes a
queue-status line before concatenated JSON; skip that prefix when decoding.
These proofs assume existing utf8_byte_spec numeric and to_utf8_char_spec
length contracts. Neither universal encoding nor those helper bodies is proved.

`verification/probes/utf8-seq-laws-decomposed/`: local candidate closure
**15 split Valid** independently raw-checked by root: empty_len1,
singleton_shape3, concat_right_empty4, flat_map_empty3, guided_singleton4.
Original flat_map_empty middle child OOM; alternate sound Why3 transformation
proves the same child. Original source-clone singleton also OOM. Preserve both.
Exact transformations after split_vc:

```
inst_rem flat_map_T_def (empty:seq(t_T)),f
inst_rem flat_map_T_def (singleton(x):seq(t_T)),f
```

They retain a ground instance and remove the universal premise, strengthening
the obligation. Export-only preflight is not proof. Evidence contains raw
actual proof outputs and exports. Local checked lemma/source linkage to actual
standard-library helper remains open. Tail/snoc and associativity targets
typecheck only; no push-back bridge or literal callers closed.

README and VERIFICATION_STATUS were updated by Luna after root's initial
review. Selectively review/stage these two probe trees and historical
`utf8-seq-laws` dependency evidence; force ignored COMAs/Cargo.lock where
needed. Do not indiscriminately include target caches or unproved universal/
literal preparations. `utf8-char-universal/` and `utf8-literal-callers/` are
translation/type-only preparations, not solver successes.

### Header initialization/borrow adapter

Probe-only `verification/probes/memory-initialization/`; no main parser
integration. Typed Initialized/Uninitialized HeaderStorage tracks used_len,
whole Header writes, four nested-borrow prophecy projections. No owned Fin
builtin/trusted prophecy. Complete-only prefix publishing must preserve real
public Partial/Error field/binding effects and existing ShrinkOnDrop semantics.

Current adapter SHA256:
`abd48063e39a036d5df401341d4a9c18f6d8c034a00d7f5afaa4d40f12f38db7`.
Private generic retain_prefix uses mem::take + split_at_mut and checked
current/future prefix/tail contracts. Frozen translation:
`evidence/retain-prefix-translation-20261005T063730Z/` (19 COMAs, selected11).
All11 typechecked. Latest actual solver run:
`evidence/selected-solver-run-retain-prefix-20261005T071056Z/`:
first6 targets 35 Valid (constructors1+1, capacity4, used4, write14,
retain_prefix11); commit_prefix7 Valid +1 Timeout30s/497620632steps.
Total42 Valid,1 Timeout. Four clients not attempted. Root has not independently
audited this new run yet. Initialized grouped ensures passed; Uninitialized
group Timeout, whole COMA281–299, not one proven-failing clause.

Astra's latest plan: body-check a generic map/subsequence commutation lemma
using length/nth/extensional equality, then instantiate via a meaningful
nonprophetic current-sequence call before retain_prefix. Future values stay in
contracts/prophetic logic. A pure Option-prefix transfer lemma may help next.
Do not reintroduce nested snapshot! or executable prophetic ghost snapshots:
both failed translation and were explicitly withdrawn. Historical OOM/Unknown/
Timeout and failed-translation archives must remain immutable. Full standard
MaybeUninit permission boundary and actual callers are OPEN.

### skip_empty_lines

Uncommitted extracted `src/skip_empty_lines.rs`, model
`src/verification/empty_lines.rs`, `verification/probes/empty-lines-harness/`.
Not included in main library. Runtime equivalence:104 tokens, normalized SHA
`cc731c7e0eeb496982e24cd47b3e563091f7a49afbbf00ed0c7b07947bdc8296`.
Initial CRLF extension proof Timeout30s; Astra required structural induction.
19-target concat induction candidate translated/typechecked. Latest
`evidence/cursor-induction-prep-20261005T071604Z/` adds endpoint recurrence
postcondition plus explicit cursor-prefix induction:64COMAs, selected20,
**all20 type-only passed**, no solver. Root has not reviewed latest candidate.
Next: inspect exact checked endpoint/induction VCs, then bounded dependency-
ordered solver run. Boolean logic body goals do exist: an earlier diagnosis
of a missing bool VC was corrected with a false-bool diagnostic. No TCB bug.

### Gate D detector linkage

Uncommitted `tools/creusot-toolpatch/patches/httparse-static-atomic-gate-d.patch`
and `tools/creusot-toolpatch/static-atomic-gate-d/` layered on frozen Gate C.
Latest `evidence/final-20261005/README.md` reports positive normal/proof audit,
three COMAs type-only (CACHE/read_cache/runtime_detect), negative lookalike,
duplicate-site/profile/feature/cfg/caller-path rejections; **no solver**.
Root has not reviewed this newly completed evidence/implementation.
Pinned sysroot exports two distinct non-generic zeroarg detectors:
`__is_feature_detected::avx2()` and `::sse4_2()`. Authenticate exact external
DefIds/artifacts; each result is a fresh boolean with only true⇒matching fixed
StaticAtomicCaps capability. No equality across observations/false⇒unsupported.
CPU detection, OS-enabled state, and stability across threads/migration are
explicit narrow TCB. Preserve cfg guards against macro compile-time shortcuts.
Actual httparse detect/get/cache writer bodies and four exact backend
target_feature call preconditions remain OPEN.

## Toolchain and proof discipline

- Upstream httparse9f29e79f9832dbd0ae5220acb17c1866745bdecd; archive SHA
  6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87.
- Creusot upstream437d3d8d00b8114d7a3b4f7b8738d594a395f5bc; Rust nightly
  2026-02-27, sysroot source6a979b3e32522049d0acb4a47f7ae44b7c8abfd5.
- `source /workspace/proof-tools/activate.sh`, then source the isolated string
  env `/workspace/httparse-tool-rebuild/string-model/creusot-env.sh` inside
  proof child wrappers to restore its config/data/XDG/DUNE paths.
- Direct Why3, Z3 4.15.3, split_vc, fixed30s, max1,1000MiB. Negatives10s.
  Exclusive `/tmp/itoa-creusot-proof.lock`, single worker solver grant at a time.
  First nonValid stops; freeze evidence, consult Astra, change proof structure.
  Sandbox solver socket needs require_escalated; type-only needs no solver.
- Why3 `-o` only exports SMT. Use stdout redirect for real proof. Raw JSON/logs
  authoritative, no fabricated session/proof.json. Literal true/empty JSON
  means no solver task, never count as Valid. Why3find may double time limits.
- Avoid cargo creusot prove: it resets global Why3 max1 to4. Translate exact
  package freshly (clean package/dedicated target), cached no-files isn't proof.
- Exact isolated creusot package via why3find query; DUNE_DIR_LOCATIONS
  `why3find:lib:<data>/share/why3find`. Canonical global Why3 stdlib path is
  valid when isolated _opam symlinks there; hash source trees rather than reject
  it as stale. Compiler/prelude matching is mandatory.
- Original string compiler SHA a1ea923760d0225f828e90e2b405c4f7d3de4177868aad6f04f158472778f293;
  prelude cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b;
  Why3config e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07.
  Version/newline shared stdnum b8b83412491707e01c46a59cb60ca5da114ada63bf0bac0bf3e52464b678929c;
  original string seed stdnum b777d9fb2cc922148df8718b9a175d249de2174fc5d57f22cf24a63af91256d2.
  Use exact per-bundle provenance, not a newer binary with an older manifest.
- Selective staging; preserve raw generated bytes including whitespace.
  Exclude COMA/patch/rawlogs from source whitespace lint. Never add target,
  caches, ICE/core dumps, or unrelated experiments merely because untracked.
- Every commit immediately push. Latest sandbox push failed to reach proxy;
  require_escalated push succeeded. No unresolved push rejection.

Remaining major work: universal UTF8 and stdhelper/literal linkage, raw-load
permissions and uninitialized headers, outer Request/Response exact field
mutation on every exit/configuration, method/URI/reason/header parsing, SWAR/
AVX2/NEON and actual SIMD dispatch/load loops, endian/feature build matrix.
Maintain explicit trusted-boundary inventory; do not call the crate proved
until all required bodies/callers/configurations and public safety entries close.
