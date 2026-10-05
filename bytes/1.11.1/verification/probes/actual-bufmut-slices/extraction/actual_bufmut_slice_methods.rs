pub struct TryGetError {
    /// The number of bytes necessary to get the value
    pub requested: usize,

    /// The number of bytes available in the buffer
    pub available: usize,
}

#[requires(false)]
#[cold]
fn panic_advance(error_info: &TryGetError) -> ! {
    panic!(
        "advance out of bounds: the len is {} but advancing by {}",
        error_info.available, error_info.requested
    );
}

pub(crate) mod slice_mut_ops {
    use creusot_std::prelude::*;
    /// Advances the mutable-slice cursor and returns the unchanged consumed prefix.
#[requires(count@ <= input@.len())]
#[ensures(result@ == input@[0..count@])]
#[ensures((^input)@ == input@[count@..])]
#[ensures((^*input)@ == (^result)@.concat((^^input)@))]
#[inline]
pub(crate) fn advance_slice_mut<'a, T>(input: &mut &'a mut [T], count: usize) -> &'a mut [T] {
    let (prefix, suffix) = core::mem::take(input).split_at_mut(count);
    *input = suffix;
    prefix
}
}

impl ClosedU8SliceBufMut for &mut [u8] {
    #[logic(open)]
    fn byte_view(&self) -> Seq<u8> { pearlite! { (**self)@ } }

    #[ensures(result@ == self.byte_view().len())]
    fn remaining_mut(&self) -> usize {
        self.len()
    }

    #[ensures(result@.len() == self.byte_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.byte_view().len() ==> result@[i] == Some(self.byte_view()[i]))]
    #[ensures((^result)@.len() == self.byte_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.byte_view().len() ==> (^result)@[i] == Some((^self).byte_view()[i]))]
    #[ensures((^self).byte_view().len() == self.byte_view().len())]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        UninitSlice::new(self)
    }

    #[requires(cnt@ <= self.byte_view().len())]
    #[ensures((^self).byte_view() == self.byte_view()[cnt@..])]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        if self.len() < cnt {
            panic_advance(&TryGetError {
                requested: cnt,
                available: self.len(),
            });
        }

        let _ = crate::slice_mut_ops::advance_slice_mut(self, cnt);
    }
}

impl ClosedMaybeUninitSliceBufMut for &mut [MaybeUninit<u8>] {
    #[logic(open)]
    fn option_view(&self) -> Seq<Option<u8>> {
        pearlite! { Seq::create((**self)@.len(), |index: Int| (**self)@[index]@) }
    }

    #[ensures(result@ == self.option_view().len())]
    fn remaining_mut(&self) -> usize {
        self.len()
    }

    #[ensures(result@.len() == self.option_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.option_view().len() ==> result@[i] == self.option_view()[i])]
    #[ensures((^result)@.len() == self.option_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.option_view().len() ==> (^result)@[i] == (^self).option_view()[i])]
    #[ensures((^self).option_view().len() == self.option_view().len())]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        UninitSlice::uninit(self)
    }

    #[requires(cnt@ <= self.option_view().len())]
    #[ensures((^self).option_view() == self.option_view()[cnt@..])]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        if self.len() < cnt {
            panic_advance(&TryGetError {
                requested: cnt,
                available: self.len(),
            });
        }

        let _ = crate::slice_mut_ops::advance_slice_mut(self, cnt);
    }
}
