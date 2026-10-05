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
    /// Copies bytes from `src` into `self`.
    ///
    /// The length of `src` must be the same as `self`.
    ///
    /// # Panics
    ///
    /// The function panics if `src` has a different length than `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::buf::UninitSlice;
    ///
    /// let mut data = [b'f', b'o', b'o'];
    /// let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
    ///
    /// slice.copy_from_slice(b"bar");
    ///
    /// assert_eq!(b"bar", &data[..]);
    /// ```
    #[inline]
    #[requires(src@.len() == self@.len())]
    #[ensures((^self)@.len() == self@.len())]
    #[ensures(forall<i> 0 <= i && i < src@.len() ==> (^self)@[i] == Some(src@[i]))]
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        assert_eq!(self.len(), src.len());

        let mut index = 0;
        #[invariant(index@ <= src@.len())]
        #[invariant(self@.len() == src@.len())]
        #[invariant(forall<i> 0 <= i && i < index@ ==> self@[i] == Some(src@[i]))]
        #[variant(src@.len() - index@)]
        while index < src.len() {
            self.0[index] = MaybeUninit::new(src[index]);
            index += 1;
        }
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
    #[trusted]
    #[cfg_attr(creusot, check(ghost))]
    #[ensures(result@.len() == slice@.len())]
    #[ensures(result@ == Seq::create(slice@.len(), |index: Int| slice@[index]@))]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == slice@[i]@)]
    fn uninit_ref(slice: &[MaybeUninit<u8>]) -> &UninitSlice {
        unsafe { &*(slice as *const [MaybeUninit<u8>] as *const UninitSlice) }
    }
    /// Return a raw pointer to the slice's buffer.
    ///
    /// # Safety
    ///
    /// The caller **must not** read from the referenced memory and **must not**
    /// write **uninitialized** bytes to the slice either.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BufMut;
    ///
    /// let mut data = [0, 1, 2];
    /// let mut slice = &mut data[..];
    /// let ptr = BufMut::chunk_mut(&mut slice).as_mut_ptr();
    /// ```
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr() as *mut _
    }
    /// Return a `&mut [MaybeUninit<u8>]` to this slice's buffer.
    ///
    /// # Safety
    ///
    /// The caller **must not** read from the referenced memory and **must not** write
    /// **uninitialized** bytes to the slice either. This is because `BufMut` implementation
    /// that created the `UninitSlice` knows which parts are initialized. Writing uninitialized
    /// bytes to the slice may cause the `BufMut` to read those bytes and trigger undefined
    /// behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BufMut;
    ///
    /// let mut data = [0, 1, 2];
    /// let mut slice = &mut data[..];
    /// unsafe {
    ///     let uninit_slice = BufMut::chunk_mut(&mut slice).as_uninit_slice_mut();
    /// };
    /// ```
    #[inline]
    #[ensures(result@.len() == self@.len())]
    #[ensures(forall<i> 0 <= i && i < self@.len() ==> result@[i]@ == self@[i])]
    #[ensures((^result)@.len() == (^self)@.len())]
    #[ensures(forall<i> 0 <= i && i < (^self)@.len() ==> (^result)@[i]@ == (^self)@[i])]
    pub unsafe fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.0
    }
}
impl<'a> From<&'a mut [u8]> for &'a mut UninitSlice {
    #[ensures(result@.len() == slice@.len())]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == Some(slice@[i]))]
    #[ensures((^result)@.len() == slice@.len())]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> (^result)@[i] == Some((^slice)@[i]))]
    #[ensures((^slice)@.len() == slice@.len())]
    fn from(slice: &'a mut [u8]) -> Self {
        UninitSlice::new(slice)
    }
}
impl<'a> From<&'a mut [MaybeUninit<u8>]> for &'a mut UninitSlice {
    #[ensures(result@.len() == slice@.len())]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == slice@[i]@)]
    #[ensures((^result)@.len() == slice@.len())]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> (^result)@[i] == (^slice)@[i]@)]
    #[ensures((^slice)@.len() == slice@.len())]
    fn from(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        UninitSlice::uninit(slice)
    }
}

use core::ops::{Index, IndexMut, RangeFull};
macro_rules! impl_index {
	(@range_full) => {
		impl Index<RangeFull> for UninitSlice {
			type Output = UninitSlice;

			#[inline]
			#[cfg_attr(creusot, check(ghost))]
			#[cfg_attr(creusot, ensures(result@.len() == self@.len()))]
			#[cfg_attr(creusot, ensures(result@ == self@))]
			fn index(&self, index: RangeFull) -> &UninitSlice {
				UninitSlice::uninit_ref(&self.0[index])
			}
		}

		impl IndexMut<RangeFull> for UninitSlice {
			#[inline]
			#[cfg_attr(creusot, check(ghost))]
			#[cfg_attr(creusot, ensures(result@.len() == self@.len()))]
			#[cfg_attr(creusot, ensures(result@ == self@))]
			#[cfg_attr(creusot, ensures((^result)@.len() == (^self)@.len()))]
			#[cfg_attr(creusot, ensures((^result)@ == (^self)@))]
			fn index_mut(&mut self, index: RangeFull) -> &mut UninitSlice {
				UninitSlice::uninit(&mut self.0[index])
			}
		}
	};
    ($($t:ty),*) => {
        $(
            impl Index<$t> for UninitSlice {
                type Output = UninitSlice;

                #[inline]
                fn index(&self, index: $t) -> &UninitSlice {
                    UninitSlice::uninit_ref(&self.0[index])
                }
            }

            impl IndexMut<$t> for UninitSlice {
                #[inline]
                fn index_mut(&mut self, index: $t) -> &mut UninitSlice {
                    UninitSlice::uninit(&mut self.0[index])
                }
            }
        )*
    };
}
impl_index!(@range_full);
