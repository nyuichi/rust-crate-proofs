//! Native allocation events for the exact extracted explicit control helpers.
use bytes_sequential_bytesmut_split::{left_then_right, right_then_left};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static EVENTS: Cell<(usize, usize, usize)> = const { Cell::new((0, 0, 0)) };
}

struct CountingAllocator;
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn record(allocation: bool) {
    let _ = TRACKING.try_with(|tracking| {
        if tracking.get() {
            let _ = EVENTS.try_with(|events| {
                let (allocated, freed, reallocated) = events.get();
                events.set((allocated + usize::from(allocation), freed + usize::from(!allocation), reallocated));
            });
        }
    });
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() { record(true); }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() { record(true); }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(false);
        unsafe { System.dealloc(pointer, layout); }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let _ = TRACKING.try_with(|tracking| {
            if tracking.get() {
                let _ = EVENTS.try_with(|events| {
                    let (allocated, freed, reallocated) = events.get();
                    events.set((allocated, freed, reallocated + 1));
                });
            }
        });
        unsafe { System.realloc(pointer, layout, size) }
    }
}

struct TrackingScope;
impl Drop for TrackingScope {
    fn drop(&mut self) { TRACKING.with(|tracking| tracking.set(false)); }
}

fn events_for(action: impl FnOnce()) -> (usize, usize, usize) {
    EVENTS.with(|events| events.set((0, 0, 0)));
    TRACKING.with(|tracking| tracking.set(true));
    let scope = TrackingScope;
    action();
    drop(scope);
    EVENTS.with(Cell::get)
}

#[test]
fn explicit_release_frees_buffer_and_control_in_both_orders() {
    for release in [left_then_right, right_then_left] {
        for split in 0..=4 {
            // A already exists; inside the scope only S is allocated.
            let input = vec![1, 2, 3, 4];
            assert_eq!(events_for(|| release(input, split)), (1, 2, 0));
        }
        let reserved = Vec::with_capacity(32);
        assert_eq!(events_for(|| release(reserved, 0)), (1, 2, 0));
        // Capacity zero has no A allocation to free, but S still exists.
        assert_eq!(events_for(|| release(Vec::new(), 0)), (1, 1, 0));
    }
}

#[test]
fn mutable_views_preserve_exact_allocation_recovery() {
    use bytes_sequential_bytesmut_split::{access_both, mutate_both};
    for right_first in [false, true] {
        for split in 1..4 {
            let mut input = Vec::with_capacity(32);
            input.extend_from_slice(&[1, 2, 3, 4]);
            assert_eq!(events_for(|| mutate_both(input, split, 71, 93, right_first)), (1, 2, 0));
        }
        let reserved = Vec::with_capacity(32);
        assert_eq!(events_for(|| access_both(reserved, 0, right_first)), (1, 2, 0));
        assert_eq!(events_for(|| access_both(Vec::new(), 0, right_first)), (1, 1, 0));
    }
}

#[test]
fn split_off_spare_ranges_recover_without_reallocation() {
    use bytes_sequential_bytesmut_split::split_off_both;
    for right_first in [false, true] {
        for (len, capacity) in [(0usize, 0usize), (0, 8), (4, 4), (4, 8)] {
            for at in 0..=capacity {
                let mut input = Vec::with_capacity(capacity);
                input.extend_from_slice(&[1, 2, 3, 4][..len]);
                assert_eq!(input.capacity(), capacity);
                let expected_frees = if capacity == 0 { 1 } else { 2 };
                assert_eq!(events_for(|| split_off_both(input, at, right_first)), (1, expected_frees, 0));
            }
        }
    }
}

#[test]
fn shrinking_lengths_retains_allocation_recovery() {
    use bytes_sequential_bytesmut_split::shrink_split_off;
    for right_first in [false, true] {
        for (len, capacity) in [(0usize, 0usize), (0, 8), (4, 4), (4, 8)] {
            for at in 0..=capacity {
                for keep in [0, 1, 4, 9] {
                    let mut input = Vec::with_capacity(capacity);
                    input.extend_from_slice(&[1, 2, 3, 4][..len]);
                    let expected_frees = if capacity == 0 { 1 } else { 2 };
                    assert_eq!(events_for(|| shrink_split_off(input, at, keep, right_first)), (1, expected_frees, 0));
                }
            }
        }
    }
}

#[test]
fn advanced_views_retain_prefix_for_complete_recovery() {
    use bytes_sequential_bytesmut_split::advance_split_off;
    for right_first in [false, true] {
        for (len, capacity) in [(0usize, 0usize), (0, 8), (4, 4), (4, 8)] {
            for at in 0..=capacity {
                for left_count in [0, len.min(at), at] {
                    for right_count in [0, len.saturating_sub(at), capacity - at] {
                        let mut input = Vec::with_capacity(capacity);
                        input.extend_from_slice(&[1, 2, 3, 4][..len]);
                        let expected_frees = if capacity == 0 { 1 } else { 2 };
                        assert_eq!(events_for(|| advance_split_off(input, at, left_count, right_count, right_first)), (1, expected_frees, 0));
                    }
                }
            }
        }
    }
}

#[test]
fn initializing_spare_bytes_preserves_exact_recovery() {
    use bytes_sequential_bytesmut_split::{initialize_split_spare, reinitialize_split_prefix};
    for right_first in [false, true] {
        for (len, capacity) in [(0usize, 0usize), (0, 8), (4, 4), (4, 8)] {
            for at in 0..=capacity {
                let mut input = Vec::with_capacity(capacity);
                input.extend_from_slice(&[1, 2, 3, 4][..len]);
                let expected_frees = if capacity == 0 { 1 } else { 2 };
                assert_eq!(events_for(|| initialize_split_spare(input, at, right_first)), (1, expected_frees, 0));
            }
        }
        for capacity in [4, 8] {
            let mut input = Vec::with_capacity(capacity);
            input.extend_from_slice(&[1, 2, 3, 4]);
            assert_eq!(events_for(|| reinitialize_split_prefix(input, right_first)), (1, 2, 0));
        }
    }
}

#[test]
fn shared_reads_and_disjoint_writes_preserve_recovery() {
    use bytes_sequential_bytesmut_split::readonly_split;
    for right_first in [false, true] {
        for (len, capacity) in [(0usize, 0usize), (0, 8), (4, 4), (4, 8)] {
            for at in 0..=capacity {
                let mut input = Vec::with_capacity(capacity);
                input.extend_from_slice(&[1, 2, 3, 4][..len]);
                let expected_frees = if capacity == 0 { 1 } else { 2 };
                assert_eq!(events_for(|| readonly_split(input, at, right_first)), (1, expected_frees, 0));
            }
        }
    }
}

#[test]
fn unique_access_has_no_control_allocation() {
    use bytes_sequential_bytesmut_split::unique_access;
    for (len, capacity) in [(0usize, 0usize), (0, 8), (4, 4), (4, 8)] {
        for keep in [0, 2, 4, 9] {
            for value in [0, 255] {
                let mut input = Vec::with_capacity(capacity);
                input.extend_from_slice(&[1, 2, 3, 4][..len]);
                let expected_frees = usize::from(capacity != 0);
                assert_eq!(events_for(|| unique_access(input, keep, value)), (0, expected_frees, 0));
            }
        }
    }
}

#[test]
fn resize_and_append_within_capacity_do_not_allocate_or_reallocate() {
    use bytes_sequential_bytesmut_split::{noalloc_split, noalloc_unique};
    for new_len in 0..=8 {
        let mut input = Vec::with_capacity(8);
        input.extend_from_slice(&[1, 2, 3, 4]);
        let extension = &[9u8; 8][..8 - new_len];
        assert_eq!(events_for(|| noalloc_unique(input, new_len, 7, extension)), (0, 1, 0));
    }
    for left_first in [false, true] {
        for new_len in 0..=7 {
            let mut input = Vec::with_capacity(8);
            input.extend_from_slice(&[1, 2, 3, 4]);
            let extension = &[9u8; 8][..7 - new_len];
            assert_eq!(events_for(|| noalloc_split(input, 1, new_len, 7, extension, left_first)), (1, 2, 0));
        }
        assert_eq!(events_for(|| noalloc_split(Vec::new(), 0, 0, 7, &[], left_first)), (1, 1, 0));
    }
    assert_eq!(events_for(|| noalloc_unique(Vec::new(), 0, 7, &[])), (0, 0, 0));
}

#[test]
fn advanced_unique_cleanup_uses_the_original_allocation_without_a_control_block() {
    use bytes_sequential_bytesmut_split::unique_advance;
    for first in 0..=8 {
        for second in 0..=8 - first {
            let mut input = Vec::with_capacity(8);
            input.extend_from_slice(&[1, 2, 3, 4]);
            assert_eq!(events_for(|| unique_advance(input, first, second, 9)), (0, 1, 0));
        }
    }
    assert_eq!(events_for(|| unique_advance(Vec::new(), 0, 0, 9)), (0, 0, 0));
}
