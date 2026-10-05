# Codex Cloud durable handoff

Use repository `nyuichi/rust-crate-proofs`, branch `main`, for the next Cloud
task. The prior container is not required to read this handoff or recover its
work-in-progress source and evidence. Select this repository in the new task,
then read the adjacent `../THREAD-HANDOFF.md` for priorities and proof status.

## Recover the uncommitted work

`work-in-progress.tar.gz` contains 1,593 files, captured from the original
workspace and byte-checked against it. `FILES.SHA256` hashes each archived
file; `ARCHIVE.SHA256` hashes the compressed archive. `SNAPSHOT.json` records
scope and exclusions. No new solver ran while preparing this handoff.
The capsule preserves incomplete candidates and failed proof evidence as well
as successful selected proofs. It does not turn those candidates into accepted
proofs, change runtime code on main, or claim complete crate verification.

With `repo` set to the absolute path of the new checkout, run the following
commands sequentially and stop on any failure. Choose a new, empty sibling
directory for `resume`; substitute another path if it already exists.

```sh
repo=$(git rev-parse --show-toplevel)
capsule="$repo/httparse/1.10.1/cloud-handoff"
(cd "$capsule" && sha256sum -c ARCHIVE.SHA256)
resume="$(dirname "$repo")/httparse-cloud-resume"
git -C "$repo" worktree add -b httparse-cloud-resume "$resume" 61840c8
tar -xzf "$capsule/work-in-progress.tar.gz" -C "$resume"
(cd "$resume" && sha256sum -c "$capsule/FILES.SHA256")
git -C "$resume" status --short
```

This deliberately restores onto the exact original base `61840c8`. Other
tasks have since advanced main, including work on other crates. Before
committing reviewed verification changes, integrate current main carefully,
retaining the capsule as the immutable recovery source. Stage accepted changes
selectively; never blindly add the entire capsule as proved source. Push every
new commit to `origin/main` without force, as the user instructed.

## Rebuild the proof environment

The archive excludes compiler/solver binaries, build caches, credentials,
the old `/workspace/proof-tools` installation and `/workspace/scratch` trees.
It preserves generated COMAs, available exact source snapshots, tool identities,
configs, raw proof logs, and historical results. Restoring it is not equivalent
to rebuilding tools or replaying proofs.

1. Read repository AGENTS.md, `.agents/playbooks/verification.md`, and the
   cloud-environment-runtime skill if available. Query the new task's own
   environment/network readiness. Obtain authentication through that task's
   configured GitHub access; do not copy credentials from the old container.
2. Inspect `tools/creusot-toolpatch/records/versions.md` and the chosen proof
   bundle's exact provenance. Pins include Rust nightly-2026-02-27, Creusot
   `437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, Why3
   `2c0f2992af85f82f3eda0f158dcf10e62e0db875`, Z3 4.15.3. Install their build
   prerequisites and sources if the new image does not already provide them.
3. Existing repository builders include
   `tools/creusot-toolpatch/scripts/build-creusot-rustc.sh`,
   `scripts/build-httparse-string-model.sh`, and Gate C/D `build-compiler.sh`.
   Inspect their absolute paths and prerequisites before running: they are not
   a self-contained bootstrap installer. Set up the expected directories or
   adapt paths explicitly in the new environment. Offline Cargo builds require
   provisioned dependencies; old caches must not be assumed to exist.
4. Recreate each isolated standard-library version from its frozen snapshots
   and pinned source, not by copying whatever happens to be newest on main.
   In particular, the old UTF8 string seed's std/num.rs differs from the later
   native-byte model. The string builder copies current libraries; invoking it
   on a newer main without reconciliation does not reproduce the old proof
   inputs. Verify exact source/package hashes before calling a run a replay.
5. Recreate the matching Why3 package, config and child-process environment.
   Use one prover, 1000 MiB, fixed30s per goal and a shared exclusive lock.
   Typecheck exact archived COMAs first. If a rebuilt binary differs, record a
   new tool identity and new replay evidence rather than overwrite old logs.

The previous task discovered a safe-public-entry parse_method UTF8 safety gap;
its source diagnosis and proposed next steps are in THREAD-HANDOFF.md and the
archived VERIFIER-SUPPORT-CONSULT.md. Prioritize that finding. Full verification
remains OPEN. Resume with the primary agent itself orchestrating, Luna xhigh
doing implementation, and Astra consulting on blocked proofs.
