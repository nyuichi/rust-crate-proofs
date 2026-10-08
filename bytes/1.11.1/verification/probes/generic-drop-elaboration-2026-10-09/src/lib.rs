#![allow(unexpected_cfgs, unused_variables, dead_code)]
#[cfg(not(creusot))]
#[path = "../native.rs"]
pub mod native;
#[cfg(creusot)]
#[path = "../generated/active.rs"]
mod shadow;
#[cfg(creusot)]
mod loan_frame;
