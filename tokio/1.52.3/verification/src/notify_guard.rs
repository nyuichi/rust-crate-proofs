use crate::publication::PublishedOnce;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;
use vstd::rwlock::{RwLock, RwLockPredicate, WriteHandle};

verus! {

pub open spec fn guarded_set_step<T>(
    before: MemContents<T>,
    input: T,
    result: Result<(), T>,
    after: MemContents<T>,
) -> bool {
    match before {
        MemContents::Uninit => result == Ok(()) && after == MemContents::Init(input),
        MemContents::Init(previous) => {
            result == Err(input) && after == MemContents::Init(previous)
        },
    }
}

pub struct NotifyLockState {
    lock_id: u64,
}

impl NotifyLockState {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
}

pub ghost struct NotifyLockPredicate {
    lock_id: u64,
}

impl NotifyLockPredicate {
    pub closed spec fn lock_id(self) -> u64 { self.lock_id }
}

impl RwLockPredicate<NotifyLockState> for NotifyLockPredicate {
    open spec fn inv(self, state: NotifyLockState) -> bool {
        state.lock_id() == self.lock_id()
    }
}

/// Linear guard issued by the verified lock. Its private write handle cannot
/// be cloned and must be consumed to return the protected state.
pub struct NotifyGuardPermission<'a> {
    state: NotifyLockState,
    handle: WriteHandle<'a, NotifyLockState, NotifyLockPredicate>,
}

impl<'a> NotifyGuardPermission<'a> {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.handle.rwlock().inv(self.state)
    }

    pub closed spec fn lock_id(&self) -> u64 { self.state.lock_id() }

    pub fn notify_waiters(self)
    {
        proof { use_type_invariant(&self); }
        let NotifyGuardPermission { state, handle } = self;
        handle.release_write(state);
    }
}

/// Body-proved concurrent mutex lifecycle used by SetOnce. vstd's verified
/// writer handle supplies the unique, non-cloneable capability; the remaining
/// production boundary is representation correspondence with Tokio's loom
/// mutex and `NotifyGuard`.
pub struct NotifyMutexModel {
    lock_id: u64,
    lock: RwLock<NotifyLockState, NotifyLockPredicate>,
}

impl NotifyMutexModel {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.lock.pred().lock_id() == self.lock_id
    }

    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }

    pub fn new(lock_id: u64) -> (result: Self)
        ensures
            result.lock_id() == lock_id,
    {
        let state = NotifyLockState { lock_id };
        let lock = RwLock::new(state, Ghost(NotifyLockPredicate { lock_id }));
        NotifyMutexModel { lock_id, lock }
    }

    pub fn lock(&self) -> (guard: NotifyGuardPermission<'_>)
        ensures
            guard.lock_id() == self.lock_id(),
    {
        proof { use_type_invariant(self); }
        let (state, handle) = self.lock.acquire_write();
        NotifyGuardPermission { state, handle }
    }
}

/// SetOnce refinement with the exact optimistic-check, lock, second-check,
/// publication, and consuming notification shape used by production.
pub struct GuardedSetOnce<T> {
    mutex: NotifyMutexModel,
    target: PublishedOnce<T>,
}

impl<T> GuardedSetOnce<T> {
    pub closed spec fn contents(&self) -> MemContents<T> { self.target.contents() }
    pub closed spec fn well_formed(&self) -> bool {
        self.target.well_formed()
    }

    pub fn empty(lock_id: u64) -> (result: Self)
        ensures
            result.well_formed(),
            result.contents() == MemContents::Uninit,
    {
        GuardedSetOnce {
            mutex: NotifyMutexModel::new(lock_id),
            target: PublishedOnce::empty(),
        }
    }

    pub fn set(&mut self, value: T) -> (result: Result<(), T>)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            guarded_set_step(old(self).contents(), value, result, final(self).contents()),
    {
        if self.target.initialized() {
            return Err(value);
        }

        let guard = self.mutex.lock();
        let result;
        if self.target.initialized() {
            result = Err(value);
        } else {
            self.target.publish(value);
            result = Ok(());
        }
        guard.notify_waiters();
        result
    }

    pub fn get<'a>(&'a self) -> (result: Option<&'a T>)
        requires
            self.well_formed(),
        ensures
            match result {
                Some(value) => self.contents() == MemContents::Init(*value),
                None => self.contents() == MemContents::Uninit,
            },
        no_unwind
    {
        self.target.get()
    }
}

pub fn verify_guard_is_returned_on_success_and_failure(
    first: u64,
    rejected: u64,
    lock_id: u64,
)
{
    let mut once = GuardedSetOnce::empty(lock_id);
    let first_result = once.set(first);
    assert(first_result == Ok(()));

    let rejected_result = once.set(rejected);
    assert(rejected_result == Err(rejected));

    let observed = once.get().unwrap();
    assert(*observed == first);
}

} // verus!
