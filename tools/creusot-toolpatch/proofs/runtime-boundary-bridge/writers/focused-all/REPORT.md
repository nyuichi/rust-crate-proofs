# Concrete runtime writer proof

The source-level trait ensures are repeated on all 12 concrete sealed writer
implementations so Creusot checks each body against the canonical output
sequence and maximum-length contract. Removing the redundant
`where Self: Integer` from the concrete impl methods lets Creusot normalize
`MAX_STR_LEN` to its concrete value and prove the safe prefix conversion.

Command, run from `itoa/1.0.18`:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh cargo creusot \
  --simple-triggers=false prove \
  verif/itoa_rlib/runtime/impl_Sealed_for_*/write.coma \
  verif/itoa_rlib/runtime/impl_Sealed_for_*/write__refines.coma \
  --why3session --no-cache
```

Result: all 12 `write` bodies and all 12 `write__refines` goals passed. The
run discharged 120 VCs: 20 for `u8::write`, 8 for each other writer, and 1
for each refinement goal. The exact stdout log is `focused-proof.log`; each
type directory contains the proof JSON, Why3 session, and compressed COMA
body and refinement sources.
