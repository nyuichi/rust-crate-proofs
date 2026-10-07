# Modified bytes 1.11.1: remaining responsibilities

This is an obligation inventory for the user-selected changed API, not approval
to omit legacy responsibilities without recording correspondence. Read the
frozen decisions before attempting a blocked unchanged representation.
Latest committed production proof: production-closed-io-263,
std/x86_64, normal return. Counts describe proof files, not API completion.

| Responsibility | Current connected evidence | Next obligation |
|---|---|---|
| Sharing/concurrency | Arbitrary finite scoped tree; two reusable HRTB callbacks; actual weak Release/Acquire retirement | Checked ranged HRTB views proved183; chained/limited scoped views proved255; explicit split correspondence remains; escaping Clone not claimed |
| Cleanup | Receipt-derived exactly-one retirement and consuming B3 | Failure/unwind/allocation-error behavior; no automatic Drop claim |
| Mutable storage | Ordinary Vec-backed ExclusiveBytes, mutation-to-shared caller | Typed DerefMut and framing/erasure controls proved136; closed append/clear/zeroed additions pass production263; capacity remains |
| Freeze/thaw | Vec moves into B1 and returns to B3 | Complete affine recovery/B2 thaw proved143; formal returned pointer/capacity identity not supplied by stock Vec model |
| Capacity | Reserve preserves contents; native Vec behavior | Formal numeric requested capacity and attached Vec identity; no extra bytes-specific trust |
| Extend/write | set/push/pop/truncate/resize | Slice extension and fixed/variable signed/unsigned/endian writes proved229; representative downstream public-model clients proved6 with native3; generic iterator extension remains |
| Cursor | Checked u8 and signed/unsigned 16/32/64/128 BE/LE, unsigned variable width, advance/copy | Integer families/aliases proved183; exact Float32Bits/64Bits transport proved229; native float materialization frozen D08; result-buffer/adapters correspondence remains |
| Read/write adapters | Concrete Cursor/Read and ExclusiveBytes/Write proved263; reusable immutable callbacks | Concrete chaining/limit/reader/writer/iteration responsibilities connected to actual entry; no universal downstream Buf laws |
| Conversions | Vec move-in/move-out ExclusiveBytes | Slice/static copies, String/Box ownership transfer and IteratorSpec conversion proved183; generic-owner replacement inventory and allocation correspondence remain |
| Trait observers | Exclusive immutable slice; current trait experiment | Equality proved183, PartialOrd/Ord and cleanup proved229; borrowed iteration and explicit OwnedByteCursor proved255; Hash/Formatter unchanged contracts frozen D12 after real failures; explicit digest/hex replacement in progress |
| no_std | Production alloc core221 and relocated std255 proved; native14/32 | Stock proof-only dependency still enables std; isolated genuine alloc model port and actual MSP UInt16 integration in progress |
| serde | Legacy serde gated away | Actual Serialize/Deserialize failures reviewed and frozen D09; generic interface semantics remain uncovered |
| extra-platforms | Actual portable RMW and matching Acquire fence; host263/native37, missing-Acquire control fails | Final API/cross-target configurations; generic primitive TCB remains explicit |
| Width/alignment/target | x86_64 current configuration | 183 source passed configured i68632/little and powerpc64/big proofs with native cross checks; final API graph and unsupported target inventory remain |
| Termination/failure | Normal-return partial-correctness | Callback catch/unwind attempt frozen D10; Copy-slot fallible spawn and try-reserve candidates in progress; allocation/termination obligations remain, actual attempts and Astra review before blocking |

The frozen original Clone(&self), raw mutable B4 ghost classification, MIR Drop
handling, opaque dispatch/static materialization, and trusted open trait laws
are not to be retried unchanged. Changed representations above are separate
experiments with preserved failures and explicit stop conditions.

Order: current typed mutation/thaw/numeric/conversion/ordering increments are
validated. Finish concrete adapters and downstream contract usability; then
core/std/serde/atomic/target configurations and failure-aware paths; finally
reconcile every retained/replaced/blocked responsibility and audit the exact
source/configuration evidence. Commit and push each validated increment.

Active distinguishing experiments are isolated: generic Vec numeric capacity and
try-reserve contracts; generic B1/B2 capacity metadata framing; Copy-only explicit
slot spawning with actual failure and mutation-negative checks; existing alloc
model exposure for real 16-bit no_std; matching portable RMW/acquire-fence backend;
explicit digest/hex bytes; copied checked splits with independently cleaned owners.
These are pending, not completed obligations. D12 excludes repeating unchanged
trait interfaces, not these explicit alternatives.
