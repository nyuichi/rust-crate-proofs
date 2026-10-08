use crate::loom::sync::atomic::{AtomicUsize, Ordering};

/// Guard the actual expected value before committing an increment. The update
/// closure can be retried, so it contains only pure arithmetic.
#[inline]
pub(crate) fn try_increment(counter: &AtomicUsize) -> Result<usize, usize> {
    counter.fetch_update(
        Ordering::Relaxed,
        Ordering::Relaxed,
        crate::ref_count_limit::next_ref_count,
    )
}

#[inline]
pub(crate) fn increment(counter: &AtomicUsize) {
    if try_increment(counter).is_err() {
        crate::abort();
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use crate::ref_count_limit::MAX_REF_COUNT;

    #[test]
    fn guarded_increment_accepts_last_allowed_value_without_wrap() {
        let counter = AtomicUsize::new(MAX_REF_COUNT);
        increment(&counter);
        assert_eq!(counter.load(Ordering::Relaxed), MAX_REF_COUNT + 1);
        assert_eq!(
            try_increment(&counter),
            Err(MAX_REF_COUNT + 1),
        );
        assert_eq!(counter.load(Ordering::Relaxed), MAX_REF_COUNT + 1);
    }

    #[test]
    fn guarded_increment_refuses_word_maximum_without_store() {
        let counter = AtomicUsize::new(usize::MAX);
        assert_eq!(
            try_increment(&counter),
            Err(usize::MAX),
        );
        assert_eq!(counter.load(Ordering::Relaxed), usize::MAX);
    }
}
