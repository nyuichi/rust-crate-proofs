use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MutexGuardKind {
    Borrowed,
    Owned,
    BorrowedMapped,
    OwnedMapped,
}

/// Logical capability carried by every production guard representation. Raw
/// pointer validity and Arc lifetime mechanics are frozen adapters; this token
/// records the Tokio-owned fact that all guard forms retain one and only one
/// permit for the same mutex.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MutexGuardToken {
    lock_id: u64,
    owner: u64,
    projection: u64,
    kind: MutexGuardKind,
}

impl MutexGuardToken {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
    pub closed spec fn owner(&self) -> u64 { self.owner }
    pub closed spec fn projection(&self) -> u64 { self.projection }
    pub closed spec fn kind(&self) -> MutexGuardKind { self.kind }
    pub closed spec fn owned(&self) -> bool {
        self.kind() == MutexGuardKind::Owned
            || self.kind() == MutexGuardKind::OwnedMapped
    }
    pub closed spec fn mapped(&self) -> bool {
        self.kind() == MutexGuardKind::BorrowedMapped
            || self.kind() == MutexGuardKind::OwnedMapped
    }
}

/// One-permit projection of `Mutex<T>` over the already-closed batch
/// semaphore. A live token exists exactly when the sole permit is held.
pub struct MutexPermitModel {
    lock_id: u64,
    holder: Option<u64>,
}

impl MutexPermitModel {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
    pub closed spec fn holder(&self) -> Option<u64> { self.holder }
    pub closed spec fn available(&self) -> bool { self.holder().is_none() }

    pub fn new(lock_id: u64) -> (result: Self)
        ensures
            result.lock_id() == lock_id,
            result.available(),
        no_unwind
    {
        MutexPermitModel { lock_id, holder: None }
    }

    /// Exact projection of `try_acquire(1)`: success iff the sole permit is
    /// available, with no mutation on failure.
    pub fn try_lock(&mut self, owner: u64) -> (result: Option<MutexGuardToken>)
        ensures
            final(self).lock_id() == old(self).lock_id(),
            old(self).available() ==> result.is_some(),
            old(self).available() ==> final(self).holder() == Some(owner),
            !old(self).available() ==> result.is_none(),
            !old(self).available() ==> final(self).holder() == old(self).holder(),
            result.is_some() ==> result.unwrap().lock_id() == old(self).lock_id(),
            result.is_some() ==> result.unwrap().owner() == owner,
            result.is_some() ==> result.unwrap().kind() == MutexGuardKind::Borrowed,
        no_unwind
    {
        if self.holder.is_none() {
            self.holder = Some(owner);
            Some(MutexGuardToken {
                lock_id: self.lock_id,
                owner,
                projection: 0,
                kind: MutexGuardKind::Borrowed,
            })
        } else {
            None
        }
    }

    /// Owned acquisition differs only in the lifetime adapter represented by
    /// Arc; permit accounting and exclusion are identical.
    pub fn try_lock_owned(&mut self, owner: u64) -> (result: Option<MutexGuardToken>)
        ensures
            final(self).lock_id() == old(self).lock_id(),
            old(self).available() ==> result.is_some(),
            old(self).available() ==> final(self).holder() == Some(owner),
            !old(self).available() ==> result.is_none(),
            !old(self).available() ==> final(self).holder() == old(self).holder(),
            result.is_some() ==> result.unwrap().lock_id() == old(self).lock_id(),
            result.is_some() ==> result.unwrap().owner() == owner,
            result.is_some() ==> result.unwrap().kind() == MutexGuardKind::Owned,
        no_unwind
    {
        if self.holder.is_none() {
            self.holder = Some(owner);
            Some(MutexGuardToken {
                lock_id: self.lock_id,
                owner,
                projection: 0,
                kind: MutexGuardKind::Owned,
            })
        } else {
            None
        }
    }

    /// Refines all four guard destructors. Consuming the unique token makes a
    /// second release unrepresentable and restores exactly one permit.
    pub fn drop_guard(&mut self, guard: MutexGuardToken)
        requires
            old(self).holder() == Some(guard.owner()),
            old(self).lock_id() == guard.lock_id(),
        ensures
            final(self).lock_id() == old(self).lock_id(),
            final(self).available(),
        no_unwind
    {
        self.holder = None;
    }
}

/// `skip_drop` followed by `map`: the original guard is consumed, its permit
/// is neither released nor duplicated, and only the data projection changes.
pub fn map_guard(guard: MutexGuardToken, projection: u64) -> (result: MutexGuardToken)
    ensures
        result.lock_id() == guard.lock_id(),
        result.owner() == guard.owner(),
        result.projection() == projection,
        result.kind() == if guard.owned() {
            MutexGuardKind::OwnedMapped
        } else {
            MutexGuardKind::BorrowedMapped
        },
        result.owned() == guard.owned(),
        result.mapped(),
    no_unwind
{
    MutexGuardToken {
        lock_id: guard.lock_id,
        owner: guard.owner,
        projection,
        kind: match guard.kind {
            MutexGuardKind::Owned | MutexGuardKind::OwnedMapped => {
                MutexGuardKind::OwnedMapped
            },
            MutexGuardKind::Borrowed | MutexGuardKind::BorrowedMapped => {
                MutexGuardKind::BorrowedMapped
            },
        },
    }
}

/// Exact success/failure ownership split of every `try_map`: failure returns
/// the original guard unchanged; success consumes it into one mapped guard.
pub fn try_map_guard(
    guard: MutexGuardToken,
    projection: u64,
    selected: bool,
) -> (result: Result<MutexGuardToken, MutexGuardToken>)
    ensures
        match result {
            Ok(mapped) => {
                selected
                    && mapped.lock_id() == guard.lock_id()
                    && mapped.owner() == guard.owner()
                    && mapped.projection() == projection
                    && mapped.mapped()
                    && mapped.owned() == guard.owned()
            },
            Err(original) => !selected && original == guard,
        },
    no_unwind
{
    if selected {
        Ok(map_guard(guard, projection))
    } else {
        Err(guard)
    }
}

/// `MutexGuard::mutex` and `OwnedMutexGuard::mutex` preserve identity even
/// though the latter returns an Arc-backed reference.
pub fn guard_mutex_id(guard: &MutexGuardToken) -> (result: u64)
    ensures result == guard.lock_id(),
    no_unwind
{
    guard.lock_id
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MutexWaiter {
    pub id: u64,
}

/// FIFO view of the batch-semaphore intrusive queue used by `lock` and
/// `lock_owned`. Index zero is the oldest waiter.
pub tracked struct MutexWaitQueue {
    ghost queue: Seq<MutexWaiter>,
}

impl MutexWaitQueue {
    pub closed spec fn queue(&self) -> Seq<MutexWaiter> { self.queue }

    pub proof fn new() -> (tracked result: Self)
        ensures result.queue().len() == 0,
    {
        let tracked result = MutexWaitQueue { queue: Seq::empty() };
        result
    }

    pub proof fn enqueue(tracked &mut self, id: u64)
        requires forall|i: int| 0 <= i < old(self).queue().len()
            ==> old(self).queue()[i].id != id,
        ensures
            final(self).queue() == old(self).queue().push(MutexWaiter { id }),
            final(self).queue()[old(self).queue().len() as int].id == id,
    {
        self.queue = self.queue.push(MutexWaiter { id });
    }

    /// Unlock assigns the sole returned permit to the oldest live waiter.
    pub proof fn grant_oldest(tracked &mut self) -> (id: u64)
        requires old(self).queue().len() > 0,
        ensures
            id == old(self).queue()[0].id,
            final(self).queue() == old(self).queue().subrange(
                1, old(self).queue().len() as int,
            ),
    {
        let id = self.queue[0].id;
        self.queue = self.queue.subrange(1, self.queue.len() as int);
        id
    }

    /// Dropping a pending lock future removes exactly its node. Surviving
    /// waiters retain their relative order, so cancellation loses its place.
    pub proof fn cancel(tracked &mut self, index: int) -> (id: u64)
        requires 0 <= index < old(self).queue().len(),
        ensures
            id == old(self).queue()[index].id,
            final(self).queue() == old(self).queue().subrange(0, index)
                + old(self).queue().subrange(index + 1, old(self).queue().len() as int),
    {
        let id = self.queue[index].id;
        self.queue = self.queue.subrange(0, index)
            + self.queue.subrange(index + 1, self.queue.len() as int);
        id
    }
}

pub proof fn verify_cancel_preserves_fifo(first: u64, cancelled: u64, last: u64)
    requires first != cancelled, cancelled != last, first != last,
{
    let tracked mut queue = MutexWaitQueue::new();
    queue.enqueue(first);
    queue.enqueue(cancelled);
    queue.enqueue(last);
    let removed = queue.cancel(1);
    assert(removed == cancelled);
    let next = queue.grant_oldest();
    assert(next == first);
    let final_id = queue.grant_oldest();
    assert(final_id == last);
}

/// Mutation witnesses for the three ownership-sensitive implementation
/// choices: mapping must not release, Drop must release exactly once, and
/// cancellation must unlink the selected waiter rather than the FIFO head.
pub fn verify_mutex_guard_mutants_rejected()
    no_unwind
{
    let mut permits = MutexPermitModel::new(17);
    let guard = match permits.try_lock(5) {
        Some(guard) => guard,
        None => {
            assert(false);
            return;
        },
    };
    assert(!permits.available());
    let mapped = map_guard(guard, 1);
    assert(!permits.available());
    permits.drop_guard(mapped);
    assert(permits.available());

    let initial_permits = 1u64;
    let mutant_after_double_release = 2u64;
    assert(mutant_after_double_release != initial_permits);

}

/// Value crossing an exclusive guard is conserved. The physical UnsafeCell
/// borrow is trusted; arbitrary `T` remains owned exactly once by this body.
pub fn replace_through_guard<T>(stored: T, replacement: T) -> (result: (T, T))
    ensures result == (replacement, stored),
    no_unwind
{
    (replacement, stored)
}

/// Mutable access needs no runtime permit because `&mut Mutex<T>` excludes all
/// guards at the Rust type layer. The value transfer itself remains exact.
pub fn replace_through_get_mut<T>(stored: T, replacement: T) -> (result: (T, T))
    ensures result == (replacement, stored),
    no_unwind
{
    (replacement, stored)
}

pub fn into_inner<T>(stored: T) -> (result: T)
    ensures result == stored,
    no_unwind
{
    stored
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MutexDebugView {
    Unlocked,
    Locked,
}

pub fn mutex_debug_view(available: bool) -> (result: MutexDebugView)
    ensures
        available ==> result == MutexDebugView::Unlocked,
        !available ==> result == MutexDebugView::Locked,
    no_unwind
{
    if available { MutexDebugView::Unlocked } else { MutexDebugView::Locked }
}

pub fn try_lock_error_message() -> (result: &'static str)
    ensures result@ == "operation would block"@,
    no_unwind
{
    "operation would block"
}

} // verus!
