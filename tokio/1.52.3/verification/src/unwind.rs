use crate::publication::PublishedOnce;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NotificationPhase {
    Idle,
    Waking,
    Complete,
    Unwound,
}

/// Explicit exceptional-state projection for the only SetOnce operation that
/// invokes arbitrary user code after publication: waking registered wakers.
pub struct NotificationUnwind<T> {
    cell: PublishedOnce<T>,
    phase: NotificationPhase,
}

impl<T> NotificationUnwind<T> {
    pub closed spec fn contents(&self) -> MemContents<T> { self.cell.contents() }
    pub closed spec fn phase(&self) -> NotificationPhase { self.phase }
    pub closed spec fn well_formed(&self) -> bool {
        self.cell.well_formed()
            && (self.phase == NotificationPhase::Idle ==> self.contents() == MemContents::Uninit)
            && (self.phase != NotificationPhase::Idle ==> self.contents().is_init())
    }

    pub fn empty() -> (result: Self)
        ensures
            result.well_formed(),
            result.contents() == MemContents::Uninit,
            result.phase() == NotificationPhase::Idle,
    {
        NotificationUnwind {
            cell: PublishedOnce::empty(),
            phase: NotificationPhase::Idle,
        }
    }

    /// Release publication occurs before control enters arbitrary Waker code.
    pub fn publish_and_start_waking(&mut self, value: T)
        requires
            old(self).well_formed(),
            old(self).phase() == NotificationPhase::Idle,
        ensures
            final(self).well_formed(),
            final(self).contents() == MemContents::Init(value),
            final(self).phase() == NotificationPhase::Waking,
        no_unwind
    {
        self.cell.publish(value);
        self.phase = NotificationPhase::Waking;
    }

    pub fn finish_waking(&mut self)
        requires
            old(self).well_formed(),
            old(self).phase() == NotificationPhase::Waking,
        ensures
            final(self).well_formed(),
            final(self).contents() == old(self).contents(),
            final(self).phase() == NotificationPhase::Complete,
        no_unwind
    {
        self.phase = NotificationPhase::Complete;
    }

    /// Logical exceptional edge corresponding to a Waker panic. It changes
    /// only notification progress; publication and ownership are preserved.
    pub fn unwind_waking(&mut self)
        requires
            old(self).well_formed(),
            old(self).phase() == NotificationPhase::Waking,
        ensures
            final(self).well_formed(),
            final(self).contents() == old(self).contents(),
            final(self).phase() == NotificationPhase::Unwound,
        no_unwind
    {
        self.phase = NotificationPhase::Unwound;
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
        self.cell.get()
    }
}

pub fn verify_notification_unwind_keeps_value(value: u64)
{
    let mut protocol = NotificationUnwind::empty();
    protocol.publish_and_start_waking(value);
    protocol.unwind_waking();
    assert(protocol.phase() == NotificationPhase::Unwound);
    let observed = protocol.get().unwrap();
    assert(*observed == value);
}

} // verus!
