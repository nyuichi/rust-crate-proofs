use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

/// Repository-local vstd-style contract for std thread-local Cell mechanics.
/// Only this primitive correspondence is trusted by the Tokio refinements.
#[derive(Copy, Clone, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub enum LocalState<V> {
    Accessible(V),
    Borrowed,
    TornDown,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TryAccess {
    ClosureOnce,
    ClosureZeroTeardown,
}

pub struct ThreadLocalCell<V> {
    thread: u64,
    key: u64,
    state: LocalState<V>,
    access: Option<TryAccess>,
}

pub struct TlsSession<V> {
    thread: u64,
    key: u64,
    value: Option<V>,
    access: TryAccess,
    original: LocalState<V>,
    modified: bool,
}

impl<V: Copy> ThreadLocalCell<V> {
    pub closed spec fn thread(&self) -> u64 { self.thread }
    pub closed spec fn key(&self) -> u64 { self.key }
    pub closed spec fn state(&self) -> LocalState<V> { self.state }
    pub closed spec fn access(&self) -> Option<TryAccess> { self.access }

    pub fn new(thread: u64, key: u64, value: V) -> (result: Self)
        ensures result.thread() == thread, result.key() == key,
            result.state() == LocalState::Accessible(value),
            result.access().is_none(),
        no_unwind
    {
        ThreadLocalCell { thread, key, state: LocalState::Accessible(value), access: None }
    }

    pub fn teardown(&mut self, thread: u64)
        requires thread == old(self).thread(), matches!(old(self).state(), LocalState::Accessible(_)),
        ensures final(self).thread() == old(self).thread(),
            final(self).key() == old(self).key(),
            final(self).state() == LocalState::TornDown,
            final(self).access().is_none(),
        no_unwind
    {
        self.state = LocalState::TornDown;
        self.access = None;
    }

    /// Begins one dynamic try_with extent. Accessible state is moved into the
    /// linear session and the Cell cannot be accessed again until end.
    pub fn try_with_begin(&mut self, thread: u64) -> (session: TlsSession<V>)
        requires thread == old(self).thread(),
            !matches!(old(self).state(), LocalState::Borrowed),
        ensures final(self).thread() == old(self).thread(),
            final(self).key() == old(self).key(),
            session.well_formed(), !session.modified(),
            session.original() == old(self).state(),
            match old(self).state() {
                LocalState::Accessible(v) => {
                    &&& final(self).state() == LocalState::Borrowed
                    &&& session.value() == Some(v)
                    &&& session.thread() == old(self).thread()
                    &&& session.key() == old(self).key()
                    &&& session.access() == TryAccess::ClosureOnce
                    &&& final(self).access() == Some(TryAccess::ClosureOnce)
                },
                LocalState::TornDown => {
                    &&& final(self).state() == LocalState::TornDown
                    &&& session.value().is_none()
                    &&& session.thread() == old(self).thread()
                    &&& session.key() == old(self).key()
                    &&& session.access() == TryAccess::ClosureZeroTeardown
                    &&& final(self).access() == Some(TryAccess::ClosureZeroTeardown)
                },
                LocalState::Borrowed => false,
            },
        no_unwind
    {
        match self.state {
            LocalState::Accessible(value) => {
                self.state = LocalState::Borrowed;
                self.access = Some(TryAccess::ClosureOnce);
                TlsSession {
                    thread: self.thread,
                    key: self.key,
                    value: Some(value),
                    access: TryAccess::ClosureOnce,
                    original: LocalState::Accessible(value),
                    modified: false,
                }
            },
            LocalState::TornDown => {
                self.access = Some(TryAccess::ClosureZeroTeardown);
                TlsSession {
                    thread: self.thread,
                    key: self.key,
                    value: None,
                    access: TryAccess::ClosureZeroTeardown,
                    original: LocalState::TornDown,
                    modified: false,
                }
            },
            LocalState::Borrowed => unreached(),
        }
    }

    pub fn try_with_end(&mut self, session: TlsSession<V>)
        requires session.well_formed(), session.thread() == old(self).thread(), session.key() == old(self).key(),
            session.access() == TryAccess::ClosureOnce ==>
                matches!(old(self).state(), LocalState::Borrowed) && session.value().is_some(),
            session.access() == TryAccess::ClosureZeroTeardown ==>
                old(self).state() == LocalState::TornDown && session.value().is_none(),
        ensures final(self).thread() == old(self).thread(),
            final(self).key() == old(self).key(),
            session.access() == TryAccess::ClosureOnce ==>
                final(self).state() == LocalState::Accessible(session.value().unwrap()),
            session.access() == TryAccess::ClosureZeroTeardown ==>
                final(self).state() == LocalState::TornDown,
            final(self).access() == Some(session.access()),
        no_unwind
    {
        match session.access {
            TryAccess::ClosureOnce => {
                match session.value {
                    Some(value) => self.state = LocalState::Accessible(value),
                    None => unreached(),
                }
            },
            TryAccess::ClosureZeroTeardown => {},
        }
        self.access = Some(session.access);
    }
}

impl<V: Copy> TlsSession<V> {
    pub closed spec fn thread(&self) -> u64 { self.thread }
    pub closed spec fn key(&self) -> u64 { self.key }
    pub closed spec fn access(&self) -> TryAccess { self.access }
    pub closed spec fn value(&self) -> Option<V> { self.value }
    pub closed spec fn original(&self) -> LocalState<V> { self.original }
    pub closed spec fn modified(&self) -> bool { self.modified }
    pub closed spec fn well_formed(&self) -> bool {
        &&& ((self.access() == TryAccess::ClosureOnce) == self.value().is_some())
        &&& (self.access() == TryAccess::ClosureOnce ==>
            matches!(self.original, LocalState::Accessible(_)))
        &&& (self.access() == TryAccess::ClosureZeroTeardown ==>
            self.original == LocalState::TornDown)
        &&& (!self.modified() ==> match self.original() {
            LocalState::Accessible(v) => self.value() == Some(v),
            LocalState::TornDown => self.value().is_none(),
            LocalState::Borrowed => false,
        })
    }

    pub fn get(&self) -> (value: Option<V>)
        requires self.well_formed(),
        ensures value == self.value(),
            value.is_some() == (self.access() == TryAccess::ClosureOnce),
        no_unwind
    {
        self.value
    }

    pub fn set(&mut self, value: V)
        requires old(self).well_formed(), old(self).access() == TryAccess::ClosureOnce,
        ensures final(self).access() == old(self).access(),
            final(self).value() == Some(value), final(self).well_formed(),
            final(self).thread() == old(self).thread(),
            final(self).key() == old(self).key(),
            final(self).original() == old(self).original(),
            final(self).modified(),
        no_unwind
    {
        self.value = Some(value);
        self.modified = true;
    }
}

/// Body-proved outer callback wrapper (no external body or admitted premise):
/// successful TLS access executes the TLS
/// closure once; teardown executes it zero times, then invokes fallback. The
/// caller-owned callback executes exactly once in either case.
pub struct TotalCallback {
    tls_closure_runs: u8,
    callback_runs: u8,
}

impl TotalCallback {
    pub closed spec fn tls_closure_runs(&self) -> u8 { self.tls_closure_runs }
    pub closed spec fn callback_runs(&self) -> u8 { self.callback_runs }

    pub fn runs(&self) -> (result: (u8, u8))
        ensures result.0 == self.tls_closure_runs(),
            result.1 == self.callback_runs(),
        no_unwind
    {
        (self.tls_closure_runs, self.callback_runs)
    }

    pub fn from_access(access: TryAccess) -> (result: Self)
        ensures result.callback_runs() == 1,
            access == TryAccess::ClosureOnce ==> result.tls_closure_runs() == 1,
            access == TryAccess::ClosureZeroTeardown ==> result.tls_closure_runs() == 0,
        no_unwind
    {
        match access {
            TryAccess::ClosureOnce => TotalCallback { tls_closure_runs: 1, callback_runs: 1 },
            TryAccess::ClosureZeroTeardown => TotalCallback { tls_closure_runs: 0, callback_runs: 1 },
        }
    }
}

/// Body-proved scoped-binding/LIFO model. Only ThreadLocalCell's generic
/// standard-library TLS/Cell correspondence above is trusted.
pub struct ScopedBinding<V> {
    thread: u64,
    stack: Ghost<Seq<V>>,
    current: Option<V>,
}

pub struct ScopeGuard<V> {
    previous: Ghost<Seq<V>>,
    previous_current: Option<V>,
}

impl<V> ScopeGuard<V> {
    pub closed spec fn previous(&self) -> Seq<V> { self.previous@ }
    pub closed spec fn previous_current(&self) -> Option<V> { self.previous_current }
}

impl<V: Copy> ScopedBinding<V> {
    pub closed spec fn thread(&self) -> u64 { self.thread }
    pub closed spec fn stack(&self) -> Seq<V> { self.stack@ }
    pub closed spec fn current(&self) -> Option<V> { self.current }

    pub fn get_current(&self) -> (result: Option<V>)
        ensures result == self.current(),
        no_unwind
    {
        self.current
    }

    pub fn new(thread: u64) -> (result: Self)
        ensures result.thread() == thread, result.stack().len() == 0,
            result.current().is_none(),
        no_unwind
    {
        ScopedBinding { thread, stack: Ghost(Seq::empty()), current: None }
    }

    pub fn enter(&mut self, thread: u64, value: V) -> (guard: ScopeGuard<V>)
        requires thread == old(self).thread(),
        ensures final(self).thread() == old(self).thread(),
            final(self).stack() == old(self).stack().push(value),
            final(self).current() == Some(value),
            guard.previous() == old(self).stack(),
            guard.previous_current() == old(self).current(),
        no_unwind
    {
        let previous = self.stack;
        let previous_current = self.current;
        self.stack = Ghost(self.stack@.push(value));
        self.current = Some(value);
        ScopeGuard { previous, previous_current }
    }

    pub fn exit(&mut self, thread: u64, guard: ScopeGuard<V>)
        requires thread == old(self).thread(), old(self).stack().len() > 0,
            old(self).stack().drop_last() == guard.previous(),
        ensures final(self).thread() == old(self).thread(),
            final(self).stack() == guard.previous(),
            final(self).current() == guard.previous_current(),
        no_unwind
    {
        self.stack = guard.previous;
        self.current = guard.previous_current;
    }
}

} // verus!
