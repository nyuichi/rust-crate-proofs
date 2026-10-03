# itoa 1.0.18: runtime memory-boundary status and historical audits

The current source takes the actual optimized `Buffer::format` path under
`cfg(creusot)`. The integrated proof passes in default and all-features
configurations (254 libraries / 2,083 VCs and 269 / 2,145, respectively).
The two source changes
explicitly accepted for that path are recorded in
[RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md): Boundary A uses a
safe typed-prefix `.try_into().unwrap()`; Boundary B isolates the initialized
`MaybeUninit<u8>`-to-`u8` slice view in one narrow local trusted helper. These
changes are acknowledged departures from the preferred unchanged source. The
caller proves the helper's initialization precondition in the Creusot writer
model; the trusted helper covers the representation conversion and
byte/length relation only. It assumes no ASCII or decimal fact. There is no
Verus proof or cross-tool correspondence claim for it. The 16- and 32-bit
`usize`/`isize` fallback adapters remain
outside the current x86_64 scope.

## Current evidence and exact status

The existing numeric-writer contracts establish initialized canonical output
in `[o,N)` and preserve the `MaybeUninit` state of `[0,o)`. The physical
`Buffer.bytes` array has 40 slots; each sealed typed buffer length `N` is at
most 40. The accepted Boundary A source edit replaces the old raw typed-prefix
cast with a safe fixed-array conversion. Focused body/refinement proofs have
established successful conversion for all 12 concrete x86_64 writers; the
original raw A cast remains unproved.

The accepted Boundary B edit keeps the raw slice reinterpretation inside
`assume_init_slice`. Its contract requires all selected slots to be initialized
and preserves slice length and each byte value. The caller proves the logical
initialization precondition; the helper trusts only the representation
conversion and its memory-validity relation. The integrated proof establishes
the suffix bounds, initialization, ASCII/UTF-8, exact returned bytes, and the
public `unreachable_unchecked` length guard. The physical memory-permission
transfer for the raw view remains trusted.

The caller proves the initialization precondition in Creusot's logical writer
model. The physical permission transfer and soundness of the helper's raw byte
view remain inside the trusted contract; Creusot does not prove those concrete
permissions. The old Verus audit is separate historical evidence and provides
no correspondence proof for the current Creusot assumption.

The Phase 9/10 diagnostics and Phase 11 Verus API audit below are historical
investigations of the original raw expressions. They explain why A and B were
changed; they do not describe the current source path.

**Historical Creusot probe status:** the Phase 9 concrete i8 cast probe and
Phase 10 slice probe stopped during translation at raw-pointer dereferences,
before Why3 and before VCs. Neither probe installed a trusted contract or
changed the source tree. Phase 10 did not reach the ASCII-to-UTF-8 obligation.

The exact translator diagnostics and rejected source expressions were:

```text
Phase 9 — scratch only: /tmp/itoa-phase9-cast/itoa/1.0.18/src/runtime.rs
error: Dereference of a raw pointer is forbidden in creusot: use `creusot_std::ghost::perm::Perm<*const T>` instead
  --> src/runtime.rs:153:14
153 | unsafe { &mut *buf_ptr }
    |          ^^^^^^^^^^^^^

Phase 10 — scratch only: /tmp/phase10-slice/itoa/1.0.18/src/runtime.rs
error: Dereference of a raw pointer is forbidden in creusot:
       use `creusot_std::ghost::perm::Perm<*const T>` instead
 --> src/runtime.rs:365:39
365 | unsafe { str::from_utf8_unchecked(&*(written as *const [MaybeUninit<u8>] as *const [u8])) }
    |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

These line numbers belong to the two isolated scratch probes, not the checked-in `runtime.rs`; both diagnostics occurred during translation, before Why3 or VC generation.

**Historical Verus status:** the Phase 11 cast audit was model-blocked; no raw-memory proof completed. The pinned vstd models did not provide the typed-prefix permission for the borrowed stack array. The Phase 11 slice probe proved an ASCII-to-UTF-8/string lemma for an already-formed `&[u8]` (1 verified, 0 errors), but did not prove the preceding `MaybeUninit<u8>`-to-`u8` conversion.

**Current cross-tool status:** the trusted Creusot helper is documented as a local assumption in the source and in `RUNTIME_BOUNDARY_BRIDGE.md`. No Verus proof or cross-tool correspondence assertion is made. Arithmetic and decimal facts are proved as Creusot obligations; none is
imported as a Verus premise.

## Phase 12 status summary

| Area | Current evidence and status |
| --- | --- |
| A. Creusot functional decimal model | The existing canonical decimal model remains the specification. The strengthened ASCII precondition on the former recursive-model string leaf and its ordinary body pass in the integrated proof. The actual unsigned/signed writer and `mulhi` body results are recorded in `RUNTIME_VERIFICATION.md`. |
| B. Runtime operation coverage | The actual public `Buffer::format` path is compiled under `cfg(creusot)`. The integrated proof establishes safe prefix conversions, all writer contracts, suffix bounds/initialization/ASCII facts, returned string bytes, and guard unreachability. Boundary B's byte-slice representation view alone is trusted under the exact contract in `RUNTIME_BOUNDARY_BRIDGE.md`. |
| C. Verus evidence | The standalone [`verus/ascii_bytes_to_str.rs`](verus/ascii_bytes_to_str.rs) artifact proves an ASCII fact for an already-formed byte slice using vstd's assumed `from_utf8_unchecked` specification (historical partial result). It does not prove the trusted helper's raw conversion. |
| D. Cross-tool correspondence | None is asserted. No decimal correctness fact crosses from Creusot to Verus. |
| E. Separate assumptions | Current local runtime trust: `assume_init_slice` only. Narrow Creusot standard/core models remain separately documented. The former trusted recursive-model string leaf was removed and its ASCII precondition strengthened; its body and caller pass. The Verus standard-library model applies only to the separate partial artifact. |

The end-to-end x86_64 result is proved under the stated Creusot models and the
local Boundary B trust. User-requested source deviations A and B are explicit;
the helper contract does not include decimal correctness, and no other
formatter or arithmetic fact was accepted without a proved body. Native tests
are reported separately from the deductive result.

## Supplementary checker feasibility: Kani and Miri

This is a capability assessment, not a completed Kani or Miri verification.
Neither `cargo-kani` nor `kani` is installed in this workspace. A `cargo-miri`
launcher exists, but the selected nightly toolchain lacks the Miri component;
no Miri run was performed.

| Tool | Useful check against the actual runtime path | Limit for the outstanding claim |
| --- | --- | --- |
| Kani / CBMC | Separate concrete-type harnesses can make the integer input symbolic, call the production `Buffer::format`, assert returned length and bytes, exercise buffer reuse, and check modeled bounds/initialization failures. Smaller harnesses around each actual raw operation could check capacities, offsets, ASCII, and suffix bytes. A successful exhaustive bitvector check requires complete type-specific loop unwinding; solver feasibility, especially for `u128`, and support for this crate's `MaybeUninit` and raw slice casts must be measured with a pinned Kani version. | A pass means no counterexample within Kani's configured and supported memory/UB model. It does not on its own establish Rust abstract-machine provenance, reference uniqueness, or borrow lifetime for the cast/reborrow, nor does it provide the missing cross-tool contract. It must not be labeled a proof of those obligations without checking that version's exact semantics. |
| Miri | Execute the unchanged public function on boundary values, small exhaustive domains such as `u8`, and repeated use of one buffer. Its Rust interpreter can reveal undefined behavior in executed raw-pointer, initialization, and aliasing paths under its chosen borrow/provenance mode. | Dynamic runs cover only executed values and paths. They cannot establish the forall-input property for `u128` or supply a compositional contract for Creusot. |

If Kani is added later, the harness should explicitly pin target width,
unwinding bounds, active undefined-behavior checks, and any unsupported
intrinsic/model assumptions. Test the raw cast and suffix conversion in source
or in extracted source-identical helpers, and state how a helper corresponds to
the production call. An independent decimal oracle can check end-to-end output,
but it does not replace the already-proved Creusot decimal specification.
Kani and Miri would improve confidence and may expose a counterexample; neither
proves the original raw A cast or removes the remaining physical memory-permission
trust in Boundary B as a Rust-semantic proof.

## Historical Boundary A analysis — original raw prefix cast

The contract exploration below refers to the source before the accepted
`.try_into().unwrap()` change. It records why a direct raw cast was difficult
to model; it is not the current source contract. The current A contract and
removal condition are in [RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md).

### Candidate contract target

For each sealed implementation, let `N` be the concrete length of `I::Buffer = [MaybeUninit<u8>; N]`. The memory-only operation should provide a reborrow of the input allocation's prefix, with these obligations:

- `N <= 40`; source and target have alignment 1; the target extent is exactly `N` slots.
- The returned reference has the same base address and allocation provenance as `bytes`, and its lifetime is bounded by the exclusive input borrow.
- The returned `&mut I::Buffer` grants unique mutable access to `[0,N)` for that lifetime. It must not create a second live mutable alias; the physical suffix `[N,40)` remains outside the typed view and is unchanged by this view-formation step.
- The preexisting per-slot `MaybeUninit` state in `[0,N)` is represented without claiming that any payload is initialized. Forming this view is valid for uninitialized `MaybeUninit<u8>` slots. Subsequent writer reads still require their slots to have been initialized; subsequent writer writes must be confined to `[0,N)`.
- Any operation that observes slots `[N,40)` must be disjoint from this borrow. A writer postcondition is a separate step: it may change the selected prefix according to its writer contract, while preserving the unselected physical tail.

The source-backed capacity table is: `u8:3`, `u16:5`, `u32:10`, `u64:20`, `u128:39`; `i8:4`, `i16:6`, `i32:11`, `i64:20`, `i128:40`; `usize/isize:20` on the x86_64 target. No portability claim is made for the 16/32-bit pointer-width adapters.

| Creusot candidate | Verus candidate |
| --- | --- |
| Model a helper from `&'a mut [MaybeUninit<u8>;40]` to `&'a mut I::Buffer` with preconditions for the concrete `N`, `N <= 40`, equal alignment, and the source's unique borrow. Its postcondition should relate the target view element-for-element to the source prefix at the same base/provenance, frame `[N,40)`, and preserve the abstract `MaybeUninit` slot states. The later writer call consumes only this prefix. This is a specification target, not current Creusot syntax: the Phase 9 translation fails at the raw dereference before Why3. | A future sound vstd helper would need to start from the live borrowed `[MaybeUninit<u8>;40]`, derive a unique typed-prefix capability for `[MaybeUninit<u8>;N]`, and expose the same pointer/provenance, exact range, alignment, lifetime, and frame facts. It must allow reference formation for `MaybeUninit` even when payload bytes are uninitialized; it must not fabricate readable payload permission. Current vstd has no borrowed stack-array-to-`PointsTo`/typed-prefix bridge. A hypothetical `PointsToRaw` token plus `into_typed` is insufficient because the resulting typed token is `Uninit`, while `ptr_mut_ref` requires `PointsTo::is_init()`. This contract cannot currently be discharged with the audited APIs. |

### Missing model and removal condition

The cast itself has a pointer-cast address/provenance model in vstd, and Creusot's pointer cast contract is not the issue identified by the Phase 9 probe. The blocked operation is reference formation and permission ownership for the typed prefix. The Verus audit further identifies the specific missing relation: turn an existing exclusive borrow of this stack array into a typed-prefix capability for `MaybeUninit` elements, while preserving uninitialized slot state. `MaybeUninit`'s equal-size/alignment fact alone does not provide that permission transformation.

A future proof may close this only with a validated safe reborrow/splitting model or a proved standard-model helper for the concrete stack storage and prefix. Adding an `assume_specification`, external body, raw permission axiom, or trusted cast contract would instead create a new trusted memory-safety boundary and would not prove this operation. Removal condition: prove the prefix reborrow and frame from the source borrow (including its uninitialized `MaybeUninit` validity), then compose it with the existing writer contracts.

## Historical Boundary B analysis — combined suffix and string conversion

The candidate below combined byte-view formation, ASCII validity, and `str`
construction. The accepted current helper intentionally has a narrower
contract: it assumes only initialized input and preserves byte values/length;
it contains no ASCII premise. See
[RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md). The remainder records
the earlier tool analysis, not the current trusted contract.

### Candidate contract target

Let `buf` have length `N` and `0 <= offset <= N`. For each `i` in `[offset,N)`, require that the outer slot is a valid initialized `MaybeUninit<u8>` value and that the nested `MaybeUninit` payload state is `Init(b_i)` for some `b_i < 128`. The writer ledger establishes these facts for the formatter-selected suffix; its canonical output sequence is also ASCII. The raw slice-conversion step should then:

- return an immutable `&[u8]` of length `N-offset`, over the exact original suffix range with the same allocation provenance;
- establish `result[j] == b_(offset+j)` for every `j < N-offset`;
- preserve the shared borrow lifetime and prevent conflicting mutation while the result is live;
- establish `valid_utf8(result)` from the ASCII-byte range;
- let `from_utf8_unchecked` return a `&str` whose bytes equal the same selected byte sequence, with the same suffix length.

The byte-slice reinterpretation and the unchecked UTF-8 conversion are conceptually separate contracts. In Verus terminology, the memory fact requires both the outer per-element permission (`PointsTo<MaybeUninit<u8>>` records `Init(slot_i)`) and the nested `slot_i.mem_contents() == Init(b_i)` fact. A raw range token alone is not enough to read these payloads. The `&str` step additionally consumes a proved `valid_utf8` fact.

| Creusot candidate | Verus candidate |
| --- | --- |
| Give the suffix-conversion helper preconditions `offset <= len`, initialized suffix slots, and an ASCII bound for every selected byte; postcondition `result@.to_bytes()` equals the modeled `MaybeUninit` suffix values. Then show the ASCII witness discharges `from_utf8_unchecked`'s UTF-8 precondition and the result's modeled bytes equal the same sequence. Phase 10 used `result@.to_bytes()` because calling executable `result.as_bytes()` in a logic attribute was rejected. The probe translated `get_unchecked`, then rejected the unchanged raw slice dereference before UTF-8 proof obligations. Thus neither conversion nor the ASCII-to-UTF-8 witness was proved for the runtime function. | First prove a suffix conversion from the nested initialized payload permissions to a shared `&[u8]` preserving values, length, pointer range, provenance, and borrow lifetime. Require both initialized outer slot permission and `MaybeUninitAdditionalSpecFns::mem_contents() == Init(byte)` for every element, plus `byte < 128`. The post-conversion sub-lemma is now proved in checked-in artifact [`verus/ascii_bytes_to_str.rs`](verus/ascii_bytes_to_str.rs): given an already-formed `&[u8]` whose elements are at most 127, it proves `valid_utf8(bytes@)` with `partial_valid_utf8_extend_ascii_block` and calls `str::from_utf8_unchecked`. Verus reports `1 verified, 0 errors`. Its postcondition `result.spec_bytes() =~= bytes@` consumes vstd’s existing assumed `from_utf8_unchecked` specification. This is not a proof of the raw suffix conversion or of that standard-library implementation. |

### Missing model and removal condition

Phase 10 precisely located the Creusot blocker at `&*(written as *const [MaybeUninit<u8>] as *const [u8])`. The preconditions (suffix bounds, initialized slots, ASCII) were written as a probe target, and that probe's body was unchanged; translation stopped before Why3. At that checkpoint, the recursive verification model still had a trusted initialized-`[u8;40]` ASCII-to-`str` leaf. That leaf has since been removed with an ASCII precondition and the same byte postcondition; neither version establishes the runtime `assume_init_slice` representation conversion.

The Phase 11 Verus slice audit confirms that pointer casts preserve address, provenance, and slice length but do not transfer typed memory permission or byte values. `ptr_ref` requires an initialized `PointsTo<T>`; `PointsToRaw::into_typed` yields `Uninit`; the nested `MaybeUninit::mem_contents()` model has single-value operations but no slice-wide conversion to readable `[u8]` permissions. The missing relation remains conversion of initialized `MaybeUninit<u8>` payload permissions into a shared readable `u8` slice while preserving the selected range. The proved ASCII helper starts after this missing step, so it cannot be composed with `slice_buffer_to_str` yet.

At the time of the Phase 10/11 audit, the proposed removal condition was to
prove the per-element payload-to-byte view plus shared-slice formation from
existing permissions, prove the ASCII implication to `valid_utf8`, and then
establish the `from_utf8_unchecked` byte-view postcondition. Boundary B was
later accepted explicitly as a local trusted source change; it remains an
assumption, not a proof of that physical conversion. The Verus audit and its
post-conversion UTF-8 lemma do not discharge or justify this current Creusot
trust. That Verus lemma uses vstd's assumed `from_utf8_unchecked`
specification, which is another separate trust boundary.

## Current proof obligation: `unreachable_unchecked` length guard

The guard in `runtime.rs` is now part of the current `Buffer::format` proof
target. Its branch calls `unreachable_unchecked()` when
`string.len() > I::MAX_STR_LEN`; the integrated Creusot proof establishes for
every sealed implementation that `string.len() <= I::MAX_STR_LEN` from the
actual writer contract and shows the guard is unreachable on x86_64. No Creusot writer fact is
imported as a Verus premise and no cross-tool length theorem is claimed.

## Keep independent assumptions separate

- The proved arithmetic contract is exactly `u128_ext::mulhi`'s result equation `result == x * y / 2^128`; the joint-helper proof derives it from `mulhi_core`; the wrapper has no
`#[trusted]` annotation and its body proof discharged 2/2 goals. It supports the `u128` formatter and, through the magnitude formatter, the signed `i128` writer; it supplies no pointer provenance, reference validity, initialization, or string-conversion fact.
- Existing standard/core models are separate from local raw-memory contracts. The runtime proof consumes `MaybeUninit::write`, slice `get_unchecked`, and narrow models for signed `unsigned_abs`, array `IndexMut`, and mutable-slice-to-array borrowing as documented in `RUNTIME_VERIFICATION.md` and `RUNTIME_MEMORY_LEDGER.md`. Their use does not prove either raw boundary.
- The recursive verification model's former trusted ASCII-suffix-to-`str` leaf has been removed. Its replacement requires ASCII and retains the same byte postcondition; its body and caller pass in the integrated proof. It is distinct from the runtime `assume_init_slice` helper.
- The x86_64 restriction here covers the 64-bit `usize`/`isize` writer adapters. It does not silently extend evidence to the 16/32-bit pointer-width adapters or to other targets.

## Source and report basis

The historical translator errors and rejected expressions are reproduced above, so the original probe results remain understandable if temporary files are later removed. The `/tmp` paths below identify session reports and scratch artifacts used to assemble that historical evidence.

- `itoa/1.0.18/RUNTIME_MEMORY_LEDGER.md` and `RUNTIME_VERIFICATION.md`, in the Phase 8 tree at commit `8204b28`.
- `PROVENANCE.md`, which records current source changes, the one accepted local trust, the proved `mulhi` result contract, and the integrated runtime result.
- [`pow2-composition/REPORT.md`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-composition/REPORT.md): current joint-helper proof, driver selection, replay command, and integrated 224-library / 1,917-VC run evidence. The focused arithmetic result does not prove the raw runtime memory path.
- `/tmp/PHASE9_CAST_REPORT.md`: i8 pointer-cast probe; raw dereference translation rejection before VCs.
- `/tmp/PHASE10_SLICE_REPORT.md`: runtime slice-conversion probe; `get_unchecked` translated, raw dereference rejected before VCs.
- `/tmp/VERUS_RAW_API_AUDIT.md`: pinned vstd API inventory; not a proof.
- `/tmp/PHASE11_CAST_VERUS_REPORT.md`: borrowed stack-array typed-prefix model gap; no solver/proof and no source edit.
- `/tmp/PHASE11_SLICE_VERUS_REPORT.md` and `/tmp/phase11-verus-slice/ascii_bytes_to_str.rs`: bounded ASCII `&[u8]` to `valid_utf8` / `&str` proof, 1 verified / 0 errors; no `MaybeUninit` raw conversion. The replay command used Verus 0.2026.09.27.3cf1832 through the shared wrapper with `--rlimit 300`. The checked-in proof artifact is `itoa/1.0.18/verus/ascii_bytes_to_str.rs`. Its SHA-256 is `4fbc8f8e19a47692539210b04f0ea1b9a341901da574516d243aea29bfdfcd7a`, matching the Phase 11 scratch proof byte-for-byte. It was checked with Verus `0.2026.09.27.3cf1832` (commit `3cf18325f0fd0c3040fbdec8c0f2255c0504c91a`), using one prover and the wrapper 1024 MiB limit. The ASCII-to-UTF-8 argument is proved; the postcondition that the returned `str` has those bytes consumes the existing assumed specification for `str::from_utf8_unchecked` in vstd `string.rs:159-164`. This is a standard-library trust boundary, not a body proof of that library function. To replay the checked-in artifact from the crate directory, run:

```sh
/workspace/rust-crate-proofs/tools/creusot-toolpatch/scripts/run-proof.sh \
  env PATH=/tmp/verus-cargo-home/bin:/tmp/verus-current/verus-x86-linux:/usr/bin:/bin \
  RUSTUP_HOME=/tmp/verus-rustup-home CARGO_HOME=/tmp/verus-cargo-home \
  WHY3DATA=/tmp/phase11-verus-unused-why3-data \
  timeout 120s /tmp/verus-current/verus-x86-linux/verus \
  /workspace/rust-crate-proofs/itoa/1.0.18/verus/ascii_bytes_to_str.rs --crate-type=lib --rlimit 300
```

No Verus raw-memory trusted contract or proof was added. The local Creusot
`assume_init_slice` contract is the accepted Boundary B source change; it is
not proved by the Verus artifact. The Verus proof covers only the post-
conversion ASCII byte slice and does not establish that helper's operation.
