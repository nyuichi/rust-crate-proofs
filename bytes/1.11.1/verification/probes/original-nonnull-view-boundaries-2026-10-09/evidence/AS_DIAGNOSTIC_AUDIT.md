# AS immutable diagnostic audit

This is an archive-only audit of the AS diagnostic and its three completed
negative controls. No Cargo build, Rust build, proof run, or source edit was
performed. The only replay was the saved-task printer over archived COMA.

## Archive closure and scope

The `as-full-diagnostic-v2` archive SHA-256 is
`abb0e4749cb8df54714cca8b689d559de920ba31c27cde2f1c9f661c937e6ea1`.
All 1,421 unique tar members match the receipt's member hashes. Recounting
the archived target list and proof trees gives 150 files, 1,328 prover
leaves, zero null leaves, and zero structural leaves. The archived target
policy has all 150 COMA files, no exclusions or features, and checker status
2 with `diagnostic: true`; this capture is not a correspondence pass or
admission.

The archive contains 110 private-Std files, 61 production `src` files plus
the captured package manifests, 7 tool/config inputs, and its captured AR
ancestor `ar-positive-canonical-v2.tar.gz` with SHA-256
`b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7`.
The run script records one proof worker and a 1,024 MiB Why3 limit. The
toolchain and Cargo target directory remain external/location-dependent.

The AS diagnostic's 39 native-capture/control members (30 MIR bodies plus
capture metadata and harness/checker artifacts) are byte-identical to the
AR canonical archive. The inherited native-control receipt records 76/76
rejections, no accepted mutations, and no Cargo/Rust build or solver. This
supports reuse of the frozen AR native gate; it is not a new AS native run.

## Negative controls and exact failed tasks

All control archive member hashes and proof trees were independently
recounted. Each failed task below was reconstructed from its archived COMA
and proof-tree tactic path using the printer; output matched the saved
sidecar byte for byte and its SHA-256.

| Archive | Statistics (files/prover/null/structural) | Failed task | Goal | SHA-256 |
| --- | ---: | --- | --- | --- |
| `as-negative-omit-view-nonnull-v1` (`ab5bbabd8fb0d784b6d7a7a1df4cca140fcd26fc6b83e9203db952a1429d526b`) | 150/1458/3/0 | `read_view`, path `0,12,1` | `not is_null_ptr_u8(value.ptr)` | `c33d5c80df4726da835207ef55b3d03172bb2c5fb33638537d42780e2c15b53f` |
| same | same | `slice_cursor_entry`, path `0,35,1,2,1,0` | `not is_null_ptr_u8(source.ptr)` | `c1d4dad66e8578a82d8f23cd338c96499e73611aa362eb9232e771a840d5663f` |
| same | same | `slice_view`, path `0,35,1,2,1,0` | `not is_null_ptr_u8(source.ptr)` | `b476075575f3d3823440c3d64dc1b77222a17871468fb48c5a3baf08ff73ab83` |
| `as-negative-weak-empty-boundary-v1` (`a706540e8c784903c0a051f5d22ffdd7a894211e6db0a290698943a9f0ab295d`) | 151/1332/1/0 | `weak_empty_boundary`, path `0,0,1` | `not is_null_ptr_u8(pointer)` given `inv_BoundPtr(bound)` and `pointer = raw_pointer(bound)` | `382c51cd2fac5e2bf97b33aeeee3d0152914de0c1127e35587ad5f61b65bf968` |
| `as-negative-weak-wrapping-boundary-v1` (`f6cea2ce8ef71ca69187810ecc35f9d8057cf2bc0d3d47e7b0c9509bd5cde078`) | 151/1334/1/0 | `weak_wrapping_boundary`, path `0,0,1` | `not is_null_ptr_u8(pointer)` given `inv_BoundPtr(bound)`, `bound@ = None`, and `pointer = raw_pointer(bound)` | `f147e0c7fd77fb86be0244a0e0783b4de4b5ed42f33beb8ab360f5e32cd42251` |

For `omit_view_nonnull`, the archived COMA proof goals show that the missing
generic nonnull fact is needed at the read/slice boundaries. For
`weak_empty_boundary` and `weak_wrapping_boundary`, the generic contracts do
not export nonnull from the stated metadata facts; the wrapping case
explicitly permits unbound metadata with `bound@ = None`. These are
specification-sensitivity results, not evidence that native `Bytes` can
contain a null pointer. In particular, `BoundPtr` stores a Std `NonNull`
value, but that representation fact must be made available to the generic
proof interface before a caller can rely on it.

## Reproduction caveat

The archived printer replay exited successfully for all five tasks and
matched all five sidecars. Why3 emitted dynamic-plugin load warnings, but the
printer exited 0 and produced the exact expected task text. This replay
invokes no prover and does not change the diagnostic/checker status.
