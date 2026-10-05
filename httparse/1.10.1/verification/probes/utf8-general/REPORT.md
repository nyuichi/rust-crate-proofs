# Pinned standard UTF-8 body proof

## Scope

The proof targets are the actual bodies in the pinned `creusot-std` package:

| Function | Source location | Selected Why3 module | Split VCs |
|---|---|---|---:|
| `utf8_byte` | `creusot-std/src/std/char.rs:78` | `verif/creusot_std_rlib/std/char/utf8_byte.coma` | 8 Valid |
| `CharExt::to_utf8` | `creusot-std/src/std/char.rs:47` | `verif/creusot_std_rlib/std/char/impl_CharExt_for_char/to_utf8.coma` | 14 Valid |

Both raw solver outputs contain no counterexamples or non-`Valid` results. The
Why3 process exit codes are 0. Proof settings were one Z3 4.15.3 prover, 30
seconds and 1000 MiB per goal, under `run-proof.bash` and its shared lock.
The raw command, target/tool/config hashes, stdout, stderr, and exit code are
retained in the corresponding `proof-utf8-byte/` and `proof-to-utf8/`
directories.

## Exact translation and linkage

The translation used the rebuilt string-model profile. Its pinned compiler,
Rust sysroot, standard-library seed, applied patches, Why3, Z3, config, and
installed prelude identities and full source manifests are recorded in the
existing [`tool-rebuild` evidence](../method-utf8/evidence/tool-rebuild-20261005/).
This bundle retains the exact profile snapshot and the actual `char.rs` used by the
translation byte-for-byte. `source-and-config.sha256` hashes the local source
snapshot; the central [`creusot-libs.sha256`](../method-utf8/evidence/tool-rebuild-20261005/creusot-libs.sha256)
checks the corresponding toolchain source. The selected `.coma` modules retain
the source path and line spans.
`Cargo.lock` is preserved and `cargo fetch --locked` left it unchanged.

`creusot-std` was the primary package. Its fresh translation produced 1,775
COMA modules in the separate output directory. Only the two listed modules
were type-checked and sent to the solver. The generated tree is retained in
this environment for subsequent universal-lemma work but is not itself a
proof of the other library bodies. The published body bundle records only the
selected module paths and hashes; it omits the unselected generated tree and
its full inventory manifest.

Only the two selected body modules are needed for the frozen proof targets:
both pass a fresh Why3 type-only check with the evidence bundle as the only
local COMA load path. Their imported model closure is copied under `model/`
and indexed by `model-dependencies.sha256`; the replay does not need the other
1,773 generated modules.

The `utf8_byte` target contains the real recursive implementation. Its split
obligations include the `0..=255` precondition for recursion, the decreasing
integer variant, the `previous + 1` `u8` range check, and the postcondition
that the result's mathematical value is the input. Thus its contract is
discharged by the body rather than assumed from a duplicate helper.

The `to_utf8` target contains the actual four-branch standard-library
definition. It proves the documented length range and all `utf8_byte`
argument preconditions for an arbitrary `char`. Its COMA uses the same
`utf8_byte` definition and contract; that contract's body was separately
proved in the preceding target under the same source and profile.

The exact imports and model files are listed in [`DEPENDENCIES.md`](DEPENDENCIES.md).
In particular, the caller target uses the pinned `Char.to_int` scalar bounds,
the standard sequence constructors, and the `utf8_byte` numeric contract,
which the separate recursive body proof closes. No local encoding axiom is
asserted.

## Imported model and remaining work

The target modules import the matching prelude's `Char`, `UInt8`, `Seq`,
integer, and computer-division models. The frozen `Char.to_int` model states
the Unicode scalar bounds and surrogate exclusion. The `Seq` and machine-byte
operations remain standard-library/prelude semantic boundaries; their
installed package hashes are recorded in `prelude-package.sha256`. Neither
`injective_to_utf8` nor `injective_to_bytes` is used.

These results establish the two standard-library bodies and their recorded
contracts. They do not yet prove that the emitted bytes satisfy RFC 3629's
canonical byte ranges, nor that encoding any finite or arbitrary-length
`Seq<char>` yields a valid UTF-8 byte sequence. The next target is a separate
universal RFC predicate and arbitrary-character prefix theorem, followed by
induction over character sequences. No full-string or httparse integration
claim is made here.
