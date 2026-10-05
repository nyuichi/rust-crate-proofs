impl Clone for Bytes {
    #[inline]
    fn clone(&self) -> Bytes {
        unsafe { (self.vtable.clone)(&self.data, self.ptr.as_ptr() as *const u8, self.len, self.vtable) }
    }
}