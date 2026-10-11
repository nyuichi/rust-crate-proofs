//! Affine owner registrations for a shared control block.
//!
//! The authority map is the exact set of registered live owner IDs. Each
//! `OwnerTicket` owns one exclusive singleton fragment in that map. This
//! module proves only the generic registration algebra and its count
//! projection; a caller must connect count changes to the actual native
//! atomic event and prove that every handle transfer calls the matching
//! transition.

use creusot_std::{
    ghost::resource::{Authority, Fragment},
    logic::{FMap, Id, Int, ra::{RA, UnitRA, auth::CancelLocalUpdateUnit, excl::Excl}},
    prelude::*,
};

use crate::ref_count_limit::MAX_REF_COUNT;

pub(crate) type ActiveOwners = FMap<Int, Excl<()>>;

/// One non-duplicable registration for a particular owner in a control block.
#[allow(missing_debug_implementations)]
pub(crate) struct OwnerTicket {
    ledger_id: Id,
    owner_id: Int,
    fragment: Fragment<ActiveOwners>,
}

impl OwnerTicket {
    #[logic(open(self))]
    pub(crate) fn ledger_id(self) -> Id {
        self.ledger_id
    }

    #[logic(open(self))]
    pub(crate) fn owner_id(self) -> Int {
        self.owner_id
    }

    #[logic(open(self))]
    pub(crate) fn fragment_resource_id(self) -> Id {
        self.fragment.id()
    }

    #[logic(open(self))]
    pub(crate) fn fragment_map(self) -> ActiveOwners {
        pearlite! { self.fragment@ }
    }

    #[logic(prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.fragment.id() == self.ledger_id &&
            self.fragment@ == FMap::singleton(self.owner_id, Excl(()))
        }
    }
}

impl Invariant for OwnerTicket {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

/// An affine proof that the owner map is empty after the final release.
#[allow(missing_debug_implementations)]
pub(crate) struct RetirePermit {
    ledger_id: Id,
    authority: Authority<ActiveOwners>,
}

impl RetirePermit {
    #[logic(open(self))]
    pub(crate) fn ledger_id(self) -> Id {
        self.ledger_id
    }

    #[logic(open(self))]
    pub(crate) fn active_ids(self) -> ActiveOwners {
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

    /// A same-ledger, valid owner ticket would have to compose with this empty
    /// authority. The exclusive singleton fragment makes that impossible.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(ticket.invariant())]
    #[requires(ticket.ledger_id() == self.ledger_id())]
    #[ensures(false)]
    pub(crate) fn contradicts_live_ticket(&self, ticket: &OwnerTicket) {
        self.authority.frag_lemma(&ticket.fragment);
        let fragment: Snapshot<ActiveOwners> = snapshot!(ticket.fragment@);
        let empty: Snapshot<ActiveOwners> = snapshot!(self.authority@);
        proof_assert!((*fragment).incl(*empty));
        proof_assert!((*fragment).get(ticket.owner_id()) == Some(Excl(())));
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

/// Current affine inventory for one native shared control block.
///
/// `ledger_id` is the stable Resource identity. Moving the empty authority
/// into a `RetirePermit` never changes that identity.
#[allow(missing_debug_implementations)]
pub(crate) struct OwnerLedger {
    ledger_id: Id,
    next_id: Int,
    live: bool,
    authority: Option<Authority<ActiveOwners>>,
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
    pub(crate) fn active_ids(self) -> ActiveOwners {
        pearlite! { match self.authority {
            Some(authority) => authority@,
            None => FMap::empty(),
        } }
    }

    /// The logical shared count is exactly the cardinality of the authoritative
    /// active-ID map; no separately mutable count is stored here.
    #[logic(open(self))]
    pub(crate) fn count(self) -> Int {
        self.active_ids().len()
    }

    /// Relation consumed by the enclosing atomic protocol.
    #[logic(open(self), prophetic)]
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
                    forall<id: Int> authority@.contains(id) ==>
                        0 <= id && id < self.next_id,
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
            self.active_ids().get(ticket.owner_id()) == Some(Excl(()))
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
    fn prove_ticket_is_active(&self, ticket: &OwnerTicket) {
        match &self.authority {
            Some(authority) => {
                authority.frag_lemma(&ticket.fragment);
                let fragment: Snapshot<ActiveOwners> = snapshot!(ticket.fragment@);
                let active: Snapshot<ActiveOwners> = snapshot!(authority@);
                proof_assert!((*fragment).incl(*active));
                proof_assert!((*fragment).get(ticket.owner_id()) == Some(Excl(())));
                proof_assert!((*active).get(ticket.owner_id()) == Some(Excl(())));
            }
            None => proof_assert!(false),
        }
    }

    /// Construct a count-one shared block and its sole root ticket.
    #[check(ghost)]
    #[ensures(result.inner_logic().0.invariant())]
    #[ensures(result.inner_logic().0.count() == 1)]
    #[ensures(result.inner_logic().0.agrees_with_value(1))]
    #[ensures(result.inner_logic().1.invariant())]
    #[ensures(result.inner_logic().0.has_ticket(result.inner_logic().1))]
    pub(crate) fn new_root() -> Ghost<(OwnerLedger, OwnerTicket)> {
        ghost! {
        let mut authority: Authority<ActiveOwners> = Authority::alloc().into_inner();
        let empty_map: Snapshot<ActiveOwners> = snapshot!(ActiveOwners::unit());
        proof_assert!(authority@ == *empty_map);
        let root_id: Int = *Int::new(0i128);
        let root_map: Snapshot<ActiveOwners> =
            snapshot!(FMap::singleton(root_id, Excl(())));
        let root_fragment = authority.add_fragment(root_map);
        proof_assert!(authority@.ext_eq(*root_map));
        let ledger_id = authority.id_ghost();
        let ledger = OwnerLedger {
            ledger_id,
            next_id: *Int::new(1i128),
            live: true,
            authority: Some(authority),
        };
        let ticket = OwnerTicket {
            ledger_id,
            owner_id: root_id,
            fragment: root_fragment,
        };
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
    #[ensures(result.inner_logic().0.count() == 2)]
    #[ensures(result.inner_logic().0.agrees_with_value(2))]
    #[ensures(result.inner_logic().1.invariant() && result.inner_logic().2.invariant())]
    #[ensures(result.inner_logic().0.has_ticket(result.inner_logic().1) &&
        result.inner_logic().0.has_ticket(result.inner_logic().2))]
    #[ensures(result.inner_logic().1.owner_id() == 0 && result.inner_logic().2.owner_id() == 1)]
    pub(crate) fn new_promoted_pair() -> Ghost<(OwnerLedger, OwnerTicket, OwnerTicket)> {
        ghost! {
        let mut authority: Authority<ActiveOwners> = Authority::alloc().into_inner();
        let empty_map: Snapshot<ActiveOwners> = snapshot!(ActiveOwners::unit());
        proof_assert!(authority@ == *empty_map);
        let first_id: Int = *Int::new(0i128);
        let second_id: Int = *Int::new(1i128);
        let first_map: Snapshot<ActiveOwners> =
            snapshot!(FMap::singleton(first_id, Excl(())));
        let first_fragment = authority.add_fragment(first_map);
        proof_assert!(authority@.ext_eq(*first_map));
        let after_first: Snapshot<ActiveOwners> = snapshot!(authority@);
        proof_assert!((*after_first).contains(second_id) == false);
        let second_map: Snapshot<ActiveOwners> =
            snapshot!(FMap::singleton(second_id, Excl(())));
        map_op_fresh_singleton(after_first, snapshot!(second_id));
        let second_fragment = authority.add_fragment(second_map);
        let pair_map: Snapshot<ActiveOwners> =
            snapshot!((*first_map).insert(second_id, Excl(())));
        proof_assert!(authority@.ext_eq(*pair_map));
        let ledger_id = authority.id_ghost();
        let ledger = OwnerLedger {
            ledger_id,
            next_id: *Int::new(2i128),
            live: true,
            authority: Some(authority),
        };
        let first = OwnerTicket {
            ledger_id,
            owner_id: first_id,
            fragment: first_fragment,
        };
        let second = OwnerTicket {
            ledger_id,
            owner_id: second_id,
            fragment: second_fragment,
        };
        proof_assert!(ledger.invariant());
        proof_assert!(first.invariant() && second.invariant());
        ledger.prove_ticket_is_active(&first);
        ledger.prove_ticket_is_active(&second);
        (ledger, first, second)
        }
    }

    /// Register a child only after the caller has established a successful
    /// native increment. A failed/refused update has no child-ticket result.
    #[check(ghost)]
    #[requires(ledger.inner_logic().invariant() && ledger.inner_logic().is_live())]
    #[requires(ledger.inner_logic().count() <= MAX_REF_COUNT@)]
    #[requires(parent.inner_logic().invariant())]
    #[requires(parent.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).invariant() && (^ledger.inner_logic()).is_live())]
    #[ensures((^ledger.inner_logic()).ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).count() == ledger.inner_logic().count() + 1)]
    #[ensures((^ledger.inner_logic()).agrees_with_value(ledger.inner_logic().count() + 1))]
    #[ensures((^ledger.inner_logic()).active_ids() ==
        ledger.inner_logic().active_ids().insert(ledger.inner_logic().next_id(), Excl(())))]
    #[ensures((^ledger.inner_logic()).next_id() == ledger.inner_logic().next_id() + 1)]
    #[ensures((^ledger.inner_logic()).has_ticket(*parent.inner_logic()))]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures(result.inner_logic().owner_id() == ledger.inner_logic().next_id())]
    #[ensures(result.inner_logic().fragment_resource_id() == ledger.inner_logic().ledger_id())]
    #[ensures(result.inner_logic().fragment_map() ==
        FMap::singleton(ledger.inner_logic().next_id(), Excl(())))]
    #[ensures((^ledger.inner_logic()).has_ticket(result.inner_logic()))]
    pub(crate) fn split_owner(
        ledger: Ghost<&mut OwnerLedger>,
        parent: Ghost<&OwnerTicket>,
    ) -> Ghost<OwnerTicket> {
        ghost! {
            let ledger = ledger.into_inner();
            let parent = parent.into_inner();
            ledger.prove_ticket_is_active(parent);
            let old_map: Snapshot<ActiveOwners> = snapshot!(ledger.active_ids());
            let new_id = ledger.next_id;
            proof_assert!(!(*old_map).contains(new_id));
            map_op_fresh_singleton(old_map, snapshot!(new_id));
            let new_fragment = ledger.authority.as_mut().unwrap().add_fragment(
                snapshot!(FMap::singleton(new_id, Excl(()))),
            );
            ledger.next_id = ledger.next_id + 1int;
            let child = OwnerTicket {
                ledger_id: ledger.ledger_id,
                owner_id: new_id,
                fragment: new_fragment,
            };
            proof_assert!(child.invariant());
            ledger.prove_ticket_is_active(parent);
            ledger.prove_ticket_is_active(&child);
            child
        }
    }

    /// Consume exactly one owner registration. The final ticket is converted
    /// into a same-generation permit carrying the empty authority.
    #[check(ghost)]
    #[requires(ledger.inner_logic().invariant() && ledger.inner_logic().is_live())]
    #[requires(ticket.inner_logic().invariant())]
    #[requires(ticket.inner_logic().ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).invariant())]
    #[ensures((^ledger.inner_logic()).ledger_id() == ledger.inner_logic().ledger_id())]
    #[ensures((^ledger.inner_logic()).count() + 1 == ledger.inner_logic().count())]
    #[ensures((^ledger.inner_logic()).active_ids() ==
        ledger.inner_logic().active_ids().remove(ticket.inner_logic().owner_id()))]
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
    pub(crate) fn drop_owner(
        ledger: Ghost<&mut OwnerLedger>,
        ticket: Ghost<OwnerTicket>,
    ) -> Ghost<Option<RetirePermit>> {
        ghost! {
            let ledger = ledger.into_inner();
            let ticket = ticket.into_inner();
            ledger.prove_ticket_is_active(&ticket);
            let before: Snapshot<ActiveOwners> = snapshot!(ledger.active_ids());
            let old_count: Snapshot<Int> = snapshot!((*before).len());
            let old_count = old_count.into_ghost().into_inner();
            let owner_id = ticket.owner_id;
            if old_count == 1int {
                sole_active_id(snapshot!(*ledger), snapshot!(ticket));
            }
            let mut fragment = ticket.fragment;
            proof_assert!((*before).get(owner_id) == Some(Excl(())));
            factor_singleton_removal(before, snapshot!(owner_id));
            ledger.authority
                .as_mut()
                .unwrap()
                .update(&mut fragment, CancelLocalUpdateUnit);
            let expected_after: Snapshot<ActiveOwners> =
                snapshot!((*before).remove(owner_id));
            let authority = ledger.authority.as_ref().unwrap();
            proof_assert!(authority@.ext_eq(*expected_after));

            if old_count == 1int {
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
            }
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
#[ensures((*map).op(FMap::singleton(*id, Excl(()))) ==
    Some((*map).insert(*id, Excl(()))))]
fn map_op_fresh_singleton(map: Snapshot<ActiveOwners>, id: Snapshot<Int>) {
    let single: Snapshot<ActiveOwners> = snapshot!(FMap::singleton(*id, Excl(())));
    proof_assert!(forall<key: Int>
        (*map).get(key).op((*single).get(key)) != None);
    proof_assert!(forall<key: Int>
        (*map).get(key).op((*single).get(key)) ==
            Some((*map).insert(*id, Excl(())).get(key)));
    let composed: Snapshot<ActiveOwners> = snapshot!((*map).total_op(*single));
    proof_assert!((*composed).ext_eq((*map).insert(*id, Excl(()))));
    proof_assert!((*map).op(*single) == Some((*map).insert(*id, Excl(()))));
}

/// A length-one active map containing the caller's valid ticket has exactly
/// that ticket's ID. The finite-map proof removes that entry, obtains an empty
/// remainder, and uses the removal frame for every other key.
#[check(ghost)]
#[requires((*ledger).invariant() && (*ledger).count() == 1)]
#[requires((*ledger).has_ticket(*ticket))]
#[ensures((*ledger).active_ids() ==
    FMap::singleton((*ticket).owner_id(), Excl(())))]
pub(crate) fn sole_active_id(ledger: Snapshot<OwnerLedger>, ticket: Snapshot<OwnerTicket>) {
    let map: Snapshot<ActiveOwners> = snapshot!((*ledger).active_ids());
    let id: Snapshot<Int> = snapshot!((*ticket).owner_id());
    map_singleton_from_len_one(map, id);
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
#[requires((*map).get(*id) == Some(Excl(())))]
#[ensures(*map == FMap::singleton(*id, Excl(())))]
fn map_singleton_from_len_one(map: Snapshot<ActiveOwners>, id: Snapshot<Int>) {
    let remainder: Snapshot<ActiveOwners> =
        snapshot!((*map).remove(*id));
    proof_assert!((*remainder).len() == 0);
    zero_length_map_has_no_ids(remainder);
    proof_assert!(forall<other: Int> other != *id ==> (*map).get(other) == None);
    proof_assert!((*map).get(*id) == Some(Excl(())));
    proof_assert!((*map).ext_eq(FMap::singleton(*id, Excl(()))));
}

/// FMap's remove/len contracts imply that a map of length zero cannot contain
/// a key: removing any alleged key would give a map with negative length.
#[check(ghost)]
#[requires((*map).len() == 0)]
#[ensures(forall<key: Int> !(*map).contains(key))]
fn zero_length_map_has_no_ids(map: Snapshot<ActiveOwners>) {
    proof_assert!(forall<key: Int> (*map).remove(key).len() >= 0);
    proof_assert!(forall<key: Int> (*map).contains(key) ==>
        (*map).remove(key).len() == -1);
    proof_assert!(forall<key: Int> !(*map).contains(key));
}

/// Cancelling one exclusive singleton registration factors precisely the
/// finite map with that ID removed.
#[check(ghost)]
#[requires((*map).get(*id) == Some(Excl(())))]
#[ensures((*map).factor(FMap::singleton(*id, Excl(()))) == (*map).remove(*id))]
fn factor_singleton_removal(map: Snapshot<ActiveOwners>, id: Snapshot<Int>) {
    let singleton: Snapshot<ActiveOwners> =
        snapshot!(FMap::singleton(*id, Excl(())));
    proof_assert!((*singleton).incl(*map));
    let remainder: Snapshot<ActiveOwners> = snapshot!((*map).factor(*singleton));
    proof_assert!((*singleton).op(*remainder) == Some(*map));
    proof_assert!(forall<key: Int>
        (*remainder).get(key) == (*map).remove(*id).get(key));
    proof_assert!((*remainder).ext_eq((*map).remove(*id)));
}
