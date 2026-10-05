# Bounded public Shared `try_reclaim`

This probe extracts the actual `BytesMut::try_reclaim` method with
`bytes_proof_shared_reclaim`. The method receives a matching exclusive external
coordinator and requires initialized registered storage and
`offset + len + additional <= isize::MAX`.

The ordinary sufficient-capacity fast path is unchanged. The cfg slow path
returns false for a live sibling or when the singleton allocation cannot fit
the request under the native movement policy. Otherwise it calls the existing
public `reserve` cfg after checking the B6 offset and physical capacity.
That guard excludes the reallocation branch. `reserve` supplies the proved
singleton lease, reuse/movement, reactivation and byte-preservation transition.

Contracts specify the exact success condition, unchanged visible bytes and
length, sufficient capacity on success, unchanged handle on failure, unchanged
allocation identity and physical capacity, unchanged registration count, and
preserved unrelated registrations. The strengthened `reserve_lease` contract
exposes preservation of its original Recovery authority for fitting requests;
its body is checked without adding trust. Public `reserve` exposes the matching
conditional allocation-preservation result to the `try_reclaim` caller.

The native matrix covers 512 prefix/suffix, capacity, request, and sibling-count
cases. It checks both output bytes and the native success condition. The proof
caller explicitly retires every surviving owner. The positive gate proves 108
files with zero unproved leaves. Clean retranslation reproduces both generated
Rust and the source-fragment map byte-for-byte. Saved evidence records the
first composition failure, reviewed interface repair, and final proof separately
from native execution.

```sh
bash verify.bash
cargo test --locked
bash verify.bash --features negative_allocation_changed
```

The negative requests an impossible changed allocation identity after the
nonallocating operation and rejects exactly that assertion (40/41 caller goals).
Ordinary automatic Drop, concurrent native ownership,
and requests outside the stated arithmetic bound remain outside this gate.
