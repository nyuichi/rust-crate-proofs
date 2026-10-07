# MSP430 16-bit alloc-only configuration audit

- Target: `msp430-none-elf`; `rustc-cfg-msp430.txt` records `target_pointer_width="16"`, `target_endian="little"`, and `target_arch="msp430"`.
- Native configuration: `verified` only, which enables `alloc` and does not enable `std`. The native gate used `cargo -Z build-std=core,alloc check ... --target msp430-none-elf` and passed. The proof-effective graph adds the Creusot and nightly features on the private `creusot-std` path package.
- Full proof: `Proved (247 files)`, exit 0; 247 Coma files, 247 proof JSON files, zero null proof-result leaves. Archive: `runs/positive/msp430-verified-alloc-final-config/evidence.tar.gz` (SHA-256 `4d1acfa3e0fee3ae9b548a0a0b927829384f47df97e24955c46e420783e79c6c`).

## Capacity and `usize`

The target-specific `verified/exclusive/reserve_then_close.coma` uses `UInt16.t`; line 30 sets its maximum to `65535`, and lines 34–36 bound the vector-view length by that maximum. Lines 45–61 define the opaque capacity observer for the actual vector and the capacity/reserve postconditions. Lines 69–94 show the translated reserve, content-preservation assertion, capacity lower-bound assertion, close, and return. The corresponding caller proof is `verified/exclusive/reserve_then_close/proof.json`.

## Native-endian call selection

For this little-endian target, the actual translated native-endian write bodies call their LE implementations:

- `verified/writes/impl_ExclusiveBytes/write_u16_ne.coma`, line 139, calls `write_u16_le`; its postcondition uses `model_le_bytes` at line 149. See `write_u16_ne/proof.json`.
- `verified/writes/impl_ExclusiveBytes/try_write_uint_ne.coma`, line 148, calls `try_write_uint_le`; its postcondition uses `model_le_bytes` and preserves the sequence on failure at lines 157–164. See `try_write_uint_ne/proof.json`.

These are actual target-specific Coma bodies, paired with the captured 16-bit little-endian target cfg.
