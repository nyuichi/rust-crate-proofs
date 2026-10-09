//! Quota-free sparse registration joined to arbitrary-order Release retirement.
//!
//! Ticket IDs are monotonically allocated ghost integers. `alive` is a sparse
//! authority map: registration inserts at `next`, while retirement removes the
//! ID carried by the consumed ticket. No fixed number of ticket classes or
//! byte-specific last-owner rule is used.

use super::*;
use creusot_std::{
    ghost::{invariant::Protocol, resource::{Authority, Fragment}},
    logic::{real::Real, ra::{agree::Ag, auth::CancelLocalUpdateUnit}},
    std::sync::view::AtView,
};
use crate::{event::{EventProtocol, ScopedProtocol}, fraction_map::{self as fm, LiveFractions}};

// Avoid a second usize addition on the imported MAX_REF_COUNT constant. The
// verifier keeps that imported const opaque while checking overflow here.
const MAX_LIVE: usize = usize::MAX / 2 + 1;

pub trait RecoveryPayload {
    type Metadata;
    #[logic] fn metadata(self) -> Self::Metadata;
    #[logic(prophetic)] fn wellformed(self) -> bool;
}

pub struct Ticket<T: RecoveryPayload> {
    pub(crate) token: LifetimeToken,
    pub(crate) id: Int,
    pub(crate) fragment: Fragment<LiveFractions>,
    pub(crate) recovery: Option<T>,
}

type Publication<T> = Option<Ag<(Int, AtView<Option<T>>)>>;

/// The ticket converted to a caller-view observation before its Release event.
pub struct Retiring<T: RecoveryPayload> {
    pub(crate) token: LifetimeToken,
    id: Int,
    fragment: Fragment<LiveFractions>,
    sealed: AtView<Option<T>>,
}

impl<T: RecoveryPayload> Retiring<T> {
    #[logic] pub fn id(self) -> Int { self.id }
    #[logic(open(crate))] pub fn lifetime(self) -> Lifetime { self.token.lft() }

    #[logic(prophetic)]
    pub fn valid(self, public: <State<T> as Protocol>::Public, current: SyncView) -> bool {
        pearlite! {
            0 <= self.id &&
            self.fragment.id() == public.1 &&
            self.fragment@ == FMap::singleton(self.id, Excl(self.token.frac())) &&
            self.token.lft() == public.2 && self.sealed.view() <= current &&
            (if self.id == 0 {
                self.sealed.val() != None && self.sealed.val().unwrap_logic().wellformed() &&
                    self.sealed.val().unwrap_logic().metadata() == public.3
            } else { self.sealed.val() == None })
        }
    }
}

pub struct RetiringRest<T: RecoveryPayload> {
    id: Int,
    fragment: Fragment<LiveFractions>,
    sealed: AtView<Option<T>>,
}

impl<T: RecoveryPayload> RetiringRest<T> {
    #[logic]
    pub fn paired(self, token: LifetimeToken) -> Retiring<T> {
        Retiring { token, id: self.id, fragment: self.fragment, sealed: self.sealed }
    }

    #[check(ghost)]
    #[ensures(result.inner_logic() == rest.paired(*token))]
    pub fn with_token(
        rest: Ghost<Self>,
        token: Ghost<LifetimeToken>,
    ) -> Ghost<Retiring<T>> {
        ghost! {
            let rest = rest.into_inner();
            Retiring {
                token: token.into_inner(),
                id: rest.id,
                fragment: rest.fragment,
                sealed: rest.sealed,
            }
        }
    }
}

impl<T: RecoveryPayload> Retiring<T> {
    #[check(ghost)]
    #[ensures(result.inner_logic().1.paired(result.inner_logic().0) == *input)]
    #[ensures(result.inner_logic().0.lft() == input.lifetime())]
    pub fn split_token(input: Ghost<Self>) -> Ghost<(LifetimeToken, RetiringRest<T>)> {
        ghost! {
            let input = input.into_inner();
            (
                input.token,
                RetiringRest {
                    id: input.id,
                    fragment: input.fragment,
                    sealed: input.sealed,
                },
            )
        }
    }
}

pub struct Pending<T: RecoveryPayload> {
    sealed: AtView<Option<T>>,
    full: LifetimeToken,
    fragment: Fragment<Publication<T>>,
}

impl<T: RecoveryPayload> Pending<T> {
    #[logic] pub fn acquired(self, current: SyncView) -> bool { self.sealed.view() <= current }

    #[logic] pub fn token(self) -> LifetimeToken { self.full }

    #[check(ghost)]
    #[ensures(*result == self.token())]
    pub fn borrow_token(&self) -> &LifetimeToken { &self.full }

    #[logic(prophetic)]
    pub fn valid(self, public: <State<T> as Protocol>::Public, current: SyncView) -> bool {
        pearlite! {
            self.sealed.val() != None && self.sealed.val().unwrap_logic().wellformed() &&
            self.sealed.val().unwrap_logic().metadata() == public.3 &&
            self.full.lft() == public.2 && self.full.frac() == PositiveReal::from_int(1) &&
            self.fragment.id() == public.4 && self.fragment@ != None &&
            self.fragment@.unwrap_logic().0.1 == self.sealed &&
            self.fragment@.unwrap_logic().0.0 <= public.0.get_timestamp(current)
        }
    }

    #[check(ghost)]
    #[requires(self.valid(*public, *current))]
    #[ensures(*result == self.token())]
    #[ensures(result.lft() == public.2 && result.frac() == PositiveReal::from_int(1))]
    pub fn borrow_token_for(
        &self,
        public: Snapshot<<State<T> as Protocol>::Public>,
        current: Snapshot<SyncView>,
    ) -> &LifetimeToken {
        &self.full
    }

    #[check(ghost)]
    #[requires(owned.valid(*public, *current) && owned.acquired(*current))]
    #[ensures(result.inner_logic().0.wellformed() && result.inner_logic().0.metadata() == public.3)]
    #[ensures(result.inner_logic().1.lft() == public.2 && result.inner_logic().1.frac() == PositiveReal::from_int(1))]
    pub fn recover(
        owned: Ghost<Self>,
        public: Snapshot<<State<T> as Protocol>::Public>,
        current: Ghost<SyncView>,
    ) -> Ghost<(T, LifetimeToken)> {
        ghost! {
            let owned = owned.into_inner();
            (owned.sealed.sync(current.into_inner()).unwrap(), owned.full)
        }
    }
}

#[requires(ticket.valid(*public))]
#[ensures(result.1.inner_logic().valid(*public, *result.0))]
#[ensures(result.1.inner_logic().id() == ticket.id)]
#[ensures(result.1.inner_logic().lifetime() == public.2)]
pub fn prepare<T: RecoveryPayload>(
    ticket: Ghost<Ticket<T>>,
    public: Snapshot<<State<T> as Protocol>::Public>,
) -> (Ghost<SyncView>, Ghost<Retiring<T>>) {
    let (rest, recovery) = ghost! {
        let t = ticket.into_inner();
        ((t.id, (t.token, t.fragment)), t.recovery)
    }.split();
    let (current, sealed) = AtView::new(recovery).split();
    (current, ghost! {
        let (id, (token, fragment)) = rest.into_inner();
        Retiring { token, id, fragment, sealed: sealed.into_inner() }
    })
}

pub struct State<T: RecoveryPayload> {
    own: Perm<ModelAtomic>,
    first: Int,
    latest: Int,
    alive: Authority<LiveFractions>,
    pub(crate) next: Int,
    lifetime: Lifetime,
    pool: Option<LifetimeToken>,
    expected: Snapshot<T::Metadata>,
    recovery: Option<AtView<Option<T>>>,
    withdrawn: bool,
    publication: Authority<Publication<T>>,
}

impl<T: RecoveryPayload> Protocol for State<T> {
    type Public = (ModelAtomic, Id, Lifetime, T::Metadata, Id);

    #[logic]
    fn public(self) -> Self::Public {
        (*self.own.ward(), self.alive.id(), self.lifetime, *self.expected,
            self.publication.id())
    }

    #[logic(prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
            self.next >= 1 && fm::ids_bounded(self.alive@, self.next) &&
            self.own.val().get(self.latest) != None &&
            self.own.val()[self.latest].0@ == self.alive@.len() &&
            self.alive@.len() <= MAX_LIVE@ &&
            self.first <= self.latest &&
            (forall<t: Int> self.own.val().get(t) != None ==
                (self.first <= t && t <= self.latest)) &&
            (self.withdrawn == (self.alive@.len() == 0)) &&
            ((self.pool == None) == self.withdrawn) &&
            (self.pool != None ==>
                self.pool.unwrap_logic().lft() == self.lifetime &&
                self.pool.unwrap_logic().frac().to_real() +
                    fm::sum_prefix(self.alive@, self.next) == Real::from_int(1)) &&
            (if self.withdrawn { self.recovery == None } else {
                ((self.recovery == None) == self.alive@.contains(0)) &&
                (self.recovery != None ==>
                    self.recovery.unwrap_logic().val() != None &&
                    self.recovery.unwrap_logic().val().unwrap_logic().wellformed() &&
                    self.recovery.unwrap_logic().val().unwrap_logic().metadata() == *self.expected &&
                    self.recovery.unwrap_logic().view() <= self.own.val()[self.latest].1)
            }) &&
            ((self.publication@ != None) == self.withdrawn) &&
            (self.publication@ != None ==>
                self.publication@.unwrap_logic().0.0 == self.latest &&
                self.publication@.unwrap_logic().0.1.view() == self.own.val()[self.latest].1)
        }
    }
}

impl<T: RecoveryPayload> EventProtocol for State<T> {
    #[logic] fn atomic(self) -> ModelAtomic { *self.own.ward() }
}

impl<T: RecoveryPayload> ScopedProtocol for State<T> {
    type Observation = (LiveFractions, Int);
    #[logic(open(crate))] fn observe(self) -> Self::Observation { (self.live_map(), self.next_id()) }
}

impl<T: RecoveryPayload> Ticket<T> {
    #[logic(open(crate))] pub fn id(self) -> Int { self.id }
    #[logic(open(crate))] pub fn fraction(self) -> PositiveReal { self.token.frac() }

    #[logic(open(crate), prophetic)]
    pub fn valid(self, public: <State<T> as Protocol>::Public) -> bool {
        pearlite! {
            0 <= self.id &&
            self.fragment.id() == public.1 &&
            self.fragment@ == FMap::singleton(self.id, Excl(self.token.frac())) &&
            self.token.lft() == public.2 &&
            (if self.id == 0 {
                self.recovery != None && self.recovery.unwrap_logic().wellformed() &&
                    self.recovery.unwrap_logic().metadata() == public.3
            } else { self.recovery == None })
        }
    }
}

#[check(ghost)]
#[requires(!authority@.contains(*id))]
#[ensures((^authority)@ == authority@.insert(*id, Excl(*fraction)))]
#[ensures(result@ == FMap::singleton(*id, Excl(*fraction)))]
#[ensures(result.id() == authority.id() && (^authority).id() == authority.id())]
fn issue(
    authority: &mut Authority<LiveFractions>,
    id: Snapshot<Int>,
    fraction: Snapshot<PositiveReal>,
) -> Fragment<LiveFractions> {
    let before = snapshot!(authority@);
    let singleton = snapshot!(FMap::singleton(*id, Excl(*fraction)));
    proof_assert!(forall<key: Int> before.get(key).op(singleton.get(key)) != None);
    let result = authority.add_fragment(singleton);
    proof_assert!(authority@.ext_eq(before.insert(*id, Excl(*fraction))));
    result
}

#[check(ghost)]
#[requires(fragment.id() == authority.id() && fragment@ == FMap::singleton(*id, Excl(*fraction)))]
#[ensures((^authority)@ == authority@.remove(*id))]
#[ensures(authority@.get(*id) == Some(Excl(*fraction)))]
#[ensures((^authority).id() == authority.id())]
fn remove(
    authority: &mut Authority<LiveFractions>,
    mut fragment: Fragment<LiveFractions>,
    id: Snapshot<Int>,
    fraction: Snapshot<PositiveReal>,
) {
    authority.frag_lemma(&fragment);
    let before = snapshot!(authority@);
    proof_assert!({ fm::singleton_ticket_factor(*before, *id, *fraction); true });
    authority.update(&mut fragment, CancelLocalUpdateUnit);
}

#[logic]
#[requires(next >= 0 && fm::ids_bounded(live, next))]
#[ensures(fm::ids_bounded(live.remove(id), next))]
fn remove_preserves_bound(live: LiveFractions, id: Int, next: Int) {
    proof_assert!(forall<key: Int> live.remove(id).get(key) != None ==>
        live.get(key) != None && key != id);
    proof_assert!(forall<key: Int> live.remove(id).get(key) != None ==>
        0 <= key && key < next);
}

impl<T: RecoveryPayload> State<T> {
    #[logic] pub fn live_count(self) -> Int { pearlite! { self.alive@.len() } }
    #[logic] pub fn live_map(self) -> LiveFractions { pearlite! { self.alive@ } }
    #[logic(open(crate))] pub fn next_id(self) -> Int { self.next }
    /// Resource-free observation of the remaining lifetime fraction. This
    /// exposes neither the private pool field nor its affine LifetimeToken.
    #[logic]
    pub fn pool_fraction(self) -> Option<PositiveReal> {
        match self.pool { Some(token) => Some(token.frac()), None => None }
    }

    /// Export the body-proved relationship needed by cross-module consumers.
    #[check(ghost)]
    #[ensures(state.atomic() == state.public().0)]
    pub fn atomic_is_public(state: Ghost<&Self>) -> Ghost<()> {
        ghost! {
            let state = state.into_inner();
            proof_assert!(state.atomic() == state.public().0);
        }
    }

    #[check(ghost)]
    #[requires(payload.wellformed() && full.frac() == PositiveReal::from_int(1))]
    #[requires(own.val() == FMap::singleton((*own.ward()).get_timestamp(*current), (1usize, *current)))]
    #[ensures(result.inner_logic().0.protocol() && result.inner_logic().0.atomic() == *own.ward())]
    #[ensures(result.inner_logic().0.public().0 == *own.ward())]
    #[ensures(result.inner_logic().0.public().2 == full.lft() && result.inner_logic().0.public().3 == payload.metadata())]
    #[ensures(result.inner_logic().1.token.frac() + result.inner_logic().1.token.frac() == PositiveReal::from_int(1))]
    #[ensures(result.inner_logic().1.valid(result.inner_logic().0.public()))]
    #[ensures(result.inner_logic().1.id() == 0)]
    #[ensures(result.inner_logic().0.live_map() == FMap::singleton(
        result.inner_logic().1.id(), Excl(result.inner_logic().1.token.frac())))]
    #[ensures(result.inner_logic().0.next_id() == 1)]
    pub fn initialize(
        own: Ghost<Perm<ModelAtomic>>,
        current: Snapshot<SyncView>,
        payload: Ghost<T>,
        full: Ghost<LifetimeToken>,
    ) -> Ghost<(Self, Ticket<T>)> {
        ghost! {
            let own = own.into_inner();
            let payload = payload.into_inner();
            let full = full.into_inner();
            let (first_token, pool) = full.split();
            let mut alive = Authority::<LiveFractions>::alloc().into_inner();
            let zero: Snapshot<Int> = snapshot!(0);
            let one: Snapshot<Int> = snapshot!(1);
            let fraction = snapshot!(first_token.frac());
            let fragment = issue(&mut alive, zero, fraction);
            proof_assert!({ fm::empty_prefix(*zero); true });
            proof_assert!({ fm::insert_fresh(LiveFractions::empty(), *zero, *fraction); true });
            proof_assert!({ fm::insert_fresh_preserves_bound(LiveFractions::empty(), *zero, *fraction); true });
            proof_assert!({ fm::cardinality_prefix_is_len(alive@, *one); true });
            proof_assert!(fm::sum_prefix(alive@, *one) == fraction.to_real());
            proof_assert!(first_token.frac() + pool.frac() == PositiveReal::from_int(1));
            let expected = snapshot!(payload.metadata());
            let first: Snapshot<Int> = snapshot!((*own.ward()).get_timestamp(*current));
            let lifetime: Snapshot<Lifetime> = snapshot!(pool.lft());
            let ticket = Ticket {
                token: first_token, id: Int::new(0).into_inner(), fragment,
                recovery: Some(payload),
            };
            let state = State {
                own,
                first: first.into_ghost().into_inner(),
                latest: first.into_ghost().into_inner(),
                alive,
                next: Int::new(1).into_inner(),
                lifetime: lifetime.into_ghost().into_inner(),
                pool: Some(pool), expected, recovery: None, withdrawn: false,
                publication: Authority::alloc().into_inner(),
            };
            (state, ticket)
        }
    }

    /// Initialize the actual two-owner count and issue both tickets directly.
    /// The root keeps half of the lifetime; the child and residual pool each
    /// receive one quarter. No native RMW or synthetic registration is used.
    #[check(ghost)]
    #[requires(payload.wellformed() && full.frac() == PositiveReal::from_int(1))]
    #[requires(own.val() == FMap::singleton((*own.ward()).get_timestamp(*current), (2usize, *current)))]
    #[ensures(result.inner_logic().0.protocol() && result.inner_logic().0.atomic() == *own.ward())]
    #[ensures(result.inner_logic().0.public().0 == *own.ward())]
    #[ensures(result.inner_logic().0.public().2 == full.lft() && result.inner_logic().0.public().3 == payload.metadata())]
    #[ensures(result.inner_logic().1.valid(result.inner_logic().0.public()))]
    #[ensures(result.inner_logic().2.valid(result.inner_logic().0.public()))]
    #[ensures(result.inner_logic().1.id() == 0 && result.inner_logic().2.id() == 1)]
    #[ensures(result.inner_logic().1.id() != result.inner_logic().2.id())]
    #[ensures(result.inner_logic().1.token.frac() == PositiveReal::from_int(1) / PositiveReal::from_int(2))]
    #[ensures(result.inner_logic().2.token.frac() == PositiveReal::from_int(1) / PositiveReal::from_int(4))]
    #[ensures(result.inner_logic().0.pool_fraction() == Some(PositiveReal::from_int(1) / PositiveReal::from_int(4)))]
    #[ensures(result.inner_logic().1.token.frac() + result.inner_logic().2.token.frac() +
        result.inner_logic().0.pool_fraction().unwrap_logic() == PositiveReal::from_int(1))]
    #[ensures(result.inner_logic().0.live_map() == FMap::singleton(
        result.inner_logic().1.id(), Excl(result.inner_logic().1.token.frac())).insert(
        result.inner_logic().2.id(), Excl(result.inner_logic().2.token.frac())))]
    #[ensures(result.inner_logic().0.live_map().len() == 2)]
    #[ensures(result.inner_logic().0.next_id() == 2)]
    pub fn initialize_pair(
        own: Ghost<Perm<ModelAtomic>>,
        current: Snapshot<SyncView>,
        payload: Ghost<T>,
        full: Ghost<LifetimeToken>,
    ) -> Ghost<(Self, Ticket<T>, Ticket<T>)> {
        ghost! {
            let own = own.into_inner();
            let payload = payload.into_inner();
            let full = full.into_inner();
            let (root_token, pool) = full.split();
            let (child_token, pool) = pool.split();

            let mut alive = Authority::<LiveFractions>::alloc().into_inner();
            let zero: Snapshot<Int> = snapshot!(0);
            let one: Snapshot<Int> = snapshot!(1);
            let two: Snapshot<Int> = snapshot!(2);
            let root_fraction = snapshot!(root_token.frac());
            let child_fraction = snapshot!(child_token.frac());

            let root_fragment = issue(&mut alive, zero, root_fraction);
            proof_assert!({ fm::empty_prefix(*zero); true });
            proof_assert!({ fm::insert_fresh(LiveFractions::empty(), *zero, *root_fraction); true });
            proof_assert!({ fm::insert_fresh_preserves_bound(LiveFractions::empty(), *zero, *root_fraction); true });
            proof_assert!({ fm::cardinality_prefix_is_len(alive@, *one); true });
            proof_assert!(fm::sum_prefix(alive@, *one) == root_fraction.to_real());
            proof_assert!(child_token.frac() + pool.frac() == PositiveReal::from_int(1) /
                PositiveReal::from_int(2));

            let after_root = snapshot!(alive@);
            let child_fragment = issue(&mut alive, one, child_fraction);
            proof_assert!({ fm::insert_fresh(*after_root, *one, *child_fraction); true });
            proof_assert!({ fm::insert_fresh_preserves_bound(*after_root, *one, *child_fraction); true });
            proof_assert!({ fm::cardinality_prefix_is_len(alive@, *two); true });
            proof_assert!(fm::sum_prefix(alive@, *two) ==
                child_fraction.to_real() + root_fraction.to_real());
            proof_assert!(pearlite! {
                (root_token.frac() + child_token.frac() + pool.frac())
                    .ext_eq(PositiveReal::from_int(1))
            });
            proof_assert!(pearlite! {
                root_token.frac() + child_token.frac() + pool.frac() == PositiveReal::from_int(1)
            });
            proof_assert!(pool.frac().to_real() + fm::sum_prefix(alive@, *two) ==
                Real::from_int(1));
            proof_assert!(pearlite! {
                root_token.frac().ext_eq(PositiveReal::from_int(1) / PositiveReal::from_int(2))
            });
            proof_assert!(pearlite! {
                child_token.frac().ext_eq(PositiveReal::from_int(1) / PositiveReal::from_int(4))
            });
            proof_assert!(pearlite! {
                pool.frac().ext_eq(PositiveReal::from_int(1) / PositiveReal::from_int(4))
            });
            proof_assert!(pearlite! {
                root_token.frac() == PositiveReal::from_int(1) / PositiveReal::from_int(2)
            });
            proof_assert!(pearlite! {
                child_token.frac() == PositiveReal::from_int(1) / PositiveReal::from_int(4)
            });
            proof_assert!(pearlite! {
                pool.frac() == PositiveReal::from_int(1) / PositiveReal::from_int(4)
            });

            let expected = snapshot!(payload.metadata());
            let initial: Snapshot<Int> = snapshot!((*own.ward()).get_timestamp(*current));
            let lifetime: Snapshot<Lifetime> = snapshot!(root_token.lft());
            let root = Ticket {
                token: root_token, id: Int::new(0).into_inner(), fragment: root_fragment,
                recovery: Some(payload),
            };
            let child = Ticket {
                token: child_token, id: Int::new(1).into_inner(), fragment: child_fragment,
                recovery: None,
            };
            let state = State {
                own,
                first: initial.into_ghost().into_inner(),
                latest: initial.into_ghost().into_inner(),
                alive,
                next: Int::new(2).into_inner(),
                lifetime: lifetime.into_ghost().into_inner(),
                pool: Some(pool),
                expected,
                recovery: None,
                withdrawn: false,
                publication: Authority::alloc().into_inner(),
            };
            (state, root, child)
        }
    }

    /// Create a fresh sparse-map key and one affine ticket from the residual
    /// lifetime pool. The borrowed source ticket is unchanged.
    #[check(ghost)]
    #[requires(state.protocol() && source.valid(state.public()))]
    #[requires(!committer.shot_store() && committer.ward() == state.atomic())]
    #[requires(committer.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && committer.val_store()@ == committer.val_load()@ + 1)]
    #[ensures((^state).protocol() && (^state).public() == state.public())]
    #[ensures((^state).atomic() == state.atomic())]
    #[ensures((^committer).shot_store() && committer.hist_inv(^committer))]
    #[ensures(result.inner_logic().valid(state.public()))]
    #[ensures(result.inner_logic().id() == state.next_id() && result.inner_logic().id() > source.id())]
    #[ensures(source.valid(state.public()) && (^state).next_id() == state.next_id() + 1)]
    #[ensures((^state).live_map() == state.live_map().insert(
        result.inner_logic().id(), Excl(result.inner_logic().token.frac())))]
    #[ensures(committer.val_load()@ == state.live_map().len())]
    #[ensures(!state.live_map().contains(result.inner_logic().id()))]
    pub fn on_register(
        state: Ghost<&mut Self>,
        committer: Ghost<&mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>,
        source: Ghost<&Ticket<T>>,
        current: Ghost<&mut SyncView>,
        release: Ghost<ReleaseSyncView>,
    ) -> Ghost<Ticket<T>> {
        ghost! {
            let state = state.into_inner();
            let committer = committer.into_inner();
            let source = source.into_inner();
            let current = current.into_inner();
            let release = release.into_inner();
            state.alive.frag_lemma(&source.fragment);
            proof_assert!(state.alive@.contains(source.id) && source.id < state.next);
            proof_assert!({ contains_implies_nonempty(state.alive@, source.id); true });
            proof_assert!(state.alive@.len() > 0);
            proof_assert!(state.pool != None);
            relaxed::relaxed_rmw(committer, &mut state.own, current, release);
            proof_assert!(committer.timestamp() == state.latest);
            proof_assert!(committer.val_load()@ == state.live_count());
            let next = snapshot!(state.next);
            let before = snapshot!(state.alive@);
            let prior_pool = snapshot!(state.pool.unwrap_logic().frac());
            let token = state.pool.as_mut().unwrap().split_off();
            let fraction = snapshot!(token.frac());
            proof_assert!(token.frac() + state.pool.unwrap_logic().frac() == *prior_pool);
            proof_assert!(before.get(*next) == None);
            let fragment = {
                #[cfg(feature = "negative_core_lost_insert")]
                { Fragment::new_unit(source.fragment.id_ghost()) }
                #[cfg(not(feature = "negative_core_lost_insert"))]
                { issue(&mut state.alive, next, fraction) }
            };
            proof_assert!({ fm::insert_fresh(*before, *next, *fraction); true });
            proof_assert!({ fm::insert_fresh_preserves_bound(*before, *next, *fraction); true });
            proof_assert!({ fm::cardinality_prefix_is_len(state.alive@, *next + 1); true });
            proof_assert!(fm::sum_prefix(state.alive@, *next + 1) +
                state.pool.unwrap_logic().frac().to_real() == Real::from_int(1));
            proof_assert!(committer.timestamp() == state.latest);
            let latest: Snapshot<Int> = snapshot!(committer.timestamp() + 1);
            state.latest = latest.into_ghost().into_inner();
            state.next = state.next + 1int;
            Ticket { token, id: next.into_ghost().into_inner(), fragment, recovery: None }
        }
    }

    #[check(ghost)]
    #[requires(state.protocol() && input.valid(state.public(), **current))]
    #[requires(!committer.shot_store() && committer.ward() == state.atomic())]
    #[requires(if committer.val_load() == 0usize {
        committer.val_store() == usize::MAX
    } else { committer.val_store()@ + 1 == committer.val_load()@ })]
    #[ensures((^state).protocol() && (^state).public() == state.public())]
    #[ensures((^state).atomic() == state.atomic())]
    #[ensures((^committer).shot_store() && committer.hist_inv(^committer))]
    #[ensures(**current <= ^current)]
    #[ensures((result.inner_logic() != None) == (committer.val_load() == 1usize))]
    #[ensures(result.inner_logic() != None ==> result.inner_logic().unwrap_logic().valid(state.public(), ^current))]
    #[ensures((^state).live_map() == state.live_map().remove(input.inner_logic().id()))]
    #[ensures(result.inner_logic() != None ==> (^state).live_count() == 0)]
    #[ensures(result.inner_logic() != None ==> (^state).live_map().len() == 0)]
    #[ensures((^state).next_id() == state.next_id())]
    #[ensures(committer.val_load()@ == state.live_map().len())]
    pub fn on_release(
        state: Ghost<&mut Self>,
        committer: Ghost<&mut Committer<ModelAtomic, usize, Relaxed, Release>>,
        input: Ghost<Retiring<T>>,
        current: Ghost<&mut SyncView>,
    ) -> Ghost<Option<Pending<T>>> {
        ghost! {
            let state = state.into_inner();
            let committer = committer.into_inner();
            let input = input.into_inner();
            let current = current.into_inner();
            let Retiring { token, id, fragment, sealed } = input;
            state.alive.frag_lemma(&fragment);
            proof_assert!(state.alive@.contains(id) && 0 <= id && id < state.next);
            let before = snapshot!(state.alive@);
            let fraction = snapshot!(token.frac());
            let identity = snapshot!(id);
            #[cfg(feature = "negative_core_lost_removal")]
            { let _ = fragment; }
            #[cfg(not(feature = "negative_core_lost_removal"))]
            {
                remove(&mut state.alive, fragment, identity, fraction);
                proof_assert!((*before).get(id) == Some(Excl(*fraction)) && (*before).len() > 0);
                proof_assert!({ remove_preserves_bound(*before, id, state.next); true });
                proof_assert!({ fm::remove_known(*before, id, *fraction, state.next); true });
                proof_assert!({ fm::cardinality_prefix_is_len(*before, state.next); true });
                proof_assert!({ fm::cardinality_prefix_is_len(state.alive@, state.next); true });
                proof_assert!(state.alive@.len() + 1 == (*before).len());
            }
            state.pool.as_mut().unwrap().join_in(token);
            let published = release::release_rmw(committer, &mut state.own, current);
            proof_assert!(committer.timestamp() == state.latest && committer.val_load()@ == (*before).len());
            let latest: Snapshot<Int> = snapshot!(committer.timestamp() + 1);
            state.latest = latest.into_ghost().into_inner();
            if id == Int::new(0).into_inner() { state.recovery = Some(sealed); }
            let last: Snapshot<bool> = snapshot!(committer.val_load() == 1usize);
            let mut pending = None;
            if last.into_ghost().into_inner() {
                proof_assert!({ empty_live_map(state.alive@); true });
                proof_assert!({ fm::empty_prefix(state.next); true });
                let full = state.pool.take().unwrap();
                proof_assert!(full.frac().ext_eq(PositiveReal::from_int(1)));
                let mut recovery = state.recovery.take().unwrap();
                recovery.weaken(published);
                let stamp = snapshot!(Some(Ag((state.latest, recovery))));
                let fragment = state.publication.add_fragment(stamp);
                pending = Some(Pending { sealed: recovery, full, fragment });
                state.withdrawn = true;
            }
            pending
        }
    }

    /// The caller still owns `peer` after consuming `input`; this fragment
    /// proves that the Release removes one ID while at least one other ID
    /// remains. The event and state transition are delegated to `on_release`.
    #[check(ghost)]
    #[requires(state.protocol() && input.valid(state.public(), **current) && peer.valid(state.public()))]
    #[requires(!committer.shot_store() && committer.ward() == state.atomic())]
    #[requires(if committer.val_load() == 0usize {
        committer.val_store() == usize::MAX
    } else { committer.val_store()@ + 1 == committer.val_load()@ })]
    #[ensures((^state).protocol() && (^state).public() == state.public())]
    #[ensures((^state).atomic() == state.atomic())]
    #[ensures((^committer).shot_store() && committer.hist_inv(^committer))]
    #[ensures(**current <= ^current)]
    #[ensures(result.inner_logic() == None)]
    pub fn on_release_with_live_peer(
        state: Ghost<&mut Self>,
        committer: Ghost<&mut Committer<ModelAtomic, usize, Relaxed, Release>>,
        input: Ghost<Retiring<T>>,
        current: Ghost<&mut SyncView>,
        peer: Ghost<&Ticket<T>>,
    ) -> Ghost<Option<Pending<T>>> {
        ghost! {
            let state = state.into_inner();
            let committer = committer.into_inner();
            let mut input = input.into_inner();
            let current = current.into_inner();
            let peer = peer.into_inner();
            state.alive.frag_lemma(&input.fragment);
            state.alive.frag_lemma(&peer.fragment);
            input.fragment.valid_op_lemma(&peer.fragment);
            proof_assert!(input.id != peer.id);
            proof_assert!(state.alive@.contains(peer.id));
            let before = snapshot!(state.live_map());
            let input_id = snapshot!(input.id);
            let result = Self::on_release(
                Ghost::new(&mut *state),
                Ghost::new(&mut *committer),
                Ghost::new(input),
                Ghost::new(&mut *current),
            ).into_inner();
            proof_assert!(state.live_map() == (*before).remove(*input_id));
            proof_assert!(state.live_map().contains(peer.id));
            proof_assert!({ contains_implies_nonempty(state.alive@, peer.id); true });
            proof_assert!(state.live_count() > 0);
            proof_assert!(result == None);
            result
        }
    }

    #[check(ghost)]
    #[requires(state.protocol() && pending.valid(state.public(), **current))]
    #[requires(!committer.shot_store() && committer.ward() == state.atomic())]
    #[ensures((^state).protocol() && (^state).public() == state.public())]
    #[ensures((^state).atomic() == state.atomic())]
    #[ensures(**current <= ^current && pending.valid(state.public(), ^current))]
    #[ensures(pending.acquired(^current))]
    #[ensures((^state).observe() == state.observe())]
    pub fn on_acquire(
        state: Ghost<&mut Self>,
        committer: Ghost<&Committer<ModelAtomic, usize, Acquire, NoStore>>,
        pending: Ghost<&Pending<T>>,
        current: Ghost<&mut SyncView>,
    ) -> Ghost<()> {
        ghost! {
            let state = state.into_inner();
            let committer = committer.into_inner();
            let pending = pending.into_inner();
            let current = current.into_inner();
            state.publication.frag_lemma(&pending.fragment);
            committer.shoot_load(&state.own, current);
            proof_assert!(committer.timestamp() == state.latest);
        }
    }
}

/// Negative control: an already issued exclusive ticket cannot be inserted
/// into the authority a second time.
#[cfg(feature = "negative_core_duplicate_ticket")]
#[check(ghost)]
#[requires(state.protocol() && ticket.valid(state.public()))]
pub fn negative_duplicate_ticket<T: RecoveryPayload>(
    state: Ghost<&mut State<T>>,
    ticket: Ghost<&Ticket<T>>,
) -> Ghost<()> {
    ghost! {
        let state = state.into_inner();
        let ticket = ticket.into_inner();
        state.alive.frag_lemma(&ticket.fragment);
        proof_assert!(state.alive@.contains(ticket.id));
        let duplicate = issue(
            &mut state.alive,
            snapshot!(ticket.id),
            snapshot!(ticket.token.frac()),
        );
        let _ = duplicate;
    }
}

#[logic]
#[requires(live.len() == 0)]
#[ensures(live == LiveFractions::empty())]
fn empty_live_map(live: LiveFractions) {
    proof_assert!(forall<id: Int> live.remove(id).len() >= 0);
    proof_assert!(forall<id: Int> live.get(id) == None);
    proof_assert!(live.ext_eq(LiveFractions::empty()));
}

/// A fragment proving presence in the finite live map rules out an empty map.
#[logic]
#[requires(live.get(id) != None)]
#[ensures(live.len() > 0)]
fn contains_implies_nonempty(live: LiveFractions, id: Int) {
    if live.len() == 0 {
        empty_live_map(live);
        proof_assert!(live.get(id) == None);
    }
}
