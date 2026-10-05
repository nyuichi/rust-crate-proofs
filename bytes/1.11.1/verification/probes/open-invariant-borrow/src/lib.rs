use creusot_std::prelude::*;

pub struct Even(usize);
impl Invariant for Even {
    #[logic(open(self))]
    fn invariant(self) -> bool { pearlite! { self.0@ % 2 == 0 } }
}

#[open_inv_result]
#[ensures(result.0 == 1usize)]
#[ensures(^result == ^value)]
fn leave_open(#[creusot::open_inv] value: &mut Even) -> &mut Even {
    value.0 = 1;
    value
}

#[ensures((^value).0 == 2usize)]
pub fn restore_after_private_transition(value: &mut Even) {
    let open = leave_open(value);
    open.0 = 2;
}

#[ensures(result.0 == 2usize)]
pub fn restore_already_open(mut value: Even) -> Even {
    value.0 = 3;
    let open = leave_open(&mut value);
    open.0 = 2;
    value
}

#[open_inv_result]
#[ensures(result.0 == 1usize)]
#[ensures(^result == ^value)]
fn relay_open(#[creusot::open_inv] value: &mut Even) -> &mut Even {
    leave_open(value)
}

#[ensures((^value).0 == 2usize)]
pub fn restore_after_relay(value: &mut Even) {
    let open = relay_open(value);
    open.0 = 2;
}
