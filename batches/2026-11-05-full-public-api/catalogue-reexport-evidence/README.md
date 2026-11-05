# Original public rand_core crate reexports

Existing compiler rustdoc JSON proves these are public, non-glob external-crate-root module uses. Their referenced IDs have `paths` entries of kind `module`, path `["rand_core"]`, with external crate_id2 resolving through external_crates. Those IDs are absent from the local index, which is expected for this external module and does not mean the original public use is unresolved. Source bytes are the original archived `pub use rand_core;` line. The packets retain raw use items, spans, IDs, external artifact metadata, rustdoc hashes and original archive hashes for default/no-default/all-features.

This metadata distinguishes the external module boundary; it does not expand all dependency APIs into local declarations or prove any external blanket specialization. Proof counts and guarantees are unchanged. No source edit, compiler/solver execution, CLI modification, API request or deployment was performed for these packets.
