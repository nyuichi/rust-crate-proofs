use creusot_std::prelude::*;
use creusot_std::std::iter::{IteratorSpec, ExactSizeIteratorSpec};
#[ensures(result@ == input@.len())]
fn remaining(input: &&[u8]) -> usize {
        input.len()
    }
#[ensures(result@ == input@)]
fn chunk<'a>(input: &'a &[u8]) -> &'a [u8] {
        input
    }
#[requires(cnt@ <= input@.len())]
#[ensures((^input)@ == input@[cnt@..])]
fn advance(input: &mut &[u8], cnt: usize) {
        if input.len() < cnt {
            panic_advance(&TryGetError {
                requested: cnt,
                available: input.len(),
            });
        }

        crate::slice_ops::advance_slice(input, cnt);
    }
#[ensures(result == (input@.len() > 0))]
fn has_remaining(input: &&[u8]) -> bool {
        remaining(input) > 0
    }
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
pub struct IntoIter<T> {
    inner: T,
}
impl<T> IntoIter<T> {
#[ensures(result.inner == inner)]
pub fn new(inner: T) -> IntoIter<T> {
        IntoIter { inner }
    }
#[ensures(result == self.inner)]
pub fn into_inner(self) -> T {
        self.inner
    }
#[ensures(*result == self.inner)]
pub fn get_ref(&self) -> &T {
        &self.inner
    }
#[ensures(*result == self.inner && ^result == (^self).inner)]
pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
impl<'a> Iterator for IntoIter<&'a [u8]> {
type Item = u8;
#[ensures(match result { Some(byte) => self.inner@.len() > 0 && byte == self.inner@[0] && (^self).inner@ == self.inner@[1..], None => self.inner@.len() == 0 && (^self).inner@ == self.inner@ })]
fn next(&mut self) -> Option<u8> {
        if !has_remaining(&self.inner) {
            return None;
        }

        let b = chunk(&self.inner)[0];
        advance(&mut self.inner, 1);

        Some(b)
    }
#[ensures(result.0@ == self.inner@.len() && result.1 == Some(result.0))]
fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = remaining(&self.inner);
        (rem, Some(rem))
    }

}
#[cfg(creusot)]
impl<'a> IteratorSpec for IntoIter<&'a [u8]> {
    #[logic(prophetic)]
    fn produces(self, visited: Seq<u8>, other: Self) -> bool {
        pearlite! { self.inner@ == visited.concat(other.inner@) }
    }
    #[logic(prophetic)]
    fn completed(&mut self) -> bool { pearlite! { self.inner@.len() == 0 } }
    #[logic(law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) { let _ = Seq::<u8>::concat_empty; }
    #[logic(law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<u8>, b: Self, bc: Seq<u8>, c: Self) {
        let _ = Seq::<u8>::concat_assoc;
    }
}
impl<'a> ExactSizeIterator for IntoIter<&'a [u8]> {}
#[cfg(creusot)]
impl<'a> ExactSizeIteratorSpec for IntoIter<&'a [u8]> {
    #[logic(law)]
    #[requires(Self::size_hint.postcondition((self,), hint))]
    #[ensures(hint.1 == Some(hint.0))]
    fn size_hint_exact(&self, hint: (usize, Option<usize>)) {}
}
