use bytes_actual_shared_reserve::{reserve_shared, reserve_shared_with_sibling};
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
fn public_reserve_explicit_cleanup_allocations() {
    for (len, cap, cut, additional, expected) in [
        (8usize,16usize,4usize,8usize,(1usize,2usize,0usize)),
        (8,16,4,64,(1,2,1)),
        (8,16,8,12,(1,2,0)),
        (0,0,0,0,(1,1,0)),
        (0,0,0,8,(2,2,0)),
    ] {
        let mut input=Vec::with_capacity(cap);
        input.resize(len,3);
        assert_eq!(events_for(|| { assert!(reserve_shared(input,cut,additional,9)>=len-cut+additional); }),expected);
    }
    let mut input=Vec::with_capacity(16);input.resize(8,3);
    assert_eq!(events_for(|| { assert!(reserve_shared_with_sibling(input,64)>=72); }),(2,3,0));
}
