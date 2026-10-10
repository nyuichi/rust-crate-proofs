#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite};

use super::protocol::Protocol;

#[derive(Clone, Debug)]
pub(in crate::uri) enum Scheme2<T> {
    None,
    Standard(Protocol),
    Other(T),
}

impl<T> Scheme2<T> {
    #[ensures(result == (match self {
        Scheme2::None => true,
        Scheme2::Standard(_) | Scheme2::Other(_) => false,
    }))]
    pub(in crate::uri) fn is_none(&self) -> bool {
        matches!(*self, Scheme2::None)
    }
}

impl<T> From<Protocol> for Scheme2<T> {
    #[ensures(match (src, result) {
        (Protocol::Http, Scheme2::Standard(Protocol::Http)) => true,
        (Protocol::Https, Scheme2::Standard(Protocol::Https)) => true,
        _ => false,
    })]
    fn from(src: Protocol) -> Self {
        Scheme2::Standard(src)
    }
}
