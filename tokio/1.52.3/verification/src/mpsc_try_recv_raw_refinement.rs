use crate::mpsc_queue_refinement::{RawMpscQueue, RawRead, RawValueSlot};
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RawTryPhase { NeedRegister, Registered, NeedPark, Terminal }

/// Arbitrary-length safety and conditional termination of production
/// `Chan::try_recv` after its initial raw Busy observation. The scheduler
/// liveness foundation supplies a finite number of Busy head observations for
/// the in-flight sender; `remaining_busy` is that single canonical rank. No
/// machine-word trace counter is used.
pub tracked struct RawTryRecvLoop {
    ghost remaining_busy: nat,
    ghost registrations: nat,
    ghost parks: nat,
    ghost phase: RawTryPhase,
}

impl RawTryRecvLoop {
    pub closed spec fn remaining_busy(&self) -> nat { self.remaining_busy }
    pub closed spec fn registrations(&self) -> nat { self.registrations }
    pub closed spec fn parks(&self) -> nat { self.parks }
    pub closed spec fn phase(&self) -> RawTryPhase { self.phase }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.parks() <= self.registrations()
        &&& self.registrations() <= self.parks() + 1
        &&& (self.phase() == RawTryPhase::Registered
            || self.phase() == RawTryPhase::NeedPark
            ==> self.registrations() == self.parks() + 1)
        &&& (self.phase() == RawTryPhase::NeedRegister
            ==> self.registrations() == self.parks())
    }

    pub proof fn new(remaining_busy: nat) -> (tracked result: Self)
        ensures result.well_formed(), result.remaining_busy() == remaining_busy,
            result.registrations() == 0, result.parks() == 0,
            result.phase() == RawTryPhase::NeedRegister,
    {
        let tracked result = RawTryRecvLoop {
            remaining_busy, registrations: 0, parks: 0,
            phase: RawTryPhase::NeedRegister,
        };
        result
    }

    /// AtomicWaker registration uses the already-proved S09 machine; the
    /// CachedParkThread Waker is obtained from the frozen park adapter before
    /// this transition.
    pub proof fn register(tracked &mut self)
        requires old(self).well_formed(), old(self).phase() == RawTryPhase::NeedRegister,
        ensures final(self).well_formed(), final(self).phase() == RawTryPhase::Registered,
            final(self).registrations() == old(self).registrations() + 1,
            final(self).parks() == old(self).parks(),
            final(self).remaining_busy() == old(self).remaining_busy(),
    {
        self.registrations = self.registrations + 1;
        self.phase = RawTryPhase::Registered;
    }

    pub proof fn observe_busy(tracked &mut self)
        requires old(self).well_formed(), old(self).phase() == RawTryPhase::Registered,
            old(self).remaining_busy() > 0,
        ensures final(self).well_formed(), final(self).phase() == RawTryPhase::NeedPark,
            final(self).remaining_busy() + 1 == old(self).remaining_busy(),
            final(self).registrations() == old(self).registrations(),
            final(self).parks() == old(self).parks(),
    {
        self.remaining_busy = (self.remaining_busy - 1) as nat;
        self.phase = RawTryPhase::NeedPark;
    }

    pub proof fn park(tracked &mut self)
        requires old(self).well_formed(), old(self).phase() == RawTryPhase::NeedPark,
        ensures final(self).well_formed(), final(self).phase() == RawTryPhase::NeedRegister,
            final(self).parks() == old(self).parks() + 1,
            final(self).registrations() == old(self).registrations(),
            final(self).remaining_busy() == old(self).remaining_busy(),
    {
        self.parks = self.parks + 1;
        self.phase = RawTryPhase::NeedRegister;
    }

    pub proof fn observe_terminal(tracked &mut self)
        requires old(self).well_formed(), old(self).phase() == RawTryPhase::Registered,
            old(self).remaining_busy() == 0,
        ensures final(self).well_formed(), final(self).phase() == RawTryPhase::Terminal,
            final(self).registrations() == old(self).registrations(),
            final(self).parks() == old(self).parks(),
            final(self).remaining_busy() == 0,
    {
        self.phase = RawTryPhase::Terminal;
    }

    pub proof fn finish_finite_prefix(tracked &mut self)
        requires old(self).well_formed(), old(self).phase() == RawTryPhase::NeedRegister,
        ensures final(self).well_formed(), final(self).phase() == RawTryPhase::Terminal,
            final(self).remaining_busy() == 0,
            final(self).registrations()
                == old(self).registrations() + old(self).remaining_busy() + 1,
            final(self).parks() == old(self).parks() + old(self).remaining_busy(),
        decreases self.remaining_busy(),
    {
        if self.remaining_busy > 0 {
            self.register();
            self.observe_busy();
            self.park();
            self.finish_finite_prefix();
        } else {
            self.register();
            self.observe_terminal();
        }
    }
}

/// Representative raw Busy is not an oracle: a later slot may already be
/// ready while the FIFO head is claimed but unpublished. Once the head's
/// release publication occurs, the same raw classifier produces Value and the
/// payload permission moves exactly once.
pub proof fn verify_raw_busy_register_park_then_value(
    value: u64,
)
{
    let tracked mut queue = RawMpscQueue::new(2);
    let first = queue.claim_value();
    let tracked mut first_slot = RawValueSlot::claimed(first);
    let second = queue.claim_value();
    let tracked mut second_slot = RawValueSlot::claimed(second);
    queue.publish_slot(&mut second_slot, value + 1);
    let initial = queue.classify_head();
    assert(initial == RawRead::Busy);

    let tracked mut loop_state = RawTryRecvLoop::new(1);
    loop_state.register();
    loop_state.observe_busy();
    loop_state.park();

    queue.publish_slot(&mut first_slot, value);
    loop_state.register();
    let ready = queue.classify_head();
    assert(ready == RawRead::Value);
    loop_state.observe_terminal();
    let received = queue.pop_slot(&mut first_slot);
    assert(received == value);
    assert(loop_state.registrations() == 2 && loop_state.parks() == 1);
}

pub proof fn verify_arbitrary_finite_busy_prefix_terminates(
    busy_iterations: nat,
)
{
    let tracked mut state = RawTryRecvLoop::new(busy_iterations);
    state.finish_finite_prefix();
    assert(state.phase() == RawTryPhase::Terminal);
    assert(state.registrations() == busy_iterations + 1);
    assert(state.parks() == busy_iterations);
}

} // verus!
