#![allow(unexpected_cfgs)]

#[cfg(creusot)]
use creusot_std::prelude::*;

pub mod actual {
    include!(concat!(env!("OUT_DIR"), "/actual_adapter_metadata.rs"));
}

#[cfg(creusot)]
#[ensures(result.0@ == updated_inner@)]
#[ensures(result.1@ == updated_inner@)]
#[ensures(result.2@ == new_limit@)]
pub fn take_metadata_round_trip(
    inner: usize,
    initial_limit: usize,
    updated_inner: usize,
    new_limit: usize,
) -> (usize, usize, usize) {
    let mut value = actual::take::new(inner, initial_limit);
    *value.get_mut() = updated_inner;
    let observed = *value.get_ref();
    value.set_limit(new_limit);
    let limit = value.limit();
    let owned = value.into_inner();
    (owned, observed, limit)
}

#[cfg(creusot)]
#[ensures(result.0 == first)]
#[ensures(result.1 == second)]
pub fn chain_round_trip<T, U>(first: T, second: U) -> (T, U) {
    actual::chain::Chain::new(first, second).into_inner()
}

#[cfg(creusot)]
#[ensures(result == buf)]
pub fn reader_metadata_round_trip<B>(buf: B) -> B {
    let mut value = actual::reader::new(buf);
    let _ = value.get_ref();
    value.into_inner()
}

#[cfg(creusot)]
#[ensures(result == buf)]
pub fn writer_metadata_round_trip<B>(buf: B) -> B {
    let mut value = actual::writer::new(buf);
    let _ = value.get_ref();
    value.into_inner()
}

#[cfg(test)]
mod tests {
    use super::actual::{chain::Chain, limit, reader, take, writer};

    #[test]
    fn take_fields_and_accessors_preserve_metadata() {
        let mut value = take::new(11usize, 4);
        assert_eq!(value.limit(), 4);
        *value.get_mut() = 12;
        assert_eq!(*value.get_ref(), 12);
        value.set_limit(7);
        assert_eq!(value.limit(), 7);
        assert_eq!(value.into_inner(), 12);
    }

    #[test]
    fn limit_fields_and_accessors_preserve_metadata() {
        let value = limit::metadata_round_trip(21usize, 5, 9);
        assert_eq!(value, (21, 9));
        let mut adapter = limit::new_for_native(31usize, 6);
        assert_eq!(adapter.limit(), 6);
        *adapter.get_mut() = 32;
        assert_eq!(*adapter.get_ref(), 32);
        adapter.set_limit(10);
        assert_eq!(adapter.limit(), 10);
        assert_eq!(adapter.into_inner(), 32);
    }

    #[test]
    fn chain_constructor_and_destructor_preserve_both_fields() {
        let pair = Chain::new(41u8, 0xfeed_u16).into_inner();
        assert_eq!(pair, (41, 0xfeed));
    }

    #[test]
    fn reader_accessors_preserve_the_stored_value() {
        let mut value = reader::new(51usize);
        assert_eq!(*value.get_ref(), 51);
        *value.get_mut() = 52;
        assert_eq!(*value.get_ref(), 52);
        assert_eq!(value.into_inner(), 52);
    }

    #[test]
    fn writer_accessors_preserve_the_stored_value() {
        let mut value = writer::new(61usize);
        assert_eq!(*value.get_ref(), 61);
        *value.get_mut() = 62;
        assert_eq!(*value.get_ref(), 62);
        assert_eq!(value.into_inner(), 62);
    }
}
