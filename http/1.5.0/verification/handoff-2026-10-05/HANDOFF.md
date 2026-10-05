# HTTP 1.5.0 verification handoff — 2026-10-05

## User authorization and execution model

Continue all feasible actual-source HTTP verification to the limit. Assume bytes
1.11.1 is fully verified as the user's explicit counterfactual premise. The
locally verified itoa 1.0.18 contract is another dependency premise. Do not wait
for dependency/Creusot changes where a local proof can still progress.

Root orchestrates and checks evidence/Git. Actual source/proof work must go to
**gpt-6-luna, xhigh**; consult **gpt-6-astra, xhigh** for blockers and independent
audits. The initial Astra plan was already done. Push **after every commit** to
`origin/main`, without force push. This is already authorized; do not ask again.
The user requested stopping this thread and continuing in a fresh thread.

## Repository and saved work

- Repository: `/workspace/rust-crate-proofs`, branch `work`.
- Origin: `https://github.com/nyuichi/rust-crate-proofs.git`.
- Last accepted published commit before this handoff: `98abda83d2d0a2b3aa632ef9c852dc170eb6e11d`.
- All commits below were individually pushed successfully.
- The worktree has uncommitted **unproved** source preparation. Do not reset it.
- `manifest.json` pins 77 files copied in `unproved-worktree-snapshot.tar.gz`;
  `worktree-diff.patch` and `git-status.txt` preserve the pending changes/status.
  The archive is a recovery artifact, not a proof claim. Same-workspace resume
  should use the existing worktree. For a fresh checkout, inspect the archive
  manifest and extract it at repository root only after checking for newer work.
- There is an applied but retained stash `http-checkpoint-shared-support` from
  integrating remote `61840c8`; do not apply it again or discard blindly.
- Many old exploration/ICE/failed directories are untracked. Do not blanket-add
  the verification tree. Stage exact accepted evidence and proof-time sources.

## Published progress in this segment

| Commit | Result and important qualification |
| --- | --- |
| `f2ccba2` | HeaderValue remaining numeric/TryFrom: 18 targets, 55 own leaves, all closed. Fresh source 5420fc, all 106 COMAs archived. |
| `09f7682` | Actual HeaderMap `find_with_hash`: 1 own + 8 support leaves. Conditional nonempty ready-map premise; `Some` bounds/hash/key relation. **Not** None iff absent or constructor/mutation closure. |
| `84ddeb4` | Actual URI Builder/Parts/Error helper: 15 targets, 46 own + 36 support leaves. Correct eight Parts presence cases and error precedence; callback contracts. Fresh gated source full printed tasks identical to prior complete proofs. Native Builder tests 8/8. |
| `2b167e4` | HeaderName source650: manual Debug/rank 9 targets, 144 own leaves; refreshed baseline23/209; 19 exact reusable targets192. Native Debug parity passed. |
| `611032a` | Request/Response Builder relays/shortcuts: 35 targets, 116 own + 164 support =280 complete leaves. Two FnOnce stubs are support. Current source reconciled by exact full printed task/context identity for9, raw COMA identity for26. |
| `71221f5` | Correct HeaderName snapshot summaries to point at accepted23/209 and19/192 reuse; preserve initial capture fingerprints. |
| `782f9de` | Error formatter bodies4/15 own proved; trait refinements4 incomplete. Preserve HeaderValue Debug failed attempt separately. No complete-formatter claim. |
| `98abda8` | Astra independent HeaderName audit: exact full printed streams + initial/nested arities for28 targets336 own leaves, all complete. |

Earlier published `a7a3d35` includes construction/default composition15 targets
(15 own26 support), and exact production frontend snapshots whose remaining19
errors are dyn/experimental limitations. **Current WIP has changed since those
snapshots:** do not call the current full crate proved or frontend-only19 without
a new exact check. Production default and all-features checks emitted no COMAs.

## Evidence to read

All paths below are relative to `http/1.5.0/verification/`.

- Value: `headers/evidence/archived_snapshots/value_profile_5420_full_emission_v1/`.
- Name baseline: `headers/evidence/archived_snapshots/name_ghost_bounds_reemission_v1/`;
  `proof_batches/name_frontier_run_v1.json` holds23/209, including parse nested split.
- Name Debug: `headers/evidence/archived_snapshots/name_manual_debug_refinements_v1/`.
- Name current650/reuse/independent audit:
  `headers/evidence/archived_snapshots/name_source_650_full_translation_v1/`.
  `independent_astra_name_9_19_task_audit/report.json` SHA
  `1f59732c8991fbe1867fda0f7d732a807d84e86f738f2776fc14b07b1eb45f82`.
- Map: `header-map-api/evidence/run-2026-10-05-find-with-hash/`.
- URI: `uri/evidence/uri-builder-api-2026-10-05/`.
- Composition: `composition/evidence/run-2026-10-05-current-builder-relays-and-shortcuts/`.
  Its v2 `independent-astra-task-streams/report.json` SHA
  `9d6f872143ce85bf80e6bd590c8136dcc2173bc5cb601596d4b5907580a7fcf3`;
  18 full stdout streams losslessly archived, no normalization.
- Public Request/Response inherent inventory:
  `composition/evidence/request-response-public-api-inventory-2026-10-05.json`:
  67 APIs,63 body-target evidence rows,4 open (`Builder::header` twice,
  `Builder::extension` twice). Counts overlap previous composition groups.
- Formatter failed attempts:
  `headers/evidence/attempts/value_header_debug_body_2026-10-05_5420/` and
  `headers/evidence/attempts/value_error_formatters_2026-10-05_5420/`.

## Exact pending work and next order

1. **Map lookup/getter batch** was granted the last solver lease to
   `luna_audit_resume`: actual `find`, `get`, `get_mut`, `get_all`, `contains_key`
   plus required sealed helpers under named map/find leaf. Frontend and native
   check passed at source `e39268a7…` before capture. At handoff root's process
   inspection saw no live prover/compiler (only old zombies); do not assume a
   fresh proof completed. Check any newly written batch manifest first. Root
   instructed the owner to stop new work and preserve the current bounded run.
2. **Name remaining18**: current WIP `8c4633906a8ecd6e30e7eb75f8ace00c9b9f89d5e5f4c325f7e6b10a4eee94d6`
   had frontend check0, no new emission/proof. Standard lookup None post excludes
   all81 canonical byte views. The body is an unrolled macro, not a loop; its
   `bytes_equal` helper already has exact comparison and processed-prefix invariant.
   Generic `Repr<T> -> Bytes` contract is conditional on real Into callback
   pre/post for Custom and exact Standard spelling; no arbitrary Into assumption.
   Remaining18 source650 scope is140 initial own tasks,6 zero-own excluded;
   strengthened current contexts need fresh arity, not automatic reuse.
3. **Value formatter repairs** are WIP, no proof: Astra identified five Debug
   preconditions (enumerate bound, two indexes, two UTF8 conversions), not a
   dependency/tool blocker. Luna is replacing enumerate with indexed scan,
   `from<=i<=len` and pending-run ASCII invariants, splitting actual ASCII-run and
   escaped-byte writers. Add real formatter-extension post. Error formatter4
   bodies passed15 own; four refinements each had one null out of2 (19/23 partial
   overall). Attach existing Debug/Display append post; ToStrError derived Debug
   becomes runtime-equivalent manual debug_struct/field/finish chain. Preserve
   native output and callback behavior, run parity, then clean/emit/freeze/prove.
4. **URI next**: actual From<Authority>/From<PathAndQuery> exact component posts
   and Uri Display/Debug append posts frontend-checked only. Local evidence under
   `uri-conversion-api-2026-10-05/` and `uri-format-api-2026-10-05/` is source-only;
   no emission/solver. URI owner was also preparing component formatting/getter
   contracts in authority/path/scheme; inspect archive rather than guess completed
   status. Continue exact rendered-text, getter/conversion, Eq/Hash/parser models.
   Uri Eq is exposed component equality, Uri-vs-str consumes a prefix and accepts
   trailing fragments; cannot substitute serialized-string equality.
5. **Method/Status current-source reconciliation**: historical Method103 proof
   snapshot source a643240c differs from current683c060a (nine crate-private
   constructors separately proved in composition). Historical Status80 source
   4a65809a differs from current59136c92 (current Default separately proved).
   Version a6507932 still exactly matches scalar snapshot;22 Version/Http targets
   closed. Re-emit current profiles, compare full printed streams and reuse exact
   complete trees only. Audit all non-ghost inherent/trait APIs and strength of
   contracts. Status62 associated constants have recorded translator limitation.
6. **Real Map mutation closure** remains local unfinished work, not external wait:
   actual append_value tail bounds + extra push/link update + frame, then
   try_insert_entry prefix frame, cyclic Pos shifts, reserve/grow/rebuild.
   Current `header_map_find_ready` is vacuously true for empty entries even with
   invalid indices/mask, so cannot serve as insertion invariant. Absence needs
   slot coverage, Robin Hood probe-prefix order, and equality/hash coherence;
   current Hash contracts preserve only invariant and do not supply coherence.
   Builder.header must wait for a **proved actual Map summary**, not an added
   trusted external HTTP contract. The actual-Map composition profile is wired
   in `verification/composition/src/header.rs`; generic proof-only model bounds
   and Link model were added. Out-of-path ValueIter/Drain IteratorSpec/raw-pointer
   frontend errors remained; use honest named leaf exclusions and preserve native
   API/body. Do not claim this profile green yet.
7. Global inventory/provenance/checkpoint/Map README have WIP updates captured in
   archive. Finish validating their scopes and publish separately. After all
   feasible leaf work, fresh actual production default/all-feature checks and
   relevant native regressions must precisely account for remaining blockers.

## Tool, resource, and soundness rules

- Read `AGENTS.md`, `.agents/playbooks/verification.md`, and cloud runtime skill.
- Source `/workspace/proof-tools/activate.sh`.
- ALL proof launches must request `require_escalated` on the **first** attempt:
  Why3 socket/Seccomp sandbox failures are environmental. Use
  `http/1.5.0/scripts/run-proof.sh`, jobs1/memory1024/shared lock. Root grants one
  explicit solver lease at a time. Free lock/process list alone is not a grant.
- No timer increases. Same logical leaf >150s without structural progress means
  stop/restructure/consult Astra; track subtree/input identity, not one SMT process.
- Native Rust proof attrs need
  `RUSTFLAGS='-Zcrate-attr=feature(stmt_expr_attributes,proc_macro_hygiene)'`
  and same RUSTDOCFLAGS. Do not pass these to cargo creusot (duplicate features).
- Cargo `--check` emits no COMA. Creusot flags are not cargo-tracked; profile
  changes require package clean + actual emit. Name/Value share headers package;
  freeze all source inputs/COMAs before another clean. Never label late-source
  fingerprints as proof-time source. Model/refinement-only type bounds must be
  explicitly scoped; native generics remain unchanged.
- Exact arity independent of JSON/log counts: why3find `--preprocess split_vc`
  uses independent `why3 prove ... -a split_vc -D why3` with **no prover**. Cargo
  creusot prove --no-cache uses **no preprocessing**, even config tactics split_vc;
  omit -a for its initial roots. Independently check each nested tactic's exact
  parent->children, not blanket second splitting successful siblings.
- Separate HTTP body obligations from imported std/callee support; zero-own
  marker/spec targets are not function-body proofs. Failed batch may contain
  individually closed targets, but record the failed batch and exact target scope.
- Named string logical setters are UNSOUND in this toolchain: emitted ComaDefn
  without `!` plus Fanytrue erased assertions, false post passed. Quarantine those
  old header spelling/character-constant artifacts. Use safe numeric pure models;
  program named setters with `!bb0` are a different, checked case.
- No trusted HTTP axioms. True std external contracts are explicit TCB. std char
  prelude excludes legal U+10FFFF; qualify Unicode scope accordingly. Preserve
  original anomalies/panic domains: SchemeNone formatting, H2 quote classifier,
  empty-vs-slash Eq/rawOrd, Authority prefix/static behavior, empty path selector.
- Actual runtime-http dependency payloads are genuine upstream types but their
  bodies are opaque in a leaf profile, not locally verified stand-ins.
- Genuine integrated external limits include Any/AnyClone/Error dyn elaboration
  (installed Creusot handles dyn Debug/Write only), not merely suppressed lints.
  Map two generic layout obligations lack nonzero abstract element-size support;
  this does not block all other Map work.

## Copy/paste prompt for the new thread

`rust-crate-proofs の http 1.5.0 検証を再開して。
http/1.5.0/verification/handoff-2026-10-05/HANDOFF.md と同ディレクトリの
manifest.json を読んで、未証明WIPを確認して引き継いで。
bytes は検証完了と仮定。君がオーケストレーション、実作業は Luna xhigh、
詰まりは Astra xhigh。依存クレート・Creusot変更待ち以外は全て限界まで検証し、
commitのたびに origin/main にpushして。引き継ぎ済みの計画・証拠はやり直さず続けて。`
