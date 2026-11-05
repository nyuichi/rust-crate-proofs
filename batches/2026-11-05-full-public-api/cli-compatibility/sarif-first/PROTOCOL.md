# Official recorded-source publication protocol

Public upstream revision: `0a39304e702b196e6dff917bbf17c63dc01abcc6` at https://github.com/proofs-rs/proofs-rs . Its docs/backend-design.md lines 342–371 state that valid SARIF publication is independent of successful rustdoc import and older /publish/prepare publishers must update. The reference cli/src/publish.rs uses /runs/prepare for dry-run and immutable upload for real publication, then exact returned bindings.

This compatibility update follows that route without changing source implementations, proof mappings, mathematical conditions, captured SARIF or its source commit. It retains source rediscovery, exact callable uniqueness, harness/precondition, run ID, original-byte SHA/size, tool and author-visible metadata checks. Each indexed returned path must be a recorded source spelling, and IDs/indexes must be unique. The endian66 typed-key opt-in remains checked. Recorded-origin rows are author-observed source APIs, not independent rustdoc signatures or safety assertions.

The failed PCG normal import was accepted after the user's limit increase but stopped at unresolved_public_reexport. It is retained as a separate catalogue error; the supported SARIF-first route does not retry that job or bypass a quota. The actual publication succeeded with unchanged twenty-one claims.

Current binary SHA-256: `dbd1b83f25044387a90083527561b89c261c0e543075f17cdbf409d7b63e0bd5`. Tests: seventy passed, three ignored; real immutable endian66 and PCG21/source-only Xoshiro30 checks are archived here. Apply sarif-first.patch after the earlier complete official-0.4.1-to-current.patch; credentials remain external.
