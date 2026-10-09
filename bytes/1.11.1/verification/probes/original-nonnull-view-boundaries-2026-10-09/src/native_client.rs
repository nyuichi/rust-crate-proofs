//! Native source client mirrored by the scoped proof shadow in `public_shared.rs`.
//!
//! Keep this body as ordinary public-API use: construction remains
//! `Bytes::from`, clones remain `Clone::clone(&self)`, and explicit cleanup
//! consumes the original handles. The borrowed slice deliberately survives a
//! peer cleanup before it is copied. The independent correspondence checker
//! compares this closed client with the proof shadow and extracted production
//! constructor, trait, vtable, and callback sources.

use super::*;
use alloc::vec::Vec;

#[cfg_attr(creusot, requires(input@.len() < creusot_std::std::vec::capacity_model(input)))]
#[cfg_attr(creusot, ensures(result@ == input@))]
pub(crate) fn actual_public_shared_driver(input: Vec<u8>) -> Vec<u8> {
    let first = Bytes::from(input);
    let second = first.clone();
    let third = first.clone();
    third.cleanup();
    first.cleanup();
    let fourth = second.clone();
    let borrowed = AsRef::<[u8]>::as_ref(&fourth);
    second.cleanup();
    let observed = borrowed.to_vec();
    fourth.cleanup();
    observed
}
