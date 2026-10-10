# Verification scope

Target: the actual bytes 1.11.1 API and representation, with small source changes
where needed for verification. Work on default `std`, x86_64 and original atomic
orderings; optional features and other platforms are outside current validation.

The [current result](README.md) covers selected sequential normal-return paths.
Complete verification still requires byte correctness, memory/resource safety,
concurrency, abnormal paths and source correspondence across the original API.
Independent models, tests and assumed bytes ownership laws are not completion.
