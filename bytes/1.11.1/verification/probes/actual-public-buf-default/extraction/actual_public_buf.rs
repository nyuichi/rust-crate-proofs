
use creusot_std::prelude::*;

#[derive(core::fmt::Debug)]
pub struct TryGetError {
    /// The number of bytes necessary to get the value
    pub requested: usize,

    /// The number of bytes available in the buffer
    pub available: usize,
}

#[requires(false)]
fn panic_advance(error_info: &TryGetError) -> ! {
    panic!(
        "advance out of bounds: the len is {} but advancing by {}",
        error_info.available, error_info.requested
    );
}

/// A source-sliced copy of the actual public `Buf` trait surface needed by the
/// default `try_get_u8`. The runtime method signatures and bodies are extracted
/// exactly from `src/buf/buf_impl.rs`; `unread` and the contracts are proof-only
/// additions for evaluating the minimum public trait law model.
pub trait Buf {
    #[logic]
    fn unread(&self) -> Seq<u8>;

    #[ensures(result@ == self.unread().len())]
    fn remaining(&self) -> usize;

    #[ensures(result@.len() <= self.unread().len())]
    #[ensures(result@ == self.unread().subsequence(0, result@.len()))]
    #[ensures(self.unread().len() > 0 ==> result@.len() > 0)]
    fn chunk(&self) -> &[u8];

    #[requires(cnt@ <= self.unread().len())]
    #[ensures((^self).unread() == self.unread().subsequence(cnt@, self.unread().len()))]
    fn advance(&mut self, cnt: usize);

    #[ensures(match result {
        Ok(value) => self.unread().len() > 0
            && value == self.unread()[0]
            && (^self).unread() == self.unread().subsequence(1, self.unread().len()),
        Err(error) => self.unread().len() == 0
            && error.requested == 1usize
            && error.available@ == 0
            && (^self).unread() == self.unread(),
    })]
    fn try_get_u8(&mut self) -> Result<u8, TryGetError> {
        if self.remaining() < 1 {
            return Err(TryGetError {
                requested: 1,
                available: self.remaining(),
            });
        }
        let ret = self.chunk()[0];
        self.advance(1);
        Ok(ret)
    }
}

impl<'a> Buf for &'a [u8] {
    #[logic(open)]
    fn unread(&self) -> Seq<u8> { pearlite! { (*self)@ } }

    #[ensures(result@ == self.unread().len())]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[ensures(result@.len() <= self.unread().len())]
    #[ensures(result@ == self.unread().subsequence(0, result@.len()))]
    #[ensures(self.unread().len() > 0 ==> result@.len() > 0)]
    fn chunk(&self) -> &[u8] {
        self
    }

    #[requires(cnt@ <= self.unread().len())]
    #[ensures((^self).unread() == self.unread().subsequence(cnt@, self.unread().len()))]
    fn advance(&mut self, cnt: usize) {
        if self.len() < cnt {
            panic_advance(&TryGetError {
                requested: cnt,
                available: self.len(),
            });
        }

        crate::slice_ops::advance_slice(self, cnt);
    }
}

/// Generic dispatch through the extracted public-trait default.
#[ensures(match result {
    Ok(value) => input.unread().len() > 0
        && value == input.unread()[0]
        && (^input).unread() == input.unread().subsequence(1, input.unread().len()),
    Err(error) => input.unread().len() == 0
        && error.requested == 1usize && error.available@ == 0
        && (^input).unread() == input.unread(),
})]
pub fn generic_try_get_u8<B: Buf + ?Sized>(input: &mut B) -> Result<u8, TryGetError> {
    input.try_get_u8()
}

/// Concrete call through the selected actual `&[u8]` implementation.
#[ensures(match result {
    Ok(value) => input.unread().len() > 0
        && value == input.unread()[0]
        && (^input).unread() == input.unread().subsequence(1, input.unread().len()),
    Err(error) => input.unread().len() == 0
        && error.requested == 1usize && error.available@ == 0
        && (^input).unread() == input.unread(),
})]
pub fn slice_try_get_u8(input: &mut &[u8]) -> Result<u8, TryGetError> {
    generic_try_get_u8(input)
}
