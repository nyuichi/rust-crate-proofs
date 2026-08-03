#![allow(non_snake_case)]

use core::sync::atomic::{AtomicBool, Ordering};
use verus_state_machines_macros::tokenized_state_machine;
use vstd::atomic::AtomicCellId;
use vstd::invariant::{AtomicInvariant, InvariantPredicate};
use vstd::open_atomic_invariant;
use vstd::pervasive::unreached;
use vstd::prelude::*;

tokenized_state_machine! {
    PublicationTokens<T> {
        fields {
            #[sharding(variable)]
            pub flag: bool,

            #[sharding(persistent_option)]
            pub published: Option<T>,

            #[sharding(option)]
            pub writer: Option<()>,
        }

        init! {
            initialize() {
                init flag = false;
                init published = Option::None;
                init writer = Option::Some(());
            }
        }

        transition! {
            publish(value: T) {
                remove writer -= Some(());
                update flag = true;
                add published (union)= Some(value);
            }
        }

        property! {
            published_values_agree(left: T, right: T) {
                have published >= Some(left);
                have published >= Some(right);
                assert(left == right);
            }
        }

        property! {
            publication_excludes_writer(value: T) {
                have published >= Some(value);
                have writer >= Some(());
                assert(false);
            }
        }

        property! {
            publication_implies_flag(value: T) {
                have published >= Some(value);
                assert(pre.flag);
            }
        }

        property! {
            writer_implies_clear() {
                have writer >= Some(());
                assert(!pre.flag);
            }
        }

        #[invariant]
        pub fn flag_matches_publication(&self) -> bool {
            self.flag == self.published.is_some()
                && self.flag == self.writer.is_none()
        }

        #[inductive(initialize)]
        fn initialize_inductive(post: Self) {}

        #[inductive(publish)]
        fn publish_inductive(pre: Self, post: Self, value: T) {}
    }
}

verus! {

/// Foundational vstd extension: a permissioned bool atomic whose executable
/// implementation uses the exact Rust Acquire and Release orderings.
///
/// Like vstd's existing PAtomic types, these raw operations are the trusted
/// bridge to Rust's atomic memory model. All publication-token logic built on
/// top of them is body-proved below.
#[verifier::external_body]
pub struct RawReleaseAcquireBool {
    atomic: AtomicBool,
}

#[verifier::external_body]
pub tracked struct RawPermissionBool {
    no_copy: NoCopy,
    unused: bool,
}

pub ghost struct RawPermissionData {
    pub atomic_id: int,
    pub value: bool,
}

impl RawPermissionBool {
    #[verifier::external_body]
    pub uninterp spec fn view(self) -> RawPermissionData;

    pub open spec fn id(&self) -> AtomicCellId {
        self.view().atomic_id
    }

    pub open spec fn value(&self) -> bool {
        self.view().value
    }
}

impl RawReleaseAcquireBool {
    pub uninterp spec fn id(&self) -> AtomicCellId;

    #[verifier::external_body]
    pub fn new(value: bool) -> (result: (Self, Tracked<RawPermissionBool>))
        ensures
            result.1@.id() == result.0.id(),
            result.1@.value() == value,
        no_unwind
    {
        let atomic = RawReleaseAcquireBool { atomic: AtomicBool::new(value) };
        (atomic, Tracked::assume_new())
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn load_acquire(
        &self,
        Tracked(permission): Tracked<&RawPermissionBool>,
    ) -> (result: bool)
        requires
            permission.id() == self.id(),
        ensures
            result == permission.value(),
        opens_invariants none
        no_unwind
    {
        self.atomic.load(Ordering::Acquire)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn load_relaxed(
        &self,
        Tracked(permission): Tracked<&RawPermissionBool>,
    ) -> (result: bool)
        requires
            permission.id() == self.id(),
        ensures
            result == permission.value(),
        opens_invariants none
        no_unwind
    {
        self.atomic.load(Ordering::Relaxed)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn store_release(
        &self,
        Tracked(permission): Tracked<&mut RawPermissionBool>,
        value: bool,
    )
        requires
            old(permission).id() == self.id(),
        ensures
            final(permission).id() == self.id(),
            final(permission).value() == value,
        opens_invariants none
        no_unwind
    {
        self.atomic.store(value, Ordering::Release)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn store_relaxed(
        &self,
        Tracked(permission): Tracked<&mut RawPermissionBool>,
        value: bool,
    )
        requires
            old(permission).id() == self.id(),
        ensures
            final(permission).id() == self.id(),
            final(permission).value() == value,
        opens_invariants none
        no_unwind
    {
        self.atomic.store(value, Ordering::Relaxed)
    }
}

pub tracked struct PublicationGhost<T> {
    flag: PublicationTokens::flag<T>,
    published: Option<PublicationTokens::published<T>>,
}

pub struct PublicationInvariant<T> {
    marker: core::marker::PhantomData<T>,
}

impl<T> InvariantPredicate<(Tracked<PublicationTokens::Instance<T>>, AtomicCellId),
    (RawPermissionBool, PublicationGhost<T>)> for PublicationInvariant<T>
{
    closed spec fn inv(
        constant: (Tracked<PublicationTokens::Instance<T>>, AtomicCellId),
        state: (RawPermissionBool, PublicationGhost<T>),
    ) -> bool {
        let (instance, atomic_id) = constant;
        let (permission, ghost) = state;
        &&& permission.id() == atomic_id
        &&& ghost.flag.instance_id() == instance@.id()
        &&& permission.value() == ghost.flag.value()
        &&& match ghost.published {
            Option::Some(token) => {
                permission.value() && token.instance_id() == instance@.id()
            },
            Option::None => !permission.value(),
        }
    }
}

/// Ordering-aware publication flag. A Relaxed load returns readiness only;
/// only an Acquire load that observes true returns persistent publication
/// knowledge.
pub struct ReleaseAcquireFlag<T> {
    raw: RawReleaseAcquireBool,
    invariant: Tracked<AtomicInvariant<
        (Tracked<PublicationTokens::Instance<T>>, AtomicCellId),
        (RawPermissionBool, PublicationGhost<T>),
        PublicationInvariant<T>,
    >>,
    instance: Tracked<PublicationTokens::Instance<T>>,
    writer: Option<Tracked<PublicationTokens::writer<T>>>,
    published: Option<Tracked<PublicationTokens::published<T>>>,
}

impl<T> ReleaseAcquireFlag<T> {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.invariant@.constant().0@.id() == self.instance@.id()
            && self.invariant@.constant().1 == self.raw.id()
            && match self.writer {
                Some(writer) => writer@.instance_id() == self.instance@.id(),
                None => true,
            }
            && match self.published {
                Some(published) => published@.instance_id() == self.instance@.id(),
                None => true,
            }
    }

    pub closed spec fn well_formed(&self) -> bool {
        self.writer.is_some() == self.published.is_none()
    }

    pub closed spec fn instance_id(&self) -> InstanceId {
        self.instance@.id()
    }

    pub closed spec fn unpublished(&self) -> bool {
        self.writer.is_some()
    }

    pub closed spec fn published_value(&self) -> Option<T> {
        match self.published {
            Some(token) => Some(token@.value()),
            None => None,
        }
    }

    pub proof fn unpublished_iff_no_value(&self)
        requires
            self.well_formed(),
        ensures
            self.unpublished() == self.published_value().is_none(),
    {
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.unpublished(),
            result.published_value().is_none(),
        no_unwind
    {
        let (raw, Tracked(permission)) = RawReleaseAcquireBool::new(false);
        let tracked (Tracked(instance), Tracked(flag), Tracked(published), Tracked(writer)) =
            PublicationTokens::Instance::initialize();
        let tracked writer = match writer {
            Option::Some(writer) => writer,
            Option::None => proof_from_false(),
        };
        let tracked publication_state = PublicationGhost { flag, published };
        let instance_for_invariant: Tracked<PublicationTokens::Instance<T>> =
            Tracked(instance.clone());
        let tracked invariant = AtomicInvariant::new(
            (instance_for_invariant, raw.id()),
            (permission, publication_state),
            0,
        );
        ReleaseAcquireFlag {
            raw,
            invariant: Tracked(invariant),
            instance: Tracked(instance),
            writer: Some(Tracked(writer)),
            published: None,
        }
    }

    pub fn store_release(&mut self, Ghost(value): Ghost<T>)
        requires
            old(self).well_formed(),
            old(self).unpublished(),
        ensures
            final(self).well_formed(),
            !final(self).unpublished(),
            final(self).published_value() == Some(value),
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        let writer = match self.writer.take() {
            Option::Some(writer) => writer,
            Option::None => unreached(),
        };
        let tracked writer = writer.get();
        let tracked mut stored_output: Option<PublicationTokens::published<T>> = Option::None;
        open_atomic_invariant!(self.invariant.borrow() => pair => {
            let tracked (mut permission, mut ghost) = pair;
            self.raw.store_release(Tracked(&mut permission), true);
            proof {
                let tracked token = self.instance.borrow().publish(
                    value,
                    &mut ghost.flag,
                    writer,
                );
                stored_output = Option::Some(token.clone());
                ghost.published = Option::Some(token);
            }
            proof { pair = (permission, ghost); }
        });
        let tracked stored = match stored_output {
            Option::Some(stored) => stored,
            Option::None => proof_from_false(),
        };
        self.published = Some(Tracked(stored));
    }

    pub fn load_acquire(&self) -> (result: Option<Acquired<T>>)
        requires
            self.well_formed(),
        ensures
            match result {
                Some(acquired) => {
                    acquired.well_formed()
                        && acquired.instance_id() == self.instance_id()
                        && self.published_value() == Some(acquired.value())
                },
                None => self.published_value().is_none(),
            },
        no_unwind
    {
        proof { use_type_invariant(self); }
        let loaded;
        let tracked mut output: Option<PublicationTokens::published<T>> = Option::None;
        open_atomic_invariant!(self.invariant.borrow() => pair => {
            let tracked (permission, ghost) = pair;
            loaded = self.raw.load_acquire(Tracked(&permission));
            proof {
                match &self.published {
                    Some(stored) => {
                        self.instance.borrow().publication_implies_flag(
                            stored@.value(),
                            &ghost.flag,
                            stored.borrow(),
                        );
                    },
                    None => {
                        let tracked writer = match &self.writer {
                            Some(writer) => writer.borrow(),
                            None => proof_from_false(),
                        };
                        self.instance.borrow().writer_implies_clear(&ghost.flag, writer);
                    },
                }
                if loaded {
                    let tracked token = match &ghost.published {
                        Option::Some(token) => token.clone(),
                        Option::None => proof_from_false(),
                    };
                    output = Option::Some(token);
                }
            }
            proof { pair = (permission, ghost); }
        });
        if loaded {
            let tracked token = match output {
                Option::Some(token) => token,
                Option::None => proof_from_false(),
            };
            let stored = match &self.published {
                Some(stored) => stored,
                None => {
                    proof {
                        let tracked writer = match &self.writer {
                            Some(writer) => writer.borrow(),
                            None => proof_from_false(),
                        };
                        self.instance.borrow().publication_excludes_writer(
                            token.value(),
                            &token,
                            writer,
                        );
                    }
                    unreached()
                },
            };
            proof {
                let tracked stored_token = stored.borrow();
                self.instance.borrow().published_values_agree(
                    token.value(),
                    stored_token.value(),
                    &token,
                    stored_token,
                );
            }
            let tracked instance = self.instance.borrow().clone();
            Some(Acquired {
                instance: Tracked(instance),
                token: Tracked(token),
            })
        } else {
            None
        }
    }

    pub fn load_relaxed(&self) -> (result: bool)
        requires
            self.well_formed(),
        ensures
            result == self.published_value().is_some(),
        no_unwind
    {
        proof { use_type_invariant(self); }
        let loaded;
        open_atomic_invariant!(self.invariant.borrow() => pair => {
            let tracked (permission, ghost) = pair;
            loaded = self.raw.load_relaxed(Tracked(&permission));
            proof {
                match &self.published {
                    Some(stored) => {
                        self.instance.borrow().publication_implies_flag(
                            stored@.value(),
                            &ghost.flag,
                            stored.borrow(),
                        );
                    },
                    None => {
                        let tracked writer = match &self.writer {
                            Some(writer) => writer.borrow(),
                            None => proof_from_false(),
                        };
                        self.instance.borrow().writer_implies_clear(&ghost.flag, writer);
                    },
                }
            }
            proof { pair = (permission, ghost); }
        });
        loaded
    }

    /// Consume the exclusively owned epoch, clear its runtime bit with the
    /// production Relaxed ordering, and begin a fresh unpublished epoch. Any
    /// persistent reader tokens remain valid only for the consumed instance.
    #[verifier::external_body]
    pub fn reset_relaxed_owned(&mut self)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).unpublished(),
            final(self).published_value().is_none(),
        no_unwind
    {
        self.raw.atomic.store(false, Ordering::Relaxed);
        *self = ReleaseAcquireFlag::new();
    }
}

/// Persistent knowledge returned only by an Acquire load which observes the
/// one-shot Release publication.
pub struct Acquired<T> {
    instance: Tracked<PublicationTokens::Instance<T>>,
    token: Tracked<PublicationTokens::published<T>>,
}

impl<T> Acquired<T> {
    pub closed spec fn well_formed(&self) -> bool {
        self.instance@.id() == self.token@.instance_id()
    }

    pub closed spec fn instance_id(&self) -> InstanceId {
        self.instance@.id()
    }

    pub closed spec fn value(&self) -> T {
        self.token@.value()
    }

    pub proof fn agrees(tracked &self, tracked other: &Acquired<T>)
        requires
            self.well_formed(),
            other.well_formed(),
            self.instance_id() == other.instance_id(),
        ensures
            self.value() == other.value(),
    {
        let tracked left = self.token.borrow();
        let tracked right = other.token.borrow();
        self.instance.borrow().published_values_agree(left.value(), right.value(), left, right);
    }
}

pub fn verify_persistent_publication_token(value: u64)
{
    let tracked (Tracked(instance), Tracked(mut flag), Tracked(initial), Tracked(writer)) =
        PublicationTokens::Instance::initialize();
    assert(initial.is_none());

    let tracked writer = match writer {
        Option::Some(writer) => writer,
        Option::None => proof_from_false(),
    };
    let tracked token = instance.publish(value, &mut flag, writer);
    let tracked cloned = token.clone();
    let acquired: Acquired<u64> = Acquired {
        instance: Tracked(instance.clone()),
        token: Tracked(token),
    };
    let acquired_again: Acquired<u64> = Acquired {
        instance: Tracked(instance),
        token: Tracked(cloned),
    };
    assert(acquired.well_formed());
    assert(acquired_again.well_formed());
    proof { acquired.agrees(&acquired_again); }
    assert(acquired.value() == value);
    assert(acquired_again.value() == value);
}

pub fn verify_ordered_publication_roundtrip(value: u64)
{
    let mut flag = ReleaseAcquireFlag::<u64>::new();
    let initially_ready = flag.load_relaxed();
    assert(!initially_ready);
    let initially_acquired = flag.load_acquire();
    assert(initially_acquired.is_none());

    flag.store_release(Ghost(value));
    let ready = flag.load_relaxed();
    assert(ready);

    let first = flag.load_acquire().unwrap();
    let second = flag.load_acquire().unwrap();
    assert(first.value() == value);
    proof { first.agrees(&second); }
    assert(second.value() == value);
}

} // verus!
