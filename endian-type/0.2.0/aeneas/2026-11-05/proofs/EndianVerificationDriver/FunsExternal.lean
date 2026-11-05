-- Model boundary: target-specific Rust scalar endian operations supplied using official Aeneas byte primitives.
-- Fixed signed values are handled as same-width bit patterns, not absolute values.
-- Pointer-sized swaps fail on unsupported Lean platform widths; public specs require numBits=64.
-- Explicit executable x86_64 little-endian integer models; official template retained.
-- [endian_verification_driver]: external functions.
-- This is a template file: rename it to "FunsExternal.lean" and fill the holes.
module
public import Aeneas
public import EndianVerificationDriver.Types
@[expose] public section
open Aeneas Aeneas.Std Result ControlFlow Error
set_option linter.dupNamespace false
set_option linter.hashCommand false
set_option linter.unusedVariables false
set_option linter.style.whitespace false
set_option linter.style.setOption false
set_option linter.style.longLine false
set_option linter.internalConstructors false

/- You can set the `maxHeartbeats` value with the `-max-heartbeats` CLI option -/
set_option maxHeartbeats 1000000

/- You can set the `maxRecDepth` value with the `-max-recdepth` CLI option -/
set_option maxRecDepth 2048
open endian_verification_driver

/-- [core::num::{i16}::to_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 540:8-540:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 405:4-424:5
    Name pattern: [core::num::{i16}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{i16}::to_be"]
def core.num.I16.to_be (x : Std.I16) : Result Std.I16 :=
  .ok (core.num.I16.from_le_bytes (core.num.I16.to_be_bytes x))

/-- [core::num::{i16}::to_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 573:8-573:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 405:4-424:5
    Name pattern: [core::num::{i16}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{i16}::to_le"]
def core.num.I16.to_le (x : Std.I16) : Result Std.I16 :=
  .ok x

/-- [core::num::{i16}::from_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 479:8-479:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 405:4-424:5
    Name pattern: [core::num::{i16}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{i16}::from_be"]
def core.num.I16.from_be (x : Std.I16) : Result Std.I16 :=
  .ok (core.num.I16.from_le_bytes (core.num.I16.to_be_bytes x))

/-- [core::num::{i16}::from_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 507:8-507:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 405:4-424:5
    Name pattern: [core::num::{i16}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{i16}::from_le"]
def core.num.I16.from_le (x : Std.I16) : Result Std.I16 :=
  .ok x

/-- [core::num::{i16}::swap_bytes]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 433:8-433:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 405:4-424:5
    Name pattern: [core::num::{i16}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{i16}::swap_bytes"]
def core.num.I16.swap_bytes (x : Std.I16) : Result Std.I16 :=
  .ok (core.num.I16.from_le_bytes (core.num.I16.to_be_bytes x))

/-- [core::num::{i32}::to_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 540:8-540:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 430:4-449:5
    Name pattern: [core::num::{i32}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{i32}::to_be"]
def core.num.I32.to_be (x : Std.I32) : Result Std.I32 :=
  .ok (core.num.I32.from_le_bytes (core.num.I32.to_be_bytes x))

/-- [core::num::{i32}::to_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 573:8-573:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 430:4-449:5
    Name pattern: [core::num::{i32}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{i32}::to_le"]
def core.num.I32.to_le (x : Std.I32) : Result Std.I32 :=
  .ok x

/-- [core::num::{i32}::from_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 479:8-479:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 430:4-449:5
    Name pattern: [core::num::{i32}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{i32}::from_be"]
def core.num.I32.from_be (x : Std.I32) : Result Std.I32 :=
  .ok (core.num.I32.from_le_bytes (core.num.I32.to_be_bytes x))

/-- [core::num::{i32}::from_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 507:8-507:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 430:4-449:5
    Name pattern: [core::num::{i32}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{i32}::from_le"]
def core.num.I32.from_le (x : Std.I32) : Result Std.I32 :=
  .ok x

/-- [core::num::{i32}::swap_bytes]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 433:8-433:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 430:4-449:5
    Name pattern: [core::num::{i32}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{i32}::swap_bytes"]
def core.num.I32.swap_bytes (x : Std.I32) : Result Std.I32 :=
  .ok (core.num.I32.from_le_bytes (core.num.I32.to_be_bytes x))

/-- [core::num::{i64}::to_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 540:8-540:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 455:4-474:5
    Name pattern: [core::num::{i64}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{i64}::to_be"]
def core.num.I64.to_be (x : Std.I64) : Result Std.I64 :=
  .ok (core.num.I64.from_le_bytes (core.num.I64.to_be_bytes x))

/-- [core::num::{i64}::to_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 573:8-573:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 455:4-474:5
    Name pattern: [core::num::{i64}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{i64}::to_le"]
def core.num.I64.to_le (x : Std.I64) : Result Std.I64 :=
  .ok x

/-- [core::num::{i64}::from_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 479:8-479:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 455:4-474:5
    Name pattern: [core::num::{i64}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{i64}::from_be"]
def core.num.I64.from_be (x : Std.I64) : Result Std.I64 :=
  .ok (core.num.I64.from_le_bytes (core.num.I64.to_be_bytes x))

/-- [core::num::{i64}::from_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 507:8-507:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 455:4-474:5
    Name pattern: [core::num::{i64}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{i64}::from_le"]
def core.num.I64.from_le (x : Std.I64) : Result Std.I64 :=
  .ok x

/-- [core::num::{i64}::swap_bytes]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 433:8-433:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 455:4-474:5
    Name pattern: [core::num::{i64}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{i64}::swap_bytes"]
def core.num.I64.swap_bytes (x : Std.I64) : Result Std.I64 :=
  .ok (core.num.I64.from_le_bytes (core.num.I64.to_be_bytes x))

/-- [core::num::{i128}::to_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 540:8-540:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 480:4-501:5
    Name pattern: [core::num::{i128}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{i128}::to_be"]
def core.num.I128.to_be (x : Std.I128) : Result Std.I128 :=
  .ok (core.num.I128.from_le_bytes (core.num.I128.to_be_bytes x))

/-- [core::num::{i128}::to_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 573:8-573:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 480:4-501:5
    Name pattern: [core::num::{i128}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{i128}::to_le"]
def core.num.I128.to_le (x : Std.I128) : Result Std.I128 :=
  .ok x

/-- [core::num::{i128}::from_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 479:8-479:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 480:4-501:5
    Name pattern: [core::num::{i128}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{i128}::from_be"]
def core.num.I128.from_be (x : Std.I128) : Result Std.I128 :=
  .ok (core.num.I128.from_le_bytes (core.num.I128.to_be_bytes x))

/-- [core::num::{i128}::from_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 507:8-507:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 480:4-501:5
    Name pattern: [core::num::{i128}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{i128}::from_le"]
def core.num.I128.from_le (x : Std.I128) : Result Std.I128 :=
  .ok x

/-- [core::num::{i128}::swap_bytes]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 433:8-433:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 480:4-501:5
    Name pattern: [core::num::{i128}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{i128}::swap_bytes"]
def core.num.I128.swap_bytes (x : Std.I128) : Result Std.I128 :=
  .ok (core.num.I128.from_le_bytes (core.num.I128.to_be_bytes x))

/-- [core::num::{isize}::to_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 540:8-540:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 560:4-579:5
    Name pattern: [core::num::{isize}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{isize}::to_be"]
def core.num.Isize.to_be (x : Std.Isize) : Result Std.Isize :=
  if System.Platform.numBits = 64 then .ok (IScalar.cast .Isize (core.num.I64.from_le_bytes (core.num.I64.to_be_bytes (IScalar.cast .I64 x)))) else .fail .panic

/-- [core::num::{isize}::to_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 573:8-573:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 560:4-579:5
    Name pattern: [core::num::{isize}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{isize}::to_le"]
def core.num.Isize.to_le (x : Std.Isize) : Result Std.Isize :=
  .ok x

/-- [core::num::{isize}::from_be]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 479:8-479:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 560:4-579:5
    Name pattern: [core::num::{isize}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{isize}::from_be"]
def core.num.Isize.from_be (x : Std.Isize) : Result Std.Isize :=
  if System.Platform.numBits = 64 then .ok (IScalar.cast .Isize (core.num.I64.from_le_bytes (core.num.I64.to_be_bytes (IScalar.cast .I64 x)))) else .fail .panic

/-- [core::num::{isize}::from_le]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 507:8-507:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 560:4-579:5
    Name pattern: [core::num::{isize}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{isize}::from_le"]
def core.num.Isize.from_le (x : Std.Isize) : Result Std.Isize :=
  .ok x

/-- [core::num::{isize}::swap_bytes]:
    Source: '/rustc/library/core/src/num/int_macros.rs', lines 433:8-433:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 560:4-579:5
    Name pattern: [core::num::{isize}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{isize}::swap_bytes"]
def core.num.Isize.swap_bytes (x : Std.Isize) : Result Std.Isize :=
  if System.Platform.numBits = 64 then .ok (IScalar.cast .Isize (core.num.I64.from_le_bytes (core.num.I64.to_be_bytes (IScalar.cast .I64 x)))) else .fail .panic

/-- [core::num::{u16}::to_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 908:8-908:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1236:4-1260:5
    Name pattern: [core::num::{u16}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{u16}::to_be"]
def core.num.U16.to_be (x : Std.U16) : Result Std.U16 :=
  .ok (core.num.U16.from_le_bytes (core.num.U16.to_be_bytes x))

/-- [core::num::{u16}::to_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 936:8-936:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1236:4-1260:5
    Name pattern: [core::num::{u16}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{u16}::to_le"]
def core.num.U16.to_le (x : Std.U16) : Result Std.U16 :=
  .ok x

/-- [core::num::{u16}::from_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 853:8-853:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1236:4-1260:5
    Name pattern: [core::num::{u16}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{u16}::from_be"]
def core.num.U16.from_be (x : Std.U16) : Result Std.U16 :=
  .ok (core.num.U16.from_le_bytes (core.num.U16.to_be_bytes x))

/-- [core::num::{u16}::from_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 880:8-880:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1236:4-1260:5
    Name pattern: [core::num::{u16}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{u16}::from_le"]
def core.num.U16.from_le (x : Std.U16) : Result Std.U16 :=
  .ok x

/-- [core::num::{u16}::swap_bytes]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 772:8-772:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1236:4-1260:5
    Name pattern: [core::num::{u16}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{u16}::swap_bytes"]
def core.num.U16.swap_bytes (x : Std.U16) : Result Std.U16 :=
  .ok (core.num.U16.from_le_bytes (core.num.U16.to_be_bytes x))

/-- [core::num::{u32}::to_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 908:8-908:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1292:4-1316:5
    Name pattern: [core::num::{u32}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{u32}::to_be"]
def core.num.U32.to_be (x : Std.U32) : Result Std.U32 :=
  .ok (core.num.U32.from_le_bytes (core.num.U32.to_be_bytes x))

/-- [core::num::{u32}::to_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 936:8-936:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1292:4-1316:5
    Name pattern: [core::num::{u32}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{u32}::to_le"]
def core.num.U32.to_le (x : Std.U32) : Result Std.U32 :=
  .ok x

/-- [core::num::{u32}::from_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 853:8-853:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1292:4-1316:5
    Name pattern: [core::num::{u32}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{u32}::from_be"]
def core.num.U32.from_be (x : Std.U32) : Result Std.U32 :=
  .ok (core.num.U32.from_le_bytes (core.num.U32.to_be_bytes x))

/-- [core::num::{u32}::from_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 880:8-880:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1292:4-1316:5
    Name pattern: [core::num::{u32}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{u32}::from_le"]
def core.num.U32.from_le (x : Std.U32) : Result Std.U32 :=
  .ok x

/-- [core::num::{u32}::swap_bytes]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 772:8-772:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1292:4-1316:5
    Name pattern: [core::num::{u32}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{u32}::swap_bytes"]
def core.num.U32.swap_bytes (x : Std.U32) : Result Std.U32 :=
  .ok (core.num.U32.from_le_bytes (core.num.U32.to_be_bytes x))

/-- [core::num::{u64}::to_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 908:8-908:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1324:4-1348:5
    Name pattern: [core::num::{u64}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{u64}::to_be"]
def core.num.U64.to_be (x : Std.U64) : Result Std.U64 :=
  .ok (core.num.U64.from_le_bytes (core.num.U64.to_be_bytes x))

/-- [core::num::{u64}::to_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 936:8-936:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1324:4-1348:5
    Name pattern: [core::num::{u64}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{u64}::to_le"]
def core.num.U64.to_le (x : Std.U64) : Result Std.U64 :=
  .ok x

/-- [core::num::{u64}::from_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 853:8-853:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1324:4-1348:5
    Name pattern: [core::num::{u64}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{u64}::from_be"]
def core.num.U64.from_be (x : Std.U64) : Result Std.U64 :=
  .ok (core.num.U64.from_le_bytes (core.num.U64.to_be_bytes x))

/-- [core::num::{u64}::from_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 880:8-880:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1324:4-1348:5
    Name pattern: [core::num::{u64}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{u64}::from_le"]
def core.num.U64.from_le (x : Std.U64) : Result Std.U64 :=
  .ok x

/-- [core::num::{u64}::swap_bytes]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 772:8-772:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1324:4-1348:5
    Name pattern: [core::num::{u64}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{u64}::swap_bytes"]
def core.num.U64.swap_bytes (x : Std.U64) : Result Std.U64 :=
  .ok (core.num.U64.from_le_bytes (core.num.U64.to_be_bytes x))

/-- [core::num::{u128}::to_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 908:8-908:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1356:4-1382:5
    Name pattern: [core::num::{u128}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{u128}::to_be"]
def core.num.U128.to_be (x : Std.U128) : Result Std.U128 :=
  .ok (core.num.U128.from_le_bytes (core.num.U128.to_be_bytes x))

/-- [core::num::{u128}::to_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 936:8-936:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1356:4-1382:5
    Name pattern: [core::num::{u128}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{u128}::to_le"]
def core.num.U128.to_le (x : Std.U128) : Result Std.U128 :=
  .ok x

/-- [core::num::{u128}::from_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 853:8-853:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1356:4-1382:5
    Name pattern: [core::num::{u128}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{u128}::from_be"]
def core.num.U128.from_be (x : Std.U128) : Result Std.U128 :=
  .ok (core.num.U128.from_le_bytes (core.num.U128.to_be_bytes x))

/-- [core::num::{u128}::from_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 880:8-880:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1356:4-1382:5
    Name pattern: [core::num::{u128}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{u128}::from_le"]
def core.num.U128.from_le (x : Std.U128) : Result Std.U128 :=
  .ok x

/-- [core::num::{u128}::swap_bytes]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 772:8-772:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1356:4-1382:5
    Name pattern: [core::num::{u128}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{u128}::swap_bytes"]
def core.num.U128.swap_bytes (x : Std.U128) : Result Std.U128 :=
  .ok (core.num.U128.from_le_bytes (core.num.U128.to_be_bytes x))

/-- [core::num::{usize}::to_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 908:8-908:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1454:4-1478:5
    Name pattern: [core::num::{usize}::to_be]
    Visibility: public -/
@[rust_fun "core::num::{usize}::to_be"]
def core.num.Usize.to_be (x : Std.Usize) : Result Std.Usize :=
  if System.Platform.numBits = 64 then .ok (UScalar.cast .Usize (core.num.U64.from_le_bytes (core.num.U64.to_be_bytes (UScalar.cast .U64 x)))) else .fail .panic

/-- [core::num::{usize}::to_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 936:8-936:40; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1454:4-1478:5
    Name pattern: [core::num::{usize}::to_le]
    Visibility: public -/
@[rust_fun "core::num::{usize}::to_le"]
def core.num.Usize.to_le (x : Std.Usize) : Result Std.Usize :=
  .ok x

/-- [core::num::{usize}::from_be]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 853:8-853:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1454:4-1478:5
    Name pattern: [core::num::{usize}::from_be]
    Visibility: public -/
@[rust_fun "core::num::{usize}::from_be"]
def core.num.Usize.from_be (x : Std.Usize) : Result Std.Usize :=
  if System.Platform.numBits = 64 then .ok (UScalar.cast .Usize (core.num.U64.from_le_bytes (core.num.U64.to_be_bytes (UScalar.cast .U64 x)))) else .fail .panic

/-- [core::num::{usize}::from_le]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 880:8-880:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1454:4-1478:5
    Name pattern: [core::num::{usize}::from_le]
    Visibility: public -/
@[rust_fun "core::num::{usize}::from_le"]
def core.num.Usize.from_le (x : Std.Usize) : Result Std.Usize :=
  .ok x

/-- [core::num::{usize}::swap_bytes]:
    Source: '/rustc/library/core/src/num/uint_macros.rs', lines 772:8-772:45; this code is generated from a macro invocation at: '/rustc/library/core/src/num/mod.rs', lines 1454:4-1478:5
    Name pattern: [core::num::{usize}::swap_bytes]
    Visibility: public -/
@[rust_fun "core::num::{usize}::swap_bytes"]
def core.num.Usize.swap_bytes (x : Std.Usize) : Result Std.Usize :=
  if System.Platform.numBits = 64 then .ok (UScalar.cast .Usize (core.num.U64.from_le_bytes (core.num.U64.to_be_bytes (UScalar.cast .U64 x)))) else .fail .panic

