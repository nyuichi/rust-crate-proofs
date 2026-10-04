#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;
use std::any::{Any, TypeId};

pub fn erase_shared<T: Any>(value: &T) -> &dyn Any {
    value
}

pub fn erase_mut<T: Any>(value: &mut T) -> &mut dyn Any {
    value
}

pub fn project_shared<T: Any>(value: &dyn Any) -> Option<&T> {
    value.downcast_ref::<T>()
}

pub fn project_mut<T: Any>(value: &mut dyn Any) -> Option<&mut T> {
    value.downcast_mut::<T>()
}

pub fn same_type<A: Any, B: Any>() -> bool {
    TypeId::of::<A>() == TypeId::of::<B>()
}

fn main() {}
