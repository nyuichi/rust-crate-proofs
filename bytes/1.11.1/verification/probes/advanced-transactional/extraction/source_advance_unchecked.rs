    // BEGIN EXACT ADVANCE_UNCHECKED
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), requires(self.proof_empty_valid() || (self.ptr.invariant() && self.ptr@ != None)))]
    #[cfg_attr(all(creusot, not(bytes_proof_valid_handle)), requires(self.ptr.invariant() && self.ptr@ != None))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), requires(self.proof_empty_valid() ||
        self.ptr@.unwrap_logic().2 + self.cap@ <= self.ptr@.unwrap_logic().1))]
    #[cfg_attr(all(creusot, not(bytes_proof_valid_handle)), requires(
        self.ptr@.unwrap_logic().2 + self.cap@ <= self.ptr@.unwrap_logic().1))]
    #[cfg_attr(creusot, requires(count <= self.cap && self.len <= self.cap))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), requires(self.proof_empty_valid() ==> count@ == 0int))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), requires(self.proof_empty_valid() ||
        (self.data.addr_logic() & KIND_MASK == KIND_ARC && self.proof_shared_allocation_registered()) ||
        (self.proof_unique_owned() && self.ptr@.unwrap_logic().2 + count@ <= crate::capacity_ops::MAX_VEC_POS@)))]
    #[cfg_attr(all(creusot, not(bytes_proof_valid_handle)), requires(
        (self.data.addr_logic() & KIND_MASK == KIND_ARC && self.proof_shared_allocation_registered()) ||
        (self.proof_unique_owned() && self.ptr@.unwrap_logic().2 + count@ <= crate::capacity_ops::MAX_VEC_POS@)))]
    #[cfg_attr(creusot, ensures((^self).ptr.invariant()))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.proof_empty_valid() ||
        (^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(all(creusot, not(bytes_proof_valid_handle)), ensures(
        (^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.proof_empty_valid() ==> (^self).proof_empty_valid()))]
    #[cfg_attr(creusot, ensures((^self).len@ == (if count <= self.len { self.len@ - count@ } else { 0int }) && (^self).cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures(self.data.addr_logic() & KIND_MASK == KIND_ARC ==> (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(self.proof_unique_owned() ==> (^self).proof_unique_owned()))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(creusot, ensures(self.proof_registered_valid() ==> (^self).proof_registered_valid()))]
    #[cfg_attr(creusot, ensures(self.proof_initialized() ==> (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        (^self).proof_view_slot(index) == self.proof_view_slot(index + count@)))]
    pub(crate) unsafe fn advance_unchecked(&mut self, count: usize) {
        // Setting the start to 0 is a no-op, so return early if this is the
        // case.
        if count == 0 {
            return;
        }

        debug_assert!(count <= self.cap, "internal: set_start out of bounds");

        let kind = self.kind();

        #[cfg(all(creusot, bytes_proof_valid_handle))]
        if kind == KIND_VEC {
            let empty = BytesMut {
                ptr: crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling()),
                len: 0,
                cap: 0,
                data: invalid_ptr(KIND_VEC),
                unique_at_zero: ghost! { None },
                pending_control: ghost! { None },
                shared_registration: ghost! { None },
                shared_context: ghost! { None },
            };
            let old = mem::replace(self, empty);
            let raw = RawTransition::from_valid(old);
            let updated = raw.advance_to_valid(count);
            // The result is already valid. Do not let BytesMut::drop inspect
            // the resource-free placeholder returned by this replacement.
            let placeholder = mem::replace(self, updated);
            mem::forget(placeholder);
            return;
        }

        #[cfg(not(all(creusot, bytes_proof_valid_handle)))]
        if kind == KIND_VEC {
            // Setting the start when in vec representation is a little more
            // complicated. First, we have to track how far ahead the
            // "start" of the byte buffer from the beginning of the vec. We
            // also have to ensure that we don't exceed the maximum shift.
            let pos = self.get_vec_pos() + count;

            if pos <= MAX_VEC_POS {
                self.set_vec_pos(pos);
            } else {
                // The repr must be upgraded to ARC. This will never happen
                // on 64 bit systems and will only happen on 32 bit systems
                // when shifting past 134,217,727 bytes. As such, we don't
                // worry too much about performance here.
                #[cfg(not(any(creusot, bytes_proof_probe)))]
                self.promote_to_shared(/*ref_count = */ 1);
                #[cfg(any(creusot, bytes_proof_probe))]
                panic!("unique offset overflow promotion is outside this proof gate");
            }
        }

        // Updating the start of the view is setting `ptr` to point to the
        // new start and updating the `len` field to reflect the new length
        // of the view.
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { self.ptr = vptr(self.ptr.as_ptr().add(count)); }
        #[cfg(any(creusot, bytes_proof_probe))]
        { self.ptr = self.ptr.advance_within(count); }
        self.len = self.len.saturating_sub(count);
        self.cap -= count;
    }