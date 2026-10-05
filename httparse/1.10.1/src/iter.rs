use core::convert::TryFrom;
use core::convert::TryInto;
use creusot_std::ghost::perm::Perm;
use creusot_std::prelude::*;

#[allow(missing_docs)]
pub struct Bytes<'a> {
    start: *const u8,
    end: *const u8,
    /// INVARIANT: start <= cursor && cursor <= end
    cursor: *const u8,
    /// Ghost ownership witness for the original input allocation.
    permission: Ghost<&'a Perm<*const [u8]>>,
    /// Immutable logical view of the original input bytes.
    #[cfg(creusot)]
    input: Ghost<Seq<u8>>,
    /// Original data pointer retained to express offsets after `commit`.
    origin: Ghost<*const u8>,
    phantom: core::marker::PhantomData<&'a ()>,
}

#[allow(missing_docs)]
impl<'a> Bytes<'a> {
    #[inline]
    #[ensures(result@.input == slice@)]
    #[ensures(result@.mark == 0 && result@.cursor == 0 && result@.end == slice@.len())]
    pub fn new(slice: &'a [u8]) -> Bytes<'a> {
        let (start, permission) = slice.as_ptr_perm();
        #[cfg(creusot)]
        let input = snapshot!(slice@).into_ghost();
        let origin = ghost!(start);
        // SAFETY: this witness covers every element and the one-past-end pointer
        // of the input slice's allocation.
        let end = unsafe { start.add_live(slice.len(), ghost!(permission.live())) };
        let cursor = start;
        Bytes {
            start,
            end,
            cursor,
            permission,
            #[cfg(creusot)]
            input,
            origin,
            phantom: core::marker::PhantomData,
        }
    }

    #[inline]
    #[ensures(result@ == crate::verification_model::cursor_pos(self@))]
    pub fn pos(&self) -> usize {
        self.cursor.expose_provenance() - self.start.expose_provenance()
    }

    #[inline]
    #[ensures(match result {
        None => self@.cursor == self@.end,
        Some(byte) => self@.cursor < self@.end
            && byte@ == self@.input[self@.cursor]@,
    })]
    pub fn peek(&self) -> Option<u8> {
        if self.cursor.addr() < self.end.addr() {
            // SAFETY: the element permission is selected from the retained
            // input-slice permission at this pointer's offset.
            let element = self.byte_permission(self.cursor);
            Some(unsafe { *Perm::as_ref(self.cursor, element) })
        } else {
            None
        }
    }

    /// Peek at byte `n` ahead of cursor
    ///
    /// # Safety
    ///
    /// Caller must ensure that `n <= self.len()`, otherwise `self.cursor.add(n)` is UB.
    /// That means there are at least `n-1` bytes between `self.cursor` and `self.end`
    /// and `self.cursor.add(n)` is either `self.end` or points to a valid byte.
    #[inline]
    #[requires(n@ <= crate::verification_model::cursor_len(self@))]
    #[ensures(match result {
        None => self@.cursor + n@ == self@.end,
        Some(byte) => self@.cursor + n@ < self@.end
            && byte@ == self@.input[self@.cursor + n@]@,
    })]
    pub unsafe fn peek_ahead(&self, n: usize) -> Option<u8> {
        debug_assert!(n <= self.len());
        // SAFETY: by preconditions
        let p = unsafe { self.cursor.add_live(n, ghost!(self.permission.live())) };
        if p.addr() < self.end.addr() {
            // SAFETY: by preconditions, if this is not `self.end`,
            // then it is safe to dereference
            Some(unsafe { *Perm::as_ref(p, self.byte_permission(p)) })
        } else {
            None
        }
    }

    #[inline]
    pub fn peek_n<'b: 'a, U: TryFrom<&'a [u8]>>(&'b self, n: usize) -> Option<U> {
        // TODO: once we bump MSRC, use const generics to allow only [u8; N] reads
        // TODO: drop `n` arg in favour of const
        // let n = core::mem::size_of::<U>();
        self.as_ref().get(..n)?.try_into().ok()
    }

    /// Advance by 1, equivalent to calling `advance(1)`.
    ///
    /// # Safety
    ///
    /// Caller must ensure that Bytes hasn't been advanced/bumped by more than [`Bytes::len()`].
    #[inline]
    #[requires(self@.cursor < self@.end)]
    #[ensures((^self)@.cursor == self@.cursor + 1)]
    pub unsafe fn bump(&mut self) {
        self.advance(1)
    }

    /// Advance cursor by `n`
    ///
    /// # Safety
    ///
    /// Caller must ensure that Bytes hasn't been advanced/bumped by more than [`Bytes::len()`].
    #[inline]
    #[requires(n@ <= crate::verification_model::cursor_len(self@))]
    #[ensures((^self)@.input == self@.input
        && (^self)@.mark == self@.mark
        && (^self)@.cursor == self@.cursor + n@
        && (^self)@.end == self@.end)]
    pub unsafe fn advance(&mut self, n: usize) {
        self.cursor = self.cursor.add_live(n, ghost!(self.permission.live()));
        debug_assert!(self.cursor.addr() <= self.end.addr(), "overflow");
    }

    #[inline]
    #[ensures(result@ == crate::verification_model::cursor_len(self@))]
    pub fn len(&self) -> usize {
        self.end.expose_provenance() - self.cursor.expose_provenance()
    }

    #[inline]
    #[ensures(result == (self@.cursor == self@.end))]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    #[ensures(result@ == crate::verification_model::cursor_slice(self@))]
    #[ensures((^self)@ == crate::verification_model::cursor_commit(self@))]
    pub fn slice(&mut self) -> &'a [u8] {
        // SAFETY: not moving position at all, so it's safe
        let slice = unsafe {
            slice_from_ptr_range(self.start, self.cursor, self.permission, self.origin)
        };
        self.commit();
        slice
    }

    // TODO: this is an anti-pattern, should be removed
    /// Deprecated. Do not use!
    /// # Safety
    ///
    /// Caller must ensure that `skip` is at most the number of advances (i.e., `bytes.advance(3)`
    /// implies a skip of at most 3). `skip` must also be representable as an `isize` offset.
    #[inline]
    #[requires(skip@ <= crate::verification_model::cursor_pos(self@))]
    #[requires(skip@ <= isize::MAX@)]
    #[ensures(result@ == crate::verification_model::cursor_slice_skip(self@, skip@))]
    #[ensures((^self)@ == crate::verification_model::cursor_commit(self@))]
    pub unsafe fn slice_skip(&mut self, skip: usize) -> &'a [u8] {
        debug_assert!(skip <= self.cursor.addr() - self.start.addr());
        // SAFETY: `skip` is within the range from start to cursor, and fits isize.
        let head_end = unsafe {
            self.cursor
                .offset_live(-(skip as isize), ghost!(self.permission.live()))
        };
        let head = slice_from_ptr_range(self.start, head_end, self.permission, self.origin);
        self.commit();
        head
    }

    #[inline]
    #[ensures((^self)@ == crate::verification_model::cursor_commit(self@))]
    pub fn commit(&mut self) {
        self.start = self.cursor
    }

    /// # Safety
    ///
    /// see [`Bytes::advance`] safety comment.
    #[inline]
    #[requires(n@ <= crate::verification_model::cursor_len(self@))]
    #[ensures((^self)@.input == self@.input
        && (^self)@.mark == self@.cursor + n@
        && (^self)@.cursor == self@.cursor + n@
        && (^self)@.end == self@.end)]
    pub unsafe fn advance_and_commit(&mut self, n: usize) {
        self.advance(n);
        self.commit();
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.cursor
    }

    #[inline]
    pub fn start(&self) -> *const u8 {
        self.start
    }

    #[inline]
    pub fn end(&self) -> *const u8 {
        self.end
    }

    /// Verification-only pointer to the original allocation's first byte.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub fn verification_origin(self) -> *const u8 {
        pearlite! { *self.origin }
    }

    /// Verification-only validity test for a cursor in this Bytes allocation.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub fn verification_cursor_is_valid(self, ptr: *const u8) -> bool {
        pearlite! {
            let index = ptr.sub_logic(self.verification_origin());
            self@.mark <= index && index <= self@.end
                && ptr == self.verification_origin().offset_logic(index)
        }
    }

    /// Verification-only absolute input index of a cursor.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub fn verification_cursor_index(self, ptr: *const u8) -> Int {
        pearlite! { ptr.sub_logic(self.verification_origin()) }
    }

    /// # Safety
    ///
    /// Must ensure invariant `bytes.start() <= ptr && ptr <= bytes.end()`.
    /// `ptr` must retain provenance from the same input allocation as `bytes`,
    /// and must identify an element or the one-past-end pointer between the
    /// current start and end pointers.
    #[inline]
    #[requires(self.verification_cursor_is_valid(ptr))]
    #[ensures((^self)@.input == self@.input
        && (^self)@.mark == self@.mark
        && (^self)@.cursor == self.verification_cursor_index(ptr)
        && (^self)@.end == self@.end)]
    pub unsafe fn set_cursor(&mut self, ptr: *const u8) {
        debug_assert!(ptr.addr() >= self.start.addr());
        debug_assert!(ptr.addr() <= self.end.addr());
        self.cursor = ptr;
    }

    #[inline]
    #[check(ghost)]
    #[requires(self.verification_cursor_is_valid(ptr))]
    #[requires(self.verification_cursor_index(ptr) < self@.end)]
    #[ensures(*result.ward() == ptr)]
    #[ensures(*result.val() == self@.input[self.verification_cursor_index(ptr)])]
    fn byte_permission(&self, ptr: *const u8) -> Ghost<&Perm<*const u8>> {
        #[cfg(creusot)]
        {
            ghost!({
                let origin = *self.origin;
                let index = snapshot!(ptr.sub_logic(origin)).into_ghost().into_inner();
                self.permission.index(index)
            })
        }
        #[cfg(not(creusot))]
        {
            let _ = ptr;
            Ghost::conjure()
        }
    }
}

#[cfg(creusot)]
impl creusot_std::model::View for Bytes<'_> {
    type ViewTy = crate::verification_model::CursorModel;

    #[logic(open(self))]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            crate::verification_model::CursorModel {
                input: *self.input,
                mark: self.start.sub_logic(*self.origin),
                cursor: self.cursor.sub_logic(*self.origin),
                end: self.end.sub_logic(*self.origin),
            }
        }
    }
}

#[cfg(creusot)]
impl creusot_std::invariant::Invariant for Bytes<'_> {
    #[logic(open(self))]
    fn invariant(self) -> bool {
        pearlite! {
            let permission = *self.permission;
            let origin = *self.origin;
            let model = self@;
            crate::verification_model::valid_cursor(model)
                && permission.val()@ == model.input
                && permission.ward().thin() == origin
                && origin.addr_logic()@ + model.end <= usize::MAX@
                && self.start == origin.offset_logic(model.mark)
                && self.cursor == origin.offset_logic(model.cursor)
                && self.end == origin.offset_logic(model.end)
        }
    }
}

impl AsRef<[u8]> for Bytes<'_> {
    #[inline]
    #[ensures(result@ == crate::verification_model::cursor_remaining(self@))]
    fn as_ref(&self) -> &[u8] {
        // SAFETY: both pointers share the retained input permission, and the
        // range is within its live allocation.
        unsafe { slice_from_ptr_range(self.cursor, self.end, self.permission, self.origin) }
    }
}

/// # Safety
///
/// `start` and `end` must have the provenance of `permission` and delimit an ordered
/// range within the slice represented by that permission.
#[inline]
#[requires(permission.ward().thin() == *origin)]
#[requires(0 <= start.sub_logic(*origin))]
#[requires(start.sub_logic(*origin) <= end.sub_logic(*origin))]
#[requires(end.sub_logic(*origin) <= permission.len())]
#[requires(start == (*origin).offset_logic(start.sub_logic(*origin)))]
#[requires(end == (*origin).offset_logic(end.sub_logic(*origin)))]
#[requires((*origin).addr_logic()@ + end.sub_logic(*origin) <= usize::MAX@)]
#[ensures(result@ == permission.val()@[start.sub_logic(*origin)..end.sub_logic(*origin)])]
unsafe fn slice_from_ptr_range<'a>(
    start: *const u8,
    end: *const u8,
    permission: Ghost<&'a Perm<*const [u8]>>,
    origin: Ghost<*const u8>,
) -> &'a [u8] {
    debug_assert!(start.addr() <= end.addr());
    let len = end.expose_provenance() - start.expose_provenance();
    let slice_ptr = core::ptr::slice_from_raw_parts(start, len);
    #[cfg(creusot)]
    let range_permission = {
        ghost!({
            let base = *origin;
            let from = snapshot!(start.sub_logic(base)).into_ghost().into_inner();
            let to = snapshot!(end.sub_logic(base)).into_ghost().into_inner();
            permission.split_at(from).1.split_at(to - from).0
        })
    };
    #[cfg(not(creusot))]
    let range_permission = {
        let _ = permission;
        let _ = origin;
        Ghost::conjure()
    };
    unsafe { Perm::as_ref(slice_ptr, range_permission) }
}

// TEMPORARY verification slice: the actual Iterator implementation remains
// present in normal builds, but is excluded from this Creusot translation
// until Bytes has an IteratorSpec model. Track this as an open public body.
#[cfg(not(creusot))]
impl Iterator for Bytes<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.cursor.addr() < self.end.addr() {
            // SAFETY: bounds checked dereference
            unsafe {
                let b = *Perm::as_ref(self.cursor, self.byte_permission(self.cursor));
                self.bump();
                Some(b)
            }
        } else {
            None
        }
    }
}
