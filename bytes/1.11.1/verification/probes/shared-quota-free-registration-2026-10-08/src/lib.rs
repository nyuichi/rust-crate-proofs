#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]

use creusot_std::{
    ghost::{
        invariant::Protocol,
        lifetime_logic::{Lifetime, LifetimeToken},
        resource::{Authority, Fragment},
        Perm,
    },
    logic::{
        FMap, Id, Int,
        ra::{excl::Excl, RA},
        real::{PositiveReal, Real},
    },
    prelude::*,
    std::sync::{
        atomic::{AtomicUsize as ModelAtomic, ordering::Relaxed},
        committer::Committer,
        view::{HasTimestamp, ReleaseSyncView, SyncView},
    },
};

#[path = "../../trusted-atomic-event-2026-10-08/src/event.rs"]
mod event;
#[path = "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs"]
mod fraction_map;

use event::{EventAtomic, EventProtocol, RawAtomic};
use fraction_map::{LiveFractions, self as fm};

/// Every integer in `[0, next)` is present, and every other key is absent.
#[logic(open, inline)]
fn exact_live_prefix(live: LiveFractions, next: Int) -> bool {
    pearlite! {
        next >= 0 &&
        forall<id: Int> live.get(id) != None == (0 <= id && id < next)
    }
}

/// The exact-domain invariant after adding the previously absent `next` key.
#[logic]
#[requires(exact_live_prefix(live, next))]
#[ensures(exact_live_prefix(live.insert(next, Excl(fraction)), next + 1))]
fn insert_after_exact_prefix(live: LiveFractions, next: Int, fraction: PositiveReal) {
    proof_assert!(forall<id: Int>
        live.insert(next, Excl(fraction)).get(id) != None ==
            (0 <= id && id < next + 1));
}

/// Body-checked authority update that creates precisely one fresh id.
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

struct Ticket {
    id: Int,
    token: LifetimeToken,
    fragment: Fragment<LiveFractions>,
}

impl Ticket {
    #[logic(open(self), prophetic)]
    fn valid(self, public: <State as Protocol>::Public) -> bool {
        pearlite! {
            self.fragment.id() == public.1 &&
            self.fragment@ == FMap::singleton(self.id, Excl(self.token.frac())) &&
            self.token.lft() == public.2
        }
    }
}

/// Two owned tickets from the same authority cannot carry the same exclusive
/// map key. This uses the stock resource-algebra compatibility operation; it
/// does not assume uniqueness from an address, ticket count, or public state.
#[check(ghost)]
#[requires(first.valid(*public) && second.valid(*public))]
#[ensures((^first).valid(*public))]
#[ensures(second.valid(*public))]
#[ensures((^first).id == first.id)]
#[ensures((^first).id != second.id)]
fn tickets_are_distinct(
    first: &mut Ticket,
    second: &Ticket,
    public: Snapshot<<State as Protocol>::Public>,
) {
    first.fragment.valid_op_lemma(&second.fragment);
    proof_assert!(first.fragment@.op(second.fragment@) != None);
    proof_assert!(first.id != second.id);
}

struct State {
    own: Perm<ModelAtomic>,
    first: Int,
    latest: Int,
    alive: Authority<LiveFractions>,
    next: Int,
    pool: LifetimeToken,
    lifetime: Lifetime,
}

impl Protocol for State {
    type Public = (ModelAtomic, Id, Lifetime);

    #[logic]
    fn public(self) -> Self::Public {
        (*self.own.ward(), self.alive.id(), self.lifetime)
    }

    #[logic(prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
            self.next >= 1 &&
            exact_live_prefix(self.alive@, self.next) &&
            fm::ids_bounded(self.alive@, self.next) &&
            fm::cardinality_prefix(self.alive@, self.next) == self.next &&
            fm::sum_prefix(self.alive@, self.next) + self.pool.frac().to_real() == Real::from_int(1) &&
            self.pool.lft() == self.lifetime &&
            self.own.val().get(self.latest) != None &&
            self.own.val()[self.latest].0@ == self.next % (usize::MAX@ + 1) &&
            self.first <= self.latest &&
            forall<t: Int> self.own.val().get(t) != None ==
                (self.first <= t && t <= self.latest)
        }
    }
}

impl EventProtocol for State {
    #[logic]
    fn atomic(self) -> ModelAtomic {
        *self.own.ward()
    }
}

impl State {
    #[check(ghost)]
    #[requires(own.val() == FMap::singleton(
        own.ward().get_timestamp(*current), (1usize, *current)))]
    #[requires(full.frac() == PositiveReal::from_int(1))]
    #[ensures(result.inner_logic().0.protocol())]
    #[ensures(result.inner_logic().0.atomic() == *own.ward())]
    #[ensures(result.inner_logic().1.valid(result.inner_logic().0.public()))]
    #[ensures(result.inner_logic().1.id == 0)]
    #[ensures(result.inner_logic().1.token.frac() == PositiveReal::from_int(1) / PositiveReal::from_int(2))]
    pub fn initialize(
        own: Ghost<Perm<ModelAtomic>>,
        current: Snapshot<SyncView>,
        full: Ghost<LifetimeToken>,
    ) -> Ghost<(Self, Ticket)> {
        ghost! {
            let own = own.into_inner();
            let full = full.into_inner();
            let full_fraction = snapshot!(full.frac());
            let (first_token, pool) = full.split();
            let lifetime: Snapshot<Lifetime> = snapshot!(pool.lft());
            let first: Snapshot<Int> = snapshot!((*own.ward()).get_timestamp(*current));

            let mut alive = Authority::<LiveFractions>::alloc().into_inner();
            let zero: Snapshot<Int> = snapshot!(0);
            let one: Snapshot<Int> = snapshot!(1);
            let fraction = snapshot!(first_token.frac());
            let fragment = issue(&mut alive, zero, fraction);
            proof_assert!(alive@.ext_eq(FMap::singleton(*zero, Excl(*fraction))));
            proof_assert!(exact_live_prefix(alive@, *one));
            proof_assert!({ fm::empty_prefix(*zero); true });
            proof_assert!({ fm::insert_fresh(LiveFractions::empty(), *zero, *fraction); true });
            proof_assert!(fm::cardinality_prefix(alive@, *one) == *one);
            proof_assert!(fm::sum_prefix(alive@, *one) == fraction.to_real());
            proof_assert!(first_token.frac() + pool.frac() == *full_fraction);

            let ticket = Ticket {
                id: Int::new(0).into_inner(),
                token: first_token,
                fragment,
            };
            let state = State {
                own,
                first: first.into_ghost().into_inner(),
                latest: first.into_ghost().into_inner(),
                alive,
                next: Int::new(1).into_inner(),
                pool,
                lifetime: lifetime.into_ghost().into_inner(),
            };
            (state, ticket)
        }
    }

    /// Register a fresh ticket while retaining the borrowed source ticket.
    /// The only resource split comes from the state's residual LifetimeToken.
    #[check(ghost)]
    #[requires(state.protocol() && source.valid(state.public()))]
    #[requires(!committer.shot_store() && committer.ward() == state.atomic())]
    #[requires(if committer.val_load() == usize::MAX {
        committer.val_store() == 0usize
    } else {
        committer.val_store()@ == committer.val_load()@ + 1
    })]
    #[ensures((^state).protocol() && (^state).public() == state.public())]
    #[ensures((^committer).shot_store() && committer.hist_inv(^committer))]
    #[ensures(result.inner_logic().valid(state.public()))]
    #[ensures(result.inner_logic().id == state.next)]
    #[ensures(result.inner_logic().id > source.id)]
    #[ensures(source.valid(state.public()))]
    #[ensures((^state).next == state.next + 1)]
    pub fn register(
        state: Ghost<&mut Self>,
        committer: Ghost<&mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>,
        source: Ghost<&Ticket>,
        current: Ghost<&mut SyncView>,
        release: Ghost<ReleaseSyncView>,
    ) -> Ghost<Ticket> {
        ghost! {
            let state = state.into_inner();
            let committer = committer.into_inner();
            let source = source.into_inner();
            let current = current.into_inner();
            let release = release.into_inner();

            state.alive.frag_lemma(&source.fragment);
            proof_assert!(state.alive@.contains(source.id));
            proof_assert!(source.id < state.next);

            let next = snapshot!(state.next);
            let before = snapshot!(state.alive@);
            let pool_fraction = snapshot!(state.pool.frac());
            let token = state.pool.split_off();
            let fraction = snapshot!(token.frac());
            proof_assert!(token.frac() + state.pool.frac() == *pool_fraction);
            proof_assert!(before.get(*next) == None);

            let fragment = issue(&mut state.alive, next, fraction);
            proof_assert!({ fm::insert_fresh(*before, *next, *fraction); true });
            proof_assert!({ fm::insert_fresh_preserves_bound(*before, *next, *fraction); true });
            proof_assert!({ insert_after_exact_prefix(*before, *next, *fraction); true });
            proof_assert!(fm::cardinality_prefix(state.alive@, *next + 1) == *next + 1);
            proof_assert!(fm::sum_prefix(state.alive@, *next + 1) +
                state.pool.frac().to_real() == Real::from_int(1));

            let _previous = committer.shoot_load(&state.own, current);
            committer.shoot_store(&mut state.own, current, release);
            proof_assert!(committer.timestamp() == state.latest);
            let latest: Snapshot<Int> = snapshot!(committer.timestamp() + 1);
            state.latest = latest.into_ghost().into_inner();
            state.next = state.next + 1int;

            Ticket {
                id: next.into_ghost().into_inner(),
                token,
                fragment,
            }
        }
    }

    #[cfg(feature = "negative_forgotten_map")]
    #[check(ghost)]
    #[requires(state.protocol())]
    #[requires(!committer.shot_store() && committer.ward() == state.atomic())]
    #[requires(if committer.val_load() == usize::MAX {
        committer.val_store() == 0usize
    } else {
        committer.val_store()@ == committer.val_load()@ + 1
    })]
    #[ensures((^state).protocol())]
    pub fn register_without_map(
        state: Ghost<&mut Self>,
        committer: Ghost<&mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>,
        current: Ghost<&mut SyncView>,
        release: Ghost<ReleaseSyncView>,
    ) -> Ghost<()> {
        ghost! {
            let state = state.into_inner();
            let committer = committer.into_inner();
            let current = current.into_inner();
            let release = release.into_inner();
            let _previous = committer.shoot_load(&state.own, current);
            committer.shoot_store(&mut state.own, current, release);
            let latest: Snapshot<Int> = snapshot!(committer.timestamp() + 1);
            state.latest = latest.into_ghost().into_inner();
            state.next = state.next + 1int;
        }
    }
}

struct Registry {
    atomic: EventAtomic<State>,
}

impl Registry {
    #[logic]
    fn public(self) -> <State as Protocol>::Public {
        self.atomic.public()
    }

    #[logic(open(self))]
    fn valid(self) -> bool {
        self.public().0 == self.atomic.model()
    }

    #[ensures(result.0.valid())]
    #[ensures(result.1.inner_logic().valid(result.0.public()))]
    #[ensures(result.1.inner_logic().id == 0)]
    pub fn new() -> (Self, Ghost<Ticket>) {
        let mut current = ghost! { SyncView::new().into_inner() };
        let (raw, own) = RawAtomic::new(1, current.borrow_mut());
        let full = ghost! { LifetimeToken::new() };
        let setup = ghost! {
            State::initialize(own, snapshot!(*current), full).into_inner()
        };
        let (state, first) = setup.split();
        (Self { atomic: EventAtomic::bind(raw, state) }, first)
    }

    /// No clone quota is passed. The source remains shared-borrowed across the
    /// actual Relaxed increment and is preserved for a later registration.
    #[requires(self.valid() && source.valid(self.public()))]
    #[ensures(result.1.inner_logic().valid(self.public()))]
    #[ensures(result.1.inner_logic().id > source.id)]
    #[ensures(source.valid(self.public()))]
    #[ensures(result.0 <= (usize::MAX >> 1usize))]
    pub fn register(&self, source: Ghost<&Ticket>) -> (usize, Ghost<Ticket>) {
        let mut output = ghost! { None::<Ticket> };
        let mut current = ghost! { SyncView::new().into_inner() };
        let release = ghost! { ReleaseSyncView::new().into_inner() };
        let old = self.atomic.increment(ghost! {
            |state: &mut State,
             committer: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>| {
                *output = Some(State::register(
                    Ghost::new(state),
                    Ghost::new(committer),
                    source,
                    current.borrow_mut(),
                    release,
                ).into_inner());
            }
        });
        if old > usize::MAX >> 1 {
            abort_on_refcount_overflow();
        }
        proof_assert!(old <= (usize::MAX >> 1usize));
        (old, ghost! { output.into_inner().unwrap() })
    }

    #[cfg(feature = "negative_forgotten_map")]
    pub fn increment_without_map(&self) {
        let mut current = ghost! { SyncView::new().into_inner() };
        let release = ghost! { ReleaseSyncView::new().into_inner() };
        self.atomic.increment(ghost! {
            |state: &mut State,
             committer: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>| {
                let _ = State::register_without_map(
                    Ghost::new(state),
                    Ghost::new(committer),
                    current.borrow_mut(),
                    release,
                );
            }
        });
    }
}

/// Exactly two registrations through one still-live source distinguish this
/// transition from an external one-use quota.
#[ensures(result.0 <= (usize::MAX >> 1usize))]
#[ensures(result.1 <= (usize::MAX >> 1usize))]
pub fn two_registrations_from_one_source() -> (usize, usize) {
    let (registry, first) = Registry::new();
    let (old1, mut second) = registry.register(first.borrow());
    let (old2, third) = registry.register(first.borrow());
    proof_assert!(second.id != first.id && third.id != first.id);
    ghost! {
        tickets_are_distinct(
            second.borrow_mut().into_inner(),
            third.borrow().into_inner(),
            snapshot!(registry.public()),
        );
    };
    proof_assert!(second.inner_logic().id != third.inner_logic().id);
    proof_assert!(first.valid(registry.public()));
    proof_assert!(second.valid(registry.public()) && third.valid(registry.public()));
    (old1, old2)
}

#[trusted]
#[ensures(false)]
#[cold]
fn abort_on_refcount_overflow() {
    std::process::abort()
}

#[cfg(feature = "negative_duplicate")]
fn duplicate_ticket(ticket: Ticket) -> (Ticket, Ticket) {
    (ticket, ticket)
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;

    #[test]
    fn two_sequential_registrations_keep_same_source_live() {
        for _ in 0..32 {
            assert_eq!(two_registrations_from_one_source(), (1, 2));
        }
    }
}
