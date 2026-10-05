# Cloud resume evidence and remaining integration frontier

This continuation starts from transport checkpoint `3e28a3b1`, which mixed proved
and exploratory work. The branch is `bytes-runtime-verification`; only bytes
1.11.1 is tested. The old archive was checked against its SHA256 and all 2142
members before use. Stock pinned Creusot 0.13, Why3 and why3find were rebuilt;
`CLOUD_RUNTIME_2026-10-05.json` records exact versions and binary hashes.

Current per-method scope is in `../STATUS.md` and `REMAINING_API_MATRIX.md`.
Probe receipts and immutable archives are authoritative for the selected
configuration. A snapshot may contain unselected source: its presence does not
prove those bodies. Fresh generated fragments were compared with proof-time
fragments after the public cfg increments. Counts denote proved files rather
than public APIs or percentage completion.

The native Vec-vs-BytesMut PartialOrd reversal was reproduced, fixed, and covered
by a semantic regression; a separate 103-file gate proves the actual three
Vec/BytesMut comparison implementations. Unique reserve/reclaim and growing
resize/append, sequential Shared reserve, same-control and independent-control
unsplit, frozen ticket read/recovery, concrete iterator, checked unique advance,
UninitSlice conversions and all six index families have scoped body evidence.
Raw UninitSlice construction requires existing exclusive B4 authority and a
matching sealed pointer; its wrong-pointer negative fails the caller requirement.
Unsafe projection now requires preservation of previously initialized slots;
the old deinitialization counterexample and the corrected rejection are retained.

Whole-crate completion remains false. The ordinary runtime translation was
actually attempted with `verify-all.bash`, not replaced by a helper command.
The four current frontend errors are unsafe Send/Sync marker trust requirements
and Bytes Deref / BytesMut DerefMut purity. The stock validator explicitly
requires trusted unsafe marker traits. No bytes marker or ownership/refcount
protocol is trusted to pass. Mutable B4 cannot be reclassified ghost: an archived
counterexample has native result 1 while erased ghost mutation proves result 2.
Readonly default AsRef separately lacks initialized authority. Actual Clone has
unsupported function-pointer dispatch; splitting its affine frozen fraction
through `&self` additionally fails mutable-borrow requirements. A finite private
enum experiment translates but supplies neither actual Clone nor its protocol.

Concrete public Buf integration attempts, source patches, diagnostics, and an
independent re-extraction are archived under `probes/actual-public-buf-default`.
The local unread sequence laws are not universal trusted laws on open public
traits. Chain requires saturated remaining; actual raw handles need initialized
view authority. Generic atomic publication and fixed two-ticket physical
retirement are separate proofs with primitive atomic/history TCB; they do not
prove arbitrary native Bytes refcounts or vtable concurrency. Automatic Drop,
panic/unwind and allocation failure remain outside normal-return consuming
cleanup proofs. These interfaces were tried and reviewed with Astra; simply
marking protocol contracts trusted would evade the user's requirement.

Use `/workspace/bytes-proof-tools/activate.sh` in this rebuilt environment, not
old absolute paths. Why3 must run elevated on its first attempt and through the
shared `/tmp/itoa-creusot-proof.lock`, one prover, 1024 MiB. Do not modify queued
or running wrappers. Commit each validated increment and push only
`origin bytes-runtime-verification`, without force. Do not run other crates.

Additional validated increments: `7d9c6310` contains the mixed and both-Unique
public unsplit gates (121 files each). `5c3c5217` contains both Shared growing
resize/append (115 files) and concrete mutable-slice BufMut bodies (16 files);
its short commit subject names only the latter. `eb8d80f3` adds the Shared
growth outer receipt and eleven extractor compatibility checks. The initially
incomplete Shared growth archive was detected by independent audit before
commit and replaced: the committed verified tar contains 115 actual proof JSON
files, zero null leaves, and 271 matching content hashes. No missing proof tree
is accepted as evidence.

Further increments include pure adapter metadata (28), Shared no-allocation
try_reclaim (108, native512), the Shared Unknown-publication negative with only
the Known-prefix requirement unproved, and exact BytesMut metadata selection (3).
Bytes metadata observers prove two selected bodies. Nonpromotable Bytes
truncate/clear and callers prove six files under an explicit tag precondition;
`c92ee61b` replaces private table-address classification with an equivalent
boolean, tested across all six native table sites. The promotable split/Drop
branch is excluded, not trusted. Exact Bytes new/from_static attempts are
preserved as failed frontend experiments (callback cast/static pointer scalar).
Their exploratory generic AtomicPtr constructor contract and opaque table
supply no positive ownership/refcount or constructor result.

Final native regression passes 1256 tests across 17 suites; no-default-features
check passes. Actual verify-all fails at exactly the four frontend errors above,
before the proof phase. `probes/native-integration-frontier/native-latest-manifest.json`
records current source and log hashes. No bytes-specific protocol was added to
the trusted boundary to claim completion. The remaining interface limitations
were concretely attempted and reviewed with Astra; whole-crate verification
remains incomplete.

Final Astra review confirms no further small standalone change closes Clone,
default initialized reads/mutable purity, generic implementer laws, native
concurrency/Drop, or the attempted constructor frontend limits using the current
proved interfaces. These are limits of this representation and pinned toolchain,
not mathematical impossibility claims. The fresh-layout observer replay proves
two files with zero unproved leaves; both archived source snapshots match the
final native regression code.
