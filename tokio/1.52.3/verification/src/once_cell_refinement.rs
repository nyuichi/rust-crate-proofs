use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OnceCellPhase {
    Empty,
    Initializing,
    Published,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BeginInit {
    Acquired,
    Wait,
    Ready,
}

#[derive(PartialEq, Eq)]
pub enum SetErrorModel<T> {
    AlreadyInitialized(T),
    Initializing(T),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SetErrorDisplay {
    AlreadyInitializedError,
    InitializingError,
}

/// Sequential projection of production OnceCell plus its one-permit
/// semaphore. S06 supplies the exclusive ownership of the Initializing phase;
/// this body proves every OnceCell-owned transition above that permit.
pub struct OnceCellModel<T> {
    value: Option<T>,
    initializing: bool,
}

impl<T> OnceCellModel<T> {
    pub closed spec fn value(&self) -> Option<T> { self.value }
    pub closed spec fn initializing(&self) -> bool { self.initializing }
    pub closed spec fn well_formed(&self) -> bool {
        self.value().is_some() ==> !self.initializing()
    }
    pub closed spec fn phase(&self) -> OnceCellPhase {
        if self.value().is_some() {
            OnceCellPhase::Published
        } else if self.initializing() {
            OnceCellPhase::Initializing
        } else {
            OnceCellPhase::Empty
        }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.phase() == OnceCellPhase::Empty,
            result.value().is_none(),
        no_unwind
    {
        OnceCellModel { value: None, initializing: false }
    }

    pub fn new_with(value: Option<T>) -> (result: Self)
        ensures
            result.well_formed(),
            result.value() == value,
            result.phase() == if value.is_some() {
                OnceCellPhase::Published
            } else {
                OnceCellPhase::Empty
            },
        no_unwind
    {
        OnceCellModel { value, initializing: false }
    }

    pub fn initialized(&self) -> (result: bool)
        requires self.well_formed(),
        ensures result == self.value().is_some(),
        no_unwind
    {
        self.value.is_some()
    }

    /// Exact projection of `Semaphore::acquire`: only Empty acquires the
    /// permit, Initializing waits, and closed/Published observes the value.
    pub fn begin_init(&mut self) -> (result: BeginInit)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            match old(self).phase() {
                OnceCellPhase::Empty => {
                    result == BeginInit::Acquired
                    && final(self).phase() == OnceCellPhase::Initializing
                    && final(self).value().is_none()
                },
                OnceCellPhase::Initializing => {
                    result == BeginInit::Wait
                    && final(self).phase() == OnceCellPhase::Initializing
                },
                OnceCellPhase::Published => {
                    result == BeginInit::Ready
                    && final(self).value() == old(self).value()
                },
            },
        no_unwind
    {
        if self.value.is_some() {
            BeginInit::Ready
        } else if self.initializing {
            BeginInit::Wait
        } else {
            self.initializing = true;
            BeginInit::Acquired
        }
    }

    /// Successful initializer: store value, Release-publish the flag, close
    /// the semaphore, then forget the permit.
    pub fn finish_init(&mut self, value: T)
        requires
            old(self).well_formed(),
            old(self).phase() == OnceCellPhase::Initializing,
        ensures
            final(self).well_formed(),
            final(self).phase() == OnceCellPhase::Published,
            final(self).value() == Some(value),
        no_unwind
    {
        self.value = Some(value);
        self.initializing = false;
    }

    /// Dropping the acquired semaphore permit after cancellation, panic, or
    /// `get_or_try_init` error reopens the empty cell without publishing.
    pub fn abandon_init(&mut self)
        requires
            old(self).well_formed(),
            old(self).phase() == OnceCellPhase::Initializing,
        ensures
            final(self).well_formed(),
            final(self).phase() == OnceCellPhase::Empty,
            final(self).value().is_none(),
        no_unwind
    {
        self.initializing = false;
    }

    pub fn set(&mut self, value: T) -> (result: Result<(), SetErrorModel<T>>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            match old(self).phase() {
                OnceCellPhase::Empty => {
                    result.is_ok()
                    && final(self).value() == Some(value)
                    && final(self).phase() == OnceCellPhase::Published
                },
                OnceCellPhase::Initializing => {
                    result == Err(SetErrorModel::Initializing(value))
                    && final(self).phase() == OnceCellPhase::Initializing
                },
                OnceCellPhase::Published => {
                    result == Err(SetErrorModel::AlreadyInitialized(value))
                    && final(self).value() == old(self).value()
                },
            },
        no_unwind
    {
        if self.value.is_some() {
            Err(SetErrorModel::AlreadyInitialized(value))
        } else if self.initializing {
            Err(SetErrorModel::Initializing(value))
        } else {
            self.value = Some(value);
            Ok(())
        }
    }

    /// Exclusive `get_mut` may replace only the published payload and leaves
    /// the semaphore/flag phase published.
    pub fn replace_mut(&mut self, value: T)
        requires
            old(self).well_formed(),
            old(self).phase() == OnceCellPhase::Published,
        ensures
            final(self).well_formed(),
            final(self).phase() == OnceCellPhase::Published,
            final(self).value() == Some(value),
        no_unwind
    {
        self.value = Some(value);
    }

    pub fn take(&mut self) -> (result: Option<T>)
        requires old(self).well_formed(), !old(self).initializing(),
        ensures
            result == old(self).value(),
            final(self).well_formed(),
            final(self).phase() == OnceCellPhase::Empty,
        no_unwind
    {
        let mut value = None;
        core::mem::swap(&mut self.value, &mut value);
        self.initializing = false;
        value
    }
}

pub fn set_error_display<T>(error: &SetErrorModel<T>) -> (result: SetErrorDisplay)
    ensures
        result == match error {
            SetErrorModel::AlreadyInitialized(_) => SetErrorDisplay::AlreadyInitializedError,
            SetErrorModel::Initializing(_) => SetErrorDisplay::InitializingError,
        },
    no_unwind
{
    match error {
        SetErrorModel::AlreadyInitialized(_) => SetErrorDisplay::AlreadyInitializedError,
        SetErrorModel::Initializing(_) => SetErrorDisplay::InitializingError,
    }
}

pub fn set_error_classifiers<T>(error: &SetErrorModel<T>) -> (result: (bool, bool))
    ensures
        result == match error {
            SetErrorModel::AlreadyInitialized(_) => (true, false),
            SetErrorModel::Initializing(_) => (false, true),
        },
    no_unwind
{
    match error {
        SetErrorModel::AlreadyInitialized(_) => (true, false),
        SetErrorModel::Initializing(_) => (false, true),
    }
}

pub fn set_error_debug_variant<T>(error: &SetErrorModel<T>) -> (result: SetErrorDisplay)
    ensures
        result == match error {
            SetErrorModel::AlreadyInitialized(_) => SetErrorDisplay::AlreadyInitializedError,
            SetErrorModel::Initializing(_) => SetErrorDisplay::InitializingError,
        },
    no_unwind
{
    set_error_display(error)
}

pub fn set_error_source_is_none<T>(_error: &SetErrorModel<T>) -> (result: bool)
    ensures result,
    no_unwind
{
    true
}

/// Generic Clone orchestration. The caller supplies the value-level result of
/// `T::clone`; OnceCell state selection and construction remain body-proved.
pub fn clone_from_value<T>(
    source: &OnceCellModel<T>,
    cloned: Option<T>,
) -> (result: OnceCellModel<T>)
    requires
        source.well_formed(),
        cloned == source.value(),
    ensures
        result.well_formed(),
        result.value() == source.value(),
        result.phase() == if source.value().is_some() {
            OnceCellPhase::Published
        } else {
            OnceCellPhase::Empty
        },
    no_unwind
{
    OnceCellModel::new_with(cloned)
}

/// Generic PartialEq orchestration above the supplied `T::eq` result.
pub fn eq_from_value<T>(
    left: &OnceCellModel<T>,
    right: &OnceCellModel<T>,
    initialized_values_equal: bool,
) -> (result: bool)
    requires
        left.well_formed(),
        right.well_formed(),
        left.value().is_some() && right.value().is_some() ==>
            initialized_values_equal == (left.value() == right.value()),
    ensures result == (left.value() == right.value()),
    no_unwind
{
    match (&left.value, &right.value) {
        (Some(_), Some(_)) => initialized_values_equal,
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

pub fn debug_phase<T>(cell: &OnceCellModel<T>) -> (result: OnceCellPhase)
    requires cell.well_formed(),
    ensures
        result == if cell.value().is_some() {
            OnceCellPhase::Published
        } else {
            OnceCellPhase::Empty
        },
    no_unwind
{
    if cell.value.is_some() {
        OnceCellPhase::Published
    } else {
        OnceCellPhase::Empty
    }
}

pub fn verify_success_wait_and_retry_paths(first: u64, second: u64)
{
    let mut cell = OnceCellModel::new();
    let acquired = cell.begin_init();
    let waiting = cell.begin_init();
    assert(acquired == BeginInit::Acquired);
    assert(waiting == BeginInit::Wait);
    cell.finish_init(first);
    let ready = cell.begin_init();
    assert(ready == BeginInit::Ready);
    assert(cell.value() == Some(first));

    let mut cancelled = OnceCellModel::new();
    let first_acquired = cancelled.begin_init();
    let recursive_wait = cancelled.begin_init();
    assert(first_acquired == BeginInit::Acquired);
    assert(recursive_wait == BeginInit::Wait); // recursive call waits
    cancelled.abandon_init(); // panic/cancel drops the only permit
    let retry_after_cancel = cancelled.begin_init();
    assert(retry_after_cancel == BeginInit::Acquired);
    cancelled.abandon_init(); // get_or_try_init Err
    let retry_after_error = cancelled.begin_init();
    assert(retry_after_error == BeginInit::Acquired);
    cancelled.finish_init(second);
    assert(cancelled.value() == Some(second));
}

pub fn verify_set_take_mutate_and_errors(first: u64, second: u64)
{
    let mut cell = OnceCellModel::new();
    let first_set = cell.set(first);
    assert(first_set.is_ok());
    let already = cell.set(second);
    assert(already == Err(SetErrorModel::AlreadyInitialized(second)));
    match &already {
        Err(error) => {
            let display = set_error_display(error);
            let classes = set_error_classifiers(error);
            assert(display == SetErrorDisplay::AlreadyInitializedError);
            assert(classes == (true, false));
        },
        Ok(_) => { assert(false); },
    }
    cell.replace_mut(second);
    let taken = cell.take();
    assert(taken == Some(second));
    assert(cell.phase() == OnceCellPhase::Empty);

    let acquired = cell.begin_init();
    assert(acquired == BeginInit::Acquired);
    let initializing = cell.set(first);
    assert(initializing == Err(SetErrorModel::Initializing(first)));
    match &initializing {
        Err(error) => {
            let display = set_error_display(error);
            let classes = set_error_classifiers(error);
            let debug = set_error_debug_variant(error);
            let no_source = set_error_source_is_none(error);
            assert(display == SetErrorDisplay::InitializingError);
            assert(classes == (false, true));
            assert(debug == SetErrorDisplay::InitializingError);
            assert(no_source);
        },
        Ok(_) => { assert(false); },
    }
}

pub fn verify_trait_orchestration(value: u64)
{
    let empty = OnceCellModel::<u64>::new();
    let empty_clone = clone_from_value(&empty, None);
    let empty_eq = eq_from_value(&empty, &empty_clone, false);
    let empty_debug = debug_phase(&empty);
    assert(empty_eq);
    assert(empty_debug == OnceCellPhase::Empty);

    let published = OnceCellModel::new_with(Some(value));
    let published_clone = clone_from_value(&published, Some(value));
    let published_eq = eq_from_value(&published, &published_clone, true);
    let published_debug = debug_phase(&published);
    assert(published_eq);
    assert(published_debug == OnceCellPhase::Published);

    let mut initializing = OnceCellModel::<u64>::new();
    let acquired = initializing.begin_init();
    assert(acquired == BeginInit::Acquired);
    let initializing_clone = clone_from_value(&initializing, None);
    let initializing_eq_empty = eq_from_value(&initializing, &empty, false);
    let initializing_debug = debug_phase(&initializing);
    assert(initializing_clone.phase() == OnceCellPhase::Empty);
    assert(initializing_eq_empty);
    assert(initializing_debug == OnceCellPhase::Empty);
}

} // verus!
