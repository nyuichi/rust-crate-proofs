# Pre-fix initializer typecheck rejection

This is raw evidence from the pre-fix Gate B Why3 typecheck. Why3 rejected the
initializer task before starting any prover: `value_invariant 0` supplied an
untyped `int` where `creusot.int.UInt8.t` was required. No VC result was
produced and no Z3 process was invoked. This is a generated-COMA typing failure,
not a counterexample to the invariant.

Inputs at that attempt:

- fixture source SHA-256: `73e3ab30792c22d2865d9fb3a27538ab77451f17f604508d3a7d443673132a3f`
- pre-fix compiler patch SHA-256: `ef01f18ecb5024dc436027d363e8cd742a26d0b3b4ee64e7accc4a252893b36d`
- pre-fix compiler binary SHA-256: `e487badf38c483b022e8a33cee8bd211b1e759538c7f71980328ee53a8876fff`
- pre-fix `CACHE.coma` SHA-256: `0b97167fefc9c4463d13a281ede608820fd07f5d149144eaca769ef439a0c623`
- effective Why3 config SHA-256: `17732ddfcddbe2808efb41cfe41be1aee29650ad17021f013c7723ad128e178a`; the logged settings were one prover and 1000 MiB
- Why3 package root: `/workspace/proof-tools/creusot-data/share/why3find/packages/creusot`
- package `.coma` closure digest: `851af6e8161611c660b5ae4a9c086b6a9159700b2e0d8d1d29f07897f27f15c9`
- Why3: `1.8.2+git`; Z3: `4.15.3`

The current compiler patch lowers the initializer via the existing MIR
`ConstValue::Scalar` conversion path and emits `(0: UInt8.t)`. It has only been
retranslated and regression-checked; that corrected task has not yet been
submitted to Why3 or a solver.
