#[cfg(not(all(test, loom)))]
pub(crate) mod sync {
    pub(crate) mod atomic {
        #[cfg(creusot)]
        use creusot_std::prelude::*;

        #[cfg(all(not(feature = "extra-platforms"), creusot))]
        use core::sync::atomic::AtomicPtr as RawAtomicPtr;
        #[cfg(all(feature = "extra-platforms", creusot))]
        use extra_platforms::AtomicPtr as RawAtomicPtr;

        #[cfg(not(feature = "extra-platforms"))]
        pub(crate) use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
        #[cfg(feature = "extra-platforms")]
        pub(crate) use extra_platforms::{AtomicPtr, AtomicUsize, Ordering};

        // The proof profile needs the permission for the exact pointer cell
        // that Bytes stores. Keep this wrapper representation-identical to
        // the native cell; native builds continue to use the original atomic
        // type directly.
        #[cfg(creusot)]
        #[repr(transparent)]
        pub(crate) struct NativePtr<T>(RawAtomicPtr<T>);

        #[cfg(creusot)]
        const _: [(); core::mem::size_of::<RawAtomicPtr<()>>()] =
            [(); core::mem::size_of::<NativePtr<()>>()];
        #[cfg(creusot)]
        const _: [(); core::mem::align_of::<RawAtomicPtr<()>>()] =
            [(); core::mem::align_of::<NativePtr<()>>()];
        #[cfg(creusot)]
        const _: [(); 0] = [(); core::mem::needs_drop::<NativePtr<()>>() as usize];

        #[cfg(creusot)]
        impl<T> NativePtr<T> {
            #[trusted]
            #[check(terminates)]
            #[ensures(result.0 == *result.1.ward())]
            #[ensures(result.1.val() == value)]
            pub(crate) fn new_with_permission(
                value: *mut T,
            ) -> (Self, creusot_std::ghost::Ghost<creusot_std::ghost::perm::Perm<Self>>) {
                (Self(RawAtomicPtr::new(value)), creusot_std::ghost::Ghost::conjure())
            }

            #[logic(open(self))]
            pub(crate) fn raw_cell(&self) -> &RawAtomicPtr<T> {
                &self.0
            }

            #[requires(*own.inner_logic().ward() == self)]
            #[ensures(result.0 == *self.raw_cell())]
            #[ensures(&result.0 == own.inner_logic().ward().raw_cell())]
            #[ensures(result.1.inner_logic().val() == own.inner_logic().val())]
            #[ensures(result.1.inner_logic().ward().raw_cell() == &result.0)]
            #[ensures(result.1 == own)]
            pub(crate) fn into_raw_with_permission(
                self,
                own: creusot_std::ghost::Ghost<creusot_std::ghost::perm::Perm<Self>>,
            ) -> (
                RawAtomicPtr<T>,
                creusot_std::ghost::Ghost<creusot_std::ghost::perm::Perm<Self>>,
            ) {
                (self.0, own)
            }

            #[trusted]
            #[check(terminates)]
            #[requires(own.inner_logic().ward().raw_cell() == &*cell)]
            #[ensures(result == own.inner_logic().val())]
            #[ensures((^cell) == *cell)]
            #[ensures((^own.inner_logic()).ward().raw_cell() == own.inner_logic().ward().raw_cell())]
            #[ensures((^own.inner_logic()).val() == own.inner_logic().val())]
            pub(crate) fn exclusive_read(
                cell: &mut RawAtomicPtr<T>,
                own: creusot_std::ghost::Ghost<&mut creusot_std::ghost::perm::Perm<Self>>,
            ) -> *mut T {
                *cell.get_mut()
            }

        }

        #[cfg(creusot)]
        impl<T> creusot_std::ghost::perm::PermTarget for NativePtr<T> {
            type Value<'a>
                = *mut T
            where
                Self: 'a;
            type PermPayload = ();
        }

        pub(crate) trait AtomicMut<T> {
            fn with_mut<F, R>(&mut self, f: F) -> R
            where
                F: FnOnce(&mut *mut T) -> R;
        }

        impl<T> AtomicMut<T> for AtomicPtr<T> {
            fn with_mut<F, R>(&mut self, f: F) -> R
            where
                F: FnOnce(&mut *mut T) -> R,
            {
                f(self.get_mut())
            }
        }
    }
}

#[cfg(all(test, loom))]
pub(crate) mod sync {
    pub(crate) mod atomic {
        pub(crate) use loom::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

        pub(crate) trait AtomicMut<T> {}
    }
}
