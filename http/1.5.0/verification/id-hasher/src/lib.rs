#![allow(dead_code, unexpected_cfgs)]

// The production implementation is included unchanged; only this harness
// keeps the independent IdHasher methods out of Extensions' dynamic Any map.
#[path = "../../../src/extensions/id_hasher.rs"]
mod id_hasher;
