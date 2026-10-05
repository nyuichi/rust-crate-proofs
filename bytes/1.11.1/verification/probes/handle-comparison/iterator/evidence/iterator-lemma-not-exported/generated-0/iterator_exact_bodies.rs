fn remaining(&self) -> usize {
        self.len()
    }
fn chunk(&self) -> &[u8] {
        self
    }
fn advance(&mut self, cnt: usize) {
        if self.len() < cnt {
            panic_advance(&TryGetError {
                requested: cnt,
                available: self.len(),
            });
        }

        crate::slice_ops::advance_slice(self, cnt);
    }
fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }
pub fn new(inner: T) -> IntoIter<T> {
        IntoIter { inner }
    }
pub fn into_inner(self) -> T {
        self.inner
    }
pub fn get_ref(&self) -> &T {
        &self.inner
    }
pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
fn next(&mut self) -> Option<u8> {
        if !self.inner.has_remaining() {
            return None;
        }

        let b = self.inner.chunk()[0];
        self.inner.advance(1);

        Some(b)
    }
fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.inner.remaining();
        (rem, Some(rem))
    }
