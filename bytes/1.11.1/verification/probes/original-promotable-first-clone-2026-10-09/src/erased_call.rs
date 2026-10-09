//! Generic three-argument counterpart of the admitted checked erasure rule.
use creusot_std::{prelude::*, std::ops::FnExt};

#[trusted]
#[logic(opaque)]
pub fn registered3<A,B,C,R,G>(_native:unsafe fn(A,B,C)->R,
    _shim:fn(A,B,C,Ghost<G>)->R)->bool { dead }

/// Native execution calls the original unsafe function pointer once. The
/// checked shim has the same runtime erasure; all extra state is Ghost-only.
#[trusted]
#[requires(registered3(native,spec.inner_logic()))]
#[requires(spec.inner_logic().precondition((args.0,args.1,args.2,input)))]
#[ensures(spec.inner_logic().postcondition((args.0,args.1,args.2,input),result))]
pub fn invoke3<A,B,C,R,G>(native:unsafe fn(A,B,C)->R,args:(A,B,C),input:Ghost<G>,
    spec:Ghost<fn(A,B,C,Ghost<G>)->R>)->R {
    #[cfg(not(creusot))]
    {unsafe {native(args.0,args.1,args.2)}}
    #[cfg(creusot)]
    {unreachable!("generic registered erased call")}
}
