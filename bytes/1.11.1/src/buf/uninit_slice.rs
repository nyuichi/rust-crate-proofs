use core::fmt;
use core::mem::MaybeUninit;
use core::ops::{
    Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use creusot_std::prelude::*;

/// Uninitialized byte slice.
///
/// Returned by `BufMut::chunk_mut()`, the referenced byte slice may be
/// uninitialized. The wrapper provides safe access without introducing
/// undefined behavior.
///
/// The safety invariants of this wrapper are:
///
///  1. Reading from an `UninitSlice` is undefined behavior.
///  2. Writing uninitialized bytes to an `UninitSlice` is undefined behavior.
///
/// The difference between `&mut UninitSlice` and `&mut [MaybeUninit<u8>]` is
/// that it is possible in safe code to write uninitialized bytes to an
/// `&mut [MaybeUninit<u8>]`, which this type prohibits.
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

    #[trusted]
    #[cfg_attr(creusot, check(ghost))]
    #[ensures(result@.len() == slice@.len())]
    #[ensures(result@ == Seq::create(slice@.len(), |index: Int| slice@[index]@))]
    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == slice@[i]@)]
    fn uninit_ref(slice: &[MaybeUninit<u8>]) -> &UninitSlice {
        unsafe { &*(slice as *const [MaybeUninit<u8>] as *const UninitSlice) }
    }

    /// Create a `&mut UninitSlice` from a pointer and a length.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` references a valid memory region owned
    /// by the caller representing a byte slice for the duration of `'a`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::buf::UninitSlice;
    ///
    /// let bytes = b"hello world".to_vec();
    /// let ptr = bytes.as_ptr() as *mut _;
    /// let len = bytes.len();
    ///
    /// let slice = unsafe { UninitSlice::from_raw_parts_mut(ptr, len) };
    /// ```
    #[cfg(not(bytes_proof_bound_uninit_raw))]
    #[inline]
    pub unsafe fn from_raw_parts_mut<'a>(ptr: *mut u8, len: usize) -> &'a mut UninitSlice {
        let maybe_init: &mut [MaybeUninit<u8>] =
            core::slice::from_raw_parts_mut(ptr as *mut _, len);
        Self::uninit(maybe_init)
    }

    /// Proof-only form of `from_raw_parts_mut` that requires the sealed
    /// allocation and exclusive interval capability used by the ownership
    /// model. The ordinary public signature remains active outside this probe
    /// cfg; this variant is not a capability-free raw-pointer conversion.
    #[cfg(bytes_proof_bound_uninit_raw)]
    #[inline]
    #[requires(ptr == bound.raw_pointer())]
    #[requires(bound.invariant() && bound@ != None)]
    #[requires(region.inner_logic().invariant())]
    #[requires(bound@.unwrap_logic().0 == region.inner_logic().namespace())]
    #[requires(bound@.unwrap_logic().1 == region.inner_logic().capacity())]
    #[requires(region.inner_logic().lo() <= bound@.unwrap_logic().2)]
    #[requires(bound@.unwrap_logic().2 + len@ <= region.inner_logic().hi())]
    #[ensures(result@.len() == len@ && (^result)@.len() == len@)]
    #[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
        region.inner_logic().slot(bound@.unwrap_logic().2 + offset) == Some(result@[offset]))]
    #[ensures((^region.inner_logic()).invariant())]
    #[ensures((^region.inner_logic()).namespace() == region.inner_logic().namespace())]
    #[ensures((^region.inner_logic()).capacity() == region.inner_logic().capacity())]
    #[ensures((^region.inner_logic()).lo() == region.inner_logic().lo())]
    #[ensures((^region.inner_logic()).hi() == region.inner_logic().hi())]
    #[ensures((^region.inner_logic()).resource_id() == region.inner_logic().resource_id())]
    #[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
        (^region.inner_logic()).slot(bound@.unwrap_logic().2 + offset) == Some((^result)@[offset]))]
    #[ensures(forall<index: Int>
        !(bound@.unwrap_logic().2 <= index && index < bound@.unwrap_logic().2 + len@) ==>
            (^region.inner_logic()).slot(index) == region.inner_logic().slot(index))]
    pub unsafe fn from_raw_parts_mut<'a>(
        ptr: *mut u8,
        len: usize,
        bound: crate::ownership_proof::raw_vec::BoundPtr,
        region: Ghost<&'a mut crate::ownership_proof::raw_vec::PhysicalRegion>,
    ) -> &'a mut UninitSlice {
        assert!(ptr == bound.as_ptr());
        let maybe_init = unsafe {
            crate::ownership_proof::raw_vec::borrow_bound_uninit_mut(bound, len, region)
        };
        Self::uninit(maybe_init)
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
    #[requires(forall<i: Int> 0 <= i && i < self@.len() && self@[i] != None ==> (^self)@[i] != None)]
    #[ensures(result@.len() == self@.len())]
    #[ensures(forall<i> 0 <= i && i < self@.len() ==> result@[i]@ == self@[i])]
    #[ensures((^result)@.len() == (^self)@.len())]
    #[ensures(forall<i> 0 <= i && i < (^self)@.len() ==> (^result)@[i]@ == (^self)@[i])]
    pub unsafe fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.0
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

impl fmt::Debug for UninitSlice {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("UninitSlice[...]").finish()
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

#[cfg(creusot)]
mod uninit_index_model {
    use core::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
    use creusot_std::prelude::*;

    pub(super) trait UninitSliceIndexModel {
        #[logic]
        fn uninit_in_bounds(self, source: Seq<Option<u8>>) -> bool;

        #[logic]
        fn uninit_view(self, source: Seq<Option<u8>>) -> Seq<Option<u8>>;

        #[logic]
        fn uninit_frame(
            self,
            old: Seq<Option<u8>>,
            final_view: Seq<Option<u8>>,
        ) -> bool;
    }

    impl UninitSliceIndexModel for Range<usize> {
        #[logic(open)]
        fn uninit_in_bounds(self, source: Seq<Option<u8>>) -> bool {
            pearlite! { self.start@ <= self.end@ && self.end@ <= source.len() }
        }

        #[logic(open)]
        fn uninit_view(self, source: Seq<Option<u8>>) -> Seq<Option<u8>> {
            pearlite! { source.subsequence(self.start@, self.end@) }
        }

        #[logic(open)]
        fn uninit_frame(
            self,
            old: Seq<Option<u8>>,
            final_view: Seq<Option<u8>>,
        ) -> bool {
            pearlite! {
                forall<i> 0 <= i && (i < self.start@ || self.end@ <= i) && i < old.len()
                ==> old[i] == final_view[i]
            }
        }
    }

    impl UninitSliceIndexModel for RangeFrom<usize> {
        #[logic(open)]
        fn uninit_in_bounds(self, source: Seq<Option<u8>>) -> bool {
            pearlite! { self.start@ <= source.len() }
        }

        #[logic(open)]
        fn uninit_view(self, source: Seq<Option<u8>>) -> Seq<Option<u8>> {
            pearlite! { source.subsequence(self.start@, source.len()) }
        }

        #[logic(open)]
        fn uninit_frame(
            self,
            old: Seq<Option<u8>>,
            final_view: Seq<Option<u8>>,
        ) -> bool {
            pearlite! {
                forall<i> 0 <= i && i < self.start@ && i < old.len()
                ==> old[i] == final_view[i]
            }
        }
    }

    impl UninitSliceIndexModel for RangeTo<usize> {
        #[logic(open)]
        fn uninit_in_bounds(self, source: Seq<Option<u8>>) -> bool {
            pearlite! { self.end@ <= source.len() }
        }

        #[logic(open)]
        fn uninit_view(self, source: Seq<Option<u8>>) -> Seq<Option<u8>> {
            pearlite! { source.subsequence(0, self.end@) }
        }

        #[logic(open)]
        fn uninit_frame(
            self,
            old: Seq<Option<u8>>,
            final_view: Seq<Option<u8>>,
        ) -> bool {
            pearlite! {
                forall<i> self.end@ <= i && i < old.len()
                ==> old[i] == final_view[i]
            }
        }
    }

    impl UninitSliceIndexModel for RangeToInclusive<usize> {
        #[logic(open)]
        fn uninit_in_bounds(self, source: Seq<Option<u8>>) -> bool {
            pearlite! { self.end@ < source.len() }
        }

        #[logic(open)]
        fn uninit_view(self, source: Seq<Option<u8>>) -> Seq<Option<u8>> {
            pearlite! { source.subsequence(0, self.end@ + 1) }
        }

        #[logic(open)]
        fn uninit_frame(
            self,
            old: Seq<Option<u8>>,
            final_view: Seq<Option<u8>>,
        ) -> bool {
            pearlite! {
                forall<i> self.end@ < i && i < old.len()
                ==> old[i] == final_view[i]
            }
        }
    }

    impl UninitSliceIndexModel for RangeInclusive<usize> {
        #[logic(open)]
        fn uninit_in_bounds(self, source: Seq<Option<u8>>) -> bool {
            pearlite! {
                self.end_log()@ < source.len()
                    && self.start_log()@ <= self.end_log()@ + 1
            }
        }

        #[logic(open)]
        fn uninit_view(self, source: Seq<Option<u8>>) -> Seq<Option<u8>> {
            pearlite! {
                if self.is_empty_log() {
                    Seq::empty()
                } else {
                    source.subsequence(self.start_log()@, self.end_log()@ + 1)
                }
            }
        }

        #[logic(open)]
        fn uninit_frame(
            self,
            old: Seq<Option<u8>>,
            final_view: Seq<Option<u8>>,
        ) -> bool {
            pearlite! {
                forall<i> 0 <= i
                    && (i < self.start_log()@ || self.end_log()@ < i || self.is_empty_log())
                    && i < old.len()
                ==> old[i] == final_view[i]
            }
        }
    }
}

macro_rules! impl_index {
    (@range_bounds) => {
        impl Index<Range<usize>> for UninitSlice {
            type Output = UninitSlice;

            #[inline]
            #[cfg_attr(creusot, check(ghost))]
            #[cfg_attr(creusot, requires(index.start@ <= index.end@ && index.end@ <= self@.len()))]
            #[cfg_attr(creusot, ensures(result@.len() == index.end@ - index.start@))]
            #[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < result@.len() ==> result@[i] == self@[index.start@ + i]))]
            fn index(&self, index: Range<usize>) -> &UninitSlice {
                UninitSlice::uninit_ref(&self.0[index])
            }
        }

        impl IndexMut<Range<usize>> for UninitSlice {
            #[inline]
            #[cfg_attr(creusot, check(ghost))]
            #[cfg_attr(creusot, requires(index.start@ <= index.end@ && index.end@ <= self@.len()))]
            #[cfg_attr(creusot, ensures(result@.len() == index.end@ - index.start@))]
            #[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < result@.len() ==> result@[i] == self@[index.start@ + i]))]
            #[cfg_attr(creusot, ensures((^result)@.len() == index.end@ - index.start@))]
            #[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < (^result)@.len() ==> (^result)@[i] == (^self)@[index.start@ + i]))]
            #[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < self@.len() && (i < index.start@ || index.end@ <= i) ==> (^self)@[i] == self@[i]))]
            #[cfg_attr(creusot, ensures((^self)@.len() == self@.len()))]
            fn index_mut(&mut self, index: Range<usize>) -> &mut UninitSlice {
                UninitSlice::uninit(&mut self.0[index])
            }
        }
    };
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
    (@range_family $index_ty:ty) => {
        impl Index<$index_ty> for UninitSlice {
            type Output = UninitSlice;

            #[inline]
            #[cfg_attr(creusot, check(ghost))]
            #[cfg_attr(creusot, requires(index.uninit_in_bounds(self@)))]
            #[cfg_attr(creusot, ensures(result@ == index.uninit_view(self@)))]
            fn index(&self, index: $index_ty) -> &UninitSlice {
                UninitSlice::uninit_ref(&self.0[index])
            }
        }

        impl IndexMut<$index_ty> for UninitSlice {
            #[inline]
            #[cfg_attr(creusot, check(ghost))]
            #[cfg_attr(creusot, requires(index.uninit_in_bounds(self@)))]
            #[cfg_attr(creusot, ensures(result@ == index.uninit_view(self@)))]
            #[cfg_attr(creusot, ensures((^result)@ == index.uninit_view((^self)@)))]
            #[cfg_attr(creusot, ensures(index.uninit_frame(self@, (^self)@)))]
            #[cfg_attr(creusot, ensures((^self)@.len() == self@.len()))]
            fn index_mut(&mut self, index: $index_ty) -> &mut UninitSlice {
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

impl_index!(
    Range<usize>,
    RangeFrom<usize>,
    RangeFull,
    RangeInclusive<usize>,
    RangeTo<usize>,
    RangeToInclusive<usize>
);
