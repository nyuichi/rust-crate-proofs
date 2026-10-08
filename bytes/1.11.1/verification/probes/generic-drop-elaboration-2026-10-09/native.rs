//! Immutable normal-return native input for the generic effect elaborator.
use creusot_std::prelude::*;

pub struct SetTrueOnDrop<'a>(pub &'a mut bool);
pub struct ToggleOnDrop<'a>(pub &'a mut bool);

#[ensures(^flag == true)]
fn set_true(flag: &mut bool) { *flag = true; }

impl Drop for SetTrueOnDrop<'_> {
    fn drop(&mut self) { set_true(self.0); }
}
impl Drop for ToggleOnDrop<'_> {
    fn drop(&mut self) { *self.0 = !*self.0; }
}

#[requires(*flag == false)]
#[ensures(^flag == true)]
pub fn set_true_scope(flag: &mut bool) {
    let _guard = SetTrueOnDrop(flag);
}

#[ensures(^flag == !*flag)]
pub fn toggle_scope(flag: &mut bool) {
    let _guard = ToggleOnDrop(flag);
}

#[ensures(^flag == false)]
pub fn toggle_after_write(flag: &mut bool) {
    let guard = ToggleOnDrop(flag);
    *guard.0 = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_scope_exit_effects_are_exactly_once() {
        let mut flag = false;
        set_true_scope(&mut flag);
        assert!(flag);
        toggle_scope(&mut flag);
        assert!(!flag);
        toggle_scope(&mut flag);
        assert!(flag);
        toggle_after_write(&mut flag);
        assert!(!flag);
    }
}
