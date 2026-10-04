//! Native allocation events for the exact extracted explicit control helpers.
use bytes_sequential_bytesmut_split::{left_then_right, right_then_left};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static EVENTS: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

struct CountingAllocator;
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn record(allocation: bool) {
    let _ = TRACKING.try_with(|tracking| {
        if tracking.get() {
            let _ = EVENTS.try_with(|events| {
                let (allocated, freed) = events.get();
                events.set((allocated + usize::from(allocation), freed + usize::from(!allocation)));
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
        unsafe { System.realloc(pointer, layout, size) }
    }
}

struct TrackingScope;
impl Drop for TrackingScope {
    fn drop(&mut self) { TRACKING.with(|tracking| tracking.set(false)); }
}

fn events_for(action: impl FnOnce()) -> (usize, usize) {
    EVENTS.with(|events| events.set((0, 0)));
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
            assert_eq!(events_for(|| release(input, split)), (1, 2));
        }
        let reserved = Vec::with_capacity(32);
        assert_eq!(events_for(|| release(reserved, 0)), (1, 2));
        // Capacity zero has no A allocation to free, but S still exists.
        assert_eq!(events_for(|| release(Vec::new(), 0)), (1, 1));
    }
}

#[test]
fn mutable_views_preserve_exact_allocation_recovery() {
    use bytes_sequential_bytesmut_split::{access_both, mutate_both};
    for right_first in [false, true] {
        for split in 1..4 {
            let mut input = Vec::with_capacity(32);
            input.extend_from_slice(&[1, 2, 3, 4]);
            assert_eq!(events_for(|| mutate_both(input, split, 71, 93, right_first)), (1, 2));
        }
        let reserved = Vec::with_capacity(32);
        assert_eq!(events_for(|| access_both(reserved, 0, right_first)), (1, 2));
        assert_eq!(events_for(|| access_both(Vec::new(), 0, right_first)), (1, 1));
    }
}
