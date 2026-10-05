
use creusot_std::prelude::*;

/// Deliberately narrow proof interface. This is not the public bytes::Buf trait.
pub trait BufPredicateSurface {
    #[logic]
    fn remaining_metadata(&self) -> Int;

    #[ensures(result@ == self.remaining_metadata())]
    fn remaining(&self) -> usize;

    #[ensures(result == (self.remaining_metadata() > 0))]
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }
}

/// Deliberately narrow proof interface. This is not the public bytes::BufMut trait.
pub trait BufMutPredicateSurface {
    #[logic]
    fn remaining_mut_metadata(&self) -> Int;

    #[ensures(result@ == self.remaining_mut_metadata())]
    fn remaining_mut(&self) -> usize;

    #[ensures(result == (self.remaining_mut_metadata() > 0))]
    fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }
}

impl<'a> BufPredicateSurface for &'a [u8] {
    #[logic(open)]
    fn remaining_metadata(&self) -> Int { pearlite! { (*self)@.len() } }

    #[ensures(result@ == self.remaining_metadata())]
    fn remaining(&self) -> usize {
        self.len()
    }
}

impl<'a> BufMutPredicateSurface for &'a mut [u8] {
    #[logic(open)]
    fn remaining_mut_metadata(&self) -> Int { pearlite! { (*self)@.len() } }

    #[ensures(result@ == self.remaining_mut_metadata())]
    fn remaining_mut(&self) -> usize {
        self.len()
    }
}

/// A generic caller uses the default method through the probe trait.
#[ensures(result == (buf.remaining_metadata() > 0))]
pub fn generic_buf_has_remaining<B: BufPredicateSurface + ?Sized>(buf: &B) -> bool {
    buf.has_remaining()
}

/// Concrete slice caller for the exact #[path]-extracted `&[u8]` implementation.
#[ensures(result == (input@.len() > 0))]
pub fn slice_has_remaining(input: &[u8]) -> bool {
    input.has_remaining()
}

/// A generic caller uses the default method through the probe trait.
#[ensures(result == (buf.remaining_mut_metadata() > 0))]
pub fn generic_bufmut_has_remaining<B: BufMutPredicateSurface + ?Sized>(buf: &B) -> bool {
    buf.has_remaining_mut()
}

/// Concrete mutable-slice caller for the exact #[path]-extracted implementation.
#[ensures(result == (input@.len() > 0))]
pub fn slice_mut_has_remaining(input: &mut [u8]) -> bool {
    input.has_remaining_mut()
}
