# Gate C retained-package type-only checkpoint

The source builder copied the entire generated Why3 package tree to the isolated target output and wrote a sorted SHA-256 manifest beside it before deleting the temporary compiler source tree. This evidence bundle preserves the matching full package snapshot under `prelude-package/`; its manifest digest is recorded below and is the same package identity used by the runner.

The Gate C runner verified the retained package manifest and translated the parameterized invariant fixture. It then ran `why3 prove --type-only` independently on each of the eight exact emitted positive `.coma` files, with the retained generated package first on `-L` and the COMA module directory second. The exact commands and raw logs are included. All eight exited successfully. No prover was selected or invoked.

The fixed `StaticAtomicCaps` Boolean functions remain uninterpreted and unconstrained. No detector assumption or detector refinement was added. The actual sysroot detector identities and any `#[target_feature]` branch obligations remain open; this is translation and type-only evidence, not a complete httparse dispatch proof. The existing Gate B standard atomic history TCB is unchanged.
