# httparse 1.10.1 full-verification plan

Planning owner: Astra. Execution orchestration: Sol. Implementation and proof work:
Luna with xhigh reasoning. Consult Astra at the structural stop conditions below.
This is a plan, not a claim that any httparse body is already proved.

## Fixed target and environment

- Published crate: httparse 1.10.1, latest non-yanked release in the Cargo sparse
  index checked during planning on 2026-10-04.
- Archive SHA-256: `6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87`.
- Recorded upstream Git revision: `9f29e79f9832dbd0ae5220acb17c1866745bdecd`.
- Verified archive: `/tmp/httparse-1.10.1.crate`; pristine extracted source:
  `/tmp/httparse-1.10.1`. Import the complete published tree into
  `httparse/1.10.1`, preserving licenses and original API/runtime behavior.
- Origin: `https://github.com/nyuichi/rust-crate-proofs.git`; user has authorized
  commit-and-push after every commit. Push without force, before making the next
  commit. Sol serializes commits and pushes; workers do not commit independently.
- Activate `/workspace/proof-tools/activate.sh` for every shell command needing
  Rust/provers. Available: nightly-2026-02-27, cargo-creusot 0.11.0-dev,
  Why3 1.8.2+git, why3find 1.2.0+dev, Z3 4.15.3, CVC5 1.3.1,
  CVC4 1.8, Alt-Ergo 2.6.2. Pinned repository contracts are in `creusot-libs`.
- Read `AGENTS.md` and `.agents/playbooks/verification.md`. Proof execution must
  request elevated execution initially due to Why3 Unix sockets. Do not test or
  prove unrelated crates. Use one coordinated proof queue and one prover by
  default; ordinary source work/builds can run independently.
- Network source lookup worked through elevated curl to `index.crates.io` and
  `static.crates.io`. The crates.io JSON API returned HTTP 403. Do not bypass the
  proxy. Exact import files are already available locally.

## What full means, and what is not completion

The goal is a functional and safety proof of the published crate's actual
executable behavior for arbitrary finite inputs, including invalid and partial
inputs, all seven ParserConfig flags, initialized and uninitialized header APIs,
and every reachable runtime backend. It is not just bounds checking, passing
upstream tests, bounded fuzzing, or an independently proved parser model.

Completion requires:

1. An explicit API inventory and contract for every crate-owned callable body,
   including doc-hidden `_benchable` exports, trait implementations and constants.
2. Exact `Complete`/`Partial`/error behavior, result offsets and byte contents,
   configuration effects, and mutable output state on each exit. Include error
   precedence and behavior when parsing repeatedly into preexisting objects.
3. Proof of absence of unintended panic, arithmetic overflow and invalid
   indexing; termination/progress; provenance, bounds, initialization, lifetime
   and aliasing obligations at each unsafe operation. Intentional
   `Status::unwrap(Partial)` panic is a specified precondition/exception, not a bug.
4. Proof that the actual scalar/SWAR and optimized execution paths implement the
   same strong scanner/parser contracts. A cfg(creusot) replacement is only an
   intermediate proof artifact until an actual runtime refinement bridge is
   mechanically checked. Finite differential tests do not discharge that bridge.
5. No crate-local trusted algorithm, admitted lemma, assumed postcondition or
   excluded callable body in the claimed surface. Standard Rust/Creusot library
   models, compiler and solvers form an explicitly listed TCB. New intrinsic or
   standard-library axioms require precise semantics and justification; moving a
   crate algorithm into an external specification does not close its proof gap.
6. Crate-scoped integrated proof and runtime tests, reproducible from the exact
   committed tree, across the supported configuration matrix; clean audit of
   trusted/cfg-excluded surfaces. Any architecture not mechanically checked is
   an explicit outstanding target, never silently covered by an x86-64 run.

Intermediate commits may contain temporary trusted scaffolding with strong
reviewed contracts, a status ledger and exact removal condition. Such commits
must remain labelled partial. Do not announce complete while a ledger row remains
trusted, excluded, model-only or runtime-unbridged. If full closure needs missing
verification infrastructure, continue useful independent work and consult Astra;
do not silently downgrade the requested outcome.

## Specification architecture

Use immutable input bytes plus integer offsets. A cursor model is
`(input, mark, cursor, end)` with `0 <= mark <= cursor <= end <= input.len()`;
`Bytes::pos()` is `cursor - mark`, NOT an absolute offset. `slice` and
`slice_skip` reset mark to cursor. Return spans plus optional owned scalar values
in the model, then prove zero-copy slices/strings correspond to those spans.
Model byte predicates directly, without assuming the runtime static tables.

Use an independent deterministic parsing relation/interpreter with explicit
stage transitions. Outcomes contain tag, exact consumed position, field updates,
initialized header prefix and residual capacity. The model may be decomposed by
stage to keep VCs small; do not define the public specification as the runtime
function itself. Use one canonical absolute cursor per scanning loop and a
separate header-count measure only in the header loop.

Public type invariants must reflect all constructible public values:
- `Request`, `Response`, and `Header` have public fields. They permit arbitrary
  valid Rust strings, bytes, integer options and header contents. Do not impose
  parsed-only restrictions such as version <= 1 or header-token syntax as a type
  invariant; those are successful-parse postconditions.
- ParserConfig's seven Booleans have exact observation/update models. Default is
  all false; builder mutation changes only its selected flag and returns self.
- Status and Error have exact variant models; Header/EMPTY_HEADER model exact
  bytes; no stronger precondition than the public Rust API requires.
- Bytes is publicly reachable through `_benchable`. Its safe methods and generic
  `peek_n<U: TryFrom<&[u8]>>` are part of the inventory; unsafe mutators need exact
  documented preconditions and a same-allocation provenance requirement.

Dependency layers:
`public Request/Response/parse_headers -> start/status line + header state machine
 -> token/URI/reason/version/code/newline/whitespace components
 -> maximal-prefix scanners + byte predicates + cursor/storage primitives`.
`parse_chunk_size -> hex recurrence + chunk/extension transition machine -> cursor`.

Scanner contracts state safe advance, unchanged source/mark, all skipped bytes
accepted, and the first unaccepted byte or end at the new cursor. Internal SWAR
blocks may provide a conservative prefix: header-value SWAR stops at TAB and the
scalar tail consumes it. Its block helper therefore must not falsely promise the
maximal full header-value prefix. Word-subtraction borrow effects also mean that
per-lane mask exactness must be proved, not presumed; use the weaker sufficient
prefix/first-candidate property and compose the scalar correction.

Semantic edge cases to preserve explicitly:
- CRLF and accepted bare LF; leading empty lines; truncated CR and invalid CR+byte.
- HTTP/1.0 and 1.1 only; fast eight-byte version path consumes differently from
  short-input path, observable through `_benchable::Bytes`.
- Nonempty method/URI; method tchar; URI allows high bytes but then requires valid
  UTF-8. Header values remain arbitrary allowed octets, not UTF-8 strings.
- Three decimal status digits, including values outside conventional HTTP codes.
- Optional/empty reason phrase; any accepted obs-text makes returned reason `""`.
- Capacity zero, exact capacity, and one-over capacity; TooManyHeaders is decided
  after parsing the candidate line, so earlier syntax errors/Partial can win.
- Whitespace trimming, empty values, obsolete folding, first-header whitespace,
  invalid-line skipping, and mandatory rejection of NUL/lone CR when skipping.
- Partial and error paths keep already-written fields; old fields may survive if
  their stage has not completed. Initialized-header wrappers restore the original
  slice length on failure but keep overwritten initialized slots. Uninitialized
  wrappers expose only initialized data and have different self.headers effects.
- Chunk size accepts up to 16 hex digits, including leading zeroes; the actual
  implementation accepts CRLF with zero digits and ignores arbitrary extension
  bytes except CR must be followed by LF. Do not strengthen this to RFC grammar.
- All 128 Boolean config combinations should follow parametrically from proofs;
  tests exercise their interactions, not replace the quantified proof.

## Execution phases and merge ownership

### P0: import, baseline, inventory, tooling probes

One Luna worker owns import/Cargo/build scripts/PROVENANCE/status ledger and the
API inventory. Preserve normalized manifest facts; give a verification package
an unambiguous selector if an upstream reference dependency is used. Run upstream
unit/doc tests for default and no-default before annotation. Capture exact target,
backend flags, tool identities and source checksum. Import + plan + baseline is
one honest partial checkpoint, then push.

Before building a large proof-only parser, probe five small actual-body examples:
Bytes pointer addition/read/slice, MaybeUninit header write/expose, one UTF-8
conversion, one SWAR block, one SSE intrinsic. Reuse existing pointer permissions,
`std::unsafe_collection`, MaybeUninit models and `std::string` specifications.
Record unsupported translation precisely. Decide the concrete runtime-refinement
route early; if raw pointer/intrinsic semantics cannot be linked mechanically,
consult Astra with the failing minimal examples. These probes need not block
chunk arithmetic or specification work.

### P1: reviewed model and orchestration skeleton

Worker A owns `verification/model*` and API/specification mapping. Worker B owns
cursor/storage/UTF-8 primitives; worker C owns chunk parser and simple Status/
config contracts. Use disjoint files or sequential edits for src/lib.rs.

Build strong leaf contracts and initially trusted decomposition boundaries.
Prove tiny representative callers, then public request/response orchestration
against exact outcome/state contracts. This is a scaffolding checkpoint only.
Prove constructors, config setters/getters, Status methods and chunk-size body
first to obtain useful complete vertical slices. No trivial `ensures(true)`
substitutes for content semantics. Commit/push each coherent proved milestone.

### P2: cursor and scalar components

Discharge raw cursor initialization, observations, advance/commit/slice and all
preconditions at call sites. Tie ownership/permissions to source lifetime; pointer
integer subtraction bounds alone do not prove same allocation. Prove byte-table
correspondence, token/version/code/reason/URI and whitespace/newline components.
String output needs byte identity and validity, not merely length. For checked
UTF-8 preserve invalid input behavior; for unchecked conversion prove validity.
Discharge chunk recurrence and <= 16-digit overflow safety, exact error/partial
semantics and extension handling. Integrate each group before proceeding.

### P3: headers and complete public parsing

Split the large header function by proof state: name, whitespace after colon,
value line, continuation/obs-fold, invalid-line skip, trailing trim, slot commit.
Each stage returns cursor/outcome and exact footprint. Prove capacity/error
precedence and the initialized prefix. Prove ShrinkOnDrop cleanup and every early
return, initialized/uninitialized conversions, wrapper restoration, and complete
public state transitions. A verified replacement of Drop logic needs a checked
bridge or an actual runtime refactor with exact behavior preservation.

At this point publish scalar-core status precisely. It is not full crate proof
until optimized paths and exposed auxiliary APIs are closed.

### P4: optimized backend refinement and remaining public adapters

Prove SWAR word arithmetic for supported widths/endianness, then block loops and
scalar tails. Prove SSE4.2 and AVX2 byte-lane predicates/masks/first-invalid index,
unaligned load preconditions, fallback composition and feature checks. Cover NEON
analogously. Prove dispatch cache allowed-state/feature invariant under atomic
interleavings or use a verified semantics-preserving runtime arrangement; treating
an arbitrary byte as a valid feature without evidence is unsound.

Complete `_benchable` methods and generic trait adapters, clone/equality and error
formatting behavior. Any standard-library formatting or CPU intrinsic boundary
must be declared in the TCB with actual semantics; unproved crate wrappers remain
open. If a new verifier or toolchain patch is necessary, consult Astra before
investing in a broad infrastructure migration. A separately proved bitvector
model must be tied to the exact runtime expressions, not just example-tested.

### P5: final matrix and audit

Create a target-local `verify-all.bash` that fails on any untranslated/unproved
component and records backend selection; do not rely on a green empty proof run.
Run no-default (which still uses SWAR), default std runtime dispatch, explicit
SIMD-disabled std, and forced SSE4.2/AVX2 on compatible host CPU; default and
all-features are identical Cargo feature sets. Exercise
`CARGO_CFG_HTTPARSE_DISABLE_SIMD=1` and
`CARGO_CFG_HTTPARSE_DISABLE_SIMD_COMPILETIME=1` carefully: upstream build.rs does
not declare rerun-if-env-changed, so separate target directories or explicit
rebuilds are required. Validate target-feature and endian/32-bit/NEON obligations
with mechanical proof artifacts/cross-target runs; record unavailable execution
separately from proof. Do not claim a host test ran a foreign instruction set.

Retain upstream tests. Add focused tests for discovered semantic corners,
all config interactions, every truncation of representative messages, invalid
byte positions, header capacities and initialized/uninitialized agreement, plus
scanner boundaries 4/8/16/32 and neighboring lengths. Differential comparison
against the pristine archive detects specification drift; Miri/sanitizers/fuzzing
are useful complementary evidence where available, never universal proof.

Final audit maps every public item and unsafe expression to an actual integrated
proof/standard TCB contract. Check `trusted`, `assume`, extern specs, `cfg(creusot)`
substitutions, disabled modules, dead stubs, panic paths and unconstrained models.
Re-run the relevant matrix after final source changes, record exact command/target/
feature/backend results and remaining gaps (must be none for the full claim),
update root README and PROVENANCE, commit, then push and verify remote HEAD.

## Proof budget, consultation and reporting

- One canonical progress index; stage-local invariants and variants. Prove a
  helper AND a representative caller before adding it to the large caller.
- About 100--150 VCs in one function is a split warning. Repeated equal-shaped
  failure twice requires interface review; three failures or backward progress
  requires redesign. Do not keep piling assertions or increasing timeouts.
- At 30--45 minutes with no structural progress, checkpoint and consult Astra.
  Consult earlier for unsupported raw memory, Drop, static table, intrinsic or
  foreign-target semantics that block a full runtime bridge.
- Send Astra: exact source/command/tool version, smallest reproducer and failing
  VC, current proved bodies, all trusted boundaries, two attempted approaches,
  and proposed interface or verifier change. Continue independent work while
  waiting; Sol coordinates assignments and only merges checked pieces.
- Maintain per-component columns: contract reviewed, body proved, runtime bridge
  proved, integrated configuration, trusted/excluded gap, removal condition.
  Proof session counts and tests are evidence, not a replacement for that table.
- Root communicates concise Japanese progress to the user. Every commit is
  immediately pushed. If HTTPS Git push fails but GitHub API is authenticated,
  use repository `tools/push-github-objects.py` exactly as AGENTS.md documents.
