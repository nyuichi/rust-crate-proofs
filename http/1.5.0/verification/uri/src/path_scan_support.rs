use super::{ErrorKind, MAX_LEN};

// This is the production scanner submodule. Only the PathAndQuery wrapper and
// its unrelated trait implementations are omitted from this focused harness.
#[path = "../../../src/uri/path_scan.rs"]
mod production;
