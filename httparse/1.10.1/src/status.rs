//! Runtime parse status and its exact variant behavior.

extern crate creusot_std;
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, requires};

/// The result of a successful parse pass.
///
/// `Complete` is used when the buffer contained the complete value.
/// `Partial` is used when parsing did not reach the end of the expected value,
/// but no invalid data was found.
// The proof harness checks the inherent Status operations over unconstrained
// payloads. Creusot's derived equality model currently needs `T: DeepModel`,
// and the derived formatter refinement has no current model, so these runtime
// trait implementations remain explicit, unverified surfaces until those
// models are available.
#[cfg_attr(not(creusot), derive(Copy, Clone, Eq, PartialEq, Debug))]
#[cfg_attr(creusot, derive(Copy, Clone))]
pub enum Status<T> {
    /// The completed result.
    Complete(T),
    /// A partial result.
    Partial,
}

impl<T> Status<T> {
    /// Convenience method to check if status is complete.
    #[inline]
    #[ensures(match *self {
        Status::Complete(_) => result,
        Status::Partial => !result,
    })]
    pub fn is_complete(&self) -> bool {
        match *self {
            Status::Complete(..) => true,
            Status::Partial => false,
        }
    }

    /// Convenience method to check if status is partial.
    #[inline]
    #[ensures(match *self {
        Status::Complete(_) => !result,
        Status::Partial => result,
    })]
    pub fn is_partial(&self) -> bool {
        match *self {
            Status::Complete(..) => false,
            Status::Partial => true,
        }
    }

    /// Convenience method to unwrap a Complete value. Panics if the status is
    /// `Partial`.
    #[inline]
    #[requires(match self {
        Status::Complete(_) => true,
        Status::Partial => false,
    })]
    #[ensures(match self {
        Status::Complete(value) => result == value,
        Status::Partial => false,
    })]
    pub fn unwrap(self) -> T {
        match self {
            Status::Complete(t) => t,
            Status::Partial => panic!("Tried to unwrap Status::Partial"),
        }
    }
}
