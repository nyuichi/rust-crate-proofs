//! Body-checked bytes caller for the reviewed generic scoped-slot Std boundary.
//! Startup failure retains the lease for explicit parent cleanup.
use super::*;
use creusot_std::ghost::invariant::Tokens;

#[requires(tokens.contains(retirement::PUBLICATION()))]
#[ensures(result.0 == if index@ < input@.len() {Some(input@[index@])} else {None})]
#[ensures(result.2 != result.3)]
pub fn parent_retained_builder_attempt(
    input: Vec<u8>,
    index: usize,
    force_builder_error: bool,
    mut tokens: Ghost<Tokens>,
) -> (Option<u8>, bool, bool, bool) {
    use creusot_std::std::thread::{self, JoinHandleExt};

    let mut owner = Owner::new(input);
    let (left, right, context) = owner.share_pair();
    let mut left_slot = Some(left);

    let worker = thread::scope(|scope| {
        let stack_size = if force_builder_error {
            // Native Builder rejection exercises the explicit Err branch.
            Some(usize::MAX)
        } else {
            None
        };
        match scope.try_spawn_with_slot(stack_size, &mut left_slot, |slot, _tokens| {
            let handle = slot.take().expect("worker started with occupied lease slot");
            let value = if index < handle.len() {
                Some(handle.read(index))
            } else {
                None
            };
            (value, handle)
        }) {
            Some(join) => Some(join.join_unwrap()),
            None => None,
        }
    });

    let spawned = worker.is_some();
    let (value, left) = match worker {
        Some(worker) => worker,
        None => {
            let handle = left_slot
                .take()
                .expect("failed spawn preserved the parent-owned lease");
            let value = if index < handle.len() {
                Some(handle.read(index))
            } else {
                None
            };
            (value, handle)
        }
    };

    let left = left.close(&context, ghost! { tokens.reborrow() });
    let right = right.close(&context, tokens);
    let flags = owner.close(context, left, right);
    (value, spawned, flags.0, flags.1)
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;

    fn token() -> Ghost<Tokens<'static>> {
        ghost! { Tokens::new().into_inner() }
    }

    #[test]
    fn successful_spawn_returns_the_lease_to_parent() {
        let input = vec![31, 32, 33, 34];
        let (value, spawned, a, b) = parent_retained_builder_attempt(input, 2, false, token());
        assert_eq!(value, Some(33));
        assert!(spawned);
        assert_ne!(a, b);
    }

    #[test]
    fn builder_error_leaves_the_parent_slot_for_explicit_close() {
        let input = Vec::with_capacity(16);
        let (value, spawned, a, b) = parent_retained_builder_attempt(input, 2, true, token());
        assert_eq!(value, None);
        assert!(!spawned);
        assert_ne!(a, b);
    }
}
