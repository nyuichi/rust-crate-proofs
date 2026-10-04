#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
#[path="../../../../src/comparison_ops.rs"]
mod comparison_ops;

#[ensures(result == (left.deep_model() == right.deep_model()))]
pub fn compare_equal(left: &[u8], right: &[u8]) -> bool {
    comparison_ops::equal(left, right)
}

#[cfg(feature="wrong_equal")]
#[requires(left.deep_model() != right.deep_model())]
#[ensures(result)]
pub fn wrong_equal(left: &[u8], right: &[u8]) -> bool {
    comparison_ops::equal(left, right)
}
