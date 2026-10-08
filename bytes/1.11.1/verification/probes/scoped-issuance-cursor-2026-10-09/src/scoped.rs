//! Body-proved ledger updates over the generic closed-scope observation channel.
use super::*;
use crate::{event::{ScopedEventAtomic, EventProtocol, ScopedProtocol, RawAtomic, ScopeCursor}, lifecycle::{State, Pending, prepare}};
use creusot_std::{ghost::invariant::Protocol, logic::ra::excl::Excl};

pub struct Registry<T: RecoveryPayload> { atomic: ScopedEventAtomic<State<T>> }
impl<T: RecoveryPayload> Registry<T> {
    #[logic] pub fn public(self) -> <State<T> as Protocol>::Public { self.atomic.public() }
    #[logic] fn valid(self) -> bool { self.public().0 == self.atomic.model() }
    #[logic] pub fn accepts(self, cursor: ScopeCursor<State<T>>) -> bool {
        cursor.model() == self.atomic.model() && cursor.public() == self.public()
    }

    #[requires(payload.wellformed() && full.frac() == PositiveReal::from_int(1))]
    #[ensures(result.0.valid() && result.0.accepts(result.2.inner_logic()))]
    #[ensures(result.1.inner_logic().valid(result.0.public()))]
    #[ensures(result.0.public().3 == payload.metadata())]
    #[ensures((*result.2.inner_logic().observation()).0 == FMap::singleton(
        result.1.inner_logic().id(), Excl(result.1.inner_logic().fraction())))]
    #[ensures((*result.2.inner_logic().observation()).1 == 1)]
    pub fn new(payload: Ghost<T>, full: Ghost<LifetimeToken>) -> (Self, Ghost<Ticket<T>>, Ghost<ScopeCursor<State<T>>>) {
        let mut current = ghost! { SyncView::new().into_inner() };
        let (raw, own) = RawAtomic::new(1, current.borrow_mut());
        let setup = ghost! { State::initialize(own, snapshot!(*current), payload, full).into_inner() };
        let (state, ticket) = setup.split();
        let (atomic, cursor) = ScopedEventAtomic::bind(raw, state);
        (Self { atomic }, ticket, cursor)
    }

    #[requires(self.valid() && self.accepts(*cursor.inner_logic()) && source.valid(self.public()))]
    #[ensures(self.accepts(^cursor) && source.valid(self.public()))]
    #[ensures(result.1.inner_logic().valid(self.public()))]
    #[ensures(!(*cursor.inner_logic().observation()).0.contains(result.1.inner_logic().id()))]
    #[ensures((*(^cursor).observation()).0 == (*cursor.inner_logic().observation()).0.insert(
        result.1.inner_logic().id(), Excl(result.1.inner_logic().fraction())))]
    #[ensures((*(^cursor).observation()).1 == (*cursor.inner_logic().observation()).1 + 1)]
    #[ensures(result.0@ == (*cursor.inner_logic().observation()).0.len())]
    #[cfg_attr(feature = "negative_cursor_wrong_summary", ensures(
        (*( ^cursor).observation()).0 == (*cursor.inner_logic().observation()).0))]
    pub fn register(&self, source: Ghost<&Ticket<T>>, cursor: Ghost<&mut ScopeCursor<State<T>>>) -> (usize, Ghost<Ticket<T>>) {
        let mut output = ghost! { None::<Ticket<T>> };
        let mut current = ghost! { SyncView::new().into_inner() };
        let release = ghost! { ReleaseSyncView::new().into_inner() };
        let old = self.atomic.increment(cursor, ghost! {
            |state: &mut State<T>, committer: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>| {
                *output = Some(State::on_register(Ghost::new(state), Ghost::new(committer), source,
                    current.borrow_mut(), release).into_inner());
            }
        });
        (old, ghost! { output.into_inner().unwrap() })
    }

    #[requires(self.valid() && self.accepts(*cursor.inner_logic()) && ticket.valid(self.public()))]
    #[ensures(self.accepts(^cursor))]
    #[ensures((*(^cursor).observation()).0 == (*cursor.inner_logic().observation()).0.remove(ticket.id()))]
    #[ensures((*(^cursor).observation()).1 == (*cursor.inner_logic().observation()).1)]
    #[ensures(result.0 == ((*cursor.inner_logic().observation()).0.len() == 1))]
    #[ensures(result.0 == (result.1.inner_logic() != None))]
    #[ensures(result.0 ==> result.1.inner_logic().unwrap_logic().0.wellformed() &&
        result.1.inner_logic().unwrap_logic().0.metadata() == self.public().3 &&
        result.1.inner_logic().unwrap_logic().1.lft() == self.public().2 &&
        result.1.inner_logic().unwrap_logic().1.frac() == PositiveReal::from_int(1))]
    pub fn retire(&self, ticket: Ghost<Ticket<T>>, mut cursor: Ghost<&mut ScopeCursor<State<T>>>) -> (bool, Ghost<Option<(T, LifetimeToken)>>) {
        let (mut current, input) = prepare(ticket, snapshot!(self.public()));
        let mut pending = ghost! { None::<Pending<T>> };
        let old = self.atomic.decrement(ghost! { &mut **cursor }, ghost! {
            |state: &mut State<T>, committer: &mut Committer<ModelAtomic, usize, Relaxed, Release>| {
                *pending = State::on_release(Ghost::new(state), Ghost::new(committer), input, current.borrow_mut()).into_inner();
            }
        });
        if old == 1 {
            #[cfg(not(feature = "negative_core_missing_acquire"))]
            self.atomic.acquire(cursor, ghost! {
                |state: &mut State<T>, committer: &Committer<ModelAtomic, usize, Acquire, NoStore>| {
                    State::on_acquire(Ghost::new(state), Ghost::new(committer),
                        Ghost::new(pending.as_ref().unwrap()), current.borrow_mut());
                }
            });
            (true, ghost! { Some(Pending::recover(Ghost::new(pending.into_inner().unwrap()), snapshot!(self.public()), current).into_inner()) })
        } else { (false, ghost! { None }) }
    }
}
