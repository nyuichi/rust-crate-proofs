# Phase 8 — itoa 1.0.18 runtime memory and ASCII ledger

This ledger records the writer-state audit and the Phase 8 ASCII proof added on
top of source revision `6d150d3`. The proof-only changes reuse the existing
decimal models and do not change normal runtime behavior or the proof
configuration. Repository guidance was read from `AGENTS.md` and
`.agents/playbooks/verification.md`; the playbook requires clear separation of
proved bodies, external contracts, and trusted assumptions.

## Boundary and current proof state

`runtime.rs` contains the optimized production writers. Under `cfg(creusot)`,
the numeric writer bodies and their contracts are translated and proved. The
normal runtime entry and conversion are excluded: `Buffer::format` is guarded by
`#[cfg(not(creusot))]` at `runtime.rs:128-136`, and
`slice_buffer_to_str` by the same guard at `runtime.rs:346-354`. Creusot instead
exports the separate initialized-byte model in `verification.rs:765-827`.
Phase 7 establishes writer facts about `MaybeUninit` slots. Phase 8 proves that
the existing unsigned, signed, and whole-integer decimal models contain only
ASCII digits and an optional ASCII minus sign, then consumes that fact in the
actual i8 and `i128::MIN` initialized-output witnesses. This does not establish
safety of the public runtime casts or runtime borrowed-`str` construction.

There are two runtime raw-memory boundaries left in the path:

1. `Buffer::format` at `runtime.rs:129-132` obtains `self.bytes.as_mut_ptr()`,
   casts it to `*mut I::Buffer`, and forms `&mut *buf_ptr`. This reinterprets
   the first `N` bytes of the physical `[MaybeUninit<u8>; 40]` storage as the
   sealed implementation's typed `[MaybeUninit<u8>; N]` array. For every
   implementation `N <= 40`; the exact `N` values are tabulated below. The
   runtime proof has not checked this raw cast/reference construction, its
   typed extent, or its exclusivity/lifetime relation.
2. `slice_buffer_to_str` at `runtime.rs:348-353` uses
   `buf.get_unchecked(offset..)`, casts the resulting
   `&[MaybeUninit<u8>]` through raw slice pointers to `&[u8]`, then calls
   `str::from_utf8_unchecked`. The writers prove the logical requirements for
   the returned suffix (bounds, initialization, canonical bytes), but the
   raw slice reborrow/cast and `&str` construction are not in the runtime proof.
   `Buffer::format` also contains `unreachable_unchecked` behind a length guard
   (`runtime.rs:132-134`); the public method and guard are likewise outside the
   Creusot runtime proof.

The physical buffer is always 40 slots (`runtime.rs:93-96`, constructed fully
uninitialized by `runtime.rs:120-123`). The selected typed view is only its
first `N` slots. Each formatter proves the returned offset `o` satisfies
`o + output_len = N`, proves every slot in `[o, N)` is initialized and has the
canonical output byte, and frames the prefix `[0, o)`. For the fresh buffer
from `Buffer::new`, that framed prefix stays uninitialized; for a reused buffer
it stays in its prior `MaybeUninit` state and need not be uninitialized. The
string helper slices only `[o, N)`, not the physical tail `[N, 40)`.

## Per-formatter storage and output suffix

All intervals below are half-open. `N` is the associated typed array length,
not the physical `Buffer.bytes` length. Unsigned outputs are
`decimal_values(value)`; signed outputs are
`integer_decimal_values(value) == signed_decimal_values(value)`. Every digit is
ASCII `48..=57`; a negative signed result prepends byte `45` (`'-'`). The
contracts also preserve every slot before the returned offset.

| Runtime type/path | Typed `N` | Writer and proved result relation | Returned bytes live in |
| --- | ---: | --- | --- |
| `u8` | 3 | shared `impl_Unsigned!` body | `[o, 3)` = `decimal_values(value)` |
| `u16` | 5 | shared `impl_Unsigned!` body | `[o, 5)` = `decimal_values(value)` |
| `u32` | 10 | shared `impl_Unsigned!` body | `[o, 10)` = `decimal_values(value)` |
| `u64` | 20 | shared `impl_Unsigned!` body | `[o, 20)` = `decimal_values(value)` |
| `u128` | 39 | specialized `Unsigned::fmt` body; uses `enc_16lsd` for its fixed-width 16-digit chunks | `[o, 39)` = `decimal_values(value)` |
| `i8` | 4 | `write_u8_suffix_i8` then `signed_write_i8` | `[o, 4)` = signed decimal; magnitude writer uses local suffix `[1, 4)` |
| `i16` | 6 | macro-generated `write_u16_suffix_i16` then `signed_write_i16` | `[o, 6)` = signed decimal; magnitude writer uses `[1, 6)` (typed length 5) |
| `i32` | 11 | macro-generated `write_u32_suffix_i32` then `signed_write_i32` | `[o, 11)` = signed decimal; magnitude writer uses `[1, 11)` (typed length 10) |
| `i64` | 20 | macro-generated `write_u64_suffix_i64` then `signed_write_i64` | `[o, 20)` = signed decimal; magnitude writer receives the full `[0, 20)` array |
| `i128` | 40 | `write_u128_suffix_i128` then `signed_write_i128` | `[o, 40)` = signed decimal; magnitude writer is given the 39-slot subarray `[1, 40)` |
| `usize` on 16/32-bit targets | 5/10 | fallback casts to `u16`/`u32` and delegates to that writer | `[o, N)` = unsigned decimal |
| `isize` on 16/32-bit targets | 6/11 | fallback casts to `i16`/`i32` and delegates to that writer | `[o, N)` = signed decimal |
| `usize` on the proved x86_64 target | 20 | `fmt_usize_via_u64` delegates to actual `u64::fmt` | `[o, 20)` = unsigned decimal |
| `isize` on the proved x86_64 target | 20 | `fmt_isize_via_i64` delegates to actual `signed_write_i64` | `[o, 20)` = signed decimal |

Source locations for the capacities and dispatch are `runtime.rs:190-273`,
`runtime.rs:293-336`, and the Phase 7 signed invocations at `runtime.rs:1823-1837`.
The `u128` outer/suffix bridge is `runtime.rs:1377-1491`; the 16-digit encoder
calls and output proof are `runtime.rs:947-1054`. The `i8` adapter is
`runtime.rs:1493-1606`, and its sign write is `runtime.rs:1608-1645`. The
macro-generated signed adapters use a slice-to-array view of exactly the
unsigned capacity (`runtime.rs:1654-1819`).

For `i128::MIN`, the external `unsigned_abs` model gives magnitude `2^127`,
the unsigned writer supplies its 39 ASCII digits in outer `[1, 40)`, and the
signed writer stores `'-'` at index 0. The concrete witness at
`runtime.rs:1863-1877` proves all 40 slots initialized, equal to the signed
model, and ASCII; `verification.rs:1687-1691` records length 40, leading byte
45, and the 39-digit magnitude relation. Its canonical output is
`-170141183460469231731687303715884105728`.

## Writes, table reads, and prefix discipline

- `write_decimal_digit` writes only `buf[index]` with `b'0' + digit`; its
  contract requires `digit <= 9`, states the written slot is initialized, and
  preserves every other slot (`runtime.rs:356-380`). The unsigned formatter
  proves the low digit bound before calling it (`runtime.rs:770-820` and
  `runtime.rs:1247-1288`).
- `write_decimal_pair` uses two table reads and writes exactly two slots
  (`runtime.rs:382-456`). `write_decimal_quad` uses four table reads and writes
  exactly four slots (`runtime.rs:458-564`). Both require the already-written
  tail to be initialized, return canonical fixed-width digits, and frame slots
  outside the write range. The shared unsigned loops maintain an initialized
  suffix invariant and move the offset left by exactly 4, 2, or 1 slots
  (`runtime.rs:601-864`); the specialized `u128` loop and final writes do so at
  `runtime.rs:1075-1313`.
- The `u128` fixed-width stores are separately proved in `enc_16lsd.rs`: its
  quad writer establishes initialization and exact bytes in each four-slot
  range (`enc_16lsd.rs:430-523`), and `enc_16lsd` preserves the prefix and
  outside range while establishing all 16 output bytes (`enc_16lsd.rs:525-565`
  onward). The caller only treats a chunk as bytes after proving its slots are
  initialized (`runtime.rs:957-978`, `1000-1021`).
- The pair table has exactly 200 bytes and each pair is ASCII (`decimal_pairs.rs:4-5`,
  `:26-61`). `decimal_pair_correct` proves the actual indexed cells equal
  `48 + n/10` and `48 + n%10` under `n < 100` (`decimal_pairs.rs:63-74`). In
  normal builds, const evaluation checks the proof-side scalar initializer
  byte-for-byte against the runtime byte-string table (`decimal_pairs.rs:35-55`).
- Ghost `logical_slot_bytes` maps `None` to mathematical filler zero, but its
  source comment explicitly says this is bookkeeping and does not read or
  initialize memory (`verification.rs:23-40`). `logical_slot_states` maps
  `None` to `-1` for exact state/frame reasoning (`verification.rs:42-53`).
  Output-byte assertions are paired with an initialized-state predicate, such
  as the unsigned formatter's final suffix assertions (`runtime.rs:838-862`) or
  the `enc_16lsd` caller checks above. These ghost views are not memory reads.
  Consequently, an uninitialized prefix is never read by the proof or by the
  numeric writer. The runtime conversion is source-intended to select only the
  suffix proved initialized by the writer; safety of the raw range selection and
  conversion remains outside the runtime proof.

## Creusot standard models consumed by the actual bodies

These are external/core contracts used by the verified executable writer
bodies; they are assumptions about library operations, not locally trusted
formatter functions. Their concrete preconditions are discharged in the
existing actual bodies and adapter contracts.

- `MaybeUninit::write` in `creusot-libs/creusot-std/src/std/mem.rs:68-84`
  requires the old state to be `None` or resolved and ensures the new state is
  exactly `Some(value)`. The production digit/pair/quad stores call this
  operation. Since the element type is `u8`, initialized old values are
  resolvable; the formatter invariants separately track `None` versus
  initialized suffix slots. No formatter reads the old value of an uninitialized
  prefix.
- Slice `get_unchecked` in `creusot-libs/creusot-std/src/std/slice.rs:389-393`
  requires `ix.in_bounds(self@)` and returns the selected element. For a `usize`
  index that condition is `< slice.len()` (`:145-160`). The proof-side table
  checks establish all pair-derived indices `< 200` in `runtime.rs:418-424`
  and `:491-516`; the `enc_16lsd` writer does likewise at
  `enc_16lsd.rs:457-485`. Those checked conditions discharge the model's
  in-bounds precondition for the corresponding table accesses.
- The signed adapters use the narrow local array models in
  `itoa/1.0.18/src/array_models.rs:1-45`: array `IndexMut` requires an
  whitelisted index form and the standard in-bounds condition; the formatter
  uses `usize` and `RangeFrom<usize>`. The mutable-slice-to-array `TryFrom`
  requires exact length. The adapters establish it by slicing at the explicit
  capacity gap: e.g. `[1..]` has length 3 for i8, 5 for i16, 10 for i32, and
  39 for i128 (`runtime.rs:1396-1410`, `:1510-1525`, and `:1680-1695`). Their
  `unwrap`/`expect` calls therefore consume successful conversion facts, rather
  than assuming arbitrary conversion succeeds. The standard `TryInto` and
  `Result::expect` forwarding models are in `creusot-libs/creusot-std/src/std/convert.rs:29-35`
  and `std/option.rs:139`.
- Signed magnitude calls use local external models for primitive
  `unsigned_abs` in `signed_primitive_models.rs:1-32`. The exact relation
  includes signed `MIN`, which is used by the i128 witness. These models have
  no unproved memory postcondition.
- The real runtime table is a safe static, while Creusot verifies an equivalent
  `const` because this translator cannot accept the static; normal compilation
  enforces equality of the two 200-byte initializers at compile time
  (`decimal_pairs.rs:35-61`). This is a source/data representation bridge,
  not a memory-conversion contract.
- The accepted `mulhi` high-half result contract is a separate arithmetic
  assumption inherited by `u128::fmt`; it does not justify any pointer cast,
  initialization, or string construction (`RUNTIME_VERIFICATION.md:38-43`).

The core native bodies behind these external models are not translated by the
crate proof. The status and removal conditions are detailed in
`RUNTIME_VERIFICATION.md:299-331` and `PROVENANCE.md:78-82`. There are no local
`#[trusted]` declarations for the actual optimized formatter writers.

## Phase 8 ASCII proof

`verification.rs:557-617` proves the ASCII range by recursion over the existing
`decimal_values` model, lifts it through `signed_decimal_values`, and exposes
the result through `integer_decimal_values`. These lemmas define no alternate
decimal representation. The focused proofs discharged one goal each. The
actual `check_signed_i8_write` witness discharged 3 goals, including its
initialized suffix ASCII assertion; the full-buffer `i128::MIN` witness
discharged 4 goals, including all 40 initialized bytes and their ASCII range.
The official fresh default/all-features run also proves all five targets; its
full log is `/tmp/phase8-runtime-verify-all.log`.

The source-ground `MaybeUninit::write` model is
`creusot-libs/creusot-std/src/std/mem.rs:68-84`: it requires an uninitialized
slot or a resolved old value, and ensures the written slot becomes
`Some(value)`. The actual writers establish initialized-state and exact-byte
postconditions from that contract. The audit found no missing initialized-slot
fact. The temporary accepted high-half `mulhi` contract remains the only
formatter arithmetic assumption; it does not support either raw-memory
boundary above. The x86_64 pointer-sized writers are proved, while 16- and
32-bit pointer-width fallbacks remain outside the runtime proof.

## Separate recursive-model ASCII trust

`verification.rs:922-931` contains a pre-existing trusted
`decimal_slice_to_str(&[u8], start)`. Its precondition is only `start <= len`,
and its postcondition says the returned `str` bytes equal the selected suffix.
The caller has already proved the initialized-byte recursive model's canonical
ASCII suffix. This function works on the verification model's initialized
`[u8; 40]`, not the runtime `MaybeUninit` buffer. It does not establish that a
`MaybeUninit<u8>` suffix may be reinterpreted as `[u8]`, that the native
`get_unchecked` range is valid, or that the raw `&str` conversion in
`runtime.rs:348-353` is safe. The trusted recursive-model leaf must remain a
separate ledger item and cannot close either Phase 8 runtime boundary.

## Phase 8 disposition

The body proofs establish a strong handoff fact for every formatter:
`[offset, N)` is initialized and equals the canonical decimal result;
`[0, offset)` retains its old slot state. The Phase 8 ASCII lemmas prove the
range of the canonical result, and the i8 and `i128::MIN` witnesses connect
that range to actual initialized writer output. The Phase 8 integrated proof
reported 217 proof libraries / 1,880 VCs in default and 218 / 1,884 in
all-features, with no failed goals. The three ASCII model lemmas discharged
one VC each; the i8 witness discharged 3 and the `i128::MIN` witness 4. Native
default tests passed 11 integration tests and 2 doctests; release all-features
`--tests` passed all 11 integration tests. Logs are
`/tmp/phase8-runtime-verify-all.log`, `/tmp/phase8-native-default.log`, and
`/tmp/phase8-native-release-all-features-tests.log`. No additional
initialization fact is missing. The public 40-byte-to-typed-`N` mutable reference construction
and the initialized-suffix-to-`&str` raw conversion remain unproved runtime
memory boundaries. The earlier trusted string leaf applies only to the
initialized recursive model and does not close either boundary.
