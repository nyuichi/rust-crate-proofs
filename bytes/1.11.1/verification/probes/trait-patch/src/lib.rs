#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;

pub trait Buffer {
    fn chain<U: Buffer>(self, next: U) -> (Self, U)
    where Self: Sized { (self, next) }
}
pub fn generic_chain<B: Buffer, U: Buffer>(b: B, u: U) -> (B, U) {
    b.chain(u)
}

#[cfg(feature = "negative_logic")]
mod negative_logic;
#[cfg(feature = "negative_assoc")]
mod negative_assoc;

#[cfg(feature = "negative_ghost")]
pub trait GhostBuffer {
    #[check(ghost)]
    fn chain<U: GhostBuffer>(self, next: U) -> (Self, U)
    where Self: Sized { (self, next) }
}
#[cfg(feature = "negative_termination")]
pub trait TerminatingBuffer {
    #[check(terminates)]
    fn chain<U: TerminatingBuffer>(self, next: U) -> (Self, U)
    where Self: Sized { (self, next) }
}
#[cfg(feature = "negative_mutual")]
pub trait A {
    fn cross<U: B>(self, next: U) -> (Self, U) where Self: Sized { (self, next) }
}
#[cfg(feature = "negative_mutual")]
pub trait B {
    fn cross<U: A>(self, next: U) -> (Self, U) where Self: Sized { (self, next) }
}
#[cfg(feature = "negative_contract")]
pub trait SpecifiedBuffer {
    #[ensures(false)]
    fn chain<U: SpecifiedBuffer>(self, next: U) -> (Self, U)
    where Self: Sized { (self, next) }
}
#[cfg(feature = "false_returning")]
#[ensures(false)]
pub fn false_returning() {}

#[cfg(any(feature = "strong_ghost_impl", feature = "strong_termination_impl", feature = "false_impl"))]
pub trait Strengthened {
    fn repeat(self, x: impl Strengthened) where Self: Sized;
}
#[cfg(any(feature = "strong_ghost_impl", feature = "strong_termination_impl"))]
impl Strengthened for i32 {
    #[cfg_attr(feature = "strong_ghost_impl", check(ghost))]
    #[cfg_attr(feature = "strong_termination_impl", check(terminates))]
    fn repeat(self, x: impl Strengthened) { self.repeat(x); }
}

#[cfg(feature = "false_impl")]
impl Strengthened for i32 {
    #[ensures(false)]
    fn repeat(self, _x: impl Strengthened) {}
}
