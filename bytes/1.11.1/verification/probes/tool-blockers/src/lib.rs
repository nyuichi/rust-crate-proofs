#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;

#[cfg(feature = "trait_cycle")]
pub trait Buffer {
    fn chain<U: Buffer>(self, next: U) -> (Self, U)
    where Self: Sized {
        (self, next)
    }
}

#[cfg(feature = "indirect_call")]
pub fn indirect(f: fn(usize) -> usize, input: usize) -> usize { f(input) }

#[cfg(feature = "pointer_tag")]
pub fn pointer_tag(pointer: *mut u8) -> *mut u8 {
    ((pointer as usize) | 1) as *mut u8
}

#[cfg(feature = "automatic_drop")]
struct Guard<'a>(&'a mut bool);
#[cfg(feature = "automatic_drop")]
impl Drop for Guard<'_> {
    fn drop(&mut self) { *self.0 = true; }
}
#[cfg(feature = "automatic_drop")]
#[ensures(*flag == false ==> ^flag == true)]
pub fn automatic_drop(flag: &mut bool) {
    let _guard = Guard(flag);
}
