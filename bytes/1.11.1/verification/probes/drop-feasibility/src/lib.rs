#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;

struct SetTrueOnDrop<'a>(&'a mut bool);

/// The same state change expected from the automatic destructor.
#[ensures(^flag == true)]
fn set_true(flag: &mut bool) {
    *flag = true;
}

impl Drop for SetTrueOnDrop<'_> {
    fn drop(&mut self) {
        set_true(self.0);
    }
}

/// Requires the observable effect of the real `Drop` implementation.
#[requires(*flag == false)]
#[ensures(^flag == true)]
pub fn automatic_drop(flag: &mut bool) {
    let _guard = SetTrueOnDrop(flag);
}

/// Calls the exact helper used by `Drop::drop` explicitly.
#[requires(*flag == false)]
#[ensures(^flag == true)]
pub fn explicit_helper(flag: &mut bool) {
    set_true(flag);
}

/// Negative control: this contract contradicts the effect of `set_true`.
#[cfg(feature = "wrong_explicit_helper")]
#[requires(*flag == false)]
#[ensures(^flag == false)]
pub fn wrong_explicit_helper(flag: &mut bool) {
    set_true(flag);
}
