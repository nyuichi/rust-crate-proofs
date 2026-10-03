# ASCII byte to UTF-8 proof

The proof maps every byte below 128 to its corresponding Unicode scalar, then
uses the standard character UTF-8 model and an induction over the byte
sequence to show that the resulting bytes equal the input. The character
constructor and per-byte UTF-8 facts are proved separately; this proof adds
no trust.

Command, run from `itoa/1.0.18`:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh cargo creusot \
  --simple-triggers=false prove ascii::ascii_byte_map_to_utf8 \
  ascii::ascii_bytes_are_utf8 --why3session --no-cache
```

Result: `ascii_byte_map_to_utf8` passed all 30 split goals and
`ascii_bytes_are_utf8` passed all 3 goals, with Z3 4.15.3. The exact stdout
log is `focused-proof.log`; the proof JSON, Why3 session, shape database, and
compressed COMA sources are included.
