/// Holds the allocation information released by `Shared::drop`.
pub(crate) struct Shared {
    buf: *mut u8,
    cap: usize,
    pub(crate) ref_cnt: AtomicUsize,
}
