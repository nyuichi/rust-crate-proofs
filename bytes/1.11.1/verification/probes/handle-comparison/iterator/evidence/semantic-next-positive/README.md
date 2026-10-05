# Concrete iterator semantic-next proof snapshot

This snapshot binds the current `build_iter.rs` to the generated iterator body,
the translated Why3 tasks, and their proof results. The pinned replay reported
**22 proof files passed**. `manifest.json` hashes every file in this snapshot
and the external bytes source fragments extracted by the build script.

The gate specializes the unchanged `IntoIter<T>` implementation to
`IntoIter<&[u8]>`. It proves the exact iterator body and specification laws for
suffix advancement and the returned head byte. It does not prove generic
`IntoIter<T>` behavior, arbitrary `T: Buf`, allocation ownership, or the
whole bytes crate.

`logs/pinned-replay.log` records the command output. `generated-0/` stores the
actual generated source, and `verif/` stores the exact `.coma` tasks and proof
JSON from that run. The external source hashes include the iterator source,
slice `Buf` bodies, core error declaration, and shared slice operations.
