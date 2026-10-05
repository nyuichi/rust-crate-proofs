use bytes_shared_nonunique_reserve::reserve_shared;
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
                    if control.get() == 0 { control.set(pointer as usize); } else { BUFFER.with(|buffer| buffer.set(pointer as usize)); }
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
                    if control.get() == 0 { control.set(pointer as usize); } else { BUFFER.with(|buffer| buffer.set(pointer as usize)); }
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
        let grown = unsafe { System.realloc(pointer, layout, size) };
        if !grown.is_null() { let _ = TRACKING.try_with(|tracking| { if tracking.get() { BUFFER.with(|buffer| buffer.set(grown as usize)); } }); }
        grown
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
fn shared_copy_reserve_retains_sibling_and_releases_both_allocations() {
    let cases = [
        (6usize, 12usize, 4usize, 7usize, (2usize, 3usize, 0usize)),
        (6, 12, 1, 10, (2, 3, 0)),
        (0, 0, 0, 1, (2, 2, 0)),
        (6, 12, 3, 0, (2, 3, 0)),
        (0, 12, 0, 0, (1, 2, 0)),
        (0, 0, 0, 0, (1, 1, 0)),
        (6, 12, 6, 0, (1, 2, 0)),
        (6, 2048, 3, 0, (2, 3, 0)),
        (0, 2048, 0, 0, (2, 3, 0)),
    ];
    for (len, requested_capacity, cut, additional, expected) in cases {
        let mut input = Vec::with_capacity(requested_capacity);
        input.extend_from_slice(&[1, 2, 3, 4, 5, 6][..len]);
        let has_new_buffer = len - cut + additional > 0 || requested_capacity >= 1024;
        let has_old_buffer = requested_capacity > 0;
        BUFFER.with(|buffer| buffer.set(if requested_capacity > 0 { input.as_ptr() as usize } else { 0 }));
        CONTROL.with(|control| control.set(0));
        FREE_TARGETS.with(|targets| targets.set((0, 0, 0)));
        assert_eq!(events_for(|| { let capacity = reserve_shared(input, cut, additional, 255); assert!(capacity >= len - cut + additional); if requested_capacity >= 1024 { assert!(capacity >= 1024); } }), expected);
        assert_eq!(FREE_TARGETS.with(Cell::get), (usize::from(has_new_buffer || has_old_buffer), 1, usize::from(has_new_buffer && has_old_buffer)));
    }
}
