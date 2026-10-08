//! Generic unsafe indirect-call boundary, with body-checked affine forwarding.
#![feature(fn_traits, unboxed_closures)]
#![allow(unexpected_cfgs)]
use creusot_std::{prelude::*, std::ops::FnExt};
use creusot_std::{ghost::resource::Resource, logic::ra::excl::Excl};

/// Semantic realization, not numeric address identity. Only registration below
/// establishes it. It means the native target realizes the checked shim's
/// normal-return contract for every argument satisfying that shim's precondition.
#[trusted]
#[logic(opaque)]
pub fn registered<A, R>(_native: unsafe fn(A) -> R, _spec: fn(A) -> R) -> bool { dead }

/// The native target, checked shim and certificate are generated together.
/// The body occurs once, and the shim has exactly one call to that same item.
/// This macro is private: no arbitrary pointer/spec-pair factory is exposed.
macro_rules! declare_registered {
    ($(#[$contract:meta])* $native:ident, $shim:ident, $getter:ident,
     ($x:ident: $arg:ty) -> $ret:ty $body:block) => {
        $(#[$contract])*
        unsafe fn $native($x: $arg) -> $ret $body

        $(#[$contract])*
        fn $shim($x: $arg) -> $ret { unsafe { $native($x) } }

        #[trusted]
        #[ensures(registered(result.0, result.1.inner_logic()))]
        #[ensures(forall<a: $arg>
            result.1.inner_logic().precondition((a,)) == $shim.precondition((a,)))]
        #[ensures(forall<a: $arg, r: $ret>
            result.1.inner_logic().postcondition((a,), r) == $shim.postcondition((a,), r))]
        pub fn $getter() -> (unsafe fn($arg) -> $ret, Ghost<fn($arg) -> $ret>) {
            ($native, Ghost::new($shim as fn($arg) -> $ret))
        }
    };
}

declare_registered!(
    #[requires(x < u32::MAX)]
    #[ensures(result == x)]
    native_identity, identity_shim, identity_registration, (x: u32) -> u32 { x }
);
declare_registered!(
    #[requires(x < u32::MAX)]
    #[ensures(result@ == x@ + 1)]
    native_successor, successor_shim, successor_registration, (x: u32) -> u32 { x + 1 }
);

/// Generic TCB: execute precisely the supplied native target, under a matching
/// registered contract. The spec pointer is ghost-only and never invoked.
#[trusted]
#[requires(registered(native, spec.inner_logic()))]
#[requires(spec.inner_logic().precondition((x,)))]
#[ensures(spec.inner_logic().postcondition((x,), result))]
pub fn invoke<A, R>(native: unsafe fn(A) -> R, x: A, spec: Ghost<fn(A) -> R>) -> R {
    unsafe { native(x) }
}

/// Resource transport is an ordinary checked move, not part of the trusted rule.
#[requires(registered(native, spec.inner_logic()))]
#[requires(spec.inner_logic().precondition((x,)))]
#[ensures(spec.inner_logic().postcondition((x,), result.0))]
#[ensures(result.1.inner_logic() == resource.inner_logic())]
pub fn invoke_forward<A, R, G>(
    native: unsafe fn(A) -> R, x: A, spec: Ghost<fn(A) -> R>, resource: Ghost<G>,
) -> (R, Ghost<G>) {
    let result = invoke(native, x, spec);
    (result, resource)
}

pub type Affine = Resource<Option<Excl<()>>>;

#[requires(x < u32::MAX)]
#[ensures(result.0 == x)]
#[ensures(result.1.inner_logic() == resource.inner_logic())]
pub fn affine_identity(x: u32, resource: Ghost<Affine>) -> (u32, Ghost<Affine>) {
    let (native, spec) = identity_registration();
    invoke_forward(native, x, spec, resource)
}

#[cfg(feature = "negative_wrong_certificate")]
#[requires(x < u32::MAX)]
#[ensures(result@ == x@ + 1)]
pub fn wrong_certificate(x: u32) -> u32 {
    let (native, _) = identity_registration();
    let (_, wrong_spec) = successor_registration();
    invoke(native, x, wrong_spec)
}

#[cfg(feature = "negative_duplicate_resource")]
pub fn duplicate_resource(resource: Ghost<Affine>) -> (Ghost<Affine>, Ghost<Affine>) {
    (resource, resource)
}

#[cfg(all(test, not(creusot)))]
mod tests {
    #[test]
    fn native_pointer_and_ghost_forwarding() {
        for x in [0, 1, 17, u32::MAX - 1] {
            let (f, spec) = super::identity_registration();
            assert_eq!(super::invoke_forward(f, x, spec, super::Ghost::new(())).0, x);
            let (f, spec) = super::successor_registration();
            assert_eq!(super::invoke(f, x, spec), x + 1);
        }
    }
}
