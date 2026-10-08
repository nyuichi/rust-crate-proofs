//! Changed premise: call a safe erased pointer via the shipped Fn trait contract,
//! rather than retrying the frozen direct MIR FnPtr call. Trait invocation uses
//! shipped Std contracts; exact-item reification is explicit generic TCB below.
#![feature(fn_traits, unboxed_closures)]
#![allow(unexpected_cfgs)]
use creusot_std::{prelude::*, std::ops::FnExt};

#[requires(f.precondition((x,)))]
#[ensures(f.postcondition((x,), result))]
pub fn through_trait(f: fn(u32) -> u32, x: u32) -> u32 {
    core::ops::Fn::call(&f, (x,))
}

#[requires(f.precondition((x,)))]
#[ensures(f.postcondition((x,), result))]
pub fn typed_callback<F: Fn(u32) -> u32>(f: F, x: u32) -> u32 {
    f(x)
}

// This feature asks whether an actual unsafe vtable pointer can use the same
// stock Fn entry. Rust's callable trait requirements are checked before proof.
#[cfg(feature = "unsafe_pointer")]
pub fn unsafe_through_trait(f: unsafe fn(u32) -> u32, x: u32) -> u32 {
    core::ops::Fn::call(&f, (x,))
}

#[requires(x < u32::MAX)]
#[ensures(result@ == x@ + 1)]
pub fn add_one(x: u32) -> u32 { x + 1 }

#[cfg(feature = "known_item")]
#[requires(x < u32::MAX)]
#[ensures(result@ == x@ + 1)]
pub fn known_item_through_trait(x: u32) -> u32 {
    through_trait(add_one, x)
}

#[cfg(feature = "negative_move_affine")]
mod affine_receiver {
    use creusot_std::{prelude::*, ghost::resource::Resource, logic::ra::excl::Excl};
    // Same affine resource kind as the one-clone quota; storing it on a receiver
    // does not permit moving it through &self. No custom bytes law is assumed.
    pub struct Receiver { quota: Ghost<Resource<Option<Excl<()>>>> }
    impl Receiver {
        pub fn take_through_shared(&self) -> Ghost<Resource<Option<Excl<()>>>> {
            self.quota
        }
    }
}

// Generic reification TCB proposal. The same target identifier supplies both
// the actual function item and its checked contract; there is no arbitrary
// pointer/spec pair and no function-address equality assumption.
macro_rules! register_unary {
    ($getter:ident, $target:ident, $arg:ty, $ret:ty) => {
        #[trusted]
        #[ensures(forall<x: $arg> result.precondition((x,)) == $target.precondition((x,)))]
        #[ensures(forall<x: $arg, r: $ret> result.postcondition((x,), r) == $target.postcondition((x,), r))]
        pub fn $getter() -> fn($arg) -> $ret { $target }
    };
}
register_unary!(registered_add_one, add_one, u32, u32);

#[requires(x < u32::MAX)]
#[ensures(result@ == x@ + 1)]
pub fn registered_item_call(x: u32) -> u32 {
    through_trait(registered_add_one(), x)
}

#[cfg(all(test, not(creusot)))]
mod tests {
    #[test]
    fn registered_target_matches_original_body() {
        for x in [0, 1, 17, u32::MAX - 1] {
            assert_eq!(super::registered_item_call(x), super::add_one(x));
        }
    }
}

#[cfg(feature = "negative_wrong_target")]
#[requires(x < u32::MAX - 1u32)]
#[ensures(result@ == x@ + 2)]
pub fn assumes_wrong_target(x: u32) -> u32 {
    // Registration bound add_one, so the caller cannot claim add_two's result.
    through_trait(registered_add_one(), x)
}
