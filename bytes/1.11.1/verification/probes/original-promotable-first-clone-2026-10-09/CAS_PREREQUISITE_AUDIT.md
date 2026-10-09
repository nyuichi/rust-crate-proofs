# CAS prerequisite audit

Date: 2026-10-09. Read-only audit; no prover was run.

## Archived CAS diagnostic

Audited `evidence/cas-positive-diagnostic-v1.tar.gz`, SHA-256
`30586cdeab09d8d12158a9ac899c227ed13acaf1399a60cd43ece266827fef98`. Its
1,888 unique member paths and all member SHA-256 values match the receipt. The
capture has 708 `probe/` members, 1,063 captured repository files, 110
private-Std files, six tool/configuration files, and `run.log`.

All 79 archived `.coma` files, receipt rows, and the included target list are
identical; no targets are excluded. I independently traversed all 79 proof
trees: 482 prover leaves, zero null leaves, zero structural leaves. The
`owned_pointer` targets `load_known`, `exchange_known`, and `first_strong_cas`
are included and have no null leaves. `exchange_known`'s `vc_elim_Err` and
`vc_elim_Ok` goals are both proved in the captured tree.

This is a diagnostic capture, not an accepted correspondence gate:
`target_policy.diagnostic` is true, `correspondence_exit_status` is 2, and the
archived correspondence result is `not_run` for the deliberate development
run. The 79/482/0 proof result does not establish native source correspondence
or full-gate admission.

## `owned_pointer.rs` TCB review

The archived `probe/src/owned_pointer.rs` SHA-256 is
`f9e6820f9846f1fe96666d1ec37c49180c8a8cc0cc80d772c2db9a79beb76766`. It has
two `#[trusted]` wrappers: `load_acquire` and strong `compare_exchange`. I
compared their callback preconditions and result contracts with the captured
Creusot Std 0.13.0 `src/std/sync/atomic.rs` `AtomicPtr` macro contracts.
`load_acquire` mirrors the Acquire load event/result contract;
`compare_exchange` mirrors strong-CAS success and failure contracts, including
the failure-side `deep_model()` inequality. The actual calls use Core
`AtomicPtr` with Acquire and AcqRel/Acquire orderings. The archive's proof
script rejects the `sc-drf` feature, and the captured configuration pins one
active prover worker and a 1024 MiB limit.

Those wrappers still form a generic TCB: the core field's identity with
`pointer_model(field)` and the callback invocation mapping are explicitly
assumed. The body-proved `load_known`, `exchange_known`, and
`first_strong_cas` are not trusted. In `exchange_known`, a strong-CAS failure
would require the event's loaded `deep_model()` to differ from `expected`;
the method's precondition says every existing owner-history entry is
`expected`. The captured `vc_elim_Err` goal proves that contradiction, so the
success result is derived from the failure contract rather than assumed by a
trusted postcondition. `exchange_known` takes a mutable owner permission and
exports the actual appended history value at the next timestamp; the
all-history-equals condition is a pre-CAS premise, not a read-only history
binding after the write.

The v1 audit applied to that archived file only. At that point the live file
had changed to SHA-256
`1280a37ab2f06f8c0c3c04e30305f2044d67f66e4ad052da04e4c39b3a574809`; the v2
capture below contains the subsequent terminal-read extension.

## Inputs and prover result

All 110 archived private-Std files match both the installed `creusot-std`
0.13.0 tree and its 110-entry SHA256SUMS file. The archive also carries the
complete captured repository inputs, Cargo manifests/locks, six tool/config
files, proof targets, proof JSONs, generated inputs, and log. Its log records
`Atomic event: one prover; 1024 MiB; native weak orderings; sc-drf disabled`
and ends with `Proved (79 files)`. The proof script checks the locked Cargo
feature tree for `sc-drf` and runs Why3find with one worker. The Why3 config has
`running_provers_max = 1`; four backends are configured and may be tried
sequentially. Tool versions and binary digests are recorded in the captured
installation manifest, while the binary payloads themselves are not in the
archive.

## Supplemental native MIR capture

For v1, the current `native-mir/capture.json` was supplemental and was **not
included** in that CAS tarball. Its 19 selected MIR hashes all match the current
files: one client body plus 18 production bodies, all at
`2-2-004.ElaborateDrops.after.mir`, with rustc 1.98.0-nightly commit
`91fe22da8084a1c9e993d78d4a56f22ab8396236` and MIR optimization level zero.
The production set covers Box construction, `Clone`, `cleanup`, `Drop`,
`AsRef`/`as_slice`, both promotable clone/drop branches, both shallow clone
branches, refcount increment, shared drop/release/free, pointer mapping, and
`AtomicMut::with_mut`. The manifest's production `Cargo.toml` and `src/bytes.rs`
hashes match their current files.

The separately captured native test manifest, lock, source, and run log match
the hashes in `capture.json`. The log reports one passing test; that test checks
input lengths 1, 2, 31, and 256 against the expected bytes. Its `native.rs`
client calls `Bytes::from(Box<[u8]>)`, clones once, calls `child.cleanup()`,
reads through `AsRef<[u8]>::as_ref(...).to_vec()`, then calls
`original.cleanup()`. The client MIR shows two explicit cleanup calls in that
order and no destructor call for either handle after it has been moved into
`cleanup`. This native test/MIR evidence was separate from v1's 79-target proof
run and does not prove the generic TCB or full source/shadow correspondence.

## v2 archive and terminal latest-value projection

The follow-on capture `evidence/cas-positive-diagnostic-v2.tar.gz` has SHA-256
`38dcaba847d610db040509177f0bea39764bd9a9def9d316c6ca86d45035f5b0`. Its
3,081 unique members all match the receipt's path and digest list: 763
`probe/` members, 2,201 repository-input members, 110 private-Std files, six
tool/configuration files, and `run.log`. The archived `.coma` files equal the
included translation target set: 83 targets, no exclusions, and 83 files / 507
prover leaves / zero null leaves. This remains diagnostic, with correspondence
status 2 and `not_run`; these proof results are not an independent source
correspondence pass.

The v2 `probe/src/owned_pointer.rs` SHA-256 is
`487f20a66b65140486b11dbe55424adf168f2eff74b2a4d827e9bc67c6bb9119`, matching
the current source. The archive includes the 110-file Std 0.13.0 input tree;
its members match the installed tree and the SHA256SUMS manifest.

In that captured Std source, `AtomicPtr::into_inner(self, own)` requires
`self == *own.ward()` and returns the value and a `Ghost<SyncView>`. Its
contract relates the value to the history at
`self.get_timestamp(result_view)`, bounds every history timestamp by that
timestamp, and bounds each history view by `result_view`. The v2
`get_mut_finish` adapter keeps the value and maximal-timestamp consequences:
its result is `(pointer, Snapshot<Int>)`, the archived history at that timestamp
must contain the returned pointer, and every history timestamp is at most that
timestamp. It additionally ensures the borrowed core field is unchanged.
`Snapshot<Int>` carries no `SyncView`; the adapter asserts no view-ordering
consequence and performs no Acquire, fence, or `SyncView` update.

This is a custom generic TCB projection of Std's terminal-value/maximal-time
contract, not a call to Std `into_inner` or a body proof that equates the two
APIs. The native body uses `field.get_mut()` under `&mut CoreAtomicPtr`; its
Creusot body is unreachable, with the contract carrying the assumed
native-to-model latest-history interpretation. It consumes
`Ghost<Perm<ModelAtomicPtr<()>>>` by value, and returns no permission or
`State` value. The archived Std `Ghost<T>` is Copy only for `T: Copy`, while
`Perm` has no Copy/Clone implementation, so this operation does not duplicate
the permission. This TCB concerns exclusive access to a generic core atomic
pointer; it states no `Bytes::Shared::drop`, reclamation, or refcount law.

The v2 body-proved helpers are `exchange_singleton`, `load_visible_known`,
`finish_latest`, and the closed `first_cas_read_finish` witness. `latest_is` and
`visible_is` are pure history predicates. Starting from a singleton history,
`exchange_singleton` proves the appended CAS value is maximal and visible at
the post-event view. `load_visible_known` performs a real Acquire through the
captured load adapter and preserves the visible-value condition. `finish_latest`
uses the terminal adapter under a `latest_is` premise. `first_cas_read_finish`
combines those operations into a closed postcondition. Their emitted VC tasks
are included in the all-proved v2 result. `get_mut_finish` itself is marked
`#[trusted]` and correctly remains a TCB boundary, not a proof target.

The v2 archive now includes the native MIR receipt, all 19 selected MIR files
(one client and 18 production bodies), and `native-test/native-run.log`; all
selected MIR hashes match `probe/native-mir/capture.json`. The archived and
current capture receipt hash is
`ec9cca4a2523fa9eb73c9ed57187551fa911c3c9e93ffa9b74180dcaadb541e2`; the
native test log hash is
`269a8c3c1cf251a721113cebeccd0761a1eb3e7a6e654b1c2a9d099c8384e7f5`. The corrected
production paths `../../../Cargo.toml` and `../../../src/bytes.rs` are relative
to the probe root and resolve to the captured repository inputs, whose hashes
match the receipt. The native source, test manifest/lock/source, and run log
also match the capture receipt. The native test has one passing test over
lengths 1, 2, 31, and 256. The production client MIR still shows two explicit
cleanup calls (child, then original), not an implicit `Drop` after either
handle has moved into `cleanup`. This corroborates the selected native client
execution; it does not prove the generic terminal-read TCB or the full
source/shadow gate.

## Scope

This supports a generic strong-CAS prerequisite and a small native Box-backed
clone/read/cleanup witness. It does not establish that the count-2 premise of
`initialize_pair` arose from this CAS, does not complete the integrated public
client lifecycle, and does not admit the crate's full `From` refinement,
unwind path, other representations, or arbitrary concurrent behavior.

## Archived generic CAS semantic controls

I independently checked the three immutable `evidence/cas-control-*.tar.gz`
captures and their receipts. The archive digests match the receipt and the
requested SHA-256 values; every member path is unique and every member digest
matches its receipt. The archive/receipt results are:

| Control | Archive SHA-256 | Members | Files / prover leaves / null leaves |
|---|---|---:|---:|
| Wrong strong-CAS expected pointer (`wrong_expected`) | `a5ac22d79ac061a03f9a52e857a64795a9067ffc0df94e4d7790bec97645b98a` | 236 | 8 / 67 / 1 |
| Omitted success-path `shoot_store` (`missing_store`) | `7f471e9a6468fd6b41aecc4fe65e1c4a017981049aaf99903248c04d1fe4165f` | 282 | 8 / 77 / 1 |
| Rebind stale initial value after mutation (`stale_readonly`) | `367ac1d07d4246cc90962bcfc5a4cecf26a1b8f8c2ce054d3b11f9c4f3a1eed1` | 310 | 8 / 69 / 1 |

Each archive's eight `.coma` paths exactly equal its recorded mini-crate target
list, with an empty exclusion set. The matching null `proof.json` leaf is in
the control target: `vc_wrong_expected`, `vc_missing_store`, and
`vc_stale_readonly`, respectively. The adjacent printed tasks express the
same failed obligations recorded in the README: the singleton CAS history
cannot be the wrong expected pointer when the initial value differs; a
successful callback that omits `shoot_store` cannot satisfy the final-store
obligation; and after the update, the complete history cannot still contain
only the initial pointer. The remaining leaves are proved in the captured
trees. These are deliberately generic `AtomicPtr`/owned-permission controls,
not `Bytes` lifecycle negative controls.

The printed-task sidecars were inspected, but they sit beside the archives and
are not archive members or receipt rows. Their audited SHA-256 values are
`f0afbd0275ec24be86d645c83889106f9105a62e803a71c68fb459e6b8482b73`
(`wrong_expected`),
`ee667c1c4a3e15aba2665a1140544b07b5bac2c86d8774b2c57f6d17a70f3986`
(`missing_store`), and
`458a752255922ed9ceefad9ad0df71138d40f76ba25ca4bacf2d186ce9f82a2c`
(`stale_readonly`). The archived `.coma` inputs and failed goal records are
captured; the sidecar text hashes are recorded here so their audit identity is
explicit.

The per-control captures contain 110 private-Std files, the same six tool and
configuration inputs, and exactly one imported old `pointer_event.rs` file;
their hashes agree across controls and with the v2 prerequisite capture. The
generic source `probe/src/owned_pointer.rs` is identical to the audited v2
source. Each run log ends with the named unproved control goal. The earlier
`cas-control-missing-mini-config.log` is a separate infrastructure failure
(`why3find.json` was absent; sidecar SHA-256
`690eeb90a3f9c3fcb013dd734bd8fcbe4a9a3e21e2d1b809d6299c9ce1c666fd`); it
contains no prover result and is not included in any semantic-control leaf
count.

Two reproduction qualifications apply. First, `capture_prerequisite.py` walks
the whole mini-crate tree and skips `.why3findcache`, but the actual solver
result cache is named `.why3find`; these archives therefore include 92, 138,
and 166 `.why3find` cache members, respectively. Their bytes are hashed in the
receipts, but a replay consuming those cached results is not a fresh uncached
solver run. The captured proof trees and printed null tasks still identify the
recorded failed obligations. Second, the capture script references absolute
`/workspace/bytes-proof-tools` and `/workspace/proof-tools` paths, the run
script references `/workspace/bytes-proof-tools`, and the mini-crate's
`#[path]` to the old `pointer_event.rs` expects a sibling path that is stored under
`inputs/repository/` in the archive. The source inputs are present, but the
archive is not by itself a ready-to-build directory tree; a replay must restore
the required workspace/tool layout or explicitly place the captured sibling
source at the path expected by `lib.rs`. No solver was run for this audit.
