struct Shared {
    // Holds arguments to dealloc upon Drop, but otherwise doesn't use them
    buf: *mut u8,
    cap: usize,
    ref_cnt: AtomicUsize,
}