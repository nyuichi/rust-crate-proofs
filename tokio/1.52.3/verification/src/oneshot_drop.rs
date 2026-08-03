use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

pub struct InnerCleanup<T> {
    pub value: Option<T>,
    pub rx_waker: Option<u64>,
    pub tx_waker: Option<u64>,
}

/// Ownership view of the two Arc handles and the resources destroyed by
/// `Inner<T>::drop`. The cleanup bundle is produced by exactly the endpoint
/// which releases the final strong reference.
pub struct OneshotArcModel<T> {
    tx_alive: bool,
    rx_alive: bool,
    inner_dropped: bool,
    value: Option<T>,
    rx_waker: Option<u64>,
    tx_waker: Option<u64>,
}

impl<T> OneshotArcModel<T> {
    pub closed spec fn tx_alive(&self) -> bool { self.tx_alive }
    pub closed spec fn rx_alive(&self) -> bool { self.rx_alive }
    pub closed spec fn inner_dropped(&self) -> bool { self.inner_dropped }
    pub closed spec fn value(&self) -> Option<T> { self.value }
    pub closed spec fn rx_waker(&self) -> Option<u64> { self.rx_waker }
    pub closed spec fn tx_waker(&self) -> Option<u64> { self.tx_waker }

    pub closed spec fn well_formed(&self) -> bool {
        self.inner_dropped() ==> {
            !self.tx_alive() && !self.rx_alive()
                &&
                self.value().is_none()
                    && self.rx_waker().is_none()
                    && self.tx_waker().is_none()
            }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.tx_alive(),
            result.rx_alive(),
            !result.inner_dropped(),
            result.value().is_none(),
    {
        OneshotArcModel {
            tx_alive: true,
            rx_alive: true,
            inner_dropped: false,
            value: None,
            rx_waker: None,
            tx_waker: None,
        }
    }

    pub fn store_value(&mut self, value: T)
        requires
            old(self).well_formed(),
            old(self).tx_alive(),
            old(self).value().is_none(),
        ensures
            final(self).well_formed(),
            final(self).value() == Some(value),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
        no_unwind
    {
        self.value = Some(value);
    }

    pub fn set_rx_waker(&mut self, waker: u64)
        requires old(self).well_formed(), old(self).rx_alive(),
        ensures
            final(self).well_formed(),
            final(self).rx_waker() == Some(waker),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).value() == old(self).value(),
            final(self).tx_waker() == old(self).tx_waker(),
        no_unwind
    {
        self.rx_waker = Some(waker);
    }

    pub fn set_tx_waker(&mut self, waker: u64)
        requires old(self).well_formed(), old(self).tx_alive(),
        ensures
            final(self).well_formed(),
            final(self).tx_waker() == Some(waker),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).value() == old(self).value(),
            final(self).rx_waker() == old(self).rx_waker(),
        no_unwind
    {
        self.tx_waker = Some(waker);
    }

    fn cleanup_if_last(&mut self) -> (cleanup: Option<InnerCleanup<T>>)
        requires
            old(self).well_formed(),
            !old(self).inner_dropped(),
        ensures
            final(self).well_formed(),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            match cleanup {
                Some(cleanup) => {
                    !old(self).tx_alive()
                        && !old(self).rx_alive()
                        && cleanup.value == old(self).value()
                        && cleanup.rx_waker == old(self).rx_waker()
                        && cleanup.tx_waker == old(self).tx_waker()
                        && final(self).inner_dropped()
                },
                None => {
                    (old(self).tx_alive() || old(self).rx_alive())
                        && !final(self).inner_dropped()
                        && final(self).value() == old(self).value()
                        && final(self).rx_waker() == old(self).rx_waker()
                        && final(self).tx_waker() == old(self).tx_waker()
                },
            },
        no_unwind
    {
        if !self.tx_alive && !self.rx_alive {
            let cleanup = InnerCleanup {
                value: self.value.take(),
                rx_waker: self.rx_waker.take(),
                tx_waker: self.tx_waker.take(),
            };
            self.inner_dropped = true;
            Some(cleanup)
        } else {
            None
        }
    }

    pub fn finish_sender(&mut self) -> (cleanup: Option<InnerCleanup<T>>)
        requires
            old(self).well_formed(),
            old(self).tx_alive(),
        ensures
            final(self).well_formed(),
            !final(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            cleanup.is_some() == !old(self).rx_alive(),
            cleanup.is_some() ==> final(self).inner_dropped(),
            old(self).rx_alive() ==> final(self).value() == old(self).value(),
            old(self).rx_alive() ==> final(self).rx_waker() == old(self).rx_waker(),
            old(self).rx_alive() ==> final(self).tx_waker() == old(self).tx_waker(),
            match cleanup {
                Some(cleanup) => {
                    cleanup.value == old(self).value()
                        && cleanup.rx_waker == old(self).rx_waker()
                        && cleanup.tx_waker == old(self).tx_waker()
                },
                None => true,
            },
        no_unwind
    {
        self.tx_alive = false;
        self.cleanup_if_last()
    }

    pub fn finish_receiver(&mut self) -> (result: (Option<T>, Option<InnerCleanup<T>>))
        requires
            old(self).well_formed(),
            old(self).rx_alive(),
        ensures
            final(self).well_formed(),
            !final(self).rx_alive(),
            final(self).tx_alive() == old(self).tx_alive(),
            result.0 == old(self).value(),
            final(self).value().is_none(),
            result.1.is_some() == !old(self).tx_alive(),
            result.1.is_some() ==> final(self).inner_dropped(),
            match result.1 {
                Some(cleanup) => {
                    cleanup.value.is_none()
                        && cleanup.rx_waker == old(self).rx_waker()
                        && cleanup.tx_waker == old(self).tx_waker()
                },
                None => true,
            },
        no_unwind
    {
        let value = self.value.take();
        self.rx_alive = false;
        let cleanup = self.cleanup_if_last();
        (value, cleanup)
    }
}

pub fn verify_sender_then_receiver_cleanup(value: u64, rx_waker: u64, tx_waker: u64)
{
    let mut model = OneshotArcModel::new();
    model.store_value(value);
    model.set_rx_waker(rx_waker);
    model.set_tx_waker(tx_waker);
    let sender_cleanup = model.finish_sender();
    assert(sender_cleanup.is_none());
    let (received, final_cleanup) = model.finish_receiver();
    assert(received == Some(value));
    let cleanup = match final_cleanup {
        Some(cleanup) => cleanup,
        None => unreached(),
    };
    assert(cleanup.value.is_none());
    assert(cleanup.rx_waker == Some(rx_waker));
    assert(cleanup.tx_waker == Some(tx_waker));
    assert(model.inner_dropped());
}

pub fn verify_receiver_then_sender_cleanup(value: u64)
{
    let mut model = OneshotArcModel::new();
    model.store_value(value);
    let (dropped_value, receiver_cleanup) = model.finish_receiver();
    assert(dropped_value == Some(value));
    assert(receiver_cleanup.is_none());
    let final_cleanup = model.finish_sender();
    let cleanup = match final_cleanup {
        Some(cleanup) => cleanup,
        None => unreached(),
    };
    assert(cleanup.value.is_none());
    assert(model.inner_dropped());
}

} // verus!
