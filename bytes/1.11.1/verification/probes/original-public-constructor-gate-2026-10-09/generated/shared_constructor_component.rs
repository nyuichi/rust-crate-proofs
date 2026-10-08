pub(crate) struct Payload {
    recovery: raw_vec::Recovery,
    physical_end: EndBorrow<raw_vec::PhysicalRegion>,
    control_end: EndBorrow<field_event::OwnedControl<Shared>>,
    physical: Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    control: Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    base: raw_vec::BoundPtr,
    capacity: usize,
    len: usize,
    expected: Snapshot<Seq<u8>>,
}

impl lifecycle::RecoveryPayload for Payload {
    type Metadata = (Lifetime, Id, Int, Seq<u8>, Int, *const Shared, *mut u8);

    #[logic]
    fn metadata(self) -> Self::Metadata {
        (
            self.control.val().lft(),
            self.recovery.namespace(),
            self.recovery.capacity(),
            *self.expected,
            pearlite! { self.control.val().cur().pointer().addr_logic()@ },
            self.control.val().cur().pointer(),
            self.base.raw_pointer(),
        )
    }

    #[logic(prophetic)]
    fn wellformed(self) -> bool {
        pearlite! {
            self.recovery.invariant() && self.base.invariant() &&
            self.base@ == Some((self.recovery.namespace(), self.recovery.capacity(), 0int)) &&
            self.capacity@ == self.recovery.capacity() && self.len@ == (*self.expected).len() &&
            self.capacity@ > 0 &&
            self.len@ <= self.capacity@ &&
            self.physical_end.lft() == self.physical.val().lft() &&
            self.physical.val().lft() == self.control.val().lft() &&
            ^self.physical_end == self.physical.val().cur() &&
            self.physical.val().cur().invariant() &&
            self.physical.val().cur().namespace() == self.recovery.namespace() &&
            self.physical.val().cur().capacity() == self.recovery.capacity() &&
            self.physical.val().cur().resource_id() == self.recovery.namespace() &&
            self.physical.val().cur().lo() == 0 &&
            self.physical.val().cur().hi() == self.recovery.capacity() &&
            self.control_end.lft() == self.control.val().lft() &&
            ^self.control_end == self.control.val().cur() &&
            self.control.val().cur().wellformed() &&
            self.control.val().cur().owner.val().buf == self.base.raw_pointer() &&
            self.control.val().cur().owner.val().cap@ == self.recovery.capacity() &&
            self.control.val().cur().model() ==
                field_event::atomic_model(&self.control.val().cur().owner.val().ref_cnt) &&
            (forall<i: Int> 0 <= i && i < self.len@ ==>
                self.physical.val().cur().slot(i) == Some(Some((*self.expected)[i])))
        }
    }
}


pub(crate) struct OriginalSharedProof {
    shared:*mut Shared,
    bound:raw_vec::BoundPtr,
    capacity:usize,
    control:Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    physical:Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    invariant:Ghost<GhostShared<field_event::FieldInvariant<lifecycle::State<Payload>>>>,
    ticket:Ghost<lifecycle::Ticket<Payload>>,
    data_binding:Ghost<pointer_event::ReadOnlyPointer>,
}
impl OriginalSharedProof {
    #[logic(prophetic)]
    pub(crate) fn valid_for(self,ptr:*const u8,len:usize,data:creusot_std::std::sync::atomic::AtomicPtr<()>)->bool {
        let p = self;
        pearlite! {
            p.bound.invariant() &&
            p.bound@ == Some((p.invariant.inner_logic().val().public().3.1,
                p.capacity@, 0int)) &&
            p.capacity@ == p.invariant.inner_logic().val().public().3.2 &&
            len@ == p.invariant.inner_logic().val().public().3.3.len() &&
            p.capacity@ > 0 &&
            len@ <= p.capacity@ &&
            ptr == p.bound.raw_pointer() as *const u8 &&
            p.bound.current_address() == ptr.addr_logic()@ &&
            p.data_binding.inner_logic().model() ==
                data &&
            p.data_binding.inner_logic().value() == p.shared as *mut () &&
            p.shared as *const Shared == p.control.inner_logic().val().cur().pointer() &&
            p.invariant.inner_logic().val().public().3.5 == p.shared as *const Shared &&
            p.invariant.inner_logic().val().public().3.6 == p.bound.raw_pointer() &&
            p.invariant.inner_logic().val().public().3.0 ==
                p.invariant.inner_logic().val().public().2 &&
            p.control.inner_logic().val().lft() == p.invariant.inner_logic().val().public().2 &&
            p.physical.inner_logic().val().lft() == p.invariant.inner_logic().val().public().2 &&
            p.control.inner_logic().val().cur().wellformed() &&
            p.control.inner_logic().val().cur().model() == p.invariant.inner_logic().val().model() &&
            p.control.inner_logic().val().cur().owner.val().buf == p.bound.raw_pointer() &&
            p.control.inner_logic().val().cur().owner.val().cap@ == p.capacity@ &&
            p.physical.inner_logic().val().cur().invariant() &&
            p.physical.inner_logic().val().cur().namespace() == p.bound@.unwrap_logic().0 &&
            p.physical.inner_logic().val().cur().capacity() == p.capacity@ &&
            p.physical.inner_logic().val().cur().resource_id() == p.bound@.unwrap_logic().0 &&
            p.physical.inner_logic().val().cur().lo() == 0 &&
            p.physical.inner_logic().val().cur().hi() == p.capacity@ &&
            (forall<i: Int> 0 <= i && i < len@ ==>
                p.physical.inner_logic().val().cur().slot(i) ==
                    Some(Some(p.invariant.inner_logic().val().public().3.3[i]))) &&
            p.ticket.inner_logic().token.lft() == p.invariant.inner_logic().val().public().2 &&
            p.ticket.inner_logic().valid(p.invariant.inner_logic().val().public())
        }
    }

}

#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result.original_bytes_valid())]
#[ensures(result.original_bytes_content() == input@)]
fn original_bytes_shared_from_vec(input:Vec<u8>)->Bytes {
    let mut current=ghost! {SyncView::new().into_inner()};
    let expected = snapshot!(input@);
    let (raw, len, capabilities) = raw_vec::detach_vec(input);
    let (base, cap) = raw.into_bound_ptr_at_zero();
    let ptr = base.as_ptr();
    let (ref_cnt, count_permission) = field_event::new(1, current.borrow_mut());
    let count_view = snapshot!(*current);
    let boxed = Box::new(Shared {
        buf: ptr,
        cap,
        ref_cnt,
    });
    let (shared, shared_owner) = boxed_alignment::into_raw_aligned(boxed);
    let shared_addr = crate::provenance_specs::pointer_addr(shared);
    boxed_alignment::aligned_address_has_clear_low_bit(
        shared_addr,
        core::mem::align_of::<Shared>(),
    );
    debug_assert_eq!(shared_addr & 1, 0);
    let (data, data_permission) = pointer_event::new_pointer(shared.cast::<()>(), current.borrow_mut());
    let data_binding = pointer_event::bind_read_only(&data, shared.cast::<()>(), data_permission);


    let (recovery, region) = capabilities.split();
    let lifetime = ghost! { LifetimeToken::new() };
    let lifetime_id = snapshot!(lifetime.lft());
    let (control_full, control_end) = FullBorrow::new(
        field_event::own_control(shared, shared_owner),
        lifetime_id,
    );
    let (physical_full, physical_end) = FullBorrow::new(region, lifetime_id);
    let control = ghost! { GhostShared::new(control_full).into_inner() };
    let physical = ghost! { GhostShared::new(physical_full).into_inner() };
    let payload = ghost! {
        Payload {
            recovery: recovery.into_inner(),
            physical_end: physical_end.into_inner(),
            control_end: control_end.into_inner(),
            physical,
            control,
            base,
            capacity: cap,
            len,
            expected,
        }
    };
    let initialized = lifecycle::State::<Payload>::initialize(
        count_permission,
        count_view,
        payload,
        lifetime,
    );
    let (state, rest) = initialized.split();
    let ticket = rest;

    // Borrow the typed owner through the exact first B-ticket, then select the
    // native field by ordinary body-proved projection before binding the
    // external protocol descriptor.
    let permission: Ghost<&Perm<*const Shared>> = ghost! {
        let full: &FullBorrow<field_event::OwnedControl<Shared>> = (*control).to_ref();
        let owner = full.borrow(&ticket.token);
        &**owner.owner
    };
    let shared_ref = unsafe { Perm::as_ref(shared, permission) };
    let field_invariant = field_event::FieldInvariant::bind(&shared_ref.ref_cnt, state);
    let invariant = GhostShared::new(field_invariant);

    let vtable = shared_table_reification();
    Bytes { ptr,len,data,vtable,original_bytes:ghost! {
        OriginalBytesProof::Shared(OriginalSharedProof {shared,bound:base,capacity:cap,control,physical,invariant,ticket,data_binding})
    }}
}
