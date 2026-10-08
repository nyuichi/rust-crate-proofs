//! Restricted erased unsafe invocation with a checked in-shim ghost update.
#![feature(fn_traits, unboxed_closures)]
#![allow(unexpected_cfgs)]
use creusot_std::{prelude::*, std::ops::FnExt};
use creusot_std::{ghost::resource::Resource, logic::ra::excl::{Excl, ExclUpdate}};

pub type Token = Resource<Excl<Int>>;
pub type TransformSpec = fn(u32, Ghost<Token>) -> (u32, Ghost<Token>);

/// The checked shim's runtime erasure is exactly the native pointer invocation;
/// checked, successfully terminating ghost steps account for its ghost output.
/// This is semantic erasure correspondence, never raw code-address equality.
#[trusted]
#[logic(opaque)]
pub fn registered_erasure<A, R, G, H>(
    _native: unsafe fn(A) -> R, _shim: fn(A, Ghost<G>) -> (R, Ghost<H>),
) -> bool { dead }

/// A closed generation skeleton: one native arithmetic body; exactly one shim
/// call to the same native item; every added operation inside checked ghost!.
macro_rules! declare_transform {
    ($native:ident, $shim:ident, $getter:ident, $step:literal) => {
        #[requires(x < u32::MAX)]
        #[ensures(result@ == x@ + $step)]
        unsafe fn $native(x: u32) -> u32 { x + $step }

        #[requires(x < u32::MAX)]
        #[requires(input.inner_logic()@ == Excl(x@))]
        #[ensures(result.0@ == x@ + $step)]
        #[ensures(result.1.inner_logic().id() == input.inner_logic().id())]
        #[ensures(result.1.inner_logic()@ == Excl(result.0@))]
        fn $shim(x: u32, input: Ghost<Token>) -> (u32, Ghost<Token>) {
            let native_result = unsafe { $native(x) };
            let transformed = ghost! {
                let mut token = input.into_inner();
                token.update(ExclUpdate(snapshot!(native_result@)));
                token
            };
            (native_result, transformed)
        }

        #[trusted]
        #[ensures(registered_erasure(result.0, result.1.inner_logic()))]
        #[ensures(forall<x: u32, g: Ghost<Token>>
            result.1.inner_logic().precondition((x, g)) == $shim.precondition((x, g)))]
        #[ensures(forall<x: u32, g: Ghost<Token>, r: (u32, Ghost<Token>)>
            result.1.inner_logic().postcondition((x, g), r) == $shim.postcondition((x, g), r))]
        pub fn $getter() -> (unsafe fn(u32) -> u32, Ghost<TransformSpec>) {
            ($native, ghost! { $shim as TransformSpec })
        }
    };
}

declare_transform!(native_identity, identity_shim, identity_registration, 0);
declare_transform!(native_successor, successor_shim, successor_registration, 1);

/// Generic TCB: replay the registered checked shim's ghost semantics around
/// precisely one native call. It consumes the actual affine input; no second
/// copy or unchecked ownership-update law is returned.
#[trusted]
#[requires(registered_erasure(native, spec.inner_logic()))]
#[requires(spec.inner_logic().precondition((x, input)))]
#[ensures(spec.inner_logic().postcondition((x, input), result))]
pub fn invoke_transform<A, R, G, H>(
    native: unsafe fn(A) -> R, x: A, input: Ghost<G>,
    spec: Ghost<fn(A, Ghost<G>) -> (R, Ghost<H>)>,
) -> (R, Ghost<H>) {
    #[cfg(not(creusot))]
    {
        // All Ghost values are erased. The scalar result is the actual unsafe
        // pointer call, not a replay/call of the logical shim.
        let native_result = unsafe { native(x) };
        (native_result, Ghost::conjure())
    }
    #[cfg(creusot)]
    { unreachable!("trusted erasure boundary") }
}

#[requires(x < u32::MAX)]
#[requires(input.inner_logic()@ == Excl(x@))]
#[ensures(result.0@ == x@ + 1)]
#[ensures(result.1.inner_logic().id() == input.inner_logic().id())]
#[ensures(result.1.inner_logic()@ == Excl(result.0@))]
pub fn update_inside_callback(x: u32, input: Ghost<Token>) -> (u32, Ghost<Token>) {
    let (native, spec) = successor_registration();
    invoke_transform(native, x, input, spec)
}

#[requires(x < u32::MAX)]
#[ensures(result@ == x@ + 1)]
pub fn allocate_and_transform(x: u32) -> u32 {
    let token = Resource::alloc(snapshot!(Excl(x@)));
    let (result, token) = update_inside_callback(x, token);
    proof_assert!(token.inner_logic()@ == Excl(result@));
    result
}

#[cfg(feature = "negative_wrong_certificate")]
#[requires(x < u32::MAX)]
#[requires(input.inner_logic()@ == Excl(x@))]
#[ensures(result.0@ == x@ + 1)]
#[ensures(result.1.inner_logic()@ == Excl(result.0@))]
pub fn wrong_certificate(x: u32, input: Ghost<Token>) -> (u32, Ghost<Token>) {
    let (native, _) = identity_registration();
    let (_, wrong_spec) = successor_registration();
    invoke_transform(native, x, input, wrong_spec)
}

#[cfg(feature = "negative_duplicate")]
#[requires(x < u32::MAX)]
#[requires(input.inner_logic()@ == Excl(x@))]
pub fn duplicate_input(x: u32, input: Ghost<Token>) -> (Ghost<Token>, Ghost<Token>) {
    let (_, transformed) = update_inside_callback(x, input);
    (input, transformed)
}

#[cfg(feature = "negative_ghost_write")]
#[requires(x < u32::MAX)]
pub fn ghost_write_native_result(x: u32, input: Ghost<Token>) -> (u32, Ghost<Token>) {
    let mut native_result = unsafe { native_identity(x) };
    let transformed = ghost! {
        native_result = 7;
        input.into_inner()
    };
    (native_result, transformed)
}

#[cfg(all(test, not(creusot)))]
mod tests {
    #[test]
    fn actual_native_result_and_erased_update() {
        for x in [0, 1, 17, u32::MAX - 1] {
            assert_eq!(super::allocate_and_transform(x), x + 1);
        }
    }
}
