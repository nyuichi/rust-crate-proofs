# Isolated derived-trait and discriminant checkpoint

This probe includes the real `src/error.rs` and checks the actual derived
`PartialEq` and `Debug` bodies. A second harness contains two deliberately
false claims so their solver outcomes can be run separately. The translation
checkpoint below was followed by the bounded solver runs recorded in the
solver section; this report keeps those stages distinct.

## Isolated compiler change

Compiler source is pinned to Creusot commit
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`. Apply the narrowcast patch, the
string-model patch, then
`tools/creusot-toolpatch/patches/httparse-derived-trait-discriminant.patch`.
The isolated build wrapper also applies the existing string `View` standard
library patch to its copied library tree; it preserves the current
`convert.rs` array contract. No active compiler or shared standard library
file is changed.

The compiler patch makes these changes:

- Local `#[automatically_derived]` trait methods without their own contract
  inherit the exact matching trait external specification. Creusot still
  translates and checks each actual method body.
- It recognizes the sysroot `discriminant_value` intrinsic by rustc's exact
  `TyCtxt::is_intrinsic(def_id, sym::discriminant_value)` query. It does not
  dispatch by function name.
- Both intrinsic `Call` MIR and non-switch `Rvalue::Discriminant` MIR lower to
  one typed fMIR enum-discriminant value. The pre-existing switch-on-enum
  optimization remains in place.
- Only non-empty fieldless enum ADTs with integer results are supported.
  Payload enums, empty enums, non-enums, and other result types fail closed.
- Lowering matches each actual enum constructor and emits its rustc
  discriminant. Signed bitwise results are masked to the enum repr width;
  mathematical signed results are sign-extended from that width. `isize` and
  `usize` use the target pointer width.

The emitted `Error::eq` COMA contains constructor matches for all seven
variants followed by equality of the resulting discriminants. Its inherited
postcondition states equality of `DeepModel` values. The emitted
`SignedI8`/`SignedI16` fixtures likewise contain constructor matches. For
`repr(i8)`, `-7` lowers to bit pattern `249`; for `repr(i16)`, `-300` lowers to
`65236`. Their mathematical forms are typed `-7` and `-300`. The COMA targets
show no contractless external discriminant call returning `Any`.

The fixtures exercise the intrinsic-Call path. The compiler's non-switch
`Rvalue::Discriminant` path feeds the same fMIR lowering but has no separate
source-level fixture in this checkpoint. The fixture set does not establish
semantics for payload enums or non-enum inputs.

## Translation and type checking

The positive harness generated 26 COMA files. All 26 passed direct Why3
`--type-only` parsing and type checking. The negative harness generated its
two selected claims; both also passed type checking, which is expected because
the false postconditions are solver goals rather than syntax errors.
`verify.sh` has separate translation, type-only, positive proof, and negative
proof modes. The proof modes use `run-proof.bash`, one prover, no cache, saved
Why3 sessions, and per-run prover-result JSONL files.

The selected positive set contains 21 exact COMA targets listed in
`proof-positive.targets`: Error equality and Debug body/refinement, signed
repr derived trait bodies/refinements, Eq marker bodies, caller-level same and
different variant results, and mathematical/bitwise discriminant examples.
The negative set contains exactly the two targets in
`proof-negative.targets`: a false Error equality result and a false signed
discriminant tag. Their later bounded solver outcomes are recorded below.

`evidence/selected-coma.sha256` records the content hashes of all 23 selected
COMA files. The target list hashes are:

- positive list: `e05fe3f31161e36acb6f06b9e145cb9735755b2f893c3163ea1e526e36228679`
- negative list: `48c6281da4e52194ac6bc2f6553f001e0f5179a9418e5f5eb00eb7bb0770948b`
- selected COMA hash manifest: `4374979f62f5dd2bfaa1839155806269c23a915c2b35eae5deec084bcda20522`
- positive type-only log: `b69cf3f8a3290267a411f069e41c0434fa87c3c6998a3e83078e9975ea0eb2f6`
- negative type-only log: `a6e550de6c8a1cb88b89c1e8d348928841f0b23b4513d409ca40711e06fd28cb`

The proof config remains one prover with a 1000 MiB limit (SHA-256
`e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`). The
isolated prelude package resolves to SHA-256
`cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b`. The
isolated compiler binary is SHA-256
`55799f44b6a711ff6b15e7c1d5c972ad21b8b91c5a77ec774526289f90be088a`.

## Solver checkpoint

The 21-target selected positive batch is incomplete. Eighteen target files
recorded Valid for all selected obligations. The six caller-level equality
targets use the called `PartialEq::eq` postcondition; they do not replace the
separate proof of the callee body. Three actual derived equality bodies remain
open: `Error::eq` timed out; each signed-repr `eq` body has two valid split
leaves and one timed-out split. For those bodies, the COMA declares
`deep_model_Error`/`deep_model_SignedI8`/`deep_model_SignedI16` as opaque
functions, so the discriminant match cannot establish injectivity of the
deep model. Astra identified inherited local-derived predicates in the native
parameter environment as the likely cause. Do not retry these bodies with a
longer solver limit with this old compiler snapshot; its model functions are
opaque in the generated COMA. The follow-up below uses fresh COMA and a
separate compiler fix that exposes the real model definitions.

The later isolated native-environment checkpoint at
`../derived-traits-nativeenv/REPORT.md` shows fresh `DeepModel` match
definitions and has now proved the three actual equality bodies with its new
binary. Results below this paragraph remain historical results for the older
`55799f44…` binary; none of them count toward the native-environment
checkpoint.

The derived COMA passed in this batch include both Debug body and its
formatter-extension refinement; all three `eq__refines` goals, Eq marker
methods, all six caller-level equality checks, and all four `repr(i8/i16)`
mathematical/bitwise discriminant checks. The discriminant fixtures therefore
proved their explicit signed values and width-masked bit patterns.

The two deliberately false claims were run separately with `--time 10`.
`vc_deliberately_wrong_error_eq` and its split returned `Unknown`; the
supporting `vc_eq_Error` body obligation returned `Valid`; it is separate from
the caller-level false-result goal and does not close the positive Error body
target that timed out. The deliberately wrong signed tag
goal and its split returned `Timeout` (10.35s and 20.70s). No `Invalid`/SAT
result was returned, so the negative checks remain open. Their prover JSONL
is `evidence/derived-negative-prover-results.jsonl` (SHA-256
`92103e91384d9284819c6f4c7f0ae1f638f8242d6b36abc1fed18a81cb9c555b`).

The first positive-stage JSONL is
`evidence/derived-positive-prover-results.jsonl` (SHA-256
`35dd1217addd7d83690a2bba7ed7077c15abd0f3d5c295100c8ed567b1a59746`); the
remaining-target JSONL is
`evidence/derived-positive-remaining-prover-results.jsonl` (SHA-256
`4243f8a00a2579105aedeb44e824db2c8864f0f15d1dd3c5db8ddfe6f87c0d69`). The
remaining batch was stopped before a further 61-second retry of the timed-out
signed equality body. That retry has no prover-result entry and is not counted.

The original string-model positive checkpoint recorded six open VC leaves,
including three non-ASCII UTF-8 byte examples, Error `PartialEq`, and Debug
refinement. The follow-up native-environment checkpoint proves the selected
Error equality body/refinement and Debug formatter-extension body/refinement
with a separate compiler. `Debug::fmt` checks its formatter-extension
contract; it does not establish exact rendered text. The Unicode byte
examples, exact formatting text, and full crate integration remain open.

## Patch and build provenance

- Derived compiler patch: `90bed76ac9626b5738ee55a6dfbdf909c6e972ba40063db5ca6e296e8c8dedc4`
- Build wrapper: `90c3ace4cf900cc2c1c817422ee1bb5271f09fc12fdfb1c40924fd834d7d4477`
- Narrowcast patch: `c3ac3e596b822f483d2f0f782169aacc5c752449cf66771ee7268ee2feb8eb24`
- String compiler patch: `8b3205493c45b909c8fb980b3e6cb3c4f63d52511060f12ec227c4e105d4f2ea`
- String standard-library patch: `7b92f2d54dc845a982246c220bc37004ae14d6ee5e51ff0ff4bbc87eb90b5ab9`
- Isolated `creusot-std/src/std/convert.rs`: `5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348`
- Isolated `creusot-std/src/std/string.rs`: `e51e9dd373e683c63c424aca31bb4850f0437e1c5c56c325c8590e46670957df`
