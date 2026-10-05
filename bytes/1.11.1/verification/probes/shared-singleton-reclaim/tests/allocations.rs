use bytes_shared_singleton_reclaim::reclaim_singleton;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static BUFFER: Cell<usize> = const { Cell::new(0) };
    static CONTROL: Cell<usize> = const { Cell::new(0) };
    static FREE_TARGETS: Cell<(usize, usize, usize)> = const { Cell::new((0, 0, 0)) };
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
        if !pointer.is_null() {
            record(true);
            let _ = TRACKING.try_with(|tracking| {
                if tracking.get() { let _ = CONTROL.try_with(|control| {
                    if control.get() == 0 { control.set(pointer as usize); }
                }); }
            });
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(true);
            let _ = TRACKING.try_with(|tracking| {
                if tracking.get() { let _ = CONTROL.try_with(|control| {
                    if control.get() == 0 { control.set(pointer as usize); }
                }); }
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(false);
        let _ = TRACKING.try_with(|tracking| {
            if tracking.get() {
                let address = pointer as usize;
                let buffer = BUFFER.try_with(Cell::get).unwrap_or(0);
                let control = CONTROL.try_with(Cell::get).unwrap_or(0);
                let _ = FREE_TARGETS.try_with(|targets| {
                    let (a, s, other) = targets.get();
                    targets.set((a + usize::from(address == buffer),
                        s + usize::from(address == control),
                        other + usize::from(address != buffer && address != control)));
                });
            }
        });
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
fn singleton_reactivation_preserves_original_buffer_and_control() {
    for (len, requested_capacity) in [(0usize, 0usize), (0, 12), (6, 6), (6, 12)] {
        for cut in [0, 1, 3, 6, 12, usize::MAX] {
            let mut input = Vec::with_capacity(requested_capacity);
            input.extend_from_slice(&[1, 2, 3, 4, 5, 6][..len]);
            let has_buffer = input.capacity() != 0;
            BUFFER.with(|buffer| buffer.set(if has_buffer { input.as_ptr() as usize } else { 0 }));
            CONTROL.with(|control| control.set(0));
            FREE_TARGETS.with(|targets| targets.set((0, 0, 0)));
            assert_eq!(events_for(|| reclaim_singleton(input, cut, 255)), (1, 1 + usize::from(has_buffer), 0));
            assert_eq!(FREE_TARGETS.with(Cell::get), (usize::from(has_buffer), 1, 0));
        }
    }
}
