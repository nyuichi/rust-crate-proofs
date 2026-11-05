use oorandom::{Rand32, Rand64};
pub fn into32<U: From<Rand32>>(x: Rand32) -> U { x.into() }
pub fn into64<U: From<Rand64>>(x: Rand64) -> U { x.into() }
pub fn try_into32<U: TryFrom<Rand32>>(x: Rand32) -> Result<U, U::Error> { x.try_into() }
pub fn try_into64<U: TryFrom<Rand64>>(x: Rand64) -> Result<U, U::Error> { x.try_into() }
pub fn try_from32<U: Into<Rand32>>(x: U) -> Result<Rand32, core::convert::Infallible> { Rand32::try_from(x) }
pub fn try_from64<U: Into<Rand64>>(x: U) -> Result<Rand64, core::convert::Infallible> { Rand64::try_from(x) }
pub fn core_into<T, U: From<T>>(x:T) -> U { x.into() }
pub fn core_try_into<T, U: TryFrom<T>>(x:T) -> Result<U,U::Error> { x.try_into() }
pub fn core_try_from<T, U: Into<T>>(x:U) -> Result<T,core::convert::Infallible> { T::try_from(x) }
