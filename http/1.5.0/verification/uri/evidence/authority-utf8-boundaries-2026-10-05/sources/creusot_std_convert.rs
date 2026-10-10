use crate::prelude::*;
#[cfg(all(creusot, feature = "std"))]
use alloc::alloc::Allocator;
#[cfg(all(creusot, not(feature = "std")))]
use alloc::boxed::Box;

extern_spec! {
    mod core {
        mod convert {
            trait From<T> where Self: From<T> {
                // #[requires(true)]
                fn from(value: T) -> Self;
            }
            trait Into<T> where Self: Into<T> {
                fn into(self) -> T;
            }
            trait TryFrom<T> where Self: TryFrom<T> {
                fn try_from(value: T) -> Result<Self, <Self as TryFrom<T>>::Error>;
            }
            trait TryInto<T> where Self: TryInto<T> {
                fn try_into(self) -> Result<T, <Self as TryInto<T>>::Error>;
            }
            trait AsRef<T> where T: ?Sized {
                fn as_ref(&self) -> &T;
            }
        }
}

impl<T> From<T> for T {
        #[check(ghost)]
        #[ensures(result == self)]
        fn from(self) -> T;
    }

    impl<T, U> TryInto<U> for T
    where
        U: TryFrom<T>,
    {
        #[requires(<U as TryFrom<T>>::try_from.precondition((self,)))]
        #[ensures(<U as TryFrom<T>>::try_from.postcondition((self,), result))]
        fn try_into(self) -> Result<U, <U as TryFrom<T>>::Error>;
    }

    // This is the standard blanket implementation: a conversion already
    // provided by `Into` cannot fail, and `TryInto` forwards its exact result.
    impl<T, U> TryFrom<U> for T
    where
        U: Into<T>,
    {
        #[requires(<U as Into<T>>::into.precondition((value,)))]
        #[ensures(match result {
            Ok(converted) => <U as Into<T>>::into.postcondition((value,), converted),
            Err(_) => false,
        })]
        fn try_from(value: U) -> Result<T, core::convert::Infallible> {
            Ok(value.into())
        }
    }

    impl<T, U> Into<U> for T
    where
        U: From<T>,
    {
        // FIXME: inherit terminates/ghost status
        #[requires(<U as From<T>>::from.precondition((self,)))]
        #[ensures(<U as From<T>>::from.postcondition((self,), result))]
        fn into(self) -> U {
            U::from(self)
        }
    }

    impl<T> From<T> for Option<T> {
        #[check(ghost)]
        #[ensures(result == Some(x))]
        fn from(x: T) -> Self;
    }

    impl<T> From<T> for Box<T> {
        #[check(ghost)]
        #[ensures(*result == x)]
        fn from(x: T) -> Self;
    }

    impl<T: Clone> From<&[T]> for Box<[T]>
    {
        // FIXME: inherit ghost/terminates from clone
        #[ensures(result@.len() == s@.len())]
        #[ensures(forall<i> 0 <= i && i < s@.len() ==> <T as Clone>::clone.postcondition((&s@[i],), result@[i]))]
        fn from(s: &[T]) -> Self;
        // To verify: uses CloneToUninit
    }

    impl<T: Clone> From<&mut [T]> for Box<[T]>
    {
        // FIXME: inherit ghost/terminates from clone
        #[ensures(result@.len() == s@.len())]
        #[ensures(forall<i> 0 <= i && i < s@.len() ==> <T as Clone>::clone.postcondition((&s@[i],), result@[i]))]
        #[ensures(^s == *s)]
        fn from(s: &mut [T]) -> Self {
            Box::<[T]>::from(&*s)
        }
    }

    impl<T, const N: usize> From<[T; N]> for Box<[T]> {
        #[check(ghost)]
        #[ensures(result@ == s@)]
        fn from(s: [T; N]) -> Self {
            Box::new(s)
        }
    }
}

extern_spec! {
    impl TryFrom<i32> for u8 {
        #[check(terminates)]
        #[ensures(value@ < 0 || value@ > 255 ||
            exists<x: u8> result == Ok(x) && x@ == value@)]
        fn try_from(value: i32) -> Result<u8, core::num::TryFromIntError>;
    }
}

extern_spec! {
    impl TryFrom<i32> for u16 {
        #[check(terminates)]
        #[ensures(value@ < 0 || value@ > 65_535 ||
            exists<x: u16> result == Ok(x) && x@ == value@)]
        fn try_from(value: i32) -> Result<u16, core::num::TryFromIntError>;
    }
}

extern_spec! {
    impl TryFrom<i32> for u32 {
        #[check(terminates)]
        #[ensures(value@ < 0 ||
            exists<x: u32> result == Ok(x) && x@ == value@)]
        fn try_from(value: i32) -> Result<u32, core::num::TryFromIntError>;
    }
}

extern_spec! {
    impl TryFrom<i32> for u64 {
        #[check(terminates)]
        #[ensures(value@ < 0 ||
            exists<x: u64> result == Ok(x) && x@ == value@)]
        fn try_from(value: i32) -> Result<u64, core::num::TryFromIntError>;
    }
}

#[cfg(feature = "std")]
extern_spec! {
    impl<T: Clone> From<&[T]> for Vec<T>
    {
        // FIXME: inherit ghost/terminates from clone
        #[ensures(result@.len() == s@.len())]
        #[ensures(forall<i> 0 <= i && i < s@.len() ==> <T as Clone>::clone.postcondition((&s@[i],), result@[i]))]
        fn from(s: &[T]) -> Self {
            s.to_vec()
        }
    }

    impl<T: Clone> From<&mut [T]> for Vec<T>
    {
        // FIXME: inherit ghost/terminates from clone
        #[ensures(result@.len() == s@.len())]
        #[ensures(forall<i> 0 <= i && i < s@.len() ==> <T as Clone>::clone.postcondition((&s@[i],), result@[i]))]
        #[ensures(^s == *s)]
        fn from(s: &mut [T]) -> Self {
            s.to_vec()
        }
    }

    impl<T, A: Allocator> From<Box<[T], A>> for Vec<T, A> {
        #[check(ghost)]
        #[ensures(result@ == s@)]
        fn from(s: Box<[T], A>) -> Self {
            s.into_vec()
        }
    }

    impl<T: Clone, const N: usize> From<&[T; N]> for Vec<T> {
        // FIXME: inherit ghost/terminates from clone
        #[ensures(result@.len() == N@)]
        #[ensures(forall<i> 0 <= i && i < s@.len() ==> <T as Clone>::clone.postcondition((&s@[i],), result@[i]))]
        fn from(s: &[T; N]) -> Self {
            Vec::<T>::from(s.as_slice())
        }
    }

    impl<T: Clone, const N: usize> From<&mut [T; N]> for Vec<T> {
        // FIXME: inherit ghost/terminates from clone
        #[ensures(result@.len() == N@)]
        #[ensures(forall<i> 0 <= i && i < s@.len() ==> <T as Clone>::clone.postcondition((&s@[i],), result@[i]))]
        #[ensures(^s == *s)]
        fn from(s: &mut [T; N]) -> Self {
            Vec::<T>::from(s.as_mut_slice())
        }
    }

    impl<T, const N: usize> From<[T; N]> for Vec<T> {
        #[check(ghost)]
        #[ensures(result@ == s@)]
        fn from(s: [T; N]) -> Self {
            <[T]>::into_vec(Box::new(s))
        }
    }

    impl<T, A: Allocator> From<Vec<T, A>> for Box<[T], A> {
        #[check(ghost)]
        #[ensures(result@ == v@)]
        fn from(v: Vec<T, A>) -> Self {
            v.into_boxed_slice()
        }
    }
}

// These standard implementations return the identical string value. The
// checked concrete specs are the base cases consumed by the faithful generic
// reference forwarding implementation below.
extern_spec! {
    impl AsRef<str> for str {
        #[check(ghost)]
        #[ensures(result@ == self@)]
        fn as_ref(&self) -> &str {
            self
        }
    }
}

#[cfg(feature = "std")]
extern_spec! {
    impl AsRef<str> for String {
        #[check(ghost)]
        #[ensures(result@ == self@)]
        fn as_ref(&self) -> &str {
            self
        }
    }
}

extern_spec! {
    impl<'a, T: ?Sized, U: ?Sized> AsRef<U> for &'a T
    where
        T: AsRef<U>,
    {
        #[requires(<T as AsRef<U>>::as_ref.precondition((*self,)))]
        #[ensures(<T as AsRef<U>>::as_ref.postcondition((*self,), result))]
        fn as_ref<'b>(&'b self) -> &'b U {
            <T as AsRef<U>>::as_ref(*self)
        }
    }
}

macro_rules! spec_from {
    ($src:ty => $tgt:ty) => {
        extern_spec! {
            impl From<$src> for $tgt {
                #[check(ghost)]
                #[ensures(result == (small as Self))]
                fn from(small: $src) -> Self {
                    small as Self
                }
            }
        }
    };
}

spec_from!(bool => u8);
spec_from!(bool => u16);
spec_from!(bool => u32);
spec_from!(bool => u64);
spec_from!(bool => u128);
spec_from!(bool => usize);
spec_from!(bool => i8);
spec_from!(bool => i16);
spec_from!(bool => i32);
spec_from!(bool => i64);
spec_from!(bool => i128);
spec_from!(bool => isize);

// unsigned -> unsigned
spec_from!(u8 => u16);
spec_from!(u8 => u32);
spec_from!(u8 => u64);
spec_from!(u8 => u128);
spec_from!(u8 => usize);
spec_from!(u16 => u32);
spec_from!(u16 => u64);
spec_from!(u16 => u128);
spec_from!(u16 => usize);
spec_from!(u32 => u64);
spec_from!(u32 => u128);
spec_from!(u64 => u128);

// signed -> signed
spec_from!(i8 => i16);
spec_from!(i8 => i32);
spec_from!(i8 => i64);
spec_from!(i8 => i128);
spec_from!(i8 => isize);
spec_from!(i16 => i32);
spec_from!(i16 => i64);
spec_from!(i16 => i128);
spec_from!(i16 => isize);
spec_from!(i32 => i64);
spec_from!(i32 => i128);
spec_from!(i64 => i128);

// unsigned -> signed
spec_from!(u8 => i16);
spec_from!(u8 => i32);
spec_from!(u8 => i64);
spec_from!(u8 => i128);
spec_from!(u8 => isize);
spec_from!(u16 => i32);
spec_from!(u16 => i64);
spec_from!(u16 => i128);
spec_from!(u32 => i64);
spec_from!(u32 => i128);
spec_from!(u64 => i128);
