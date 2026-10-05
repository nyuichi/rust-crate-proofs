#![allow(unexpected_cfgs)]

use core::mem::MaybeUninit;
use creusot_std::prelude::*;

pub mod buf {
    mod uninit_slice {
        include!(concat!(env!("OUT_DIR"), "/actual_uninit_slice.rs"));
    }

    pub use uninit_slice::UninitSlice;
}

use buf::UninitSlice;

/// Probe-only proof interface with one implementation: `&mut [u8]`.
pub trait ClosedU8SliceBufMut {
    #[logic]
    fn byte_view(&self) -> Seq<u8>;

    #[ensures(result@ == self.byte_view().len())]
    fn remaining_mut(&self) -> usize;

    #[ensures(result@.len() == self.byte_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.byte_view().len() ==> result@[i] == Some(self.byte_view()[i]))]
    #[ensures((^result)@.len() == self.byte_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.byte_view().len() ==> (^result)@[i] == Some((^self).byte_view()[i]))]
    #[ensures((^self).byte_view().len() == self.byte_view().len())]
    fn chunk_mut(&mut self) -> &mut UninitSlice;

    #[requires(cnt@ <= self.byte_view().len())]
    #[ensures((^self).byte_view() == self.byte_view()[cnt@..])]
    unsafe fn advance_mut(&mut self, cnt: usize);
}

/// Probe-only proof interface with one implementation:
/// `&mut [MaybeUninit<u8>]`.
pub trait ClosedMaybeUninitSliceBufMut {
    #[logic]
    fn option_view(&self) -> Seq<Option<u8>>;

    #[ensures(result@ == self.option_view().len())]
    fn remaining_mut(&self) -> usize;

    #[ensures(result@.len() == self.option_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.option_view().len() ==> result@[i] == self.option_view()[i])]
    #[ensures((^result)@.len() == self.option_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.option_view().len() ==> (^result)@[i] == (^self).option_view()[i])]
    #[ensures((^self).option_view().len() == self.option_view().len())]
    fn chunk_mut(&mut self) -> &mut UninitSlice;

    #[requires(cnt@ <= self.option_view().len())]
    #[ensures((^self).option_view() == self.option_view()[cnt@..])]
    unsafe fn advance_mut(&mut self, cnt: usize);
}

include!(concat!(env!("OUT_DIR"), "/actual_bufmut_slice_methods.rs"));
