//! Generic helper-interface control; not an extra native Drop witness.
use creusot_std::prelude::*;

#[ensures(^flag == false)]
pub fn caller_can_mutate_after_effect(flag: &mut bool) {
    let mut guard = crate::shadow::SetTrueOnDrop(flag);
    crate::shadow::set_true_drop_effect(&mut guard);
    *guard.0 = false;
}
