//! STD-STR-BYTES: the immutable UTF-8 representation of a Rust string.
//!
//! This is a narrow standard-library trusted contract, not a Bytes ownership
//! or comparison axiom. `str`'s existing Creusot view is Seq<char>; `to_bytes`
//! is the standard model's concatenation of each scalar's UTF-8 encoding.
//! Core's native as_bytes returns exactly that immutable backing slice, with
//! its lifetime tied to the string borrow. It allocates and mutates nothing.
use creusot_std::prelude::*;

extern_spec! {
    impl str {
        #[check(ghost)]
        #[ensures(result@ == self@.to_bytes())]
        fn as_bytes(&self) -> &[u8];
    }
}
