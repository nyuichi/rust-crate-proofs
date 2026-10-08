//! Immutable normal-return native input for the generic effect elaborator.
use creusot_std::prelude::*;

pub struct SetTrueOnDrop<'a>(pub &'a mut bool);
pub struct ToggleOnDrop<'a>(pub &'a mut bool);

#[ensures(^flag == true)]
fn set_true(flag: &mut bool) { *flag = true; }

#[ensures(^(guard.0) == ^((^guard).0))]
#[ensures(*(^guard).0 == true)]
pub(crate) fn set_true_drop_effect<'a>(guard: &mut SetTrueOnDrop<'a>) {
    set_true(guard.0);
}

#[ensures(^(guard.0) == ^((^guard).0))]
#[ensures(*(^guard).0 == !*guard.0)]
pub(crate) fn toggle_drop_effect<'a>(guard: &mut ToggleOnDrop<'a>) {
    *guard.0 = !*guard.0;
}





#[requires(*flag == false)]
#[ensures(^flag == true)]
pub fn set_true_scope(flag: &mut bool) {
    let mut _guard = SetTrueOnDrop(flag);
}

#[ensures(^flag == !*flag)]
pub fn toggle_scope(flag: &mut bool) {
    let mut _guard = ToggleOnDrop(flag);
}

#[ensures(^flag == false)]
pub fn toggle_after_write(flag: &mut bool) {
    let mut guard = ToggleOnDrop(flag);
    *guard.0 = true;
}


