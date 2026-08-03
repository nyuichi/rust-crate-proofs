use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use vstd::prelude::*;

verus! {

// These four declarations deliberately make the unsupported hand-written
// Future::poll surface opaque. They test whether a Tokio-shaped Future impl can
// be connected to Verus; they do not specify the behavior of these types.
#[verifier::reject_recursive_types(Ptr)]
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPin<Ptr>(core::pin::Pin<Ptr>);

#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPoll<T>(core::task::Poll<T>);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExContext<'a>(core::task::Context<'a>);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWaker(core::task::Waker);

pub struct RecvError(pub ());

// This is only the smallest shape needed to reproduce the production
// Receiver<T> Future signature. The production receiver stores an
// Option<Arc<Inner<T>>> plus tracing fields under its tracing configuration.
pub struct Receiver<T> {
    pub inner_is_present: bool,
    pub marker: core::marker::PhantomData<T>,
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, RecvError>;

    // Tokio 1.52.3 uses this exact Future::poll signature. The body remains an
    // explicit trusted boundary until Pin, Poll, Context, and Waker have usable
    // Verus specifications and the oneshot state machine is connected to them.
    #[verifier::external_body]
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let _ = (&mut self, cx);
        unimplemented!()
    }
}

}
