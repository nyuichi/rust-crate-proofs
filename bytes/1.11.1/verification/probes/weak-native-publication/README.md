# Native weak-memory publication probe

Fresh pinned replay proves all **3 files**. Native tests cover the primitive's
Release ordering and both retirement orders. Exact sources, Coma, proof JSON,
logs and hashes are in `evidence/positive.tar.gz` and `positive.json`.

The generic primitive performs native `AtomicUsize::fetch_sub(Release)` and
`load(Acquire)` under explicit trusted atomic contracts. `two_retirements`
exercises the latter with exclusively owned atomic history. The shared
`AtomicInvariant` protocol uses two affine tickets. The final retirement obtains
a stock relaxed-load `AcquireSyncView`, performs Release, and then uses the stock
Acquire fence before unsealing the other thread's `AtView<T>`. Its own resource
is synchronized only with its own current view. No bytes protocol is trusted.

The checkpoint returned publication metadata as executable `SyncView`, allowing
recovery without Acquire. The repair returns `Snapshot<SyncView>`: metadata may
be compared, but cannot become a current-thread witness. The historical archive
had compilation failures, not a successful proof.

Fresh controls reject:

- `negative_no_acquire`: sequential recovery and shared retirement cannot open
  the other resource without the Acquire load/fence (two unproved functions).
- `negative_drop_prior_publication`: the sequential release-sequence propagation
  proof has one unproved function. The immediate two-ticket predecessor can
  still synchronize through the final fence; this control is not evidence for
  an arbitrary-length registration protocol.
- `negative_publication_witness`: conversion of metadata to a witness fails
  compilation because `SyncView` does not implement `Plain`.

Run `bash run-proof.sh`, optionally with `--features FEATURE`. The wrapper uses
the common serialized proof lock. `weak-physical-retirement` separately
instantiates the protocol with actual buffer capabilities. Native Bytes vtable,
arbitrary registration, automatic Drop and Send/Sync remain separate goals.
