#![allow(unexpected_cfgs)]

// Include the production implementation itself. This harness isolates its
// overflow and load-factor leaves from HeaderMap's unrelated raw iterators.
#[path = "../../../src/header/map_capacity.rs"]
mod map_capacity;

// These index and sentinel leaves are used by the production HeaderMap.
#[path = "../../../src/header/map_index.rs"]
mod map_index;
