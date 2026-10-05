    // BEGIN EXACT SET_VEC_POS
    #[inline]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(pos <= crate::capacity_ops::MAX_VEC_POS))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == pos))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    unsafe fn set_vec_pos(&mut self, pos: usize) {
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(pos <= MAX_VEC_POS);

        self.data = invalid_ptr(crate::capacity_ops::set_vec_pos_in_data(
            crate::provenance_specs::pointer_addr(self.data),
            pos,
        ));
    }