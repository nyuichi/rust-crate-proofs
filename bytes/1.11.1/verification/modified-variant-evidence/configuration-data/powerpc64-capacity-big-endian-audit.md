# PowerPC64 capacity and native-endian translation audit

This note maps the isolated PowerPC64 proof gate to the target-width and target-endian evidence.

## Gate and source correspondence

- Target: `powerpc64-unknown-linux-gnu`; `rustc --print cfg` reports `target_pointer_width="64"` and `target_endian="big"` in `rustc-cfg-powerpc64.txt`.
- The native configuration gate passed for `verified,std,extra-platforms`; the effective proof graph adds `creusot-std/creusot` and `creusot-std/nightly`. The captured feature graphs contain no `sc-drf` feature.
- Root source snapshot: commit `dcb7e29efcf1de30f1ca1058c7255009f75ec8ba`. The copied Cargo manifests and compiled Rust source hashes match the root snapshot. `source-correspondence.json` records every copied source/manifest digest and separates the probe-only wrapper cache override.
- Full proof: `Proved (287 files)`, exit 0; 287 Coma files, 287 proof JSON files, zero null proof leaves. Archive: `runs/positive/powerpc64-verified-std-final-config/evidence.tar.gz` (SHA-256 `e60fd65a74dca7f68dc29566e03a83266d63279c4b7fdde35ab7ed518373b08b`).

## Capacity and `usize` evidence

In the translated `reserve_then_close.coma`:

- Lines 21 and 30 use `UInt64.t`; `const_MAX` is `18446744073709551615` (line 30).
- Lines 34–36 state the vector view length is bounded by that `UInt64` maximum.
- Line 45 declares the opaque `capacity_model_u8` over the actual `Vec<u8>` value. The observer is not a sequence or pointer-identity model.
- Lines 47–54 give `reserve`'s postconditions: preserve the sequence, keep capacity monotone, and raise capacity to at least old length plus `additional`.
- Lines 56–61 expose capacity as `UInt64`, equate it to the opaque observer, and require capacity at least the current sequence length.
- Lines 69–94 show the compiled caller saving the original sequence, reserving, asserting contents are unchanged, querying capacity, asserting the lower bound, closing the owner, and returning capacity.
- The caller's VCs are in `verified/exclusive/reserve_then_close/proof.json`.

## Actual native-endian branch

The translated implementation selected the big-endian branch for this target:

- In `verified/writes/impl_ExclusiveBytes/write_u64_ne.coma`, `write_u64_ne` calls `write_u64_be` at line 141; the callee contract appends `model_be_bytes` at lines 116–121. Its proof record is `write_u64_ne/proof.json`.
- In `verified/writes/impl_ExclusiveBytes/try_write_uint_ne.coma`, `try_write_uint_ne` calls `try_write_uint_be` at line 149. The translated ensures state that success appends `model_be_bytes` and failure preserves the sequence at lines 158–165. The proof record is `try_write_uint_ne/proof.json`.

These are the target-specific Coma outputs from the captured PowerPC64 run, and the accompanying target cfg records big endian. They provide direct evidence that the native-endian APIs translated through their BE implementations on this target.
