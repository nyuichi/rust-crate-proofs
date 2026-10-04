#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;
use std::any::TypeId;

type Alias = u32;
struct Newtype(u32);

fn same_type<A: 'static, B: 'static>() -> bool {
    TypeId::of::<A>() == TypeId::of::<B>()
}

#[ensures(result)]
pub fn alias_has_same_id() -> bool {
    same_type::<u32, Alias>()
}

#[ensures(!result)]
pub fn newtype_has_distinct_id() -> bool {
    same_type::<u32, Newtype>()
}

fn main() {}
