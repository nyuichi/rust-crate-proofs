//! Native allocator observations for the ordinary `BytesMut` destructor.
//!
//! These tests exercise public APIs and check the concrete allocator events.
//! They do not establish a formal ownership or concurrency proof.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

const MAX_EVENTS: usize = 4;
type AllocationEvent = (usize, usize, usize); // pointer, size, alignment

#[derive(Clone, Copy)]
struct EventLog {
    allocations: [AllocationEvent; MAX_EVENTS],
    allocation_count: usize,
    deallocations: [AllocationEvent; MAX_EVENTS],
    deallocation_count: usize,
    reallocations: usize,
}

const EMPTY_EVENT: AllocationEvent = (0, 0, 0);
const EMPTY_LOG: EventLog = EventLog {
    allocations: [EMPTY_EVENT; MAX_EVENTS],
    allocation_count: 0,
    deallocations: [EMPTY_EVENT; MAX_EVENTS],
    deallocation_count: 0,
    reallocations: 0,
};

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static EVENTS: Cell<EventLog> = const { Cell::new(EMPTY_LOG) };
}

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn record_allocation(pointer: *mut u8, layout: Layout) {
    let _ = TRACKING.try_with(|tracking| {
        if tracking.get() {
            let _ = EVENTS.try_with(|events| {
                let mut log = events.get();
                let index = log.allocation_count;
                if index < MAX_EVENTS {
                    log.allocations[index] = (pointer as usize, layout.size(), layout.align());
                }
                log.allocation_count += 1;
                events.set(log);
            });
        }
    });
}

fn record_deallocation(pointer: *mut u8, layout: Layout) {
    let _ = TRACKING.try_with(|tracking| {
        if tracking.get() {
            let _ = EVENTS.try_with(|events| {
                let mut log = events.get();
                let index = log.deallocation_count;
                if index < MAX_EVENTS {
                    log.deallocations[index] = (pointer as usize, layout.size(), layout.align());
                }
                log.deallocation_count += 1;
                events.set(log);
            });
        }
    });
}

fn record_reallocation() {
    let _ = TRACKING.try_with(|tracking| {
        if tracking.get() {
            let _ = EVENTS.try_with(|events| {
                let mut log = events.get();
                log.reallocations += 1;
                events.set(log);
            });
        }
    });
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record_allocation(pointer, layout);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record_allocation(pointer, layout);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record_deallocation(pointer, layout);
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_reallocation();
        unsafe { System.realloc(pointer, layout, size) }
    }
}

struct TrackingScope;

impl Drop for TrackingScope {
    fn drop(&mut self) {
        TRACKING.with(|tracking| tracking.set(false));
    }
}

fn capture_events(action: impl FnOnce()) -> EventLog {
    EVENTS.with(|events| events.set(EMPTY_LOG));
    TRACKING.with(|tracking| tracking.set(true));
    let scope = TrackingScope;
    action();
    drop(scope);
    EVENTS.with(Cell::get)
}

fn assert_no_unexpected_events(log: EventLog) {
    assert_eq!(log.reallocations, 0, "unexpected buffer reallocation");
    assert!(log.allocation_count <= MAX_EVENTS, "too many allocations");
    assert!(
        log.deallocation_count <= MAX_EVENTS,
        "too many deallocations"
    );
}

fn assert_unique_buffer_freed_once(log: EventLog, base: usize, capacity: usize) {
    assert_no_unexpected_events(log);
    assert_eq!(log.allocation_count, 0, "unique drop allocated");
    if capacity == 0 {
        assert_eq!(log.deallocation_count, 0, "zero-capacity buffer was freed");
        return;
    }

    assert_eq!(log.deallocation_count, 1, "expected exactly one free");
    assert_eq!(
        log.deallocations[0],
        (base, capacity, 1),
        "unique drop must free the original allocation base and full layout"
    );
}

fn assert_shared_allocations_freed_once(log: EventLog, base: usize, capacity: usize) {
    assert_no_unexpected_events(log);
    assert_eq!(
        log.allocation_count, 1,
        "split should allocate one control block"
    );
    assert_eq!(log.deallocation_count, 1 + usize::from(capacity != 0));

    let (control, control_size, control_align) = log.allocations[0];
    assert_ne!(control, 0);
    assert_ne!(control, base);

    let mut saw_control = false;
    let mut saw_buffer = capacity == 0;
    for &(pointer, size, align) in &log.deallocations[..log.deallocation_count] {
        if pointer == control {
            assert!(!saw_control, "control block was freed twice");
            assert_eq!((size, align), (control_size, control_align));
            saw_control = true;
        } else if capacity != 0 && pointer == base {
            assert!(!saw_buffer, "buffer allocation was freed twice");
            assert_eq!((size, align), (capacity, 1));
            saw_buffer = true;
        } else if capacity != 0 && pointer > base && pointer - base < capacity {
            panic!("freed an interior pointer into the original buffer allocation");
        } else {
            panic!("unexpected deallocation at {pointer:#x}");
        }
    }
    assert!(saw_control, "shared control block was not freed");
    assert!(saw_buffer, "original buffer allocation was not freed");
}

#[test]
fn ordinary_unique_advance_releases_the_original_allocation_after_spare_writes() {
    let mut buffer = BytesMut::with_capacity(32);
    buffer.put_slice(b"abcdef");
    let base = buffer.as_mut().as_mut_ptr() as usize;
    let capacity = buffer.capacity();
    assert!(capacity >= 32);

    let log = capture_events(|| {
        buffer.advance(2);
        buffer.put_slice(b"ghij");
        let remaining = buffer.len();
        buffer.advance(remaining);
        assert!(buffer.is_empty());
        assert!(buffer.capacity() < capacity);
        drop(buffer);
    });

    assert_unique_buffer_freed_once(log, base, capacity);
}

fn exercise_endpoint_split(split_to_len: bool, drop_order: [usize; 3]) {
    let mut source = BytesMut::with_capacity(32);
    source.put_slice(b"abcdef");
    let base = source.as_mut().as_mut_ptr() as usize;
    let capacity = source.capacity();
    let len = source.len();
    let first = if split_to_len { len } else { 0 };
    let second = capacity;

    assert!(capacity >= 32);
    assert!(first <= len);
    assert!(first <= second && second <= capacity);

    let log = capture_events(|| {
        let left = source.split_to(first);
        let right = source.split_off(second - first);
        let mut handles = [Some(left), Some(source), Some(right)];
        for index in drop_order {
            drop(
                handles[index]
                    .take()
                    .expect("each split handle is dropped once"),
            );
        }
    });

    assert_shared_allocations_freed_once(log, base, capacity);
}

#[test]
fn shared_endpoint_splits_free_buffer_and_control_once_in_each_drop_order() {
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    for order in ORDERS {
        // Exercise split_to(0), followed by split_off(capacity).
        exercise_endpoint_split(false, order);
        // Exercise split_to(len), followed by split_off(remaining capacity).
        exercise_endpoint_split(true, order);
    }
}

#[test]
fn zero_capacity_shared_splits_free_only_the_control_block() {
    let mut source = BytesMut::new();
    assert_eq!(source.capacity(), 0);
    let base = source.as_mut().as_mut_ptr() as usize;

    let log = capture_events(|| {
        let left = source.split_to(0);
        let right = source.split_off(0);
        drop(left);
        drop(source);
        drop(right);
    });

    assert_shared_allocations_freed_once(log, base, 0);
}

#[test]
fn zero_capacity_unique_drop_does_not_free_a_dangling_pointer() {
    let buffer = BytesMut::new();
    assert_eq!(buffer.capacity(), 0);
    let log = capture_events(|| drop(buffer));
    assert_unique_buffer_freed_once(log, 0, 0);
}

/// Select Bytes' existing len < capacity Shared constructor, then its actual
/// SHARED_VTABLE clone and Release/Acquire destructor paths. Allocator observations
/// complement the proof gate; they do not establish destructor verification.
#[test]
fn bytes_shared_clone_keeps_live_reads_and_frees_both_allocations_once() {
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2], [0, 2, 1], [1, 0, 2],
        [1, 2, 0], [2, 0, 1], [2, 1, 0],
    ];
    for input in [&b""[..], &b"x"[..], &b"shared physical bytes"[..]] {
        for order in ORDERS {
            let mut storage = Vec::with_capacity(input.len() + 8);
            storage.extend_from_slice(input);
            let base = storage.as_ptr() as usize;
            let capacity = storage.capacity();
            assert!(storage.len() < capacity);
            let log = capture_events(|| {
                let first = Bytes::from(storage);
                let second = first.clone();
                let third = second.clone();
                assert_eq!(first.as_ptr() as usize, base);
                assert_eq!(second.as_ptr(), first.as_ptr());
                assert_eq!(third.as_ptr(), first.as_ptr());
                let mut handles = [Some(first), Some(second), Some(third)];
                for (step, index) in order.into_iter().enumerate() {
                    // Observe another live owner before each release.
                    let observed = handles.iter().enumerate()
                        .find(|(other, value)| *other != index && value.is_some())
                        .map(|(_, value)| value.as_ref().unwrap().as_ref());
                    if let Some(slice) = observed { assert_eq!(slice, input); }
                    drop(handles[index].take().unwrap());
                    if step < 2 {
                        EVENTS.with(|events| assert_eq!(events.get().deallocation_count, 0));
                        for remaining in handles.iter().flatten() {
                            assert_eq!(remaining.as_ref(), input);
                        }
                    }
                }
            });
            assert_shared_allocations_freed_once(log, base, capacity);
        }
    }
}

#[test]
fn bytes_shared_slice_remains_live_across_another_owners_release() {
    let input = b"retained slice";
    let mut storage = Vec::with_capacity(input.len() + 8);
    storage.extend_from_slice(input);
    let base = storage.as_ptr() as usize;
    let capacity = storage.capacity();
    let log = capture_events(|| {
        let source = Bytes::from(storage);
        let other = source.clone();
        let slice = source.as_ref();
        drop(other);
        EVENTS.with(|events| assert_eq!(events.get().deallocation_count, 0));
        assert_eq!(slice, input);
        drop(source);
    });
    assert_shared_allocations_freed_once(log, base, capacity);
}
