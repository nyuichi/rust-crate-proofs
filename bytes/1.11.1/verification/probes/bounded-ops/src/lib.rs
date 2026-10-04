#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
#[path = "../../../../src/bounded_ops.rs"]
mod bounded_ops;

#[ensures(result@.len() <= limit@)]
#[ensures(result@.len() <= bytes@.len())]
#[ensures(forall<i: Int> 0 <= i && i < result@.len() ==> result@[i] == bytes@[i])]
pub fn take_chunk(bytes: &[u8], limit: usize) -> &[u8] {
    bounded_ops::bounded_chunk(bytes, limit)
}

#[cfg(feature = "wrong_prefix")]
#[requires(bytes@.len() > 0)]
#[ensures(result@.len() == 0)]
pub fn wrong_prefix(bytes: &[u8]) -> &[u8] {
    bounded_ops::bounded_chunk(bytes, 1)
}

#[requires(count@ <= limit@)]
#[ensures(result@ == limit@ - count@)]
pub fn consume_budget(mut limit: usize, count: usize) -> usize {
    bounded_ops::decrease_limit(&mut limit, count);
    limit
}
