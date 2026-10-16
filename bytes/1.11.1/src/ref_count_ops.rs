use crate::loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(creusot)]
use creusot_std::ghost::{perm::Perm, Ghost};

// Keep the native relaxed-load / guarded relaxed-CAS algorithm in one source
// body. The proof build supplies event-aware load/CAS adapters; native builds
// supply the original RefCountCell methods.
macro_rules! try_increment_loop {
    (
        $load:block,
        |$current:ident, $next:ident| $compare_exchange:block
        $(, #[$loop_invariant:meta])*
    ) => {{
        let mut current = $load;
        $(#[$loop_invariant])*
        loop {
            let next = match crate::ref_count_limit::next_ref_count(current) {
                Some(next) => next,
                None => break Err(current),
            };

            let $current = current;
            let $next = next;
            match $compare_exchange {
                Ok(old) => break Ok(old),
                Err(actual) => current = actual,
            }
        }
    }};
}

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

    /// Generic sequential adapter for a detached exclusive permission to
    /// this exact counter. The native operation is the same single Release
    /// decrement used by `RefCountCell`; the permission records its returned
    /// value for a later Acquire observation.
    #[cfg(creusot)]
    #[trusted]
    #[requires(owner.inner_logic().0.invariant())]
    #[requires(owner.inner_logic().0.ward() == self)]
    #[requires(owner.inner_logic().0.val() == owner.inner_logic().1)]
    #[ensures(result.0 == owner.inner_logic().1)]
    #[ensures(result.1.inner_logic().0.invariant())]
    #[ensures(result.1.inner_logic().0.ward() == owner.inner_logic().0.ward())]
    #[ensures(result.1.inner_logic().0.val() == if owner.inner_logic().1 == 0usize {
        usize::MAX
    } else {
        owner.inner_logic().1 - 1usize
    })]
    #[ensures(result.1.inner_logic().1 == if owner.inner_logic().1 == 0usize {
        usize::MAX
    } else {
        owner.inner_logic().1 - 1usize
    })]
    pub(crate) fn fetch_sub_release_owned(
        &self,
        owner: Ghost<(Perm<NativeCounter>, usize)>,
    ) -> (usize, Ghost<(Perm<NativeCounter>, usize)>) {
        let old = self.fetch_sub_release_native();
        let owner = ghost! {
            let (permission, value) = owner.into_inner();
            (permission, value.wrapping_sub(1usize))
        };
        (old, owner)
    }

    /// Generic sequential Acquire observation for the same detached counter
    /// permission. It performs exactly one native Acquire load and preserves
    /// that permission and its value.
    #[cfg(creusot)]
    #[trusted]
    #[requires(owner.inner_logic().0.invariant())]
    #[requires(owner.inner_logic().0.ward() == self)]
    #[requires(owner.inner_logic().0.val() == owner.inner_logic().1)]
    #[ensures(result.0 == owner.inner_logic().1)]
    #[ensures(result.1.inner_logic().0.invariant())]
    #[ensures(result.1.inner_logic().0.ward() == owner.inner_logic().0.ward())]
    #[ensures(result.1.inner_logic().0.val() == owner.inner_logic().1)]
    #[ensures(result.1.inner_logic().1 == owner.inner_logic().1)]
    pub(crate) fn load_acquire_owned(
        &self,
        owner: Ghost<(Perm<NativeCounter>, usize)>,
    ) -> (usize, Ghost<(Perm<NativeCounter>, usize)>) {
        let value = self.load_acquire_native();
        (value, owner)
    }
}

/// The current value and ghost state associated with one exact native
/// counter. The proof-only sidecar is zero-sized and does not change layout.
#[repr(C)]
pub(crate) struct RefCountCell {
    counter: NativeCounter,
    #[cfg(creusot)]
    invariant: Ghost<Option<RefCountInvariant>>,
}

const _: [(); core::mem::size_of::<AtomicUsize>()] = [(); core::mem::size_of::<RefCountCell>()];
const _: [(); core::mem::align_of::<AtomicUsize>()] = [(); core::mem::align_of::<RefCountCell>()];
const _: [(); 0] = [(); core::mem::needs_drop::<RefCountCell>() as usize];

impl RefCountCell {
    /// Stable logical identity of the exact native counter field. This is a
    /// projection only; it does not expose or duplicate the counter
    /// permission.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub(crate) fn counter_model(self) -> NativeCounter {
        pearlite! { self.counter }
    }

    /// Opaque logical identity of the counter field's address. The body is
    /// visible only in this module so callers can name the address without
    /// unfolding the private `counter` field in their own contracts.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub(crate) fn counter_reference(&self) -> &NativeCounter {
        &self.counter
    }

    #[cfg(creusot)]
    #[logic(open(self))]
    pub(crate) fn has_event_invariant(self) -> bool {
        pearlite! { self.invariant.inner_logic() != None }
    }

    #[cfg_attr(creusot, ensures(result.invariant()))]
    #[cfg_attr(creusot, ensures(result.has_event_invariant()))]
    pub(crate) fn new(value: usize) -> Self {
        #[cfg(creusot)]
        {
            let (counter, owner) = NativeCounter::new(value);
            let invariant = EventBoundInvariant::new(
                owner,
                ghost!(RefCountState { counter_model: snapshot!(counter), current: value }),
            );
            Self { counter, invariant: ghost!(Some(invariant.into_inner())) }
        }

        #[cfg(not(creusot))]
        {
            Self {
                counter: NativeCounter::new(value),
            }
        }
    }

    /// Construct the native counter and detach its one-shot proof permission
    /// from the cell in the same step. The returned permission is the sole
    /// permission produced for the exact counter field, and its value records
    /// the constructor argument. This is used when a higher-level header
    /// invariant will own the counter protocol after the cell is placed.
    ///
    /// This constructor has no normal-build counterpart: native construction
    /// continues through `new`, and this proof-only path performs the same
    /// single `AtomicUsize::new` operation through `NativeCounter::new`.
    #[cfg(creusot)]
    #[ensures(result.0.invariant() && !result.0.has_event_invariant())]
    #[ensures(*result.1.inner_logic().0.ward() == result.0.counter_model())]
    #[ensures(result.1.inner_logic().0.val() == value)]
    #[ensures(result.1.inner_logic().1 == value)]
    pub(crate) fn new_with_owner(
        value: usize,
    ) -> (Self, Ghost<(creusot_std::ghost::Perm<NativeCounter>, usize)>) {
        let (counter, owner) = NativeCounter::new(value);
        let cell = Self {
            counter,
            invariant: ghost!(None),
        };
        (cell, ghost!((owner.into_inner(), value)))
    }

    /// Return the exact native counter field without opening its ghost
    /// invariant. This getter preserves field identity for callers that first
    /// place the enclosing header at its final address.
    #[cfg_attr(creusot, ensures(result == self.counter_reference()))]
    #[cfg_attr(creusot, ensures(*result == self.counter_model()))]
    pub(crate) fn native_counter(&self) -> &NativeCounter {
        &self.counter
    }

    /// Consume the cell's one-shot event invariant and return its unique
    /// permission together with the value recorded by that permission.
    ///
    /// The native counter remains in place. The cell is left detached so that
    /// later scalar operations cannot reopen the old invariant after its
    /// permission has moved to a header-level owner.
    #[cfg(creusot)]
    #[check(ghost)]
    #[requires(self.invariant() && self.has_event_invariant())]
    #[ensures((^self).invariant() && !(^self).has_event_invariant())]
    #[ensures((^self).counter == self.counter)]
    #[ensures(result.inner_logic().0.ward() == &(^self).counter)]
    #[ensures(*result.inner_logic().0.ward() == (^self).counter)]
    #[ensures(result.inner_logic().0.val() == result.inner_logic().1)]
    pub(crate) fn take_bootstrap_owner(
        &mut self,
    ) -> Ghost<(creusot_std::ghost::Perm<NativeCounter>, usize)> {
        ghost! {
            let invariant = self.invariant.borrow_mut().into_inner().take().unwrap();
            let (owner, state) = invariant.into_inner().into_inner();
            (owner, state.current)
        }
    }

    #[cfg_attr(creusot, requires(self.invariant() && self.has_event_invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant() && self.has_event_invariant()))]
    pub(crate) fn load_relaxed(&self) -> usize {
        #[cfg(creusot)]
        {
            self.counter
                .load_relaxed_with(
                    ghost!(self.invariant.borrow().into_inner().as_ref().unwrap()),
                    ghost!(|state: &mut RefCountState, value| { observe_ref_count(state, value) }),
                )
                .0
        }

        #[cfg(not(creusot))]
        {
            self.counter.load_relaxed_native()
        }
    }

    #[cfg_attr(creusot, requires(self.invariant() && self.has_event_invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant() && self.has_event_invariant()))]
    pub(crate) fn load_acquire(&self) -> usize {
        #[cfg(creusot)]
        {
            self.counter
                .load_acquire_with(
                    ghost!(self.invariant.borrow().into_inner().as_ref().unwrap()),
                    ghost!(|state: &mut RefCountState, value| { observe_ref_count(state, value) }),
                )
                .0
        }

        #[cfg(not(creusot))]
        {
            self.counter.load_acquire_native()
        }
    }

    #[cfg_attr(creusot, requires(self.invariant() && self.has_event_invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant() && self.has_event_invariant()))]
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
                ghost!(self.invariant.borrow().into_inner().as_ref().unwrap()),
                expected,
                new,
                ghost!(()),
                ghost!(|state: &mut RefCountState, old, new, _frame: ()| {
                    update_ref_count(state, old, new);
                    ((), ())
                }),
            );
            ghost! {
                match callback.into_inner() {
                    Ok(((), ())) | Err((_, ())) => (),
                }
            };
            result
        }

        #[cfg(not(creusot))]
        {
            self.counter.compare_exchange_relaxed_native(expected, new)
        }
    }

    #[cfg_attr(creusot, requires(self.invariant() && self.has_event_invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant() && self.has_event_invariant()))]
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
                ghost!(self.invariant.borrow().into_inner().as_ref().unwrap()),
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

    #[cfg_attr(creusot, requires(self.invariant() && self.has_event_invariant()))]
    #[cfg_attr(creusot, ensures(self.invariant() && self.has_event_invariant()))]
    pub(crate) fn fetch_sub_release(&self) -> usize {
        #[cfg(creusot)]
        {
            self.counter
                .fetch_sub_release_with(
                    ghost!(self.invariant.borrow().into_inner().as_ref().unwrap()),
                    ghost!(()),
                    ghost!(|state: &mut RefCountState, old, new, _frame: ()| {
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
            FnGhost, FnGhostWrapper, Ghost,
        },
        prelude::*,
    };

    declare_namespace! { REF_COUNT_EVENT }

    #[logic(open)]
    pub(crate) fn event_namespace() -> creusot_std::ghost::invariant::Namespace {
        REF_COUNT_EVENT()
    }

    impl PermTarget for NativeCounter {
        type Value<'a>
            = usize
        where
            Self: 'a;
        type PermPayload = ();
    }

    pub(crate) trait CounterProtocol: Protocol {
        /// Opaque logical identity of the exact counter represented by this
        /// protocol state. This model is distinct from any native address
        /// marker used by a higher-level resource registry.
        #[logic(prophetic)]
        fn counter_model(self) -> NativeCounter;

        #[logic(prophetic)]
        fn agrees_with_value(self, value: usize) -> bool;
    }

    pub(crate) struct EventState<T: CounterProtocol> {
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
        #[ensures(result == (self.current.protocol() &&
            self.current.counter_model() == *self.owner.ward() &&
            self.current.agrees_with_value(self.owner.val())))]
        fn protocol(self) -> bool {
            self.current.protocol() &&
            self.current.counter_model() == *self.owner.ward() &&
            self.current.agrees_with_value(self.owner.val())
        }
    }

    pub(crate) struct EventBoundInvariant<T: CounterProtocol> {
        pub(super) inner: Ghost<AtomicInvariant<EventState<T>>>,
    }

    impl<T: CounterProtocol> EventBoundInvariant<T> {
        #[check(ghost)]
        #[requires(current.counter_model() == *owner.ward())]
        #[requires(current.protocol() && current.agrees_with_value(owner.val()))]
        #[ensures(result.public() == (*owner.ward(), current.public()))]
        #[ensures(result.public().0 == current.counter_model())]
        #[ensures(result.invariant())]
        #[ensures(result.namespace() == event_namespace())]
        pub(crate) fn new(
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
        pub(crate) fn namespace(self) -> creusot_std::ghost::invariant::Namespace {
            self.inner.namespace()
        }

        /// Open the exact event state with an explicitly supplied invariant
        /// token. Unlike the native counter adapters below, this helper has
        /// no atomic operation of its own: it delegates only to the existing
        /// generic `AtomicInvariant::open` contract and exposes its checked
        /// callback/frame obligations to callers.
        #[check(ghost)]
        #[requires(tokens.contains(self.namespace()))]
        #[requires(forall<s: &mut EventState<T>>
            s.public() == self.public() &&
                s.current.protocol() &&
                s.current.counter_model() == *s.owner.ward() &&
                s.current.agrees_with_value(s.owner.val()) &&
                inv(s) ==>
                f.precondition((
                    &mut s.current,
                    Snapshot::new(*s.owner.ward()),
                    Snapshot::new(s.owner.val()),
                )) &&
                (forall<res: A> f.postcondition_once((
                    &mut s.current,
                    Snapshot::new(*s.owner.ward()),
                    Snapshot::new(s.owner.val()),
                ), res) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() &&
                    (^s).current.agrees_with_value(s.owner.val()))
        )]
        #[ensures(exists<s: &mut EventState<T>>
            s.public() == self.public() &&
            s.current.protocol() &&
            s.current.counter_model() == *s.owner.ward() &&
            s.current.agrees_with_value(s.owner.val()) &&
            inv(s) &&
            f.postcondition_once((
                &mut s.current,
                Snapshot::new(*s.owner.ward()),
                Snapshot::new(s.owner.val()),
            ), *result) &&
            (^s).current.public() == s.current.public() &&
            (^s).current.counter_model() == s.current.counter_model() &&
            (^s).current.protocol() &&
            (^s).current.agrees_with_value(s.owner.val()) &&
            (^s).owner.val() == s.owner.val() &&
            (^s).owner.ward() == s.owner.ward())]
        pub(crate) fn open_with_tokens<A, F>(
            &self,
            tokens: Ghost<creusot_std::ghost::invariant::Tokens<'_>>,
            f: Ghost<F>,
        ) -> Ghost<A>
        where
            F: FnGhost + for<'s> FnOnce(
                &'s mut T,
                Snapshot<NativeCounter>,
                Snapshot<usize>,
            ) -> A,
        {
            ghost! {
                let callback = f.into_inner();
                self.inner.borrow().open(
                    tokens.into_inner(),
                    FnGhostWrapper(|state: &mut EventState<T>| {
                        let owner_model = snapshot!(*state.owner.ward());
                        let owner_value = snapshot!(state.owner.val());
                        callback(&mut state.current, owner_model, owner_value)
                    }),
                )
            }
        }

        /// Open the event state while passing a value-only frame explicitly
        /// to the callback. This keeps callback postconditions independent
        /// of captured values that the callback consumes while restoring an
        /// affine loan.
        #[check(ghost)]
        #[requires(tokens.contains(self.namespace()))]
        #[requires(forall<s: &mut EventState<T>>
            s.public() == self.public() &&
                s.current.protocol() &&
                s.current.counter_model() == *s.owner.ward() &&
                s.current.agrees_with_value(s.owner.val()) &&
                inv(s) ==>
                f.precondition((
                    &mut s.current,
                    Snapshot::new(*s.owner.ward()),
                    Snapshot::new(s.owner.val()),
                    frame,
                )) &&
                (forall<res: A> f.postcondition_once((
                    &mut s.current,
                    Snapshot::new(*s.owner.ward()),
                    Snapshot::new(s.owner.val()),
                    frame,
                ), res) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() &&
                    (^s).current.agrees_with_value(s.owner.val()))
        )]
        #[ensures(exists<s: &mut EventState<T>>
            s.public() == self.public() &&
            s.current.protocol() &&
            s.current.counter_model() == *s.owner.ward() &&
            s.current.agrees_with_value(s.owner.val()) &&
            inv(s) &&
            f.postcondition_once((
                &mut s.current,
                Snapshot::new(*s.owner.ward()),
                Snapshot::new(s.owner.val()),
                frame,
            ), *result) &&
            (^s).current.public() == s.current.public() &&
            (^s).current.counter_model() == s.current.counter_model() &&
            (^s).current.protocol() &&
            (^s).current.agrees_with_value(s.owner.val()) &&
            (^s).owner.val() == s.owner.val() &&
            (^s).owner.ward() == s.owner.ward())]
        pub(crate) fn open_with_tokens_with_frame<A, F, G>(
            &self,
            tokens: Ghost<creusot_std::ghost::invariant::Tokens<'_>>,
            frame: Snapshot<G>,
            f: Ghost<F>,
        ) -> Ghost<A>
        where
            F: FnGhost + for<'s> FnOnce(
                &'s mut T,
                Snapshot<NativeCounter>,
                Snapshot<usize>,
                Snapshot<G>,
            ) -> A,
        {
            ghost! {
                let callback = f.into_inner();
                self.inner.borrow().open(
                    tokens.into_inner(),
                    FnGhostWrapper(|state: &mut EventState<T>| {
                        let owner_model = snapshot!(*state.owner.ward());
                        let owner_value = snapshot!(state.owner.val());
                        callback(&mut state.current, owner_model, owner_value, frame)
                    }),
                )
            }
        }

        #[logic(open(self))]
        pub(crate) fn public(self) -> (NativeCounter, T::Public) {
            self.inner.public()
        }

        /// Consume the invariant and move out its one owner permission and
        /// current protocol value. This is the body-checked bootstrap path;
        /// it does not mint or duplicate either resource.
        #[check(ghost)]
        #[ensures(*result.inner_logic().0.ward() == self.public().0)]
        #[ensures(result.inner_logic().1.public() == self.public().1)]
        #[ensures(result.inner_logic().1.counter_model() ==
            *result.inner_logic().0.ward())]
        #[ensures(result.inner_logic().1.protocol())]
        #[ensures(result.inner_logic().1.agrees_with_value(result.inner_logic().0.val()))]
        pub(crate) fn into_inner(self) -> Ghost<(Perm<NativeCounter>, T)> {
            ghost! {
                let EventState { owner, current } = self.inner.into_inner().into_inner();
                (owner, current)
            }
        }
    }

    impl<T: CounterProtocol> Invariant for EventBoundInvariant<T> {
        #[logic(open(super), prophetic)]
        fn invariant(self) -> bool {
            self.inner.inner_logic().invariant()
        }
    }

    impl Invariant for RefCountCell {
        #[logic(open(super), prophetic)]
        fn invariant(self) -> bool {
            pearlite! {
                match self.invariant.inner_logic() {
                    None => true,
                    Some(invariant) => invariant.public().0 == self.counter,
                }
            }
        }
    }

    pub(super) struct RefCountState {
        pub(super) counter_model: Snapshot<NativeCounter>,
        pub(super) current: usize,
    }

    impl Protocol for RefCountState {
        type Public = NativeCounter;

        #[logic(open(super))]
        fn public(self) -> NativeCounter { *self.counter_model }

        #[logic(open(super), prophetic)]
        fn protocol(self) -> bool {
            true
        }
    }

    impl CounterProtocol for RefCountState {
        #[logic(open(super), prophetic)]
        fn counter_model(self) -> NativeCounter { *self.counter_model }

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

    /// Observe an event protocol at the value read from its native counter.
    /// This is the generic counterpart of `observe_ref_count`, used when the
    /// counter is owned by a higher-level capsule rather than a `RefCountCell`.
    #[check(ghost)]
    #[requires(state.protocol() && state.agrees_with_value(value))]
    #[ensures((^state).public() == state.public())]
    #[ensures((^state).counter_model() == state.counter_model())]
    #[ensures((^state).protocol())]
    #[ensures((^state).agrees_with_value(value))]
    pub(crate) fn observe_event_counter<T: CounterProtocol>(state: &mut T, value: usize) {
        proof_assert!(state.agrees_with_value(value));
    }

    impl NativeCounter {
        /// Opens the exact hidden state held by `invariant.inner` for this one
        /// Relaxed load, then restores it through the checked callback.
        #[trusted]
        #[check(terminates)]
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.invariant() && s.owner.ward() == self && old == s.owner.val() ==>
                f.precondition((&mut s.current, old)) &&
                (forall<a: A> f.postcondition_once((&mut s.current, old), a) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(old))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() && result.0 == old &&
            f.postcondition_once((&mut s.current, old), *result.1) &&
            (^s).public() == s.public() && (^s).protocol() &&
            (^s).current.counter_model() == s.current.counter_model() &&
            (^s).owner.val() == old && (^s).owner.ward() == s.owner.ward())]
        pub(crate) fn load_relaxed_with<T, F, A>(
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
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(old))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() && result.0 == old &&
            f.postcondition_once((&mut s.current, old), *result.1) &&
            (^s).public() == s.public() && (^s).protocol() &&
            (^s).current.counter_model() == s.current.counter_model() &&
            (^s).owner.val() == old && (^s).owner.ward() == s.owner.ward())]
        pub(crate) fn load_acquire_with<T, F, A>(
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
            s.invariant() && s.owner.ward() == self && old == s.owner.val() ==>
                (old == expected ==>
                    f.precondition((&mut s.current, old, new, frame.inner_logic())) &&
                    (forall<a: A, updated: C> f.postcondition_once(
                        (&mut s.current, old, new, frame.inner_logic()),
                        (a, updated)) ==>
                        (^s).current.public() == s.current.public() &&
                        (^s).current.counter_model() == s.current.counter_model() &&
                        (^s).current.protocol() && (^s).current.agrees_with_value(new)))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() &&
            match result.0 {
                Ok(actual) => actual == old && old == expected &&
                    (^s).owner.val() == new && (^s).owner.ward() == s.owner.ward() &&
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(new) &&
                    (^s).public() == s.public() && (^s).protocol() &&
                    (exists<a: A, updated: C> *result.1 == Ok((a, updated)) &&
                        f.postcondition_once(
                            (&mut s.current, old, new, frame.inner_logic()),
                            (a, updated))),
                Err(actual) => actual == old && old != expected &&
                    *result.1 == Err((*f, frame.inner_logic())) && (^s) == *s,
            })]
        pub(crate) fn compare_exchange_relaxed_with<T, F, A, C>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            expected: usize,
            new: usize,
            frame: Ghost<C>,
            f: Ghost<F>,
        ) -> (Result<usize, usize>, Ghost<Result<(A, C), (F, C)>>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize, usize, C) -> (A, C),
        {
            match self.compare_exchange_relaxed_native(expected, new) {
                Ok(old) => {
                    let callback_result = ghost! {
                        let mut state = Ghost::<&mut EventState<T>>::conjure();
                        let (output, updated_frame) = f.into_inner()(
                            &mut state.current,
                            old,
                            new,
                            frame.into_inner(),
                        );
                        Ok((output, updated_frame))
                    };
                    (Ok(old), callback_result)
                }
                Err(actual) => {
                    let callback_result = ghost! {
                        Err((f.into_inner(), frame.into_inner()))
                    };
                    (Err(actual), callback_result)
                }
            }
        }

        /// Run the crate's guarded relaxed-CAS increment loop against an
        /// arbitrary event invariant. A failed compare-exchange returns the
        /// same callback closure so its affine captures are retained across
        /// retries; the callback runs exactly once, after the successful
        /// native CAS. The loop is shared by the ordinary `RefCountCell`
        /// increment path and shared-owner splitting.
        #[requires(self == &invariant.public().0)]
        #[requires(forall<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.invariant() && s.owner.ward() == self && old == s.owner.val() ==>
                (old <= crate::ref_count_limit::MAX_REF_COUNT ==>
                    callback.precondition((
                        &mut s.current, old, old + 1usize, frame.inner_logic()
                    )) &&
                    (forall<a: A, updated: C> callback.postcondition_once(
                        (&mut s.current, old, old + 1usize, frame.inner_logic()),
                        (a, updated)
                    ) ==>
                        (^s).current.public() == s.current.public() &&
                        (^s).current.counter_model() == s.current.counter_model() &&
                        (^s).current.protocol() &&
                        (^s).current.agrees_with_value(old + 1usize)))
        )]
        #[ensures(match result.0 {
            Ok(old) => old <= crate::ref_count_limit::MAX_REF_COUNT &&
                exists<s: &mut EventState<T>, a: A, updated: C>
                    s.public() == invariant.public() && s.protocol() &&
                    s.owner.ward() == self && old == s.owner.val() &&
                    result.1.inner_logic() == Ok((a, updated)) &&
                    callback.postcondition_once((
                        &mut s.current, old, old + 1usize, frame.inner_logic()
                    ), (a, updated)) &&
                    (^s).public() == s.public() && (^s).protocol() &&
                    (^s).owner.ward() == s.owner.ward() &&
                    (^s).owner.val() == old + 1usize &&
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() &&
                    (^s).current.agrees_with_value(old + 1usize),
            Err(old) => old > crate::ref_count_limit::MAX_REF_COUNT &&
                result.1.inner_logic() == Err((*callback, frame.inner_logic()))
        })]
        pub(crate) fn try_increment_with<T, F, A, C>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            frame: Ghost<C>,
            callback: Ghost<F>,
        ) -> (Result<usize, usize>, Ghost<Result<(A, C), (F, C)>>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize, usize, C) -> (A, C),
        {
            // The loop owns the affine callback in `callback_slot`; this
            // snapshot records only its logical identity for the loop
            // invariant and does not duplicate the callback's captures.
            let callback_identity: Snapshot<Ghost<F>> = snapshot!(callback);
            let initial_frame: Snapshot<Ghost<C>> = snapshot!(frame);
            let mut frame_slot = Some(frame);
            let mut callback_slot = Some(callback);
            let mut callback_output = None;
            let result = try_increment_loop!(
                {
                    let (value, observation) = self.load_relaxed_with(
                        invariant,
                        ghost!(|state: &mut T, value: usize| {
                            observe_event_counter(state, value)
                        }),
                    );
                    ghost! { let _ = observation.into_inner(); };
                    value
                },
                |current, next| {
                    let (exchange, callback_result) = self.compare_exchange_relaxed_with(
                        invariant,
                        current,
                        next,
                        frame_slot.take().unwrap(),
                        callback_slot.take().unwrap(),
                    );
                    match exchange {
                        Ok(old) => {
                            callback_output = Some(ghost! {
                                callback_result.into_inner().ok().unwrap()
                            });
                            Ok(old)
                        }
                        Err(actual) => {
                            let (callback, frame) = ghost! {
                                callback_result.into_inner().err().unwrap()
                            }.split();
                            callback_slot = Some(callback);
                            frame_slot = Some(frame);
                            Err(actual)
                        }
                    }
                },
                #[invariant(callback_slot == Some(*callback_identity))],
                #[invariant(callback_output == None)],
                #[invariant(callback_output == None ==>
                    frame_slot == Some(*initial_frame))]
            );

            match result {
                Ok(old) => {
                    let output = callback_output.unwrap();
                    (Ok(old), ghost! { Ok(output.into_inner()) })
                }
                Err(old) => {
                    let unused = callback_slot.unwrap();
                    let frame = frame_slot.unwrap();
                    (Err(old), ghost! { Err((unused.into_inner(), frame.into_inner())) })
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
                        (^s).current.counter_model() == s.current.counter_model() &&
                        (^s).current.protocol() && (^s).current.agrees_with_value(new)))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() &&
            match result.0 {
                Ok(actual) => actual == old && old == expected &&
                    (^s).owner.val() == new && (^s).owner.ward() == s.owner.ward() &&
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() && (^s).current.agrees_with_value(new) &&
                    (^s).public() == s.public() && (^s).protocol() &&
                    (exists<a: A> *result.1 == Ok(a) &&
                        f.postcondition_once((&mut s.current, old, new), a)),
                Err(actual) => actual == old && old != expected &&
                    *result.1 == Err(*f) && (^s) == *s,
            })]
        pub(crate) fn compare_exchange_acqrel_relaxed_with<T, F, A>(
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
                    if old == 0usize { usize::MAX } else { old - 1usize },
                    frame.inner_logic())) &&
                (forall<a: A> f.postcondition_once((&mut s.current, old,
                    if old == 0usize { usize::MAX } else { old - 1usize },
                    frame.inner_logic()), a) ==>
                    (^s).current.public() == s.current.public() &&
                    (^s).current.counter_model() == s.current.counter_model() &&
                    (^s).current.protocol() &&
                    (^s).current.agrees_with_value(
                        if old == 0usize { usize::MAX } else { old - 1usize }))
        )]
        #[ensures(exists<s: &mut EventState<T>, old: usize>
            s.public() == invariant.public() && s.protocol() &&
            s.owner.ward() == self && old == s.owner.val() && result.0 == old &&
            f.postcondition_once((&mut s.current, old,
                if old == 0usize { usize::MAX } else { old - 1usize },
                frame.inner_logic()), *result.1) &&
            (^s).owner.val() == if old == 0usize { usize::MAX } else { old - 1usize } &&
            (old == 0usize ==> (^s).owner.val() == usize::MAX) &&
            (old > 0usize ==> (^s).owner.val() + 1usize == old) &&
            (^s).owner.ward() == s.owner.ward() &&
            (^s).current.public() == s.current.public() &&
            (^s).current.counter_model() == s.current.counter_model() &&
            (^s).current.protocol() &&
            (^s).current.agrees_with_value(
                if old == 0usize { usize::MAX } else { old - 1usize }) &&
            (^s).public() == s.public() && (^s).protocol())]
        pub(crate) fn fetch_sub_release_with<T, F, A, Frame>(
            &self,
            invariant: Ghost<&EventBoundInvariant<T>>,
            frame: Ghost<Frame>,
            f: Ghost<F>,
        ) -> (usize, Ghost<A>)
        where
            T: CounterProtocol,
            F: FnGhost + for<'s> FnOnce(&'s mut T, usize, usize, Frame) -> A,
        {
            let old = self.fetch_sub_release_native();
            let output = ghost! {
                let new = old.wrapping_sub(1usize);
                let mut state = Ghost::<&mut EventState<T>>::conjure();
                f.into_inner()(&mut state.current, old, new, frame.into_inner())
            };
            (old, output)
        }
    }

}

#[cfg(creusot)]
pub(crate) use proof::{event_namespace, CounterProtocol, EventBoundInvariant};

#[cfg(creusot)]
use proof::{observe_ref_count, update_ref_count, RefCountState};

#[cfg(creusot)]
type RefCountInvariant = EventBoundInvariant<RefCountState>;

#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(creusot, requires(cell.invariant() && cell.has_event_invariant()))]
#[cfg_attr(creusot, ensures(cell.invariant() && cell.has_event_invariant()))]
#[cfg_attr(creusot, ensures(match result {
    Ok(old) => old <= crate::ref_count_limit::MAX_REF_COUNT,
    Err(old) => old > crate::ref_count_limit::MAX_REF_COUNT,
}))]
pub(crate) fn try_increment(cell: &RefCountCell) -> Result<usize, usize> {
    #[cfg(creusot)]
    {
        let invariant = ghost! { cell.invariant.borrow().into_inner().as_ref().unwrap() };
        let (result, callback) = cell.counter.try_increment_with(
            invariant,
            ghost!(()),
            ghost!(|state: &mut RefCountState, old: usize, new: usize, _frame: ()| {
                update_ref_count(state, old, new);
                ((), ())
            }),
        );
        ghost! { let _ = callback.into_inner(); };
        return result;
    }

    #[cfg(not(creusot))]
    {
        try_increment_loop!(
            { cell.load_relaxed() },
            |current, next| { cell.compare_exchange_relaxed(current, next) }
        )
    }
}

#[inline]
#[cfg_attr(creusot, requires(cell.invariant() && cell.has_event_invariant()))]
#[cfg_attr(creusot, ensures(cell.invariant() && cell.has_event_invariant()))]
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
