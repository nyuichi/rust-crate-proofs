    // Suggested insertion into impl BytesMut; this file is NOT itself an
    // extracted-source proof. Integration and extraction are the parent's task.
    // BEGIN EXACT UNIQUE_RECLAIM
    #[cfg_attr(creusot, requires(self.proof_unique_owned() && self.proof_initialized()))]
    #[cfg_attr(creusot, requires(self.len@ <= self.ptr@.unwrap_logic().2))]
    #[cfg_attr(creusot, ensures((^self).proof_unique_owned() && (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len == self.len))]
    #[cfg_attr(creusot, ensures((^self).cap@ == self.cap@ + self.ptr@.unwrap_logic().2))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, 0int))))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    fn reclaim_unique(&mut self) {
        let offset = unsafe { self.get_vec_pos() };
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        {
            let base = unsafe { self.ptr.as_ptr().sub(offset) };
            // offset >= len makes source/destination disjoint.
            unsafe { ptr::copy_nonoverlapping(self.ptr.as_ptr(), base, self.len); }
            self.ptr = unsafe { NonNull::new_unchecked(base) };
        }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            let capabilities = ghost! { self.unique_at_zero.take().unwrap() };
            let (recovery, region) = capabilities.split();
            let (base, region) = crate::ownership_proof::unique_reclaim::reclaim(
                self.ptr, offset, self.len, region,
            );
            self.ptr = base;
            self.unique_at_zero = ghost! { Some((recovery.into_inner(), region.into_inner())) };
        }
        self.cap += offset;
        unsafe { self.set_vec_pos(0); }
    }
    // END EXACT UNIQUE_RECLAIM
