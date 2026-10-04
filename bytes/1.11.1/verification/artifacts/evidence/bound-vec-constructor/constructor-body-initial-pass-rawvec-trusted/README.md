# Initial constructor gate before conversion body proof

This checkpoint passed the exact-source `BytesMut::from_vec` and explicit
release gate with 34 generated proof artifacts. Its `proof-artifacts/`
directory contains 34 Coma files and 34 matching proof JSON files, without a
second copied proof tree.

The source snapshot and native two-test output are retained. This run used the
then-trusted `RawAllocation::into_bound_ptr_at_zero` conversion. The later
`constructor-body-body-proved-conversion` archive supersedes it for current
source and proves that conversion method's body as well.
