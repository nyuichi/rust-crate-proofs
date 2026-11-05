# New bounded codec supplement: five original APIs

Ready for review as a Kani supplementary guarantee; no Aeneas mapping/count change.
Canonical APIs are base16ct::{lower,upper,mixed}::decode and
base16ct::{lower,upper}::encode. results.json/api-bindings.json preserve each
actual call and its separate successful evidence.

Decode: entire source slice length2, entire destination slice length3, all byte
patterns in source/initial destination. Exact ASCII alphabet iff success;
returned slice length1/correct value; otherwise InvalidEncoding; suffix dst1,2
unchanged on success/error. Error-case dst0 content is explicitly not promised.
Encode: source1byte/dst4bytes, exact canonical2 ASCII output, suffix dst2,3
unchanged; returned output roundtrips through actual decode into separate dst1.

Five successful Kani harnesses,1484checks/0fail, explicit unwind8 with checks
retained. Full original release copy/archive/source/config/lock/compiler/logs
preserved, no source/generated-IR modification, no models/stubs/assumptions.
No general arbitrary-length/capacity or UTF8/alloc/timing/release/other-feature
claim. encoded_len overflow remains outside its proved precondition. This packet
was not published by its worker and does not change existing base3 claims.
