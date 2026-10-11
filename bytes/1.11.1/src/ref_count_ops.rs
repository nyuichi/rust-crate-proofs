use crate::loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(creusot)]
use creusot_std::ghost::Ghost;

/// Opaque identity for one native reference counter.
///
/// Its only operations are the event methods below. In the proof profile each
/// method is tied to the exact invariant stored next to this value in a
/// `RefCountCell`; in native builds the same method performs the same atomic
/// operation and ordering.
///
/// The proof contracts below are deliberately sequential: they assume every
/// access to this exact counter in the verified call cone is made through
/// these methods on one thread. They do not specify concurrent load/coherence
/// behavior. Native builds retain the original atomic implementation and
/// remain usable with the crate's normal concurrent semantics.
#[cfg_attr(creusot, opaque)]
#[repr(transparent)]
pub(crate) struct NativeCounter(AtomicUsize);

const _: [(); core::mem::size_of::<AtomicUsize>()] = [(); core::mem::size_of::<NativeCounter>()];
const _: [(); core::mem::align_of::<AtomicUsize>()] = [(); core::mem::align_of::<NativeCounter>()];
const _: [(); 0] = [(); core::mem::needs_drop::<NativeCounter>() as usize];

impl NativeCounter {
    #[cfg(creusot)]
    #[trusted]
    #[check(terminates)]
    #[ensures(result.0 == *result.1.ward())]
    #[ensures(result.1.val() == value)]
    fn new(value: usize) -> (Self, Ghost<creusot_std::ghost::Perm<Self>>) {
        (Self(AtomicUsize::new(value)), Ghost::conjure())
    }

    #[cfg(not(creusot))]
    fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    // These native bodies are private to this module. In the proof profile
    // only the trusted event adapters call them; ordinary crate callers use
    // RefCountCell, which always carries the matching invariant.
    #[cfg_attr(creusot, creusot::no_translate)]
    fn load_relaxed_native(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }

    #[cfg_attr(creusot, creusot::no_translate)]
    fn load_acquire_native(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    #[cfg_attr(creusot, creusot::no_translate)]
    fn compare_exchange_relaxed_native(&self, expected: usize, new: usize) -> Result<usize, usize> {
        self.0
            .compare_exchange(expected, new, Ordering::Relaxed, Ordering::Relaxed)
    }

    #[cfg_attr(creusot, creusot::no_translate)]
    fn compare_exchange_acqrel_relaxed_native(
        &self,
        expected: usize,
        new: usize,
    ) -> Result<usize, usize> {
        self.0
            .compare_exchange(expected, new, Ordering::AcqRel, Ordering::Relaxed)
    }

    #[cfg_attr(creusot, creusot::no_translate)]
    fn fetch_sub_release_native(&self) -> usize {
        self.0.fetch_sub(1, Ordering::Release)
    }
}

/// The current value and ghost state associated with one exact native
/// counter. The proof-only sidecar is zero-sized and does not change layout.
#[repr(C)]
pub(crate) struct RefCountCell {
    counter: NativeCounter,
    #[cfg(creusot)]
    invariant: Ghost<RefCountInvariant>,
}

const _: [(); core::mem::size_of::<AtomicUsize>()] = [(); core::mem::size_of::<RefCountCell>()];
const _: [(); core::mem::align_of::<AtomicUsize>()] = [(); core::mem::align_of::<RefCountCell>()];
const _: [(); 0] = [(); core::mem::needs_drop::<RefCountCell>() as usize];

impl RefCountCell {
    #[cfg_attr(creusot, ensures(result.invariant()))]
    pub(crate) fn new(value: usize) -> Self {
        #[cfg(creusot)]
        {
            let (counter, owner) = NativeCounter::new(value);
            let invariant =
                EventBoundInvariant::new(&counter, owner, ghost!(RefCountState { current: value }));
            Self { counter, invariant }
        }

        #[cfg(not(creusot))]
        {
            Self {
                counter: NativeCounter::new(value),
            }
        }
    }

    #[cfg_attr(creusot, requires(self.invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant()))]
    pub(crate) fn load_relaxed(&self) -> usize {
        #[cfg(creusot)]
        {
            self.counter
                .load_relaxed_with(
                    ghost!(&*self.invariant),
                    ghost!(|state: &mut RefCountState, value| { observe_ref_count(state, value) }),
                )
                .0
        }

        #[cfg(not(creusot))]
        {
            self.counter.load_relaxed_native()
        }
    }

    #[cfg_attr(creusot, requires(self.invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant()))]
    pub(crate) fn load_acquire(&self) -> usize {
        #[cfg(creusot)]
        {
            self.counter
                .load_acquire_with(
                    ghost!(&*self.invariant),
                    ghost!(|state: &mut RefCountState, value| { observe_ref_count(state, value) }),
                )
                .0
        }

        #[cfg(not(creusot))]
        {
            self.counter.load_acquire_native()
        }
    }

    #[cfg_attr(creusot, requires(self.invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant()))]
    #[cfg_attr(creusot, ensures(match result {
        Ok(old) => old == expected,
        Err(actual) => actual != expected,
    }))]
    pub(crate) fn compare_exchange_relaxed(
        &self,
        expected: usize,
        new: usize,
    ) -> Result<usize, usize> {
        #[cfg(creusot)]
        {
            let (result, callback) = self.counter.compare_exchange_relaxed_with(
                ghost!(&*self.invariant),
                expected,
                new,
                ghost!(|state: &mut RefCountState, old, new| { update_ref_count(state, old, new) }),
            );
            ghost! {
                match callback.into_inner() {
                    Ok(()) | Err(_) => (),
                }
            };
            result
        }

        #[cfg(not(creusot))]
        {
            self.counter.compare_exchange_relaxed_native(expected, new)
        }
    }

    #[cfg_attr(creusot, requires(self.invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant()))]
    #[cfg_attr(creusot, ensures(match result {
        Ok(old) => old == expected,
        Err(actual) => actual != expected,
    }))]
    pub(crate) fn compare_exchange_acqrel_relaxed(
        &self,
        expected: usize,
        new: usize,
    ) -> Result<usize, usize> {
        #[cfg(creusot)]
        {
            let (result, callback) = self.counter.compare_exchange_acqrel_relaxed_with(
                ghost!(&*self.invariant),
                expected,
                new,
                ghost!(|state: &mut RefCountState, old, new| { update_ref_count(state, old, new) }),
            );
            ghost! {
                match callback.into_inner() {
                    Ok(()) | Err(_) => (),
                }
            };
            result
        }

        #[cfg(not(creusot))]
        {
            self.counter
                .compare_exchange_acqrel_relaxed_native(expected, new)
        }
    }

    #[cfg_attr(creusot, requires(self.invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant()))]
    pub(crate) fn fetch_sub_release(&self) -> usize {
        #[cfg(creusot)]
        {
            self.counter
                .fetch_sub_release_with(
                    ghost!(&*self.invariant),
                    ghost!(|state: &mut RefCountState, old, new| {
                        update_ref_count(state, old, new)
                    }),
                )
                .0
        }

        #[cfg(not(creusot))]
        {
            self.counter.fetch_sub_release_native()
        }
    }
}

#[cfg(creusot)]
mod proof {
    use super::{NativeCounter, RefCountCell};
    use creusot_std::{
        ghost::{
            invariant::{declare_namespace, AtomicInvariant, Protocol},
            perm::{Perm, PermTarget},
            FnGhost, Ghost,
        },
        prelude::*,
    };

    declare_namespace! { REF_COUNT_EVENT }

    impl PermTarget for NativeCounter {
        type Value<'a>
            = usize
        where
            Self: 'a;
        type PermPayload = ();
    }

    pub(super) trait CounterProtocol: Protocol {
        #[logic(prophetic)]
        fn agrees_with_value(self, value: usize) -> bool;
    }

    pub(super) struct EventState<T: CounterProtocol> {
        pub owner: Perm<NativeCounter>,
        pub current: T,
    }

    impl<T: CounterProtocol> Protocol for EventState<T> {
        type Public = (NativeCounter, T::Public);

        #[logic(open)]
        fn public(self) -> Self::Public {
            pearlite! { (*self.owner.ward(), self.current.public()) }
        }

        #[logic(open(super), prophetic)]
        fn protocol(self) -> bool {
            self.current.protocol() && self.current.agrees_with_value(self.owner.val())
        }
    }

    pub(super) struct EventBoundInvariant<T: CounterProtocol> {
        inner: Ghost<AtomicInvariant<EventState<T>>>,
    }

    impl<T: CounterProtocol> EventBoundInvariant<T> {
        #[check(ghost)]
        #[requires(counter == owner.ward())]
        #[requires(current.protocol() && current.agrees_with_value(owner.val()))]
        #[ensures(result.public() == (*counter, current.public()))]
        pub(super) fn new(
            counter: &NativeCounter,
            owner: Ghost<Perm<NativeCounter>>,
            current: Ghost<T>,
        ) -> Ghost<Self> {
            ghost! {
                let state = EventState { owner: owner.into_inner(), current: current.into_inner() };
                let inner = AtomicInvariant::new(Ghost::new(state), snapshot!(REF_COUNT_EVENT()));
                Self { inner }
            }
        }

        #[logic(open(self))]
        pub(super) fn public(self) -> (NativeCounter, T::Public) {
            self.inner.public()
        }
    }

    impl Invariant for RefCountCell {
        #[logic(open(super), prophetic)]
        fn invariant(self) -> bool {
            self.invariant.inner_logic().public().0 == self.counter
        }
    }

    pub(super) struct RefCountState {
        pub(super) current: usize,
    }

    impl Protocol for RefCountState {
        type Public = ();

        #[logic(open)]
        fn public(self) {}

        #[logic(open(super), prophetic)]
        fn protocol(self) -> bool {
            true
        }
    }

    impl CounterProtocol for RefCountState {
        #[logic(open(super), prophetic)]
        fn agrees_with_value(self, value: usize) -> bool {
            self.current == value
        }
    }

    #[check(ghost)]
    #[requires(state.protocol() && state.agrees_with_value(value))]
    #[ensures((^state).public() == state.public())]
    #[ensures((^state).protocol())]
    #[ensures((^state).agrees_with_value(value))]
    pub(super) fn observe_ref_count(state: &mut RefCountState, value: usize) {
        assert!(state.current == value);
    }

    #[check(ghost)]
    #[requires(state.protocol() && state.current == old)]
    #[ensures((^state).public() == state.public())]
    #[ensures((^state).protocol())]
    #[ensures((^state).agrees_with_value(new))]
    pub(super) fn update_ref_count(state: &mut RefCountState, old: usize, new: usize) {
        assert!(state.current == old);
        state.current = new;
    }

    impl NativeCounter {
        /// Opens the exact hidden state held by `invariant.inner` for this one
        /// Relaxed load, then restores it through the checked callback.
        #[trusted]
        #[check(terminates)]
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() ==>
                f.precondition((&mut s.current, old)) &&
                (forall<a: A> f.postcondition_once((&mut s.current, old), a) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(old))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() && result.0 == old &&
            f.postcondition_once((&mut s.current, old), *result.1) &&
            (^s).public() == s.public() && (^s).protocol() &&
            (^s).owner.val() == old && (^s).owner.ward() == s.owner.ward())]
        pub(super) fn load_relaxed_with<T, F, A>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            f: Ghost<F>,
        ) -> (usize, Ghost<A>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize) -> A,
        {
            let value = self.load_relaxed_native();
            let output = ghost! {
                let mut state = Ghost::<&mut EventState<T>>::conjure();
                f.into_inner()(&mut state.current, value)
            };
            (value, output)
        }

        #[trusted]
        #[check(terminates)]
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() ==>
                f.precondition((&mut s.current, old)) &&
                (forall<a: A> f.postcondition_once((&mut s.current, old), a) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(old))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() && result.0 == old &&
            f.postcondition_once((&mut s.current, old), *result.1) &&
            (^s).public() == s.public() && (^s).protocol() &&
            (^s).owner.val() == old && (^s).owner.ward() == s.owner.ward())]
        pub(super) fn load_acquire_with<T, F, A>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            f: Ghost<F>,
        ) -> (usize, Ghost<A>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize) -> A,
        {
            let value = self.load_acquire_native();
            let output = ghost! {
                let mut state = Ghost::<&mut EventState<T>>::conjure();
                f.into_inner()(&mut state.current, value)
            };
            (value, output)
        }

        #[trusted]
        #[check(terminates)]
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() ==>
                (old == expected ==>
                    f.precondition((&mut s.current, old, new)) &&
                    (forall<a: A> f.postcondition_once((&mut s.current, old, new), a) ==>
                        (^s).current.public() == s.current.public() &&
                        (^s).current.protocol() && (^s).current.agrees_with_value(new)))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() &&
            match result.0 {
                Ok(actual) => actual == old && old == expected &&
                    (^s).owner.val() == new && (^s).owner.ward() == s.owner.ward() &&
                    (^s).current.public() == s.current.public() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(new) &&
                    (^s).public() == s.public() && (^s).protocol() &&
                    (exists<a: A> *result.1 == Ok(a) &&
                        f.postcondition_once((&mut s.current, old, new), a)),
                Err(actual) => actual == old && old != expected &&
                    *result.1 == Err(*f) && (^s) == *s,
            })]
        pub(super) fn compare_exchange_relaxed_with<T, F, A>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            expected: usize,
            new: usize,
            f: Ghost<F>,
        ) -> (Result<usize, usize>, Ghost<Result<A, F>>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize, usize) -> A,
        {
            match self.compare_exchange_relaxed_native(expected, new) {
                Ok(old) => {
                    let callback_result = ghost! {
                        let mut state = Ghost::<&mut EventState<T>>::conjure();
                        let output = f.into_inner()(&mut state.current, old, new);
                        Ok(output)
                    };
                    (Ok(old), callback_result)
                }
                Err(actual) => {
                    let callback_result = ghost! { Err(f.into_inner()) };
                    (Err(actual), callback_result)
                }
            }
        }

        #[trusted]
        #[check(terminates)]
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() ==>
                (old == expected ==>
                    f.precondition((&mut s.current, old, new)) &&
                    (forall<a: A> f.postcondition_once((&mut s.current, old, new), a) ==>
                        (^s).current.public() == s.current.public() &&
                        (^s).current.protocol() && (^s).current.agrees_with_value(new)))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() &&
            match result.0 {
                Ok(actual) => actual == old && old == expected &&
                    (^s).owner.val() == new && (^s).owner.ward() == s.owner.ward() &&
                    (^s).current.public() == s.current.public() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(new) &&
                    (^s).public() == s.public() && (^s).protocol() &&
                    (exists<a: A> *result.1 == Ok(a) &&
                        f.postcondition_once((&mut s.current, old, new), a)),
                Err(actual) => actual == old && old != expected &&
                    *result.1 == Err(*f) && (^s) == *s,
            })]
        pub(super) fn compare_exchange_acqrel_relaxed_with<T, F, A>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            expected: usize,
            new: usize,
            f: Ghost<F>,
        ) -> (Result<usize, usize>, Ghost<Result<A, F>>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize, usize) -> A,
        {
            match self.compare_exchange_acqrel_relaxed_native(expected, new) {
                Ok(old) => {
                    let callback_result = ghost! {
                        let mut state = Ghost::<&mut EventState<T>>::conjure();
                        let output = f.into_inner()(&mut state.current, old, new);
                        Ok(output)
                    };
                    (Ok(old), callback_result)
                }
                Err(actual) => {
                    let callback_result = ghost! { Err(f.into_inner()) };
                    (Err(actual), callback_result)
                }
            }
        }

        #[trusted]
        #[check(terminates)]
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() ==>
                f.precondition((&mut s.current, old,
                    if old == 0usize { usize::MAX } else { old - 1usize })) &&
                (forall<a: A> f.postcondition_once((&mut s.current, old,
                    if old == 0usize { usize::MAX } else { old - 1usize }), a) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.protocol() &&
                    (^s).current.agrees_with_value(
                        if old == 0usize { usize::MAX } else { old - 1usize }))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() && result.0 == old &&
            f.postcondition_once((&mut s.current, old,
                if old == 0usize { usize::MAX } else { old - 1usize }), *result.1) &&
            (^s).owner.val() == if old == 0usize { usize::MAX } else { old - 1usize } &&
            (old == 0usize ==> (^s).owner.val() == usize::MAX) &&
            (old > 0usize ==> (^s).owner.val() + 1usize == old) &&
            (^s).owner.ward() == s.owner.ward() &&
            (^s).current.public() == s.current.public() &&
            (^s).current.protocol() &&
            (^s).current.agrees_with_value(
                if old == 0usize { usize::MAX } else { old - 1usize }) &&
            (^s).public() == s.public() && (^s).protocol())]
        pub(super) fn fetch_sub_release_with<T, F, A>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            f: Ghost<F>,
        ) -> (usize, Ghost<A>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize, usize) -> A,
        {
            let old = self.fetch_sub_release_native();
            let output = ghost! {
                let new = old.wrapping_sub(1usize);
                let mut state = Ghost::<&mut EventState<T>>::conjure();
                f.into_inner()(&mut state.current, old, new)
            };
            (old, output)
        }
    }

    pub(super) use {EventBoundInvariant as EventInvariant, RefCountState as State};
}

#[cfg(creusot)]
use proof::{
    observe_ref_count, update_ref_count, EventInvariant as EventBoundInvariant,
    State as RefCountState,
};

#[cfg(creusot)]
type RefCountInvariant = EventBoundInvariant<RefCountState>;

#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(creusot, requires(cell.invariant()))]
#[cfg_attr(creusot, ensures(cell.invariant()))]
#[cfg_attr(creusot, ensures(match result {
    Ok(old) => old <= crate::ref_count_limit::MAX_REF_COUNT,
    Err(old) => old > crate::ref_count_limit::MAX_REF_COUNT,
}))]
pub(crate) fn try_increment(cell: &RefCountCell) -> Result<usize, usize> {
    let mut current = cell.load_relaxed();
    #[cfg_attr(creusot, invariant(cell.invariant()))]
    loop {
        let next = match crate::ref_count_limit::next_ref_count(current) {
            Some(next) => next,
            None => return Err(current),
        };

        match cell.compare_exchange_relaxed(current, next) {
            Ok(old) => return Ok(old),
            Err(actual) => current = actual,
        }
    }
}

#[inline]
#[cfg_attr(creusot, requires(cell.invariant()))]
#[cfg_attr(creusot, ensures(cell.invariant()))]
pub(crate) fn increment(cell: &RefCountCell) {
    if try_increment(cell).is_err() {
        crate::abort();
    }
}

#[cfg(all(test, not(loom), not(creusot)))]
mod tests {
    use super::*;
    use crate::ref_count_limit::MAX_REF_COUNT;

    #[test]
    fn guarded_increment_accepts_last_allowed_value_without_wrap() {
        let cell = RefCountCell::new(MAX_REF_COUNT);
        increment(&cell);
        assert_eq!(cell.load_relaxed(), MAX_REF_COUNT + 1);
        assert_eq!(try_increment(&cell), Err(MAX_REF_COUNT + 1));
        assert_eq!(cell.load_relaxed(), MAX_REF_COUNT + 1);
    }

    #[test]
    fn guarded_increment_refuses_word_maximum_without_store() {
        let cell = RefCountCell::new(usize::MAX);
        assert_eq!(try_increment(&cell), Err(usize::MAX));
        assert_eq!(cell.load_relaxed(), usize::MAX);
    }
}
