        let shared = Box::new(Shared {
            buf: ptr,
            cap,
            ref_cnt: AtomicUsize::new(1),
        });

        let shared = Box::into_raw(shared);
        // The pointer should be aligned, so this assert should
        // always succeed.
        debug_assert!(
            0 == (shared as usize & KIND_MASK),
            "internal: Box<Shared> should have an aligned pointer",
        );
        Bytes {
            ptr,
            len,
            data: AtomicPtr::new(shared as _),
            vtable: &SHARED_VTABLE,
        }