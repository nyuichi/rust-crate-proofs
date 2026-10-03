# Focused Creusot proof: UTF-8 logic model

This run proved the new `utf8_byte` constructor and the open `CharExt::to_utf8`
logic model in `creusot-std`. It proves the mathematical model functions; it is
not a proof of Rust core's runtime character encoder.

The exact command, run from `creusot-libs/creusot-std`, was:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh cargo creusot --simple-triggers=false prove utf8_byte to_utf8 --why3session --no-cache
```

The wrapper exited 0 with one solver job and a 1024 MiB per-prover limit. It
reported one proved VC for each item:

```text
Library verif.creusot_std_rlib.std.char.utf8_byte: ✔ (1)
Library verif.creusot_std_rlib.std.char.impl_CharExt_for_char.to_utf8: ✔ (1)
```

The source blob for `creusot-libs/creusot-std/src/std/char.rs` was
`3b0fa7b66c471dfa97a2c5de61982b856e451397`.

The proof used Creusot standard library `0.11.0-dev`, Why3 `1.8.2+git`,
why3find `v1.2.0+dev`, Z3 `4.15.3`, and Rust `1.95.0-nightly
(6a979b3e3 2026-02-26)`. Why3 sessions and proof JSON are saved beside this
report; the generated COMA tasks are gzip-compressed.

The arithmetic UTF-8 formula was also compared against Python's standard UTF-8
encoder for every Unicode scalar value. Surrogates were excluded because Rust
`char` cannot represent them. The check script and output are included here.
