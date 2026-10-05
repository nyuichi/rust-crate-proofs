// BEGIN EXACT UNIQUE ADVANCE RAW TRANSITION
// The checked unique-offset transition carries affine ownership through a
// private descriptor that has no BytesMut invariant or destructor.
#[cfg(all(creusot, bytes_proof_valid_handle))]
struct RawTransition {
    ptr: crate::ownership_proof::raw_vec::BoundPtr,
    len: usize,
    cap: usize,
    data: *mut Shared,
    unique_at_zero: Ghost<Option<(
        crate::ownership_proof::raw_vec::Recovery,
        crate::ownership_proof::raw_vec::PhysicalRegion,
    )>>,
    pending_control: Ghost<Option<sequential_shared_control::PendingControl>>,
    shared_registration: Ghost<Option<sequential_shared_control::HandleRegistration>>,
    shared_context: Ghost<Option<sequential_shared_control::ControlContext>>,
}

#[cfg(all(creusot, bytes_proof_valid_handle))]
impl RawTransition {
    #[logic]
    fn view_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(self.ptr@.unwrap_logic().2 + index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(self.ptr@.unwrap_logic().2 + index),
                },
            }
        }
    }

    #[logic]
    fn owned_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(index),
                },
            }
        }
    }

    #[logic(prophetic)]
    fn unique_owned(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_VEC &&
            self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None &&
            self.shared_context.inner_logic() == None &&
            match self.unique_at_zero.inner_logic() {
                None => false,
                Some((recovery, region)) =>
                    self.ptr.invariant() && self.ptr@ != None &&
                    self.ptr@.unwrap_logic().0 == recovery.namespace() &&
                    self.ptr@.unwrap_logic().1 == recovery.capacity() &&
                    self.ptr@.unwrap_logic().2 ==
                        (self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET)@ &&
                    self.ptr@.unwrap_logic().2 + self.cap@ == recovery.capacity() &&
                    recovery.invariant() && region.invariant() &&
                    region.capacity() == recovery.capacity() &&
                    region.namespace() == recovery.namespace() &&
                    region.resource_id() == recovery.namespace() &&
                    region.lo() == 0 && region.hi() == recovery.capacity() &&
                    self.len <= self.cap
            }
        }
    }

    #[logic(prophetic)]
    fn initialized(self) -> bool {
        pearlite! {
            self.unique_owned() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.view_slot(index))
        }
    }

    #[cfg_attr(creusot, requires(value.proof_unique_owned() && value.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result.unique_owned() && result.initialized()))]
    #[cfg_attr(creusot, ensures(
        result.ptr == value.ptr && result.len == value.len &&
        result.cap == value.cap && result.data == value.data
    ))]
    #[cfg_attr(creusot, ensures(
        result.unique_at_zero == value.unique_at_zero &&
        result.pending_control == value.pending_control &&
        result.shared_registration == value.shared_registration &&
        result.shared_context == value.shared_context
    ))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        result.view_slot(index) == value.proof_view_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        result.owned_slot(index) == value.proof_owned_slot(index)))]
    fn from_valid(value: BytesMut) -> Self {
        let mut value = value;
        let raw = Self {
            ptr: mem::replace(
                &mut value.ptr,
                crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling()),
            ),
            len: mem::replace(&mut value.len, 0),
            cap: mem::replace(&mut value.cap, 0),
            data: mem::replace(&mut value.data, invalid_ptr(KIND_VEC)),
            unique_at_zero: mem::replace(&mut value.unique_at_zero, ghost! { None }),
            pending_control: mem::replace(&mut value.pending_control, ghost! { None }),
            shared_registration: mem::replace(&mut value.shared_registration, ghost! { None }),
            shared_context: mem::replace(&mut value.shared_context, ghost! { None }),
        };
        // Keep the BytesMut destructor from interpreting the emptied shell as
        // a live Vec owner. Every field and its affine proof capabilities now
        // reside in `raw`.
        mem::forget(value);
        raw
    }

    #[cfg_attr(creusot, requires(self.unique_owned() && self.initialized()))]
    #[cfg_attr(creusot, requires(count <= self.cap))]
    #[cfg_attr(creusot, requires(
        self.ptr@.unwrap_logic().2 + count@ <= crate::capacity_ops::MAX_VEC_POS@
    ))]
    #[cfg_attr(creusot, ensures(result.proof_unique_owned() && result.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result.len@ ==
        (if count <= self.len { self.len@ - count@ } else { 0int })))]
    #[cfg_attr(creusot, ensures(result.cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures(result.ptr@ == Some((
        self.ptr@.unwrap_logic().0,
        self.ptr@.unwrap_logic().1,
        self.ptr@.unwrap_logic().2 + count@
    ))))]
    #[cfg_attr(creusot, ensures(result.unique_at_zero == self.unique_at_zero))]
    #[cfg_attr(creusot, ensures(result.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK ==
        self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures(
        result.pending_control == self.pending_control &&
        result.shared_registration == self.shared_registration &&
        result.shared_context == self.shared_context
    ))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        result.proof_view_slot(index) == self.view_slot(index + count@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        result.proof_owned_slot(index) == self.owned_slot(index)))]
    fn advance_to_valid(self, count: usize) -> BytesMut {
        let Self { ptr, len, cap, data, unique_at_zero, pending_control,
            shared_registration, shared_context } = self;
        let old_addr = crate::provenance_specs::pointer_addr(data);
        let pos = crate::capacity_ops::vec_pos_from_data(old_addr) + count;
        let packed = crate::capacity_ops::set_vec_pos_in_data(old_addr, pos);
        BytesMut {
            ptr: ptr.advance_within(count),
            len: len.saturating_sub(count),
            cap: cap - count,
            data: invalid_ptr(packed),
            unique_at_zero, pending_control, shared_registration, shared_context,
        }
    }
}
// END EXACT UNIQUE ADVANCE RAW TRANSITION