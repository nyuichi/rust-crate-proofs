//! Creusot-facing model of the collector's exclusive epoch state machine.
//!
//! A zero participant stamp means unpinned. A nonzero stamp records one plus
//! the epoch in which that participant pinned. This makes epoch zero
//! representable without conflating it with the unpinned state.

use alloc::vec::Vec;
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, requires, variant, Int, Invariant, Seq, View,
};

/// A single-owner model of an epoch collector and its registered participants.
#[allow(missing_debug_implementations)]
pub struct Collector {
    epoch: usize,
    participants: Vec<usize>,
}

impl View for Collector {
    type ViewTy = (Int, Seq<usize>);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { (self.epoch@, self.participants@) }
    }
}

impl Invariant for Collector {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.epoch@ < usize::MAX@
                && self.participants@.len() <= usize::MAX@
                && forall<i: Int> 0 <= i && i < self.participants@.len()
                    ==> self.participants@[i]@ <= self.epoch@ + 1
        }
    }
}

impl Collector {
    /// Runtime-typed participant stamp for the current epoch.
    #[logic]
    pub fn current_stamp(self) -> usize {
        pearlite! { self.epoch + 1usize }
    }

    /// Creates an empty collector at the starting epoch.
    #[ensures(result@.0 == 0)]
    #[ensures(result@.1.len() == 0)]
    #[ensures(result.invariant())]
    pub fn new() -> Self {
        Self {
            epoch: 0,
            participants: Vec::new(),
        }
    }

    /// Returns the current logical epoch.
    #[ensures(result@ == self@.0)]
    pub fn epoch(&self) -> usize {
        self.epoch
    }

    /// Returns the number of registered participants.
    #[ensures(result@ == self@.1.len())]
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Registers an initially unpinned participant and returns its identifier.
    #[requires(self.invariant())]
    #[requires(self@.1.len() < usize::MAX@)]
    #[ensures(result@ == self@.1.len())]
    #[ensures((^self)@.0 == self@.0)]
    #[ensures((^self)@.1 == self@.1.push_back(0usize))]
    #[ensures((^self).invariant())]
    pub fn register_mut(&mut self) -> usize {
        let id = self.participants.len();
        self.participants.push(0);
        id
    }

    /// Returns whether a participant is pinned in any epoch.
    #[requires(id@ < self@.1.len())]
    #[ensures(result == (self@.1[id@] != 0usize))]
    pub fn is_pinned(&self, id: usize) -> bool {
        self.participants[id] != 0
    }

    /// Returns whether a participant is pinned in the current epoch.
    #[requires(self.invariant())]
    #[requires(id@ < self@.1.len())]
    #[ensures(result == (self@.1[id@] == self.current_stamp()))]
    pub fn is_pinned_in_current_epoch(&self, id: usize) -> bool {
        self.participants[id] == self.epoch + 1
    }

    /// Pins a participant in the current epoch.
    #[requires(self.invariant())]
    #[requires(id@ < self@.1.len())]
    #[ensures((^self)@.0 == self@.0)]
    #[ensures((^self)@.1 == self@.1.set(id@, self.current_stamp()))]
    #[ensures((^self).invariant())]
    pub fn pin_mut(&mut self, id: usize) {
        self.participants[id] = self.epoch + 1;
    }

    /// Unpins a participant without changing any other participant.
    #[requires(self.invariant())]
    #[requires(id@ < self@.1.len())]
    #[ensures((^self)@.0 == self@.0)]
    #[ensures((^self)@.1 == self@.1.set(id@, 0usize))]
    #[ensures((^self).invariant())]
    pub fn unpin_mut(&mut self, id: usize) {
        self.participants[id] = 0;
    }

    /// Checks whether every pinned participant observed the current epoch.
    #[requires(self.invariant())]
    #[ensures(result == (forall<i: Int> 0 <= i && i < self@.1.len()
        ==> self@.1[i] == 0usize || self@.1[i] == self.current_stamp()))]
    pub fn can_advance(&self) -> bool {
        let mut ready = true;
        let mut index = 0usize;
        #[invariant(index@ <= self@.1.len())]
        #[invariant(ready == (forall<i: Int> 0 <= i && i < index@
            ==> self@.1[i] == 0usize || self@.1[i] == self.current_stamp()))]
        #[variant(self@.1.len() - index@)]
        while index < self.participants.len() {
            let stamp = self.participants[index];
            if stamp != 0 && stamp != self.epoch + 1 {
                ready = false;
            }
            index += 1;
        }
        ready
    }

    /// Advances once exactly when all pinned participants are current.
    #[requires(self.invariant())]
    #[requires(self@.0 + 1 < usize::MAX@)]
    #[ensures(result == (forall<i: Int> 0 <= i && i < self@.1.len()
        ==> self@.1[i] == 0usize || self@.1[i] == self.current_stamp()))]
    #[ensures((^self)@.1 == self@.1)]
    #[ensures(result ==> (^self)@.0 == self@.0 + 1)]
    #[ensures(!result ==> (^self)@.0 == self@.0)]
    #[ensures((^self).invariant())]
    pub fn try_advance_mut(&mut self) -> bool {
        if self.can_advance() {
            self.epoch += 1;
            true
        } else {
            false
        }
    }

    /// Records the epoch to attach to a newly retired object.
    #[ensures(result@ == self@.0)]
    pub fn retirement_epoch(&self) -> usize {
        self.epoch
    }

    /// Checks the two-advancement reclamation threshold.
    #[requires(retired_epoch@ + 2 <= usize::MAX@)]
    #[ensures(result == (retired_epoch@ + 2 <= self@.0))]
    pub fn is_reclaimable(&self, retired_epoch: usize) -> bool {
        retired_epoch + 2 <= self.epoch
    }
}

impl Default for Collector {
    #[ensures(result@.0 == 0)]
    #[ensures(result@.1.len() == 0)]
    #[ensures(result.invariant())]
    fn default() -> Self {
        Self::new()
    }
}
