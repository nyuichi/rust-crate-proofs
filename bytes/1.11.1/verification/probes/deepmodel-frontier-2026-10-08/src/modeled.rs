//! Positive control: the same raw-pointer shape with an explicit byte model.
//!
//! `model` is only a logical snapshot slot here. This probe does not establish
//! that it corresponds to memory reachable through `ptr` and `len`.

use core::cmp::{Ordering, PartialEq, PartialOrd};
use creusot_std::prelude::*;

pub struct RawBytes {
    pub ptr: *const u8,
    pub len: usize,
    pub model: Ghost<Seq<u8>>,
}

impl PartialEq for RawBytes {
    fn eq(&self, _other: &RawBytes) -> bool {
        false
    }
}

impl PartialOrd for RawBytes {
    fn partial_cmp(&self, _other: &RawBytes) -> Option<Ordering> {
        None
    }
}

impl DeepModel for RawBytes {
    type DeepModelTy = Seq<u8>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { *self.model }
    }
}

impl<'a, T: ?Sized + DeepModel<DeepModelTy = Seq<u8>>> PartialEq<&'a T> for RawBytes
where
    RawBytes: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized + DeepModel<DeepModelTy = Seq<u8>>> PartialOrd<&'a T> for RawBytes
where
    RawBytes: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
        self.partial_cmp(&**other)
    }
}
