# Modified bytes 1.11.1: remaining responsibilities

This is an obligation inventory for the user-selected changed API, not approval
to omit legacy responsibilities without recording correspondence. Read the
frozen decisions before attempting a blocked unchanged representation.
Latest committed production proof: adapters-public-spec-255,
std/x86_64, normal return. Counts describe proof files, not API completion.

| Responsibility | Current connected evidence | Next obligation |
|---|---|---|
| Sharing/concurrency | Arbitrary finite scoped tree; two reusable HRTB callbacks; actual weak Release/Acquire retirement | Checked ranged HRTB views proved183; chained/limited scoped views proved255; explicit split correspondence remains; escaping Clone not claimed |
| Cleanup | Receipt-derived exactly-one retirement and consuming B3 | Failure/unwind/allocation-error behavior; no automatic Drop claim |
| Mutable storage | Ordinary Vec-backed ExclusiveBytes, mutation-to-shared caller | Typed DerefMut and framing/erasure controls proved136; remaining append/capacity/adapter interfaces |
| Freeze/thaw | Vec moves into B1 and returns to B3 | Complete affine recovery/B2 thaw proved143; formal returned pointer/capacity identity not supplied by stock Vec model |
| Capacity | Reserve preserves contents; native Vec behavior | Formal numeric requested capacity and attached Vec identity; no extra bytes-specific trust |
| Extend/write | set/push/pop/truncate/resize | Slice extension and fixed/variable signed/unsigned/endian writes proved229; downstream public model usability and generic iterator extension remain |
| Cursor | Checked u8 and signed/unsigned 16/32/64/128 BE/LE, unsigned variable width, advance/copy | Integer families/aliases proved183; exact Float32Bits/64Bits transport proved229; native float materialization frozen D08; result-buffer/adapters correspondence remains |
| Read/write adapters | Concrete Cursor; reusable immutable callbacks | Concrete chaining/limit/reader/writer/iteration responsibilities connected to actual entry; no universal downstream Buf laws |
| Conversions | Vec move-in/move-out ExclusiveBytes | Slice/static copies, String/Box ownership transfer and IteratorSpec conversion proved183; generic-owner replacement inventory and allocation correspondence remain |
| Trait observers | Exclusive immutable slice; current trait experiment | Equality proved183, PartialOrd/Ord and cleanup proved229; borrowed iteration and explicit OwnedByteCursor proved255; hash/format actual attempts and replacement correspondence remain |
| no_std | verified currently forces std | Alloc-only candidate native check and 152-file proof passed; production adoption with current APIs and std relocation proof remain |
| serde | Legacy serde gated away | Actual Serialize/Deserialize failures reviewed and frozen D09; generic interface semantics remain uncovered |
| extra-platforms | Legacy portable atomics gated away | Native modified primitive mapping, generic contract adequacy and configured proof |
| Width/alignment/target | x86_64 current configuration | 183 source passed configured i68632/little and powerpc64/big proofs with native cross checks; final API graph and unsupported target inventory remain |
| Termination/failure | Normal-return partial-correctness | Callback catch/unwind attempt frozen D10; thread-creation and allocation failures plus termination remain; actual attempt and Astra review before blocking |

The frozen original Clone(&self), raw mutable B4 ghost classification, MIR Drop
handling, opaque dispatch/static materialization, and trusted open trait laws
are not to be retried unchanged. Changed representations above are separate
experiments with preserved failures and explicit stop conditions.

Order: current typed mutation/thaw/numeric/conversion/ordering increments are
validated. Finish concrete adapters and downstream contract usability; then
core/std/serde/atomic/target configurations and failure-aware paths; finally
reconcile every retained/replaced/blocked responsibility and audit the exact
source/configuration evidence. Commit and push each validated increment.
