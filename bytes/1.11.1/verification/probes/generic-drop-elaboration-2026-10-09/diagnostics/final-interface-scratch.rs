#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
struct SetTrueOnDrop<'a>(&'a mut bool);
#[ensures(^flag == true)]
fn set_true(flag:&mut bool) { *flag=true; }
#[ensures(*(^guard).0 == true)]
#[ensures(^(guard.0) == ^((^guard).0))]
fn drop_shadow(guard:&mut SetTrueOnDrop<'_>) { set_true(guard.0); }
#[requires(*flag == false)]
#[ensures(^flag == true)]
pub fn automatic_drop(flag:&mut bool) {
    let mut guard=SetTrueOnDrop(flag);
    drop_shadow(&mut guard);
}
