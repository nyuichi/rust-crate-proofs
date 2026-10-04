//! Diagnostic-only probes for the `Bytes` vtable clone recursion.
#![allow(unexpected_cfgs)]

#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg(feature = "static-cycle")]
mod static_cycle {
    pub struct Vtable {
        clone: unsafe fn() -> Bytes,
    }

    pub struct Bytes {
        vtable: &'static Vtable,
    }

    const STATIC_VTABLE: Vtable = Vtable {
        clone: static_clone,
    };

    impl Bytes {
        fn from_static() -> Bytes {
            Bytes {
                vtable: &STATIC_VTABLE,
            }
        }
    }

    impl Clone for Bytes {
        fn clone(&self) -> Bytes {
            unsafe { (self.vtable.clone)() }
        }
    }

    unsafe fn static_clone() -> Bytes {
        Bytes::from_static()
    }

    /// Reproduce `static_clone -> from_static -> STATIC_VTABLE -> static_clone`.
    pub fn clone_static() -> Bytes {
        Bytes::from_static().clone()
    }
}

#[cfg(feature = "owned-cycle")]
mod owned_cycle {
    use core::marker::PhantomData;

    pub struct Vtable {
        clone: unsafe fn() -> Bytes,
    }

    pub struct Bytes {
        vtable: &'static Vtable,
    }

    struct Owned<T>(PhantomData<T>);

    impl<T> Owned<T> {
        const VTABLE: Vtable = Vtable {
            clone: owned_clone::<T>,
        };
    }

    fn from_owned<T>() -> Bytes {
        Bytes {
            vtable: &Owned::<T>::VTABLE,
        }
    }

    impl Clone for Bytes {
        fn clone(&self) -> Bytes {
            unsafe { (self.vtable.clone)() }
        }
    }

    unsafe fn owned_clone<T>() -> Bytes {
        from_owned::<T>()
    }

    /// Reproduce `owned_clone<T> -> from_owned<T> -> Owned<T>::VTABLE -> owned_clone<T>`.
    pub fn clone_owned() -> Bytes {
        from_owned::<u8>().clone()
    }
}

#[cfg(feature = "accessor-cycle")]
mod accessor_cycle {
    pub struct Vtable {
        clone: unsafe fn() -> Bytes,
    }

    pub struct Bytes {
        vtable: &'static Vtable,
    }

    static STATIC_VTABLE: Vtable = Vtable {
        clone: static_clone,
    };

    fn static_vtable() -> &'static Vtable {
        &STATIC_VTABLE
    }

    fn from_static() -> Bytes {
        Bytes {
            vtable: static_vtable(),
        }
    }

    impl Clone for Bytes {
        fn clone(&self) -> Bytes {
            unsafe { (self.vtable.clone)() }
        }
    }

    unsafe fn static_clone() -> Bytes {
        from_static()
    }

    /// Test whether hiding the static access behind an accessor removes the cycle.
    pub fn clone_static() -> Bytes {
        from_static().clone()
    }
}

#[cfg(feature = "passed-vtable")]
mod passed_vtable {
    use core::marker::PhantomData;
    use core::ptr;
    use core::sync::atomic::AtomicPtr;

    pub struct Vtable {
        clone: unsafe fn(&'static Vtable, &AtomicPtr<()>, *const u8, usize) -> Bytes,
    }

    pub struct Bytes {
        ptr: *const u8,
        len: usize,
        data: AtomicPtr<()>,
        vtable: &'static Vtable,
    }

    impl Clone for Bytes {
        fn clone(&self) -> Bytes {
            unsafe { (self.vtable.clone)(self.vtable, &self.data, self.ptr, self.len) }
        }
    }

    const STATIC_VTABLE: Vtable = Vtable {
        clone: static_clone,
    };

    unsafe fn static_clone(
        vtable: &'static Vtable,
        _: &AtomicPtr<()>,
        ptr: *const u8,
        len: usize,
    ) -> Bytes {
        Bytes {
            ptr,
            len,
            data: AtomicPtr::new(ptr::null_mut()),
            vtable,
        }
    }

    struct Owned<T>(PhantomData<T>);

    impl<T> Owned<T> {
        const VTABLE: Vtable = Vtable {
            clone: owned_clone::<T>,
        };
    }

    unsafe fn owned_clone<T>(
        vtable: &'static Vtable,
        _: &AtomicPtr<()>,
        ptr: *const u8,
        len: usize,
    ) -> Bytes {
        Bytes {
            ptr,
            len,
            data: AtomicPtr::new(ptr::null_mut()),
            vtable,
        }
    }

    fn from_static() -> Bytes {
        Bytes {
            ptr: ptr::null(),
            len: 0,
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    fn from_owned<T>() -> Bytes {
        Bytes {
            ptr: ptr::null(),
            len: 0,
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &Owned::<T>::VTABLE,
        }
    }

    /// The callback receives the actual table, so its body does not look the table up again.
    pub fn clone_static() -> Bytes {
        from_static().clone()
    }

    /// The generic associated-const callback receives the actual table as well.
    pub fn clone_owned() -> Bytes {
        from_owned::<u8>().clone()
    }
}
