// ORIGINAL_SHARED_BEGIN shared_vtable
static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
};
// ORIGINAL_SHARED_END shared_vtable

// ORIGINAL_SHARED_BEGIN shared_table_native
// Closed native target for the generic proof-only Vtable reification boundary.
#[allow(dead_code)]
fn original_shared_table_native() -> &'static Vtable {
    &SHARED_VTABLE
}
// ORIGINAL_SHARED_END shared_table_native

// ORIGINAL_SHARED_BEGIN shared_clone
unsafe fn shared_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Relaxed);
    shallow_clone_arc(shared as _, ptr, len)
}
// ORIGINAL_SHARED_END shared_clone

// ORIGINAL_SHARED_BEGIN shallow_clone_arc
unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    crate::ref_count_ops::increment(&(*shared).ref_cnt);

    Bytes {

        #[cfg(all(creusot, bytes_original_freeze_gate))]

        original_frozen: None,
        ptr,
        len,
        data: AtomicPtr::new(shared as _),
        vtable: &SHARED_VTABLE,
    }
}
// ORIGINAL_SHARED_END shallow_clone_arc

// ORIGINAL_SHARED_BEGIN shared_drop
unsafe fn shared_drop(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    data.with_mut(|shared| {
        release_shared(shared.cast());
    });
}
// ORIGINAL_SHARED_END shared_drop

// ORIGINAL_SHARED_BEGIN release_shared
unsafe fn release_shared(ptr: *mut Shared) {
    // `Shared` storage... follow the drop steps from Arc.
    if (*ptr).ref_cnt.fetch_sub(1, Ordering::Release) != 1 {
        return;
    }

    // This fence is needed to prevent reordering of use of the data and
    // deletion of the data.  Because it is marked `Release`, the decreasing
    // of the reference count synchronizes with this `Acquire` fence. This
    // means that use of the data happens before decreasing the reference
    // count, which happens before this fence, which happens before the
    // deletion of the data.
    //
    // As explained in the [Boost documentation][1],
    //
    // > It is important to enforce any possible access to the object in one
    // > thread (through an existing reference) to *happen before* deleting
    // > the object in a different thread. This is achieved by a "release"
    // > operation after dropping a reference (any access to the object
    // > through this reference must obviously happened before), and an
    // > "acquire" operation before deleting the object.
    //
    // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
    //
    // Thread sanitizer does not support atomic fences. Use an atomic load
    // instead.
    (*ptr).ref_cnt.load(Ordering::Acquire);

    // Explicit cleanup has the same payload/control effects as dropping the Box.
    free_shared(ptr);
}
// ORIGINAL_SHARED_END release_shared

// ORIGINAL_SHARED_BEGIN free_shared
// The explicit final cleanup below bypasses Shared::drop. All of its fields
// are non-owning scalar/atomic values; re-evaluate it if an owning field is added.
// In particular a substituted atomic type must not carry destructor effects.
const _: [(); 0] = [(); mem::needs_drop::<AtomicUsize>() as usize];

unsafe fn free_shared(ptr: *mut Shared) {
    // SAFETY: release_shared has recovered the sole reference after Acquire.
    // The allocation was made by Box<Shared>; buf/cap describe its separate
    // global-allocator byte allocation. Preserve the original destruction order.
    let buf = (*ptr).buf;
    let cap = (*ptr).cap;
    dealloc(buf, Layout::from_size_align(cap, 1).unwrap());
    dealloc(ptr.cast(), Layout::new::<Shared>());
}
// ORIGINAL_SHARED_END free_shared
