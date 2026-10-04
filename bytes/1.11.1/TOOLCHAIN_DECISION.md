# Toolchain decision

## Migration result

Bytes now pins Creusot/creusot-std 0.13.0 and nightly-2026-06-22 in its own
worktree. Dedicated Cargo, Creusot data/config, targets and OCaml switch are under
`/workspace/bytes-proof-tools`; the rustup store contains both nightlies without
changing the old default. No itoa patch, prelude, driver or target cache was reused
as the bytes proof compiler. The shared proof lock remains serialized, one prover,
1024 MiB. `sc-drf` is disabled; serde/portable-atomic are not enabled.

Creusot SHA: `318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`.
Why3 SHA: `54c92f96bb0711d6e991c18f10bfbc08d90d028b` (`1.8.2+git`).
Why3find SHA: `eab37557d3e24e1913a3c4f44bc5528ef497c6c9` (`1.3.0+dev`).
Alt-Ergo 2.6.2, Z3 4.15.3, CVC4 1.8, CVC5 1.3.1.

The initial Opam install attempted full Why3 Git history and stalled. A shallow
fetch of the exact commit, local source pins and an isolated OCaml 5.3.0 switch
completed installation. The compiler/prelude use the official 0.13 source.
The native compiler is preserved at `bytes-proof-tools/vanilla/creusot-rustc`.

The old `prove` subcommand and `--simple-triggers` option are not used. The new
entry translates with `--only=coma` then proves with `--only=prove` and explicit
one-prover arguments. `verify-all.bash` targets the actual default-std runtime;
it currently fails with the recorded blockers rather than passing an old model.

## Gate status

| Gate | Established result | Still required |
|---|---|---|
| P1 | arbitrary Box storage/read/recovery; borrowed split; extracted actual free_boxed_slice body and real Box caller | Bytes constructors, all representations, vtable dispatch, automatic Drop; caller resource connection |
| P2 | borrowed permission splitting foundation only | independently owned initialized/uninitialized ranges, freeze/write, refcount lifetime, arbitrary drop order |
| P3 | original Ordering/source paths inventoried and model reviewed | native fetch_sub wrapper, release-sequence resource recovery, clone/release/unique real bodies |
| P4 | original CAS/tag/control-block paths identified | provenance-aware integer tagging, vtable contracts, winner/loser resource protocol and actual bodies |

None of P1–P4 is complete. Creusot 0.13 is useful for foundational proof work,
but is not yet adopted as a tool capable of full bytes runtime integration.

## Reproducible blockers

From `verification/probes/tool-blockers`, source the dedicated activation and run
`cargo creusot --only=coma -- --locked --features FEATURE`:

- `trait_cycle`: specification-free `Buffer::chain<U:Buffer>` rejected as illegal
  recursive trait. The isolated patch fixes that limited form while preserving
  logic/ghost/termination/associated-type/mutual/contracted negatives.
- `indirect_call`: a function-pointer call rejected as unsupported call type.
- `pointer_tag`: native pointer-to-integer exposure rejected as unsupported cast.
- `automatic_drop`: translation succeeds but omits the destructor action. Proving
  its valid destructor-effect postcondition fails. This is a supportedness gap,
  not a false claim that the real destructor violates that postcondition.

Full runtime translation also ICEs on Bytes::PartialOrd's missing DeepModel
projection. A byte model alone will not resolve heterogeneous byte/string compares:
standard comparison specs demand matching model types, whereas bytes compares
with both byte sequences and strings. Do not add public API bounds or trust an
unconnected model just to remove the ICE.

## Next work

Keep vanilla foundation results separate from the candidate trait patch. Add
minimal support gates for indirect call contracts, Drop glue/unwind and exposed
provenance before large-scale annotation work. Close real Bytes caller resources
for the proved deallocation body. Then establish owned interval permission
splitting and initialized capacity, followed by the RMW release-sequence probe.

A Verus migration has not been demonstrated. Its use requires the same actual
implementation paths, original weak Ordering and comparable resource transfer.
SeqCst wrappers or finite Loom exploration alone do not satisfy this gate.

## Revised work policy (user direction)

Large Creusot extensions are deferred. Continue component proofs that do not
require indirect-call, automatic-Drop or exposed-provenance support. Begin with
safe slice/cursor operations and bounded buffer adapters, then initialized-byte
writes where the existing permission library suffices. Any dependency on
unverified allocation/refcount/vtable contracts stays explicit and is not counted
as an integrated runtime proof.

Small target-source changes are allowed: extract shared runtime helpers, add
contracts/ghost instrumentation, and investigate finite direct dispatch replacing
an internal vtable call where representation mapping can be justified. Finite
dispatch is a candidate, not an established equivalence. Preserve public API,
byte results, ownership/drop behavior, panic behavior and native atomic Ordering.
Record original/changed source correspondence, the proof configuration, and
whether the normal runtime includes the change. Do not replace storage with a
length-only model or use stronger atomic ordering as a verification workaround.

The slice/cursor, bounded adapter accounting, initialized and uninitialized writes,
byte codecs and checked reads now have exact-source isolated proofs. Borrowed
Box regions can split, mutate and recombine under existing permission contracts.
The next full-runtime gate remains the recursive trait/comparison translation
failure, followed by indirect vtable calls, provenance casts, destructor effects
and refcount resource recovery. These are not discharged by pure helper proofs.
Large compiler patches remain experimental and outside the adopted toolchain.

Push each committed checkpoint to origin/bytes-runtime-verification without force.
