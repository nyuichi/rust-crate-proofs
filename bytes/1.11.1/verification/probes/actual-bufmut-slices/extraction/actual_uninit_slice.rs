use core::mem::MaybeUninit;
use creusot_std::prelude::*;

#[repr(transparent)]
pub struct UninitSlice([MaybeUninit<u8>]);

impl View for UninitSlice {
    type ViewTy = Seq<Option<u8>>;

    #[logic(open(self))]
    fn view(self) -> Self::ViewTy {
        pearlite! { Seq::create(self.0@.len(), |index: Int| self.0@[index]@) }
    }
}

impl UninitSlice {
    /// Creates a `&mut UninitSlice` wrapping a slice of initialised memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::buf::UninitSlice;
    ///
    /// let mut buffer = [0u8; 64];
    /// let slice = UninitSlice::new(&mut buffer[..]);
    /// ```
    #[inline]
    #[trusted]
    #[cfg_attr(creusot, check(ghost))]
    #[ensures(result@.len() == slice@.len())]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == Some(slice@[i]))]
    #[ensures((^result)@.len() == slice@.len())]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> (^result)@[i] == Some((^slice)@[i]))]
    #[ensures((^slice)@.len() == slice@.len())]
    pub fn new(slice: &mut [u8]) -> &mut UninitSlice {
        unsafe { &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>] as *mut UninitSlice) }
    }

    /// Creates a `&mut UninitSlice` wrapping a slice of uninitialised memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::buf::UninitSlice;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut buffer = [MaybeUninit::uninit(); 64];
    /// let slice = UninitSlice::uninit(&mut buffer[..]);
    ///
    /// let mut vec = Vec::with_capacity(1024);
    /// let spare: &mut UninitSlice = vec.spare_capacity_mut().into();
    /// ```
    #[inline]
    #[trusted]
    #[cfg_attr(creusot, check(ghost))]
    #[ensures(result@.len() == slice@.len())]
    #[ensures(result@ == Seq::create(slice@.len(), |index: Int| slice@[index]@))]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == slice@[i]@)]
    #[ensures((^result)@.len() == slice@.len())]
    #[ensures((^result)@ == Seq::create((^slice)@.len(), |index: Int| (^slice)@[index]@))]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> (^result)@[i] == (^slice)@[i]@)]
    #[ensures((^slice)@.len() == slice@.len())]
    pub fn uninit(slice: &mut [MaybeUninit<u8>]) -> &mut UninitSlice {
        unsafe { &mut *(slice as *mut [MaybeUninit<u8>] as *mut UninitSlice) }
    }

    /// Write a single byte at the specified offset.
    ///
    /// # Panics
    ///
    /// The function panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::buf::UninitSlice;
    ///
    /// let mut data = [b'f', b'o', b'o'];
    /// let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
    ///
    /// slice.write_byte(0, b'b');
    ///
    /// assert_eq!(b"boo", &data[..]);
    /// ```
    #[inline]
    #[requires(index@ < self@.len())]
    #[ensures((^self)@.len() == self@.len())]
    #[ensures((^self)@[index@] == Some(byte))]
    #[ensures(forall<i> 0 <= i && i < self@.len() && i != index@ ==> (^self)@[i] == self@[i])]
    pub fn write_byte(&mut self, index: usize, byte: u8) {
        assert!(index < self.len());

        self.0[index] = MaybeUninit::new(byte);
    }

    /// Returns the number of bytes in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BufMut;
    ///
    /// let mut data = [0, 1, 2];
    /// let mut slice = &mut data[..];
    /// let len = BufMut::chunk_mut(&mut slice).len();
    ///
    /// assert_eq!(len, 3);
    /// ```
    #[inline]
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
