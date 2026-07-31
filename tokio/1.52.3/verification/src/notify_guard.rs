use crate::publication::PublishedOnce;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

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

/// Linear permission issued by the waiter-list mutex. The private field and
/// lack of Clone make the capability non-duplicable by clients.
pub struct NotifyGuardPermission {
    lock_id: u64,
}

impl NotifyGuardPermission {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
}

/// Minimal body-proved mutex lifecycle needed by SetOnce. The remaining
/// production boundary is only that Tokio's loom Mutex implements this
/// acquire/release contract.
pub struct NotifyMutexModel {
    lock_id: u64,
    held: bool,
}

impl NotifyMutexModel {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
    pub closed spec fn held(&self) -> bool { self.held }

    pub fn new(lock_id: u64) -> (result: Self)
        ensures
            result.lock_id() == lock_id,
            !result.held(),
        no_unwind
    {
        NotifyMutexModel { lock_id, held: false }
    }

    pub fn lock(&mut self) -> (guard: NotifyGuardPermission)
        requires
            !old(self).held(),
        ensures
            final(self).held(),
            final(self).lock_id() == old(self).lock_id(),
            guard.lock_id() == final(self).lock_id(),
        no_unwind
    {
        self.held = true;
        NotifyGuardPermission { lock_id: self.lock_id }
    }

    /// Production `NotifyGuard::notify_waiters(self)` consumes the guard. This
    /// proof-view operation returns its exclusive capability to the mutex.
    pub fn notify_waiters(&mut self, guard: NotifyGuardPermission)
        requires
            old(self).held(),
            guard.lock_id() == old(self).lock_id(),
        ensures
            !final(self).held(),
            final(self).lock_id() == old(self).lock_id(),
        no_unwind
    {
        self.held = false;
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
    pub closed spec fn lock_held(&self) -> bool { self.mutex.held() }
    pub closed spec fn well_formed(&self) -> bool {
        self.target.well_formed() && !self.mutex.held()
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
        no_unwind
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
        self.mutex.notify_waiters(guard);
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
    assert(!once.lock_held());

    let rejected_result = once.set(rejected);
    assert(rejected_result == Err(rejected));
    assert(!once.lock_held());

    let observed = once.get().unwrap();
    assert(*observed == first);
}

} // verus!
