#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
#[cfg(creusot)]
use creusot_std::std::vec::{pointer_model, capacity_model};

// Generic getter observation, not a bytes API or an access permission.
#[cfg_attr(creusot, ensures(result.1 == pointer_model(input)))]
#[cfg_attr(creusot, ensures(pointer_model(result.0) == pointer_model(input)))]
#[cfg_attr(creusot, ensures(capacity_model(result.0) == capacity_model(input)))]
#[ensures(result.0@ == input@)]
pub fn observe_pointer(mut input: Vec<u8>) -> (Vec<u8>, *mut u8) {
    let pointer = input.as_mut_ptr();
    (input, pointer)
}

#[cfg(all(creusot, feature="wrong_vec"))]
#[ensures(result.2 == pointer_model(input))]
pub fn wrong_vec(input: Vec<u8>, mut other: Vec<u8>) -> (Vec<u8>, Vec<u8>, *mut u8) {
    let pointer = other.as_mut_ptr();
    (input, other, pointer)
}
