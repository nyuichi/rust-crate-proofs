//! Affine native-reference registrations for a shared control block.
//!
//! The authority map is the exact set of registered live native references.
//! Each `OwnerTicket` owns one exclusive singleton fragment in that map.
//! Mutable entries record the full physical interval held by their packet;
//! readonly entries record only their visible interval. The map count is the
//! native strong-reference count, while the physical pool excludes Mutable
//! intervals only. A caller must connect count-changing transitions to the
//! actual native atomic event and move the corresponding physical authority.

use creusot_std::{
    ghost::resource::{Authority, Fragment},
    logic::{FMap, Id, Int, ra::{RA, UnitRA, auth::CancelLocalUpdateUnit,
        excl::Excl, fmap::FMapKeyLocalUpdate, update::LocalUpdate}},
    prelude::*,
};

use crate::ref_count_limit::MAX_REF_COUNT;

/// Metadata identifying the allocation owned by every registration in one
/// shared control block. This records identity only; it carries no access
/// authority for the allocation.
pub type AllocationBinding = (Id, *mut u8, Int);

/// Classification of one native strong reference. Mutable entries name the
/// full interval held by their `PhysicalRegion`; readonly entries name only
/// the byte interval visible through that immutable handle.
#[derive(creusot_std::prelude::Clone, Copy)]
pub(crate) enum RefKind {
    Mutable,
    Readonly,
}

/// Allocation identity, reference class, and that handle's physical/view span.
/// The binding itself is metadata; a Mutable ticket's `PhysicalRegion` is the
/// affine authority for its interval.
pub(crate) type OwnerBinding = (AllocationBinding, RefKind, Int, Int);

pub(crate) type ActiveReferences = FMap<Int, Excl<OwnerBinding>>;

#[logic(open)]
pub(crate) fn owner_binding_valid(owner: OwnerBinding, allocation: AllocationBinding) -> bool {
    pearlite! {
        owner.0 == allocation && 0 <= owner.2 && owner.2 <= owner.3 &&
        owner.3 <= allocation.2
    }
}

#[logic]
pub(crate) fn active_mutable_contains(owners: ActiveReferences, index: Int) -> bool {
    pearlite! {
        exists<id: Int> match owners.get(id) {
            Some(Excl((_, kind, lo, hi))) => match kind {
                RefKind::Mutable => lo <= index && index < hi,
                RefKind::Readonly => false,
            },
            None => false,
        }
    }
}

/// Describe exactly how removing one registration changes the set of slots
/// covered by live Mutable owners. Disjointness of the owner map ensures that
/// no other registration covers a removed Mutable interval; removing a
/// Readonly registration leaves Mutable coverage unchanged.
#[check(ghost)]
#[requires(owner_ranges_disjoint(*before))]
#[requires(ticket.invariant())]
#[requires((*before).get(ticket.owner_id()) ==
    Some(Excl(ticket.owner_binding())))]
#[requires(*after == (*before).remove(ticket.owner_id()))]
#[ensures(forall<index: Int>
    match ticket.class() {
        RefKind::Mutable => active_mutable_contains(*after, index) ==
            (active_mutable_contains(*before, index) &&
                !(ticket.owned_lo() <= index && index < ticket.owned_hi())),
        RefKind::Readonly => active_mutable_contains(*after, index) ==
            active_mutable_contains(*before, index),
    })]
pub(crate) fn prove_owner_removal_effect(
    before: Snapshot<ActiveReferences>,
    after: Snapshot<ActiveReferences>,
    ticket: Snapshot<OwnerTicket>,
) {
    proof_assert!(forall<index: Int>
        match ticket.class() {
            RefKind::Mutable => active_mutable_contains(*after, index) ==
                (active_mutable_contains(*before, index) &&
                    !(ticket.owned_lo() <= index && index < ticket.owned_hi())),
            RefKind::Readonly => active_mutable_contains(*after, index) ==
                active_mutable_contains(*before, index),
        });
}

/// Replacing one Mutable owner's interval with its two contiguous children
/// preserves the exact set of indices covered by live Mutable owners.
#[check(ghost)]
#[requires((*before).get(*parent_id) ==
    Some(Excl((*binding, RefKind::Mutable, *lo, *hi))))]
#[requires(!(*before).contains(*child_id))]
#[requires(*lo <= *cut && *cut <= *hi)]
#[requires(*after == (*before)
    .insert(*parent_id, Excl((*binding, RefKind::Mutable, *lo, *cut)))
    .insert(*child_id, Excl((*binding, RefKind::Mutable, *cut, *hi))))]
#[ensures(forall<index: Int>
    active_mutable_contains(*after, index) ==
        active_mutable_contains(*before, index))]
pub(crate) fn prove_owner_split_preserves_mutable_coverage(
    before: Snapshot<ActiveReferences>,
    after: Snapshot<ActiveReferences>,
    parent_id: Snapshot<Int>,
    child_id: Snapshot<Int>,
    binding: Snapshot<AllocationBinding>,
    lo: Snapshot<Int>,
    cut: Snapshot<Int>,
    hi: Snapshot<Int>,
) {
    proof_assert!(forall<index: Int>
        active_mutable_contains(*after, index) ==
            active_mutable_contains(*before, index));
}

/// Reclassifying one Mutable owner as Readonly removes exactly that owner's
/// former interval from the set covered by Mutable registrations. The
/// Readonly binding remains in the authoritative map, but does not represent
/// exclusive physical ownership and therefore does not keep bytes out of the
/// physical pool.
#[check(ghost)]
#[requires(owner_ranges_disjoint(*before))]
#[requires(ticket.invariant() && ticket.class() == RefKind::Mutable)]
#[requires((*before).get(ticket.owner_id()) ==
    Some(Excl(ticket.owner_binding())))]
#[requires((*after) == (*before).insert(ticket.owner_id(), Excl((
    ticket.allocation_binding(), RefKind::Readonly, *visible_lo, *visible_hi,
))))]
#[requires(ticket.owned_lo() <= *visible_lo && *visible_lo <= *visible_hi &&
    *visible_hi <= ticket.owned_hi())]
#[ensures(forall<index: Int>
    active_mutable_contains(*after, index) ==
        (active_mutable_contains(*before, index) &&
            !(ticket.owned_lo() <= index && index < ticket.owned_hi())))]
pub(crate) fn prove_owner_reclassification_effect(
    before: Snapshot<ActiveReferences>,
    after: Snapshot<ActiveReferences>,
    ticket: Snapshot<OwnerTicket>,
    visible_lo: Snapshot<Int>,
    visible_hi: Snapshot<Int>,
) {
    proof_assert!(forall<index: Int>
        active_mutable_contains(*after, index) ==
            (active_mutable_contains(*before, index) &&
                !(ticket.owned_lo() <= index && index < ticket.owned_hi())));
}

#[logic]
fn spans_compatible(left: OwnerBinding, right: OwnerBinding) -> bool {
    pearlite! {
        match (left.1, right.1) {
            (RefKind::Readonly, RefKind::Readonly) => true,
            _ => left.2 == left.3 || right.2 == right.3 ||
                left.3 <= right.2 || right.3 <= left.2,
        }
    }
}

#[logic]
fn owner_ranges_valid(owners: ActiveReferences, allocation: AllocationBinding) -> bool {
    pearlite! {
        forall<id: Int> match owners.get(id) {
            Some(Excl(owner)) => owner_binding_valid(owner, allocation),
            None => true,
        }
    }
}

#[logic]
pub(crate) fn owner_ranges_disjoint(owners: ActiveReferences) -> bool {
    pearlite! {
        forall<left_id: Int, right_id: Int> match
            (owners.get(left_id), owners.get(right_id)) {
            (Some(Excl(left)), Some(Excl(right))) =>
                left_id == right_id || spans_compatible(left, right),
            _ => true,
        }
    }
}

#[logic]
fn binding_compatible_with_map(
    binding: OwnerBinding,
    references: ActiveReferences,
    except_id: Int,
) -> bool {
    pearlite! {
        forall<other_id: Int> other_id != except_id ==>
            match references.get(other_id) {
                Some(Excl(other)) => spans_compatible(binding, other),
                None => true,
            }
    }
}

/// Body-proved point update for one exclusive owner payload. `Excl` has no
/// composable frame, so changing this payload cannot disturb another ticket.
pub struct OwnerBindingUpdate(pub Snapshot<OwnerBinding>);

impl LocalUpdate<Excl<OwnerBinding>> for OwnerBindingUpdate {
    #[logic(open, inline)]
    fn premise(self, _from_auth: Excl<OwnerBinding>, _from_frag: Excl<OwnerBinding>) -> bool {
        true
    }

    #[logic(open, inline)]
    fn update(
        self,
        _from_auth: Excl<OwnerBinding>,
        _from_frag: Excl<OwnerBinding>,
    ) -> (Excl<OwnerBinding>, Excl<OwnerBinding>) {
        (Excl(*self.0), Excl(*self.0))
    }

    #[logic]
    fn frame_preserving(
        self,
        from_auth: Excl<OwnerBinding>,
        from_frag: Excl<OwnerBinding>,
        frame: Option<Excl<OwnerBinding>>,
    ) {
        // A nonempty frame cannot compose with Excl; the empty frame is
        // preserved because both updated payloads are equal.
    }
}

/// One non-duplicable registration for a particular native strong reference.
#[allow(missing_debug_implementations)]
pub(crate) struct OwnerTicket {
    ledger_id: Id,
    owner_id: Int,
    fragment: Fragment<ActiveReferences>,
}

impl OwnerTicket {
    /// Reify the exact singleton fragment returned by an authority operation
    /// as an owner ticket. Keeping the real affine fragment as the sole input
    /// makes the ticket invariant a constructor boundary instead of a caller
    /// assertion.
    #[check(ghost)]
    #[requires(fragment.id() == ledger_id)]
    #[requires(fragment@ == FMap::singleton(owner_id, Excl(binding)))]
    #[requires(fragment@.get(owner_id) == Some(Excl(binding)))]
    #[ensures(result.invariant())]
    #[ensures(result.ledger_id() == ledger_id)]
    #[ensures(result.owner_id() == owner_id)]
    #[ensures(result.fragment_resource_id() == ledger_id)]
    #[ensures(result.owner_binding() == binding)]
    #[ensures(result.allocation_binding() == binding.0)]
    #[ensures(result.owned_lo() == binding.2)]
    #[ensures(result.owned_hi() == binding.3)]
    #[ensures(result.fragment_map() == FMap::singleton(owner_id, Excl(binding)))]
    pub(crate) fn from_fragment(
        ledger_id: Id,
        owner_id: Int,
        binding: OwnerBinding,
        fragment: Fragment<ActiveReferences>,
    ) -> OwnerTicket {
        let ticket = OwnerTicket {
            ledger_id,
            owner_id,
            fragment,
        };
        proof_assert!(ticket.invariant());
        ticket
    }

    #[logic(open(self))]
    pub(crate) fn ledger_id(self) -> Id {
        self.ledger_id
    }

    #[logic(open(self))]
    pub(crate) fn owner_id(&self) -> Int {
        self.owner_id
    }

    #[logic(open(self))]
    pub(crate) fn fragment_resource_id(self) -> Id {
        self.fragment.id()
    }

    /// The actual singleton ticket fragment uses this ticket's ledger ID as
    /// its resource identity. This derives a reusable field relation from
    /// the ticket's own invariant; it does not create or duplicate a ticket.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(self.fragment_resource_id() == self.ledger_id())]
    pub(crate) fn prove_fragment_resource_matches_ledger(&self) {
        proof_assert!(self.fragment_resource_id() == self.ledger_id());
    }

    #[logic(open(self))]
    pub(crate) fn fragment_map(self) -> ActiveReferences {
        pearlite! { self.fragment@ }
    }

    /// Allocation metadata stored in this ticket's actual exclusive
    /// singleton fragment.
    #[logic(open(self))]
    pub(crate) fn allocation_binding(&self) -> AllocationBinding {
        pearlite! { self.owner_binding().0 }
    }

    #[logic(open(self))]
    pub(crate) fn class(&self) -> RefKind {
        pearlite! { self.owner_binding().1 }
    }

    #[logic(open(self))]
    pub(crate) fn owner_binding(&self) -> OwnerBinding {
        pearlite! { self.fragment@.get(self.owner_id).unwrap_logic().0 }
    }

    #[logic(open(self))]
    pub(crate) fn owned_lo(&self) -> Int {
        pearlite! { self.owner_binding().2 }
    }

    #[logic(open(self))]
    pub(crate) fn owned_hi(&self) -> Int {
        pearlite! { self.owner_binding().3 }
    }

    /// Replace this ticket's exact singleton payload in both its affine
    /// fragment and the matching authority. Keeping the operation here lets
    /// the ticket's private singleton invariant be proved at the update
    /// boundary instead of at each ledger transition.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(authority.invariant())]
    #[requires(authority.id() == self.ledger_id())]
    #[requires(authority@.get(self.owner_id()) ==
        Some(Excl(self.owner_binding())))]
    #[requires((*new_binding).0 == self.allocation_binding())]
    #[requires(0 <= (*new_binding).2 && (*new_binding).2 <= (*new_binding).3)]
    #[requires((*new_binding).3 <= self.allocation_binding().2)]
    #[ensures((^self).invariant())]
    #[ensures((^self).owner_id() == self.owner_id())]
    #[ensures((^self).ledger_id() == self.ledger_id())]
    #[ensures((^self).fragment_resource_id() == self.fragment_resource_id())]
    #[ensures((^self).owner_binding() == *new_binding)]
    #[ensures((^self).fragment_map() ==
        FMap::singleton(self.owner_id(), Excl(*new_binding)))]
    #[ensures((^authority).id() == authority.id())]
    #[ensures((^authority)@ == authority@.insert(
        self.owner_id(), Excl(*new_binding)))]
    #[ensures((^authority)@.get(self.owner_id()) ==
        Some(Excl(*new_binding)))]
    pub(crate) fn replace_binding(
        &mut self,
        authority: &mut Authority<ActiveReferences>,
        new_binding: Snapshot<OwnerBinding>,
    ) -> Ghost<()> {
        let owner_id: Snapshot<Int> = snapshot!(self.owner_id);
        let old_fragment: Snapshot<ActiveReferences> = snapshot!(self.fragment@);
        normalize_singleton_insert(
            old_fragment,
            owner_id,
            snapshot!(*new_binding),
        );
        authority.update(
            &mut self.fragment,
            FMapKeyLocalUpdate(
                owner_id,
                OwnerBindingUpdate(new_binding),
            ),
        );
        ghost!(())
    }

    #[logic(prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.fragment.id() == self.ledger_id &&
            self.fragment@ == FMap::singleton(
                self.owner_id,
                Excl(self.owner_binding())
            )
        }
    }
}

impl Invariant for OwnerTicket {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

/// An affine proof that the reference map is empty after the final release.
#[allow(missing_debug_implementations)]
pub(crate) struct RetirePermit {
    ledger_id: Id,
    authority: Authority<ActiveReferences>,
}

impl RetirePermit {
    #[logic(open(self))]
    pub(crate) fn ledger_id(self) -> Id {
        self.ledger_id
    }

    #[logic(open(self))]
    pub(crate) fn active_ids(self) -> ActiveReferences {
        pearlite! { self.authority@ }
    }

    #[logic(prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.authority.invariant() &&
            self.authority.id() == self.ledger_id &&
            self.authority@.is_empty()
        }
    }

    /// A same-ledger, valid reference ticket would have to compose with this empty
    /// authority. The exclusive singleton fragment makes that impossible.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(ticket.invariant())]
    #[requires(ticket.ledger_id() == self.ledger_id())]
    #[ensures(false)]
    pub(crate) fn contradicts_live_ticket(&self, ticket: &OwnerTicket) {
        self.authority.frag_lemma(&ticket.fragment);
        let fragment: Snapshot<ActiveReferences> = snapshot!(ticket.fragment@);
        let empty: Snapshot<ActiveReferences> = snapshot!(self.authority@);
        proof_assert!((*fragment).incl(*empty));
        proof_assert!((*fragment).get(ticket.owner_id()) ==
            Some(Excl(ticket.owner_binding())));
        proof_assert!((*empty).get(ticket.owner_id()) == None);
        proof_assert!((*fragment).get(ticket.owner_id()).incl(
            (*empty).get(ticket.owner_id())
        ));
        proof_assert!(false);
    }
}

impl Invariant for RetirePermit {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

/// Current affine inventory for every native strong reference to one shared
/// control block.
///
/// `ledger_id` is the stable Resource identity. Moving the empty authority
/// into a `RetirePermit` never changes that identity.
#[allow(missing_debug_implementations)]
pub(crate) struct OwnerLedger {
    ledger_id: Id,
    next_id: Int,
    live: bool,
    allocation_binding: AllocationBinding,
    authority: Option<Authority<ActiveReferences>>,
}

impl OwnerLedger {
    #[logic(open(self))]
    pub(crate) fn ledger_id(self) -> Id {
        self.ledger_id
    }

    #[logic(open(self))]
    pub(crate) fn next_id(self) -> Int {
        self.next_id
    }

    #[logic(open(self))]
    pub(crate) fn is_live(self) -> bool {
        self.live
    }

    #[logic(open(self))]
    pub(crate) fn allocation_binding(self) -> AllocationBinding {
        self.allocation_binding
    }

    #[logic(open(self))]
    pub(crate) fn active_ids(self) -> ActiveReferences {
        pearlite! { match self.authority {
            Some(authority) => authority@,
            None => FMap::empty(),
        } }
    }

    /// The logical native strong-reference count is exactly the cardinality
    /// of the authoritative map; no separately mutable count is stored here.
    #[logic(open(self))]
    pub(crate) fn count(self) -> Int {
        self.active_ids().len()
    }

    /// Relation consumed by the enclosing atomic protocol.
    #[logic(opaque, prophetic)]
    #[ensures(result == (self.invariant() && self.count() == value))]
    pub(crate) fn agrees_with_value(self, value: Int) -> bool {
        self.invariant() && self.count() == value
    }

    #[logic(prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            0 <= self.next_id &&
            match (self.live, self.authority) {
                (true, Some(authority)) =>
                    authority.invariant() && authority.id() == self.ledger_id &&
                    authority@.len() > 0 &&
                    owner_ranges_valid(authority@, self.allocation_binding) &&
                    owner_ranges_disjoint(authority@) &&
                    forall<id: Int> authority@.contains(id) ==>
                        0 <= id && id < self.next_id &&
                        match authority@.get(id) {
                            Some(Excl(owner)) => owner.0 == self.allocation_binding,
                            None => false,
                        },
                (false, None) => true,
                _ => false,
            }
        }
    }

    #[logic(prophetic)]
    pub(crate) fn has_ticket(self, ticket: OwnerTicket) -> bool {
        pearlite! {
            self.invariant() && self.live && ticket.invariant() &&
            ticket.ledger_id() == self.ledger_id &&
            ticket.allocation_binding() == self.allocation_binding &&
            self.active_ids().get(ticket.owner_id()) ==
                Some(Excl(ticket.owner_binding()))
        }
    }

    /// Prove membership from the affine fragment and the authority's resource
    /// identity. The caller never supplies an arbitrary active-ID predicate.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(self.is_live())]
    #[requires(ticket.invariant())]
    #[requires(ticket.ledger_id() == self.ledger_id())]
    #[ensures(self.has_ticket(*ticket))]
    #[ensures(ticket.owner_id() < self.next_id())]
    #[ensures(ticket.fragment_resource_id() == self.ledger_id())]
    #[ensures(ticket.allocation_binding() == self.allocation_binding())]
    #[ensures(self.active_ids().get(ticket.owner_id()) ==
        Some(Excl(ticket.owner_binding())))]
    #[ensures(self.active_ids().get(ticket.owner_id()) == Some(Excl((
        ticket.allocation_binding(), ticket.class(), ticket.owned_lo(), ticket.owned_hi(),
    ))))]
    #[ensures(ticket.owner_binding() ==
        self.active_ids().get(ticket.owner_id()).unwrap_logic().0)]
    #[ensures(owner_binding_valid(ticket.owner_binding(), self.allocation_binding()))]
    #[ensures(0 <= ticket.owned_lo())]
    #[ensures(ticket.owned_lo() <= ticket.owned_hi())]
    #[ensures(ticket.owned_hi() <= self.allocation_binding().2)]
    #[ensures(owner_ranges_disjoint(self.active_ids()))]
    pub(crate) fn prove_ticket_is_active(&self, ticket: &OwnerTicket) {
        match &self.authority {
            Some(authority) => {
                authority.frag_lemma(&ticket.fragment);
                let fragment: Snapshot<ActiveReferences> = snapshot!(ticket.fragment@);
                let active: Snapshot<ActiveReferences> = snapshot!(authority@);
                proof_assert!((*fragment).incl(*active));
                proof_assert!((*fragment).get(ticket.owner_id()) ==
                    Some(Excl(ticket.owner_binding())));
                proof_assert!((*active).get(ticket.owner_id()) ==
                    Some(Excl(ticket.owner_binding())));
                proof_assert!((*active).get(ticket.owner_id()) == Some(Excl((
                    ticket.allocation_binding(), ticket.class(),
                    ticket.owned_lo(), ticket.owned_hi(),
                ))));
                proof_assert!(ticket.owner_id() < self.next_id());
                proof_assert!(ticket.allocation_binding() == self.allocation_binding());
            }
            None => proof_assert!(false),
        }
    }

    /// A visible subspan of a live Mutable owner can be reclassified as a
    /// Readonly view without conflicting with any other registered owner.
    /// The old Mutable interval is disjoint from every other entry by the
    /// ledger invariant; shrinking that interval and changing its class to
    /// Readonly preserves compatibility with both Mutable and Readonly peers.
    #[check(ghost)]
    #[requires(self.invariant() && self.is_live())]
    #[requires(ticket.invariant() && ticket.class() == RefKind::Mutable)]
    #[requires(ticket.ledger_id() == self.ledger_id())]
    #[requires(self.has_ticket(*ticket))]
    #[requires(ticket.owned_lo() <= *visible_lo &&
        *visible_lo <= *visible_hi && *visible_hi <= ticket.owned_hi())]
    #[ensures(owner_ranges_disjoint(self.active_ids()))]
    #[ensures(self.active_ids().get(ticket.owner_id()) ==
        Some(Excl(ticket.owner_binding())))]
    #[ensures(owner_binding_valid((
        ticket.allocation_binding(), RefKind::Readonly, *visible_lo, *visible_hi,
    ), self.allocation_binding()))]
    #[ensures(binding_compatible_with_map((
        ticket.allocation_binding(), RefKind::Readonly, *visible_lo, *visible_hi,
    ), self.active_ids(), ticket.owner_id()))]
    pub(crate) fn prove_readonly_binding_compatible(
        &self,
        ticket: &OwnerTicket,
        visible_lo: Snapshot<Int>,
        visible_hi: Snapshot<Int>,
    ) {
        self.prove_ticket_is_active(ticket);
        proof_assert!(owner_ranges_disjoint(self.active_ids()));
        proof_assert!(self.active_ids().get(ticket.owner_id()) ==
            Some(Excl(ticket.owner_binding())));
        proof_assert!(owner_binding_valid((
            ticket.allocation_binding(), RefKind::Readonly, *visible_lo, *visible_hi,
        ), self.allocation_binding()));
        proof_assert!(binding_compatible_with_map((
            ticket.allocation_binding(), RefKind::Readonly, *visible_lo, *visible_hi,
        ), self.active_ids(), ticket.owner_id()));
    }

    /// A Mutable ticket's interval is represented in the live authority map.
    /// This body lemma packages the existential witness needed by physical
    /// pool proofs before the affine ticket is consumed.
    #[check(ghost)]
    #[requires(self.has_ticket(*ticket))]
    #[requires(ticket.class() == RefKind::Mutable)]
    #[ensures(forall<index: Int>
        ticket.owned_lo() <= index && index < ticket.owned_hi() ==>
            active_mutable_contains(self.active_ids(), index))]
    pub(crate) fn prove_mutable_ticket_covers_interval(&self, ticket: &OwnerTicket) {
        self.prove_ticket_is_active(ticket);
        proof_assert!(forall<index: Int>
            ticket.owned_lo() <= index && index < ticket.owned_hi() ==>
                active_mutable_contains(self.active_ids(), index));
    }

    /// An actual Readonly ticket excludes Mutable coverage throughout its
    /// recorded visible interval. The ticket's own map entry is Readonly, and
    /// every distinct Mutable entry is disjoint from that span.
    #[check(ghost)]
    #[requires(self.invariant() && self.is_live())]
    #[requires(ticket.invariant() && ticket.class() == RefKind::Readonly)]
    #[requires(ticket.ledger_id() == self.ledger_id())]
    #[requires(self.has_ticket(*ticket))]
    #[ensures(forall<index: Int>
        ticket.owned_lo() <= index && index < ticket.owned_hi() ==>
            !active_mutable_contains(self.active_ids(), index))]
    pub(crate) fn prove_readonly_ticket_excluded_from_mutable_coverage(
        &self,
        ticket: &OwnerTicket,
    ) {
        self.prove_ticket_is_active(ticket);
        proof_assert!(owner_ranges_disjoint(self.active_ids()));
        proof_assert!(self.active_ids().get(ticket.owner_id()) ==
            Some(Excl(ticket.owner_binding())));
        proof_assert!(forall<index: Int>
            ticket.owned_lo() <= index && index < ticket.owned_hi() ==>
                !active_mutable_contains(self.active_ids(), index));
    }

    /// A currently registered Readonly span cannot overlap any active
    /// Mutable owner. This projection is keyed by the authoritative map entry
    /// so callers restoring a physical lend need not duplicate or mint the
    /// affine owner ticket.
    #[check(ghost)]
    #[requires(self.invariant() && self.is_live())]
    #[requires(self.active_ids().get(owner_id) == Some(Excl((
        self.allocation_binding(), RefKind::Readonly, lo, hi,
    ))))]
    #[ensures(forall<index: Int> lo <= index && index < hi ==>
        !active_mutable_contains(self.active_ids(), index))]
    pub(crate) fn prove_readonly_owner_interval_excluded_from_mutable_coverage(
        &self,
        owner_id: Int,
        lo: Int,
        hi: Int,
    ) {
        proof_assert!(owner_ranges_disjoint(self.active_ids()));
        proof_assert!(forall<index: Int> lo <= index && index < hi ==>
            !active_mutable_contains(self.active_ids(), index));
    }

    /// Construct a count-one shared block and its sole root ticket.
    #[check(ghost)]
    #[ensures(result.inner_logic().0.invariant())]
    #[ensures(result.inner_logic().0.is_live())]
    #[ensures(result.inner_logic().0.allocation_binding() == binding.inner_logic())]
    #[ensures(result.inner_logic().0.count() == 1)]
    #[ensures(result.inner_logic().0.next_id() == 1)]
    #[ensures(result.inner_logic().0.agrees_with_value(1))]
    #[ensures(result.inner_logic().1.invariant())]
    #[ensures(result.inner_logic().0.has_ticket(result.inner_logic().1))]
    #[ensures(result.inner_logic().1.ledger_id() == result.inner_logic().0.ledger_id())]
    #[ensures(result.inner_logic().1.owner_id() == 0)]
    #[ensures(result.inner_logic().1.owner_id() >= 0)]
    #[ensures(result.inner_logic().1.owner_id() < result.inner_logic().0.next_id())]
    #[ensures(result.inner_logic().1.allocation_binding() == binding.inner_logic())]
    #[ensures(result.inner_logic().1.class() == RefKind::Mutable)]
    #[ensures(result.inner_logic().1.owned_lo() == 0)]
    #[ensures(result.inner_logic().1.owned_hi() == binding.inner_logic().2)]
    #[requires(binding.inner_logic().2 >= 0)]
    pub(crate) fn new_root(
        binding: Ghost<AllocationBinding>,
    ) -> Ghost<(OwnerLedger, OwnerTicket)> {
        ghost! {
        let binding = binding.into_inner();
        let mut authority: Authority<ActiveReferences> = Authority::alloc().into_inner();
        let empty_map: Snapshot<ActiveReferences> = snapshot!(ActiveReferences::unit());
        proof_assert!(authority@ == *empty_map);
        let root_id: Int = *Int::new(0i128);
        let root_owner: OwnerBinding = (binding, RefKind::Mutable, 0int, binding.2);
        let root_map: Snapshot<ActiveReferences> =
            snapshot!(FMap::singleton(root_id, Excl(root_owner)));
        let root_fragment = authority.add_fragment(root_map);
        proof_assert!(authority@.ext_eq(*root_map));
        let ledger_id = authority.id_ghost();
        let ledger = OwnerLedger {
            ledger_id,
            next_id: *Int::new(1i128),
            live: true,
            allocation_binding: binding,
            authority: Some(authority),
        };
        let ticket = OwnerTicket::from_fragment(
            ledger_id,
            root_id,
            root_owner,
            root_fragment,
        );
        proof_assert!(ledger.invariant());
        proof_assert!(ticket.invariant());
        ledger.prove_ticket_is_active(&ticket);
        (ledger, ticket)
        }
    }

    /// Construct the two registrations used when a Vec is first promoted with
    /// two live handles.
    #[check(ghost)]
    #[ensures(result.inner_logic().0.invariant())]
    #[ensures(result.inner_logic().0.is_live())]
    #[ensures(result.inner_logic().0.next_id() == 2)]
    #[ensures(result.inner_logic().0.allocation_binding() == binding.inner_logic())]
    #[ensures(result.inner_logic().0.count() == 2)]
    #[ensures(result.inner_logic().0.agrees_with_value(2))]
    #[ensures(forall<index: Int>
        active_mutable_contains(result.inner_logic().0.active_ids(), index) ==
            (0 <= index && index < binding.inner_logic().2))]
    #[ensures(result.inner_logic().1.invariant() && result.inner_logic().2.invariant())]
    #[ensures(result.inner_logic().0.has_ticket(result.inner_logic().1) &&
        result.inner_logic().0.has_ticket(result.inner_logic().2))]
    #[ensures(result.inner_logic().1.ledger_id() == result.inner_logic().0.ledger_id() &&
        result.inner_logic().2.ledger_id() == result.inner_logic().0.ledger_id())]
    #[ensures(result.inner_logic().1.allocation_binding() == binding.inner_logic() &&
        result.inner_logic().2.allocation_binding() == binding.inner_logic())]
    #[ensures(result.inner_logic().1.class() == RefKind::Mutable &&
        result.inner_logic().2.class() == RefKind::Mutable)]
    #[ensures(result.inner_logic().1.owner_id() == 0 && result.inner_logic().2.owner_id() == 1)]
    #[ensures(result.inner_logic().1.owner_id() < result.inner_logic().0.next_id() &&
        result.inner_logic().2.owner_id() < result.inner_logic().0.next_id())]
    #[ensures(result.inner_logic().1.owned_lo() == 0 &&
        result.inner_logic().1.owned_hi() == *cut)]
    #[ensures(result.inner_logic().2.owned_lo() == *cut &&
        result.inner_logic().2.owned_hi() == binding.inner_logic().2)]
    #[ensures(forall<owner_id: Int> result.inner_logic().0.active_ids().get(owner_id) ==
        if owner_id == 0 {
            Some(Excl((binding.inner_logic(), RefKind::Mutable, 0int, *cut)))
        } else if owner_id == 1 {
            Some(Excl((binding.inner_logic(), RefKind::Mutable, *cut,
                binding.inner_logic().2)))
        } else {
            None
        })]
    #[requires(0 <= *cut && *cut <= binding.inner_logic().2)]
    pub(crate) fn new_promoted_pair(
        binding: Ghost<AllocationBinding>,
        cut: Snapshot<Int>,
    ) -> Ghost<(OwnerLedger, OwnerTicket, OwnerTicket)> {
        ghost! {
        let binding = binding.into_inner();
        let cut = cut.into_ghost().into_inner();
        let mut authority: Authority<ActiveReferences> = Authority::alloc().into_inner();
        let empty_map: Snapshot<ActiveReferences> = snapshot!(ActiveReferences::unit());
        proof_assert!(authority@ == *empty_map);
        let first_id: Int = *Int::new(0i128);
        let second_id: Int = *Int::new(1i128);
        let first_owner: OwnerBinding = (binding, RefKind::Mutable, 0int, cut);
        let second_owner: OwnerBinding = (binding, RefKind::Mutable, cut, binding.2);
        let first_map: Snapshot<ActiveReferences> =
            snapshot!(FMap::singleton(first_id, Excl(first_owner)));
        let first_fragment = authority.add_fragment(first_map);
        proof_assert!(authority@.ext_eq(*first_map));
        let after_first: Snapshot<ActiveReferences> = snapshot!(authority@);
        proof_assert!((*after_first).contains(second_id) == false);
        let second_map: Snapshot<ActiveReferences> =
            snapshot!(FMap::singleton(second_id, Excl(second_owner)));
        map_op_fresh_singleton(after_first, snapshot!(second_id), snapshot!(second_owner));
        let second_fragment = authority.add_fragment(second_map);
        let pair_map: Snapshot<ActiveReferences> =
            snapshot!((*first_map).insert(second_id, Excl(second_owner)));
        proof_assert!(authority@.ext_eq(*pair_map));
        let ledger_id = authority.id_ghost();
        let ledger = OwnerLedger {
            ledger_id,
            next_id: *Int::new(2i128),
            live: true,
            allocation_binding: binding,
            authority: Some(authority),
        };
        proof_assert!(forall<index: Int>
            active_mutable_contains(ledger.active_ids(), index) ==
                (0 <= index && index < binding.2)
        );
        let first = OwnerTicket::from_fragment(
            ledger_id,
            first_id,
            first_owner,
            first_fragment,
        );
        let second = OwnerTicket::from_fragment(
            ledger_id,
            second_id,
            second_owner,
            second_fragment,
        );
        proof_assert!(ledger.invariant());
        proof_assert!(first.invariant() && second.invariant());
        ledger.prove_ticket_is_active(&first);
        ledger.prove_ticket_is_active(&second);
        proof_assert!(forall<owner_id: Int> ledger.active_ids().get(owner_id) ==
            if owner_id == first_id {
                Some(Excl(first_owner))
            } else if owner_id == second_id {
                Some(Excl(second_owner))
            } else {
                None
            });
        (ledger, first, second)
        }
    }

    /// Register a Mutable child only after the caller has established a
    /// successful native increment. A failed/refused update has no child;
    /// the native guard permits the final transition from MAX_REF_COUNT to
    /// MAX_REF_COUNT + 1.
    #[check(ghost)]
    #[requires(ledger.inner_logic().invariant() && ledger.inner_logic().is_live())]
    #[requires(ledger.inner_logic().count() <= MAX_REF_COUNT@)]
    #[requires(parent.inner_logic().invariant())]
    #[requires(parent.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[requires(parent.inner_logic().class() == RefKind::Mutable)]
    #[requires(parent.inner_logic().owned_lo() <= *cut &&
        *cut <= parent.inner_logic().owned_hi())]
    #[ensures((^ledger.inner_logic()).invariant() && (^ledger.inner_logic()).is_live())]
    #[ensures((^ledger.inner_logic()).ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).allocation_binding() ==
        ledger.inner_logic().allocation_binding())]
    #[ensures((^ledger.inner_logic()).count() == ledger.inner_logic().count() + 1)]
    #[ensures((^ledger.inner_logic()).count() <= MAX_REF_COUNT@ + 1)]
    #[ensures((^ledger.inner_logic()).agrees_with_value(ledger.inner_logic().count() + 1))]
    #[ensures((^ledger.inner_logic()).active_ids() ==
        ledger.inner_logic().active_ids()
            .insert(parent.inner_logic().owner_id(), Excl((
                parent.inner_logic().allocation_binding(), RefKind::Mutable,
                parent.inner_logic().owned_lo(), *cut,
            )))
            .insert(ledger.inner_logic().next_id(), Excl((
                parent.inner_logic().allocation_binding(), RefKind::Mutable,
                *cut, parent.inner_logic().owned_hi(),
            ))))]
    #[ensures(forall<index: Int>
        active_mutable_contains((^ledger.inner_logic()).active_ids(), index) ==
            active_mutable_contains(ledger.inner_logic().active_ids(), index))]
    #[ensures((^ledger.inner_logic()).next_id() == ledger.inner_logic().next_id() + 1)]
    #[ensures((^ledger.inner_logic()).has_ticket((^parent.inner_logic())))]
    #[ensures((^parent.inner_logic()).allocation_binding() ==
        parent.inner_logic().allocation_binding())]
    #[ensures(result.inner_logic().owner_id() != parent.inner_logic().owner_id())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures(result.inner_logic().owner_id() == ledger.inner_logic().next_id())]
    #[ensures(result.inner_logic().fragment_resource_id() == ledger.inner_logic().ledger_id())]
    #[ensures(result.inner_logic().fragment_resource_id() ==
        parent.inner_logic().fragment_resource_id())]
    #[ensures(result.inner_logic().allocation_binding() ==
        parent.inner_logic().allocation_binding())]
    #[ensures(result.inner_logic().class() == RefKind::Mutable)]
    #[ensures((^parent.inner_logic()).class() == RefKind::Mutable)]
    #[ensures((^parent.inner_logic()).ledger_id() == parent.inner_logic().ledger_id())]
    #[ensures((^parent.inner_logic()).owner_id() == parent.inner_logic().owner_id())]
    #[ensures((^parent.inner_logic()).fragment_resource_id() ==
        parent.inner_logic().fragment_resource_id())]
    #[ensures(result.inner_logic().owned_lo() == *cut)]
    #[ensures(result.inner_logic().owned_hi() == parent.inner_logic().owned_hi())]
    #[ensures((^parent.inner_logic()).owned_lo() == parent.inner_logic().owned_lo())]
    #[ensures((^parent.inner_logic()).owned_hi() == *cut)]
    #[ensures(result.inner_logic().fragment_map() ==
        FMap::singleton(ledger.inner_logic().next_id(),
            Excl((parent.inner_logic().allocation_binding(), RefKind::Mutable, *cut,
                parent.inner_logic().owned_hi()))))]
    #[ensures((^ledger.inner_logic()).has_ticket(result.inner_logic()))]
    pub(crate) fn split_owner(
        ledger: Ghost<&mut OwnerLedger>,
        parent: Ghost<&mut OwnerTicket>,
        cut: Snapshot<Int>,
    ) -> Ghost<OwnerTicket> {
        ghost! {
            let ledger = ledger.into_inner();
            let parent = parent.into_inner();
            let cut = cut.into_ghost().into_inner();
            ledger.prove_ticket_is_active(parent);
            let old_map: Snapshot<ActiveReferences> = snapshot!(ledger.active_ids());
            let new_id = ledger.next_id;
            let parent_id = parent.owner_id;
            let binding_snapshot: Snapshot<AllocationBinding> = snapshot!(
                parent.fragment@.get(parent.owner_id).unwrap_logic().0.0
            );
            let lo_snapshot: Snapshot<Int> = snapshot!(
                parent.fragment@.get(parent.owner_id).unwrap_logic().0.2
            );
            let hi_snapshot: Snapshot<Int> = snapshot!(
                parent.fragment@.get(parent.owner_id).unwrap_logic().0.3
            );
            let binding = binding_snapshot.into_ghost().into_inner();
            let lo = lo_snapshot.into_ghost().into_inner();
            let hi = hi_snapshot.into_ghost().into_inner();
            let left_owner: OwnerBinding = (binding, RefKind::Mutable, lo, cut);
            let right_owner: OwnerBinding = (binding, RefKind::Mutable, cut, hi);
            proof_assert!(!(*old_map).contains(new_id));
            parent.replace_binding(
                ledger.authority.as_mut().unwrap(),
                snapshot!(left_owner),
            );
            let after_parent: Snapshot<ActiveReferences> = snapshot!(ledger.active_ids());
            map_op_fresh_singleton(after_parent, snapshot!(new_id), snapshot!(right_owner));
            normalize_singleton_insert(
                snapshot!(FMap::empty()),
                snapshot!(new_id),
                snapshot!(right_owner),
            );
            let new_fragment = ledger.authority.as_mut().unwrap().add_fragment(
                snapshot!(FMap::singleton(new_id, Excl(right_owner))),
            );
            ledger.next_id = ledger.next_id + 1int;
            let child = OwnerTicket::from_fragment(
                ledger.ledger_id,
                new_id,
                right_owner,
                new_fragment,
            );
            proof_assert!(child.invariant());
            ledger.prove_ticket_is_active(parent);
            proof_assert!(ledger.has_ticket(*parent));
            ledger.prove_ticket_is_active(&child);
            let after_map: Snapshot<ActiveReferences> = snapshot!(ledger.active_ids());
            proof_assert!(*after_map == (*old_map)
                .insert(parent_id, Excl(left_owner))
                .insert(new_id, Excl(right_owner)));
            prove_owner_split_preserves_mutable_coverage(
                old_map,
                after_map,
                snapshot!(parent_id),
                snapshot!(new_id),
                binding_snapshot,
                lo_snapshot,
                snapshot!(cut),
                hi_snapshot,
            );
            child
        }
    }

    /// Register one readonly clone after its native counter increment
    /// succeeds. Readonly views may overlap other readonly views. Because the
    /// source view is already disjoint from every Mutable span, the new entry
    /// remains disjoint from all mutable physical owners as well.
    #[check(ghost)]
    #[requires(ledger.inner_logic().invariant() && ledger.inner_logic().is_live())]
    #[requires(ledger.inner_logic().count() <= MAX_REF_COUNT@)]
    #[requires(source.inner_logic().invariant())]
    #[requires(source.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[requires(source.inner_logic().class() == RefKind::Readonly)]
    #[requires(ledger.inner_logic().has_ticket(*source.inner_logic()))]
    #[ensures((^ledger.inner_logic()).invariant() && (^ledger.inner_logic()).is_live())]
    #[ensures((^ledger.inner_logic()).ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).allocation_binding() ==
        ledger.inner_logic().allocation_binding())]
    #[ensures((^ledger.inner_logic()).count() == ledger.inner_logic().count() + 1)]
    #[ensures((^ledger.inner_logic()).count() <= MAX_REF_COUNT@ + 1)]
    #[ensures((^ledger.inner_logic()).agrees_with_value(ledger.inner_logic().count() + 1))]
    #[ensures((^ledger.inner_logic()).active_ids() ==
        ledger.inner_logic().active_ids().insert(ledger.inner_logic().next_id(), Excl((
            ledger.inner_logic().allocation_binding(), RefKind::Readonly,
            source.inner_logic().owned_lo(), source.inner_logic().owned_hi(),
        ))))]
    #[ensures((^ledger.inner_logic()).next_id() == ledger.inner_logic().next_id() + 1)]
    #[ensures((^ledger.inner_logic()).has_ticket(result.inner_logic()))]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures(result.inner_logic().owner_id() == ledger.inner_logic().next_id())]
    #[ensures(result.inner_logic().class() == RefKind::Readonly)]
    #[ensures(result.inner_logic().allocation_binding() ==
        ledger.inner_logic().allocation_binding())]
    #[ensures(result.inner_logic().owner_binding() == (
        ledger.inner_logic().allocation_binding(), RefKind::Readonly,
        source.inner_logic().owned_lo(), source.inner_logic().owned_hi()))]
    #[ensures(result.inner_logic().owned_lo() == source.inner_logic().owned_lo())]
    #[ensures(result.inner_logic().owned_hi() == source.inner_logic().owned_hi())]
    pub(crate) fn register_readonly(
        ledger: Ghost<&mut OwnerLedger>,
        source: Ghost<&OwnerTicket>,
    ) -> Ghost<OwnerTicket> {
        ghost! {
            let ledger = ledger.into_inner();
            let source = source.into_inner();
            ledger.prove_ticket_is_active(source);
            let old_map: Snapshot<ActiveReferences> = snapshot!(ledger.active_ids());
            let ref_id = ledger.next_id;
            let lo_snapshot: Snapshot<Int> = snapshot!(
                source.fragment@.get(source.owner_id).unwrap_logic().0.2
            );
            let hi_snapshot: Snapshot<Int> = snapshot!(
                source.fragment@.get(source.owner_id).unwrap_logic().0.3
            );
            let lo = lo_snapshot.into_ghost().into_inner();
            let hi = hi_snapshot.into_ghost().into_inner();
            let binding: OwnerBinding = (
                ledger.allocation_binding,
                RefKind::Readonly,
                lo,
                hi,
            );
            proof_assert!(!(*old_map).contains(ref_id));
            proof_assert!(binding_compatible_with_map(binding, *old_map, source.owner_id));
            map_op_fresh_singleton(old_map, snapshot!(ref_id), snapshot!(binding));
            let fragment = ledger.authority.as_mut().unwrap().add_fragment(
                snapshot!(FMap::singleton(ref_id, Excl(binding))),
            );
            ledger.next_id = ledger.next_id + 1int;
            let ticket = OwnerTicket::from_fragment(ledger.ledger_id, ref_id, binding, fragment);
            proof_assert!(ticket.invariant());
            ledger.prove_ticket_is_active(&ticket);
            ticket
        }
    }

    /// Reclassify one existing native reference without changing its strong
    /// count or ID. Mutable-to-readonly freezes a visible subspan and returns
    /// the remainder of its former physical region to the pool. Readonly-to-
    /// mutable reclassification is allowed only for the sole reference and
    /// may install any compatible Mutable span. The caller separately recovers
    /// physical authority from the pool; for a nonzero-offset view it may
    /// recover the full allocation and return the hidden prefix to the pool.
    #[check(ghost)]
    #[requires(ledger.inner_logic().invariant() && ledger.inner_logic().is_live())]
    #[requires(ticket.inner_logic().invariant())]
    #[requires(ticket.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[requires(ledger.inner_logic().has_ticket(*ticket.inner_logic()))]
    #[requires((*new_binding).0 == ledger.inner_logic().allocation_binding())]
    #[requires(owner_binding_valid(*new_binding, ledger.inner_logic().allocation_binding()))]
    #[requires(binding_compatible_with_map(*new_binding,
        ledger.inner_logic().active_ids(), ticket.inner_logic().owner_id()))]
    #[requires(match (ticket.inner_logic().class(), (*new_binding).1) {
        (RefKind::Mutable, RefKind::Readonly) =>
            ticket.inner_logic().owned_lo() <= (*new_binding).2 &&
            (*new_binding).2 <= (*new_binding).3 &&
            (*new_binding).3 <= ticket.inner_logic().owned_hi(),
        (RefKind::Readonly, RefKind::Mutable) =>
            ledger.inner_logic().count() == 1,
        _ => true,
    })]
    #[ensures((^ledger.inner_logic()).invariant() && (^ledger.inner_logic()).is_live())]
    #[ensures((^ledger.inner_logic()).ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).allocation_binding() ==
        ledger.inner_logic().allocation_binding())]
    #[ensures((^ledger.inner_logic()).count() == ledger.inner_logic().count())]
    #[ensures((^ledger.inner_logic()).next_id() == ledger.inner_logic().next_id())]
    #[ensures((^ledger.inner_logic()).active_ids() ==
        ledger.inner_logic().active_ids().insert(
            ticket.inner_logic().owner_id(), Excl(*new_binding)))]
    #[ensures((^ticket.inner_logic()).invariant())]
    #[ensures((^ticket.inner_logic()).ledger_id() == ticket.inner_logic().ledger_id())]
    #[ensures((^ticket.inner_logic()).owner_id() == ticket.inner_logic().owner_id())]
    #[ensures((^ticket.inner_logic()).fragment_resource_id() ==
        ticket.inner_logic().fragment_resource_id())]
    #[ensures((^ticket.inner_logic()).owner_binding() == *new_binding)]
    #[ensures((^ticket.inner_logic()).class() == (*new_binding).1)]
    #[ensures((^ticket.inner_logic()).allocation_binding() == (*new_binding).0)]
    #[ensures((^ticket.inner_logic()).owned_lo() == (*new_binding).2)]
    #[ensures((^ticket.inner_logic()).owned_hi() == (*new_binding).3)]
    #[ensures((^ledger.inner_logic()).has_ticket((^ticket.inner_logic())))]
    pub(crate) fn reclassify(
        ledger: Ghost<&mut OwnerLedger>,
        ticket: Ghost<&mut OwnerTicket>,
        new_binding: Snapshot<OwnerBinding>,
    ) {
        ghost! {
            let ledger = ledger.into_inner();
            let ticket = ticket.into_inner();
            ledger.prove_ticket_is_active(ticket);
            ticket.replace_binding(
                ledger.authority.as_mut().unwrap(),
                new_binding,
            );
            ledger.prove_ticket_is_active(ticket);
        };
    }

    /// Consume exactly one native-reference registration. The final ticket is converted
    /// into a same-generation permit carrying the empty authority.
    #[check(ghost)]
    #[requires(ledger.inner_logic().invariant() && ledger.inner_logic().is_live())]
    #[requires(ticket.inner_logic().invariant())]
    #[requires(ticket.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).invariant())]
    #[ensures((^ledger.inner_logic()).ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).allocation_binding() ==
        ledger.inner_logic().allocation_binding())]
    #[ensures((^ledger.inner_logic()).count() + 1 == ledger.inner_logic().count())]
    #[ensures((^ledger.inner_logic()).active_ids() ==
        ledger.inner_logic().active_ids().remove(ticket.inner_logic().owner_id()))]
    #[ensures(result.inner_logic() != None ==>
        (^ledger.inner_logic()).active_ids() == FMap::empty())]
    #[ensures((^ledger.inner_logic()).next_id() == ledger.inner_logic().next_id())]
    #[ensures(match result.inner_logic() {
        Some(retire) => ledger.inner_logic().count() == 1 &&
            !(^ledger.inner_logic()).is_live() && (^ledger.inner_logic()).count() == 0 &&
            retire.invariant() && retire.ledger_id() == ledger.inner_logic().ledger_id() &&
            retire.active_ids().is_empty(),
        None => ledger.inner_logic().count() > 1 && (^ledger.inner_logic()).is_live(),
    })]
    #[ensures(forall<other_id: Int> other_id != ticket.inner_logic().owner_id() ==>
        (^ledger.inner_logic()).active_ids().get(other_id) ==
            ledger.inner_logic().active_ids().get(other_id))]
    #[ensures(forall<index: Int>
        match ticket.inner_logic().class() {
            RefKind::Mutable => active_mutable_contains(
                (^ledger.inner_logic()).active_ids(), index) ==
                    (active_mutable_contains(ledger.inner_logic().active_ids(), index) &&
                        !(ticket.inner_logic().owned_lo() <= index &&
                            index < ticket.inner_logic().owned_hi())),
            RefKind::Readonly => active_mutable_contains(
                (^ledger.inner_logic()).active_ids(), index) ==
                    active_mutable_contains(ledger.inner_logic().active_ids(), index),
        })]
    pub(crate) fn drop_owner(
        ledger: Ghost<&mut OwnerLedger>,
        ticket: Ghost<OwnerTicket>,
    ) -> Ghost<Option<RetirePermit>> {
        ghost! {
            let ledger = ledger.into_inner();
            let ticket = ticket.into_inner();
            ledger.prove_ticket_is_active(&ticket);
            let before: Snapshot<ActiveReferences> = snapshot!(ledger.active_ids());
            let ticket_snapshot: Snapshot<OwnerTicket> = snapshot!(ticket);
            let old_count: Snapshot<Int> = snapshot!((*before).len());
            let old_count = old_count.into_ghost().into_inner();
            let owner_id = ticket.owner_id;
            let owner_binding: Snapshot<OwnerBinding> = snapshot!(
                ticket.fragment@.get(ticket.owner_id).unwrap_logic().0
            );
            if old_count == 1int {
                sole_active_id(snapshot!(*ledger), snapshot!(ticket));
            }
            let mut fragment = ticket.fragment;
            proof_assert!((*before).get(owner_id) == Some(Excl(*owner_binding)));
            factor_singleton_removal(before, snapshot!(owner_id), owner_binding);
            ledger.authority
                .as_mut()
                .unwrap()
                .update(&mut fragment, CancelLocalUpdateUnit);
            let expected_after: Snapshot<ActiveReferences> =
                snapshot!((*before).remove(owner_id));
            let authority = ledger.authority.as_ref().unwrap();
            proof_assert!(authority@.ext_eq(*expected_after));

            let result = if old_count == 1int {
                let remaining = ledger.authority.as_ref().unwrap();
                proof_assert!(remaining@.is_empty());
                let authority = ledger.authority.take().unwrap();
                ledger.live = false;
                let retire = RetirePermit {
                    ledger_id: ledger.ledger_id,
                    authority,
                };
                proof_assert!(retire.invariant());
                Some(retire)
            } else {
                let remaining = ledger.authority.as_ref().unwrap();
                proof_assert!(remaining@.len() > 0);
                None
            };
            let after: Snapshot<ActiveReferences> = snapshot!(ledger.active_ids());
            prove_owner_removal_effect(before, after, ticket_snapshot);
            result
        }
    }
}

impl Invariant for OwnerLedger {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

/// Freshly adding one exclusive singleton makes the finite-map RA operation
/// exactly the corresponding insertion.
#[check(ghost)]
#[requires(!(*map).contains(*id))]
#[ensures((*map).op(FMap::singleton(*id, Excl(*binding))) ==
    Some((*map).insert(*id, Excl(*binding))))]
fn map_op_fresh_singleton(
    map: Snapshot<ActiveReferences>,
    id: Snapshot<Int>,
    binding: Snapshot<OwnerBinding>,
) {
    let single: Snapshot<ActiveReferences> =
        snapshot!(FMap::singleton(*id, Excl(*binding)));
    proof_assert!(forall<key: Int>
        (*map).get(key).op((*single).get(key)) != None);
    proof_assert!(forall<key: Int>
        (*map).get(key).op((*single).get(key)) ==
            Some((*map).insert(*id, Excl(*binding)).get(key)));
    let composed: Snapshot<ActiveReferences> = snapshot!((*map).total_op(*single));
    proof_assert!((*composed).ext_eq((*map).insert(*id, Excl(*binding))));
    proof_assert!((*map).op(*single) == Some((*map).insert(*id, Excl(*binding))));
}

/// A length-one active map containing the caller's valid ticket has exactly
/// that ticket's ID. The finite-map proof removes that entry, obtains an empty
/// remainder, and uses the removal frame for every other key.
#[check(ghost)]
#[requires((*ledger).invariant() && (*ledger).count() == 1)]
#[requires((*ledger).has_ticket(*ticket))]
#[ensures((*ledger).active_ids() ==
    FMap::singleton((*ticket).owner_id(), Excl((*ticket).owner_binding())))]
#[ensures(forall<index: Int>
    active_mutable_contains((*ledger).active_ids(), index) ==
        match (*ticket).class() {
            RefKind::Mutable => (*ticket).owned_lo() <= index && index < (*ticket).owned_hi(),
            RefKind::Readonly => false,
        })]
pub(crate) fn sole_active_id(ledger: Snapshot<OwnerLedger>, ticket: Snapshot<OwnerTicket>) {
    let map: Snapshot<ActiveReferences> = snapshot!((*ledger).active_ids());
    let id: Snapshot<Int> = snapshot!((*ticket).owner_id());
    let binding: Snapshot<OwnerBinding> = snapshot!((*ticket).owner_binding());
    map_singleton_from_len_one(map, id, binding);
    proof_assert!(forall<index: Int>
        active_mutable_contains(*map, index) ==
            match (*ticket).class() {
                RefKind::Mutable => (*ticket).owned_lo() <= index && index < (*ticket).owned_hi(),
                RefKind::Readonly => false,
            });
}

/// A count-one ledger cannot admit two independently borrowed tickets. The
/// active-ID map first forces both tickets to name the same ID; the affine
/// fragment lemma then rules out composing their exclusive singleton maps.
#[check(ghost)]
#[requires((*ledger).invariant() && (*ledger).count() == 1)]
#[requires((*ledger).has_ticket(*first.inner_logic()))]
#[requires((*ledger).has_ticket(*other.inner_logic()))]
#[ensures(false)]
pub(crate) fn contradict_two_tickets(
    ledger: Snapshot<OwnerLedger>,
    first: Ghost<&mut OwnerTicket>,
    other: Ghost<&OwnerTicket>,
) {
    let first_snapshot: Snapshot<OwnerTicket> = snapshot!(*first.inner_logic());
    let other_snapshot: Snapshot<OwnerTicket> = snapshot!(*other.inner_logic());
    sole_active_id(ledger, first_snapshot);
    proof_assert!((*first_snapshot).owner_id() == (*other_snapshot).owner_id());
    ghost! {
        let first = first.into_inner();
        let other = other.into_inner();
        first.fragment.valid_op_lemma(&other.fragment);
        proof_assert!(first.fragment@.op(other.fragment@) != None);
        proof_assert!(first.fragment@.op(other.fragment@) == None);
        proof_assert!(false);
    };
}

#[check(ghost)]
#[requires((*map).len() == 1)]
#[requires((*map).get(*id) == Some(Excl(*binding)))]
#[ensures(*map == FMap::singleton(*id, Excl(*binding)))]
fn map_singleton_from_len_one(
    map: Snapshot<ActiveReferences>,
    id: Snapshot<Int>,
    binding: Snapshot<OwnerBinding>,
) {
    let remainder: Snapshot<ActiveReferences> =
        snapshot!((*map).remove(*id));
    proof_assert!((*remainder).len() == 0);
    zero_length_map_has_no_ids(remainder);
    proof_assert!(forall<other: Int> other != *id ==> (*map).get(other) == None);
    proof_assert!((*map).get(*id) == Some(Excl(*binding)));
    proof_assert!((*map).ext_eq(FMap::singleton(*id, Excl(*binding))));
}

/// Replacing the only binding of a singleton map, or inserting into an empty
/// map, yields the corresponding singleton and exposes its key lookup.
#[check(ghost)]
#[requires((*map).len() == 0 ||
    ((*map).len() == 1 && (*map).get(*id) != None))]
#[ensures((*map).insert(*id, Excl(*binding)) ==
    FMap::singleton(*id, Excl(*binding)))]
#[ensures((*map).insert(*id, Excl(*binding)).get(*id) ==
    Some(Excl(*binding)))]
fn normalize_singleton_insert(
    map: Snapshot<ActiveReferences>,
    id: Snapshot<Int>,
    binding: Snapshot<OwnerBinding>,
) {
    let _ = ghost! {
        let map_len: Snapshot<Int> = snapshot!((*map).len());
        let map_len = map_len.into_ghost().into_inner();
        if map_len == 0int {
            zero_length_map_has_no_ids(map);
        } else {
            proof_assert!((*map).len() == 1);
            proof_assert!((*map).get(*id) != None);
        }
        let inserted: Snapshot<ActiveReferences> =
            snapshot!((*map).insert(*id, Excl(*binding)));
        proof_assert!((*inserted).get(*id) == Some(Excl(*binding)));
        proof_assert!((*inserted).len() == 1);
        map_singleton_from_len_one(inserted, id, binding);
    };
}

/// FMap's remove/len contracts imply that a map of length zero cannot contain
/// a key: removing any alleged key would give a map with negative length.
#[check(ghost)]
#[requires((*map).len() == 0)]
#[ensures(forall<key: Int> !(*map).contains(key))]
fn zero_length_map_has_no_ids(map: Snapshot<ActiveReferences>) {
    proof_assert!(forall<key: Int> (*map).remove(key).len() >= 0);
    proof_assert!(forall<key: Int> (*map).contains(key) ==>
        (*map).remove(key).len() == -1);
    proof_assert!(forall<key: Int> !(*map).contains(key));
}

/// No finite map entry means no Mutable registration covers any slot. Keep
/// this bridge explicit for callers that combine the ledger map with the
/// physical-pool complement.
#[check(ghost)]
#[requires((*owners).len() == 0)]
#[ensures(forall<index: Int>
    !active_mutable_contains(*owners, index))]
pub(crate) fn prove_no_active_mutable_on_empty(
    owners: Snapshot<ActiveReferences>,
) {
    zero_length_map_has_no_ids(owners);
    proof_assert!(forall<index: Int>
        !active_mutable_contains(*owners, index));
}

/// Cancelling one exclusive singleton registration factors precisely the
/// finite map with that ID removed.
#[check(ghost)]
#[requires((*map).get(*id) == Some(Excl(*binding)))]
#[ensures((*map).factor(FMap::singleton(*id, Excl(*binding))) == (*map).remove(*id))]
fn factor_singleton_removal(
    map: Snapshot<ActiveReferences>,
    id: Snapshot<Int>,
    binding: Snapshot<OwnerBinding>,
) {
    let singleton: Snapshot<ActiveReferences> =
        snapshot!(FMap::singleton(*id, Excl(*binding)));
    proof_assert!((*singleton).incl(*map));
    let remainder: Snapshot<ActiveReferences> = snapshot!((*map).factor(*singleton));
    proof_assert!((*singleton).op(*remainder) == Some(*map));
    proof_assert!(forall<key: Int>
        (*remainder).get(key) == (*map).remove(*id).get(key));
    proof_assert!((*remainder).ext_eq((*map).remove(*id)));
}
