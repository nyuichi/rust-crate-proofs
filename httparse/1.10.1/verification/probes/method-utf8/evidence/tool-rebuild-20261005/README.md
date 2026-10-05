# Fresh proof-tool profile

`../../tools/build-string-profile.sh` rebuilt the string-model compiler and generated
its matching prelude on 2026-10-05. The build used Creusot
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, then the frozen narrowcast and
string-model patches. Its binary is
`/workspace/httparse-tool-rebuild/targets/creusot-cast-compdiv/debug/creusot-rustc`
(SHA-256 `29fcf8914166db07a910c45e4c8462f5665cfe4eee482509b885e629d3ff826e`).
The profile reuses the prebuilt `castcompdiv` Cargo target to reuse dependencies.
It replaced the baseline binary recorded below. The patched compiler source
manifest excludes only generated `compiler-source/target/**` build output.

The prelude generator output and installed isolated package both hash to
`cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b`.
`prelude-package.sha256` covers every installed package file;
`compiler-source.sha256` covers the patched pinned compiler input; and
`creusot-libs.sha256` covers the isolated full standard-library tree.

The standard-library seed is commit `6263082`, whose `creusot-libs` tree is
`daf48d3435a26fa967e3e5727c0f386515a44002`. The string View patch has SHA-256
`7b92f2d54dc845a982246c220bc37004ae14d6ee5e51ff0ff4bbc87eb90b5ab9`.
The resulting `std/num.rs` is the seed version (`b777d9fb…`), and patched
`std/string.rs` hashes to `e51e9dd3…`. All 22 standard-library file hashes in
the archived newline translation manifest match this newly rebuilt tree;
`newline-stdlib-match.tsv` records each comparison. That authenticates these
22 inputs for the new remediation identity. It does not establish the full
historical library tree identity or make new runs historical replays.

The installed proof stack resolves to clean source checkouts at Why3
`2c0f2992af85f82f3eda0f158dcf10e62e0db875` and why3find
`3a98fc320b9cbf2e71860da1c8dc188a966eee96`. The active opam switch lists
`why3 git-2c0f2992` and `why3find git-3a98fc32`; binaries report Why3
`1.8.2+git`, why3find `1.2.0+dev`, and Z3 `4.15.3`. Z3's binary SHA-256 is
`80ee070b8ffc2fa964b0b879250b2110c47f84c1256a1f9ab76fc7cf95e46292`. The
copied Why3 config has `memlimit = 1000`, `running_provers_max = 1`. No solver
was started during this tool rebuild.

Activate the profile with:

```sh
source /workspace/proof-tools/activate.sh
source /workspace/httparse-tool-rebuild/string-model/creusot-env.sh
```

The shell file also exports `HTTPARSE_CREUSOT_LIBS` and
`HTTPARSE_TOOL_PROFILE` for probe wrappers.

Before this build reused the target directory, the root task reported baseline
binary SHA-256 `b64ebfbcc0c62b5a3f8e5b513f16e8ddc95c06c9d4f597827cdca2ff1fb96156`.
Its expected source recipe is the same pinned Creusot commit plus the
narrowcast patch only, as encoded by
`tools/creusot-toolpatch/scripts/build-creusot-rustc.sh`. The fresh workspace
did not contain that baseline build's raw log; `build.log` is the captured log
for this string-model rebuild.
