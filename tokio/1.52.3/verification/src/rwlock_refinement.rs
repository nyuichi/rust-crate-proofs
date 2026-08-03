use vstd::prelude::*;

verus! {

pub open spec fn rwlock_max_reads() -> nat { u32::MAX as nat / 8 }

pub fn production_max_reads() -> (result: u32)
    ensures result as nat == rwlock_max_reads(),
    no_unwind
{
    u32::MAX / 8
}

pub fn valid_max_readers(max_readers: u32) -> (result: bool)
    ensures result == (0 < max_readers as nat <= rwlock_max_reads()),
    no_unwind
{
    max_readers != 0 && max_readers <= production_max_reads()
}

#[derive(PartialEq, Eq)]
pub enum RwGuardMode {
    Read,
    Write,
}

/// Linear capability shared by borrowed/owned and mapped/unmapped production
/// guard representations. Arc and raw-pointer validity remain frozen adapters.
#[derive(PartialEq, Eq)]
pub struct RwGuardToken {
    lock_id: u64,
    owner: u64,
    mode: RwGuardMode,
    permits: u32,
    owned: bool,
    mapped: bool,
    projection: u64,
}

impl RwGuardToken {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
    pub closed spec fn owner(&self) -> u64 { self.owner }
    pub closed spec fn mode(&self) -> RwGuardMode { self.mode }
    pub closed spec fn permits(&self) -> u32 { self.permits }
    pub closed spec fn owned(&self) -> bool { self.owned }
    pub closed spec fn mapped(&self) -> bool { self.mapped }
    pub closed spec fn projection(&self) -> u64 { self.projection }
    pub closed spec fn downgradeable(&self) -> bool {
        self.mode() == RwGuardMode::Write && !self.mapped()
    }
}

/// Exact permit accounting above production's `Semaphore::new(mr)`: each
/// reader holds one permit and a writer holds all `mr` permits.
pub struct RwLockPermitModel {
    lock_id: u64,
    max_readers: u32,
    readers: u32,
    writer: Option<u64>,
}

impl RwLockPermitModel {
    pub closed spec fn lock_id(&self) -> u64 { self.lock_id }
    pub closed spec fn max_readers(&self) -> u32 { self.max_readers }
    pub closed spec fn readers(&self) -> u32 { self.readers }
    pub closed spec fn writer(&self) -> Option<u64> { self.writer }
    pub closed spec fn well_formed(&self) -> bool {
        &&& 0 < self.max_readers() as nat <= rwlock_max_reads()
        &&& self.readers() <= self.max_readers()
        &&& self.writer().is_some() ==> self.readers() == 0
    }
    pub closed spec fn available(&self) -> nat {
        if self.writer().is_some() {
            0
        } else {
            (self.max_readers() - self.readers()) as nat
        }
    }

    pub fn new(lock_id: u64, max_readers: u32) -> (result: Self)
        requires 0 < max_readers as nat <= rwlock_max_reads(),
        ensures
            result.well_formed(),
            result.lock_id() == lock_id,
            result.max_readers() == max_readers,
            result.readers() == 0,
            result.writer().is_none(),
            result.available() == max_readers as nat,
        no_unwind
    {
        RwLockPermitModel { lock_id, max_readers, readers: 0, writer: None }
    }

    pub fn try_read(&mut self, owner: u64, owned: bool) -> (result: Option<RwGuardToken>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).lock_id() == old(self).lock_id(),
            final(self).max_readers() == old(self).max_readers(),
            result.is_some() == (old(self).writer().is_none()
                && old(self).readers() < old(self).max_readers()),
            result.is_some() ==> final(self).readers() == old(self).readers() + 1,
            result.is_some() ==> final(self).writer().is_none(),
            result.is_some() ==> result.unwrap().mode() == RwGuardMode::Read,
            result.is_some() ==> result.unwrap().permits() == 1,
            result.is_some() ==> result.unwrap().lock_id() == old(self).lock_id(),
            result.is_some() ==> result.unwrap().owner() == owner,
            result.is_some() ==> result.unwrap().owned() == owned,
            result.is_some() ==> !result.unwrap().mapped(),
            !result.is_some() ==> final(self).readers() == old(self).readers(),
            !result.is_some() ==> final(self).writer() == old(self).writer(),
        no_unwind
    {
        if self.writer.is_none() && self.readers < self.max_readers {
            self.readers += 1;
            Some(RwGuardToken {
                lock_id: self.lock_id,
                owner,
                mode: RwGuardMode::Read,
                permits: 1,
                owned,
                mapped: false,
                projection: 0,
            })
        } else {
            None
        }
    }

    pub fn try_write(&mut self, owner: u64, owned: bool) -> (result: Option<RwGuardToken>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).lock_id() == old(self).lock_id(),
            final(self).max_readers() == old(self).max_readers(),
            result.is_some() == (old(self).writer().is_none() && old(self).readers() == 0),
            result.is_some() ==> final(self).writer() == Some(owner),
            result.is_some() ==> final(self).readers() == 0,
            result.is_some() ==> result.unwrap().mode() == RwGuardMode::Write,
            result.is_some() ==> result.unwrap().permits() == old(self).max_readers(),
            result.is_some() ==> result.unwrap().lock_id() == old(self).lock_id(),
            result.is_some() ==> result.unwrap().owner() == owner,
            result.is_some() ==> result.unwrap().owned() == owned,
            result.is_some() ==> !result.unwrap().mapped(),
            !result.is_some() ==> final(self).readers() == old(self).readers(),
            !result.is_some() ==> final(self).writer() == old(self).writer(),
        no_unwind
    {
        if self.writer.is_none() && self.readers == 0 {
            self.writer = Some(owner);
            Some(RwGuardToken {
                lock_id: self.lock_id,
                owner,
                mode: RwGuardMode::Write,
                permits: self.max_readers,
                owned,
                mapped: false,
                projection: 0,
            })
        } else {
            None
        }
    }

    pub fn drop_read(&mut self, guard: RwGuardToken)
        requires
            old(self).well_formed(),
            guard.mode() == RwGuardMode::Read,
            guard.permits() == 1,
            guard.lock_id() == old(self).lock_id(),
            old(self).writer().is_none(),
            old(self).readers() > 0,
        ensures
            final(self).well_formed(),
            final(self).lock_id() == old(self).lock_id(),
            final(self).max_readers() == old(self).max_readers(),
            final(self).readers() + 1 == old(self).readers(),
            final(self).writer().is_none(),
            final(self).available() == old(self).available() + 1,
        no_unwind
    {
        self.readers -= 1;
    }

    pub fn drop_write(&mut self, guard: RwGuardToken)
        requires
            old(self).well_formed(),
            guard.mode() == RwGuardMode::Write,
            guard.permits() == old(self).max_readers(),
            guard.lock_id() == old(self).lock_id(),
            old(self).writer() == Some(guard.owner()),
        ensures
            final(self).well_formed(),
            final(self).readers() == 0,
            final(self).writer().is_none(),
            final(self).available() == old(self).max_readers() as nat,
        no_unwind
    {
        self.writer = None;
    }

    /// Production retains one of the writer's `mr` permits and releases
    /// exactly `mr - 1`, atomically producing one read guard.
    pub fn downgrade(
        &mut self,
        guard: RwGuardToken,
        projection: u64,
    ) -> (result: RwGuardToken)
        requires
            old(self).well_formed(),
            guard.downgradeable(),
            guard.permits() == old(self).max_readers(),
            guard.lock_id() == old(self).lock_id(),
            old(self).writer() == Some(guard.owner()),
        ensures
            final(self).well_formed(),
            final(self).lock_id() == old(self).lock_id(),
            final(self).max_readers() == old(self).max_readers(),
            final(self).writer().is_none(),
            final(self).readers() == 1,
            final(self).available() + 1 == old(self).max_readers() as nat,
            result.lock_id() == guard.lock_id(),
            result.owner() == guard.owner(),
            result.mode() == RwGuardMode::Read,
            result.permits() == 1,
            result.owned() == guard.owned(),
            result.mapped(),
            result.projection() == projection,
        no_unwind
    {
        self.writer = None;
        self.readers = 1;
        RwGuardToken {
            lock_id: guard.lock_id,
            owner: guard.owner,
            mode: RwGuardMode::Read,
            permits: 1,
            owned: guard.owned,
            mapped: true,
            projection,
        }
    }

    pub fn try_downgrade(
        &mut self,
        guard: RwGuardToken,
        projection: u64,
        selected: bool,
    ) -> (result: Result<RwGuardToken, RwGuardToken>)
        requires
            old(self).well_formed(),
            guard.downgradeable(),
            guard.permits() == old(self).max_readers(),
            guard.lock_id() == old(self).lock_id(),
            old(self).writer() == Some(guard.owner()),
        ensures
            final(self).well_formed(),
            final(self).lock_id() == old(self).lock_id(),
            final(self).max_readers() == old(self).max_readers(),
            match result {
                Ok(read) => {
                    selected
                        && final(self).writer().is_none()
                        && final(self).readers() == 1
                        && read.mode() == RwGuardMode::Read
                        && read.permits() == 1
                        && read.mapped()
                        && read.projection() == projection
                },
                Err(original) => {
                    !selected
                        && original == guard
                        && final(self).writer() == old(self).writer()
                        && final(self).readers() == old(self).readers()
                },
            },
        no_unwind
    {
        if selected {
            Ok(self.downgrade(guard, projection))
        } else {
            Err(guard)
        }
    }
}

/// Read maps and write maps consume the original guard without changing its
/// permit count. Mapping a write guard intentionally disables downgrade.
pub fn map_guard(guard: RwGuardToken, projection: u64) -> (result: RwGuardToken)
    ensures
        result.lock_id() == guard.lock_id(),
        result.owner() == guard.owner(),
        result.mode() == guard.mode(),
        result.permits() == guard.permits(),
        result.owned() == guard.owned(),
        result.mapped(),
        result.projection() == projection,
        guard.mode() == RwGuardMode::Write ==> !result.downgradeable(),
    no_unwind
{
    RwGuardToken {
        lock_id: guard.lock_id,
        owner: guard.owner,
        mode: guard.mode,
        permits: guard.permits,
        owned: guard.owned,
        mapped: true,
        projection,
    }
}

pub fn try_map_guard(
    guard: RwGuardToken,
    projection: u64,
    selected: bool,
) -> (result: Result<RwGuardToken, RwGuardToken>)
    ensures
        match result {
            Ok(mapped) => {
                selected
                    && mapped.lock_id() == guard.lock_id()
                    && mapped.owner() == guard.owner()
                    && mapped.mode() == guard.mode()
                    && mapped.permits() == guard.permits()
                    && mapped.owned() == guard.owned()
                    && mapped.mapped()
                    && mapped.projection() == projection
            },
            Err(original) => !selected && original == guard,
        },
    no_unwind
{
    if selected { Ok(map_guard(guard, projection)) } else { Err(guard) }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RwRequestKind {
    Read,
    Write,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RwWaitRequest {
    pub id: u64,
    pub kind: RwRequestKind,
    pub requested: nat,
    pub remaining: nat,
}

/// Logical FIFO view of production's reversed intrusive semaphore list.
pub tracked struct RwWaitQueue {
    ghost queue: Seq<RwWaitRequest>,
}

impl RwWaitQueue {
    pub closed spec fn queue(&self) -> Seq<RwWaitRequest> { self.queue }

    pub proof fn new() -> (tracked result: Self)
        ensures result.queue().len() == 0,
    {
        let tracked result = RwWaitQueue { queue: Seq::empty() };
        result
    }

    pub proof fn enqueue_read(tracked &mut self, id: u64)
        ensures final(self).queue() == old(self).queue().push(RwWaitRequest {
            id, kind: RwRequestKind::Read, requested: 1, remaining: 1,
        }),
    {
        self.queue = self.queue.push(RwWaitRequest {
            id, kind: RwRequestKind::Read, requested: 1, remaining: 1,
        });
    }

    pub proof fn enqueue_write(tracked &mut self, id: u64, max_readers: nat)
        requires max_readers > 0,
        ensures final(self).queue() == old(self).queue().push(RwWaitRequest {
            id, kind: RwRequestKind::Write,
            requested: max_readers, remaining: max_readers,
        }),
    {
        self.queue = self.queue.push(RwWaitRequest {
            id, kind: RwRequestKind::Write,
            requested: max_readers, remaining: max_readers,
        });
    }

    /// Only the oldest request receives returned permits. A partially funded
    /// writer therefore prevents later readers from bypassing it.
    pub proof fn assign_front(tracked &mut self, permits: nat) -> (completed: bool)
        requires
            old(self).queue().len() > 0,
            old(self).queue()[0].remaining > 0,
        ensures
            completed == (permits >= old(self).queue()[0].remaining),
            completed ==> final(self).queue() == old(self).queue().subrange(
                1, old(self).queue().len() as int,
            ),
            !completed ==> final(self).queue().len() == old(self).queue().len(),
            !completed ==> final(self).queue()[0].id == old(self).queue()[0].id,
            !completed ==> final(self).queue()[0].kind == old(self).queue()[0].kind,
            !completed ==> final(self).queue()[0].remaining
                == old(self).queue()[0].remaining - permits,
            !completed ==> forall|i: int| 1 <= i < final(self).queue().len()
                ==> final(self).queue()[i] == old(self).queue()[i],
    {
        let front = self.queue[0];
        if permits >= front.remaining {
            self.queue = self.queue.subrange(1, self.queue.len() as int);
            true
        } else {
            self.queue = self.queue.update(0, RwWaitRequest {
                id: front.id,
                kind: front.kind,
                requested: front.requested,
                remaining: (front.remaining - permits) as nat,
            });
            false
        }
    }

    pub proof fn cancel(tracked &mut self, index: int) -> (removed: RwWaitRequest)
        requires 0 <= index < old(self).queue().len(),
        ensures
            removed == old(self).queue()[index],
            final(self).queue() == old(self).queue().subrange(0, index)
                + old(self).queue().subrange(index + 1, old(self).queue().len() as int),
    {
        let removed = self.queue[index];
        self.queue = self.queue.subrange(0, index)
            + self.queue.subrange(index + 1, self.queue.len() as int);
        removed
    }
}

pub proof fn verify_writer_preference(writer: u64, reader: u64, max_readers: nat)
    requires writer != reader, max_readers > 1,
{
    let tracked mut queue = RwWaitQueue::new();
    queue.enqueue_write(writer, max_readers);
    queue.enqueue_read(reader);
    let completed = queue.assign_front((max_readers - 1) as nat);
    assert(!completed);
    assert(queue.queue()[0].id == writer);
    assert(queue.queue()[0].remaining == 1);
    assert(queue.queue()[1].id == reader);
}

pub proof fn verify_cancelled_writer_loses_place(writer: u64, reader: u64, max_readers: nat)
    requires writer != reader, max_readers > 0,
{
    let tracked mut queue = RwWaitQueue::new();
    queue.enqueue_write(writer, max_readers);
    queue.enqueue_read(reader);
    let removed = queue.cancel(0);
    assert(removed.id == writer);
    let completed = queue.assign_front(1);
    assert(completed);
    assert(queue.queue().len() == 0);
}

pub fn replace_through_write_guard<T>(stored: T, replacement: T) -> (result: (T, T))
    ensures result == (replacement, stored),
    no_unwind
{
    (replacement, stored)
}

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
pub enum RwLockDebugView {
    Data,
    Locked,
}

pub fn debug_view(can_read: bool) -> (result: RwLockDebugView)
    ensures
        can_read ==> result == RwLockDebugView::Data,
        !can_read ==> result == RwLockDebugView::Locked,
    no_unwind
{
    if can_read { RwLockDebugView::Data } else { RwLockDebugView::Locked }
}

pub fn verify_rwlock_guard_mutants_rejected()
    no_unwind
{
    assert(0 < 4u32 as nat <= rwlock_max_reads()) by {
        reveal(rwlock_max_reads);
    }
    let mut lock = RwLockPermitModel::new(9, 4);
    let writer = match lock.try_write(3, false) {
        Some(writer) => writer,
        None => { assert(false); return; },
    };
    assert(writer.downgradeable());
    assert(lock.available() == 0);
    let reader = lock.downgrade(writer, 0);
    assert(lock.readers() == 1);
    assert(lock.available() == 3);
    lock.drop_read(reader);
    assert(lock.available() == 4);
    let writer = match lock.try_write(7, true) {
        Some(writer) => writer,
        None => {
            assert(false);
            return;
        }
    };
    let mapped = map_guard(writer, 22);
    assert(mapped.permits() == 4);
    assert(lock.available() == 0);
    lock.drop_write(mapped);
    assert(lock.available() == 4);
    assert(4u32 - 1 == 3u32);
    assert(4u32 != 1u32);
}

} // verus!
