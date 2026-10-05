#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite};

#[derive(Copy, Clone, Debug)]
pub(in crate::uri) enum Protocol {
    Http,
    Https,
}

impl Protocol {
    #[ensures(match *self {
        Protocol::Http => result@ == 4,
        Protocol::Https => result@ == 5,
    })]
    pub(in crate::uri) fn len(&self) -> usize {
        match *self {
            Protocol::Http => 4,
            Protocol::Https => 5,
        }
    }
}
