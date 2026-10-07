# Modified bytes 1.11.1: remaining responsibilities

This is an obligation inventory for the user-selected changed API, not approval
to omit legacy responsibilities without recording correspondence. Read the
frozen decisions before attempting a blocked unchanged representation.
Latest committed production proof: callbacks-133, std/x86_64, normal return.

| Responsibility | Current connected evidence | Next obligation |
|---|---|---|
| Sharing/concurrency | Arbitrary finite scoped tree; two reusable HRTB callbacks; actual weak Release/Acquire retirement | Ranged views/splits and a usable scoped interface for those views; escaping Clone not claimed |
| Cleanup | Receipt-derived exactly-one retirement and consuming B3 | Failure/unwind/allocation-error behavior; no automatic Drop claim |
| Mutable storage | Ordinary Vec-backed ExclusiveBytes, mutation-to-shared caller | Typed DerefMut framing and ghost-erasure rejection (active bounded experiment) |
| Freeze/thaw | Vec moves into B1 and returns to B3 | Retain exact RawAllocation, recover complete physical capabilities, consume existing B2 to return Vec (active design) |
| Capacity | Reserve preserves contents; native Vec behavior | Formal numeric requested capacity and attached Vec identity; no extra bytes-specific trust |
| Extend/write | set/push/pop/truncate/resize | Slice/iterator extension, concrete endian/variable writes, initialized mutation contracts |
| Cursor | Checked u8 and signed/unsigned 16/32/64/128 BE/LE, unsigned variable width, advance/copy | i8, native-endian aliases, signed variable width, floats, result-buffer copies; document checked replacement of panicking getters |
| Read/write adapters | Concrete Cursor; reusable immutable callbacks | Concrete chaining/limit/reader/writer/iteration responsibilities connected to actual entry; no universal downstream Buf laws |
| Conversions | Vec move-in/move-out ExclusiveBytes | Box/String/iterator/static/from-slice and generic-owner replacement inventory, allocation correspondence |
| Trait observers | Exclusive immutable slice; current trait experiment | Equality/order/hash/format/iteration correspondence as retained; no disconnected helper aggregation |
| no_std | verified currently forces std | Separate core exclusive/cursor surface from std scoped threads, actual compile and integrated proof |
| serde | Legacy serde gated away | Actual modified serialization/deserialization implementation or concrete recorded tool blocker |
| extra-platforms | Legacy portable atomics gated away | Native modified primitive mapping, generic contract adequacy and configured proof |
| Width/alignment/target | x86_64 current configuration | Available target inventory and actual distinguishing build/proof; state unsupported target limitations |
| Termination/failure | Normal-return partial-correctness | Thread/callback termination, panic/unwind, thread-creation and allocation failures; actual attempt plus Astra review before blocking |

The frozen original Clone(&self), raw mutable B4 ghost classification, MIR Drop
handling, opaque dispatch/static materialization, and trusted open trait laws
are not to be retried unchanged. Changed representations above are separate
experiments with preserved failures and explicit stop conditions.

Order: finish typed mutation and B2 thaw; connect remaining concrete write/read
and conversion methods; then core/std/serde/atomic/target configurations; finally
reconcile every retained/replaced/blocked responsibility and audit the exact
source/configuration evidence. Commit and push each validated increment.
